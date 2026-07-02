use std::sync::Arc;

use agent_core::{AgentContext, RecommendedProduct, ShoppingRecommendation};
use tool_gateway::{clients::llm::ChatMessage, LlmClient, ProductInfo, SearchResult, ToolGateway};

use crate::error::ShoppingError;

/// Orchestrates parallel data gathering and LLM synthesis for a shopping query.
pub struct ProductResearcher {
    gateway: Arc<ToolGateway>,
    llm: Arc<LlmClient>,
}

impl ProductResearcher {
    pub fn new(gateway: Arc<ToolGateway>, llm: Arc<LlmClient>) -> Self {
        Self { gateway, llm }
    }

    /// Research a product query and return a three-tier recommendation.
    pub async fn research(
        &self,
        query: &str,
        location: Option<&str>,
        ctx: &AgentContext,
    ) -> Result<ShoppingRecommendation, ShoppingError> {
        let loc = location.unwrap_or("");

        // Parallel data gathering: web search + nutrition DB
        #[cfg(not(target_arch = "wasm32"))]
        let (search_res, food_res) = tokio::join!(
            self.gateway.tavily_search(query, loc),
            self.gateway.open_food_facts_search(query),
        );

        // Sequential fallback for WASM (no tokio::join!)
        #[cfg(target_arch = "wasm32")]
        let search_res = self.gateway.tavily_search(query, loc).await;
        #[cfg(target_arch = "wasm32")]
        let food_res = self.gateway.open_food_facts_search(query).await;

        let search_results = search_res.unwrap_or_default();
        let food_results = food_res.unwrap_or_default();

        tracing::debug!(
            search_count = search_results.len(),
            food_count = food_results.len(),
            query,
            "gathered product data"
        );

        // Build the LLM prompt
        let prompt = build_prompt(query, loc, &search_results, &food_results, ctx);
        let messages = vec![
            ChatMessage::system(SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let llm_resp = self
            .llm
            .chat(messages, 1024)
            .await
            .map_err(|e| ShoppingError::LlmError(e.to_string()))?;

        let rec = parse_recommendation(
            query,
            location,
            &llm_resp.content,
            llm_resp.usage.total_tokens,
            &food_results,
        );
        Ok(rec)
    }
}

// ── Prompt engineering ────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = "\
You are the Freeco.AI Shopping Concierge for Swiss sustainable living. \
Given product search results and nutrition data, produce a concise recommendation \
in exactly three tiers using this pipe-delimited format (one line each):

BUDGET|ProductName|Brand|PriceCHF|Store|Reason
VALUE|ProductName|Brand|PriceCHF|Store|Reason
LUXURY|ProductName|Brand|PriceCHF|Store|Reason
ANALYSIS|Two-sentence summary of the options.

Rules:
- Use only data provided; do not invent prices or stores.
- If a tier has no suitable product output: TIER|N/A
- PriceCHF is a number only (e.g. 1.95).
- Keep Reason under 15 words.";

fn build_prompt(
    query: &str,
    location: &str,
    search: &[SearchResult],
    food: &[ProductInfo],
    ctx: &AgentContext,
) -> String {
    let mut parts = vec![format!(
        "Query: {query}\nLocation: {loc}\nSubscription: {tier}\n\nSearch Results:",
        loc = if location.is_empty() {
            "Not specified"
        } else {
            location
        },
        tier = ctx.tier
    )];

    for (i, r) in search.iter().take(5).enumerate() {
        let snippet = if r.content.len() > 200 {
            &r.content[..200]
        } else {
            &r.content
        };
        parts.push(format!("{}. {} — {}", i + 1, r.title, snippet));
    }

    if !food.is_empty() {
        parts.push("\nNutrition/Eco Data:".into());
        for (i, p) in food.iter().take(3).enumerate() {
            let brand = p.brands.first().map(String::as_str).unwrap_or("Unknown");
            parts.push(format!(
                "{}. {} by {} — Nutri: {:?}, Eco: {:?}, Vegan: {:?}",
                i + 1,
                p.name,
                brand,
                p.nutri_score,
                p.eco_score,
                p.is_vegan,
            ));
        }
    }

    parts.join("\n")
}

// ── Response parsing ──────────────────────────────────────────────────────────

fn parse_recommendation(
    query: &str,
    location: Option<&str>,
    llm_text: &str,
    tokens: u64,
    food_results: &[ProductInfo],
) -> ShoppingRecommendation {
    let mut budget = None;
    let mut value = None;
    let mut luxury = None;
    let mut analysis = String::new();

    for line in llm_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("BUDGET|") {
            budget = parse_product_line(rest, food_results);
        } else if let Some(rest) = line.strip_prefix("VALUE|") {
            value = parse_product_line(rest, food_results);
        } else if let Some(rest) = line.strip_prefix("LUXURY|") {
            luxury = parse_product_line(rest, food_results);
        } else if let Some(rest) = line.strip_prefix("ANALYSIS|") {
            analysis = rest.to_string();
        }
    }

    // Fallback: use raw LLM text as analysis if structured parsing produced nothing
    if analysis.is_empty() && budget.is_none() && value.is_none() && luxury.is_none() {
        analysis = llm_text.to_string();
    }

    ShoppingRecommendation {
        query: query.to_string(),
        location: location.map(str::to_string),
        budget,
        value,
        luxury,
        analysis,
        tokens_used: tokens,
    }
}

fn parse_product_line(line: &str, food_results: &[ProductInfo]) -> Option<RecommendedProduct> {
    if line.starts_with("N/A") {
        return None;
    }
    let parts: Vec<&str> = line.splitn(6, '|').collect();
    let name = parts.first().copied().unwrap_or("").trim().to_string();
    if name.is_empty() {
        return None;
    }

    let brand = parts
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let price_chf = parts.get(2).and_then(|s| s.trim().parse::<f32>().ok());
    let store = parts
        .get(3)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let reason = parts.get(5).copied().unwrap_or("").trim().to_string();

    // Enrich with Open Food Facts data when the product name matches
    let food_match = food_results.iter().find(|f| {
        f.name.to_lowercase().contains(&name.to_lowercase())
            || name.to_lowercase().contains(&f.name.to_lowercase())
    });
    let nutri_score = food_match.map(|f| format!("{:?}", f.nutri_score));
    let eco_score = food_match.map(|f| format!("{:?}", f.eco_score));

    Some(RecommendedProduct {
        name,
        brand,
        price_chf,
        store,
        url: None,
        nutri_score,
        eco_score,
        reason,
    })
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_food() -> Vec<ProductInfo> {
        vec![]
    }

    #[test]
    fn parse_full_llm_output() {
        let llm = "\
BUDGET|Alnatura Oat|Alnatura|1.95|Coop|Cheapest available
VALUE|Oatly Organic|Oatly|2.80|Migros|Best nutri-score
LUXURY|Oatly Barista|Oatly|3.50|Bio-Markt|Premium foamy option
ANALYSIS|Good range available in Geneva. Oatly leads on sustainability.";

        let rec = parse_recommendation("oat milk", Some("Geneva"), llm, 200, &empty_food());

        assert_eq!(rec.query, "oat milk");
        assert_eq!(rec.location.as_deref(), Some("Geneva"));
        assert_eq!(rec.tokens_used, 200);

        let budget = rec.budget.unwrap();
        assert_eq!(budget.name, "Alnatura Oat");
        assert_eq!(budget.brand.as_deref(), Some("Alnatura"));
        assert!((budget.price_chf.unwrap() - 1.95).abs() < 0.01);
        assert_eq!(budget.store.as_deref(), Some("Coop"));

        let value = rec.value.unwrap();
        assert_eq!(value.name, "Oatly Organic");

        let luxury = rec.luxury.unwrap();
        assert_eq!(luxury.name, "Oatly Barista");

        assert!(rec.analysis.contains("Geneva"));
    }

    #[test]
    fn parse_na_tier_is_none() {
        let llm = "BUDGET|N/A\nVALUE|Some Product|Brand|2.00|Store|Good\nLUXURY|N/A\nANALYSIS|Limited options.";
        let rec = parse_recommendation("rare item", None, llm, 50, &empty_food());
        assert!(rec.budget.is_none());
        assert!(rec.value.is_some());
        assert!(rec.luxury.is_none());
    }

    #[test]
    fn parse_fallback_analysis_when_no_structure() {
        let llm = "Sorry, I could not find any results for this query.";
        let rec = parse_recommendation("q", None, llm, 10, &empty_food());
        assert!(rec.budget.is_none());
        assert!(rec.value.is_none());
        assert!(rec.luxury.is_none());
        assert!(!rec.analysis.is_empty());
    }

    #[test]
    fn parse_enriches_with_food_data() {
        use tool_gateway::{EcoScore, NutriScore};
        let food = vec![ProductInfo {
            barcode: "123".into(),
            name: "Oatly Oat Milk".into(),
            brands: vec!["Oatly".into()],
            eco_score: EcoScore::B,
            nutri_score: NutriScore::A,
            nova_group: None,
            is_vegan: Some(true),
            is_vegetarian: Some(true),
            is_organic: None,
            origin_country: None,
            labels: vec![],
            image_url: None,
        }];

        let llm = "VALUE|Oatly Oat Milk|Oatly|2.80|Migros|Great eco-score\nANALYSIS|Recommended.";
        let rec = parse_recommendation("oat milk", None, llm, 100, &food);

        let value = rec.value.unwrap();
        assert_eq!(value.nutri_score.as_deref(), Some("A"));
        assert_eq!(value.eco_score.as_deref(), Some("B"));
    }

    #[test]
    fn parse_product_line_handles_missing_price() {
        let result = parse_product_line("Some Product||Migros|Reason here|extra", &empty_food());
        let prod = result.unwrap();
        assert_eq!(prod.name, "Some Product");
        assert!(prod.price_chf.is_none()); // empty price field
    }
}
