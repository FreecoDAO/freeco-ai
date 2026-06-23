use tool_gateway::{clients::llm::ChatMessage, LlmClient};

use crate::error::SecretaryError;

/// Classified user intent, used for routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Product search, price comparison, nutritional advice, store location.
    Shopping,
    /// Reminder, calendar event, or note-taking request.
    Memo,
    /// Wellness advice, exercise plans, dietary coaching.
    Wellness,
    /// Anything else — answered directly by the Secretary via LLM.
    General,
}

// ── LLM-based classification ──────────────────────────────────────────────────

const CLASSIFY_SYSTEM: &str = "\
Classify the user's request into exactly one category.
Output ONLY the single word — nothing else.
Categories: SHOPPING, MEMO, WELLNESS, GENERAL";

/// Classify intent using an LLM call.
///
/// Uses a tiny `max_tokens=4` budget; falls back to [`classify_intent_heuristic`]
/// if the LLM returns an unrecognised label.
pub async fn classify_intent_llm(
    text: &str,
    llm: &LlmClient,
) -> Result<Intent, SecretaryError> {
    let messages = vec![
        ChatMessage::system(CLASSIFY_SYSTEM),
        ChatMessage::user(text),
    ];
    let resp = llm
        .chat(messages, 4)
        .await
        .map_err(|e| SecretaryError::LlmError(e.to_string()))?;

    let label = resp.content.trim().to_uppercase();
    Ok(match label.as_str() {
        "SHOPPING" => Intent::Shopping,
        "MEMO" => Intent::Memo,
        "WELLNESS" => Intent::Wellness,
        _ => {
            // Unknown label from LLM — fall back to heuristic
            classify_intent_heuristic(text)
        }
    })
}

// ── Keyword heuristic (no LLM, WASM-safe) ────────────────────────────────────

/// Classify intent using keyword heuristics — no LLM call required.
///
/// Used as:
/// - A fallback when the LLM is unavailable or budget is exhausted.
/// - The primary method in WASM contexts where async HTTP is constrained.
/// - A fast path for obviously-typed queries.
///
/// Priority order: **Memo > Wellness > Shopping > General**
/// This prevents "remind me to buy milk" from being mis-classified as Shopping.
pub fn classify_intent_heuristic(text: &str) -> Intent {
    let lower = text.to_lowercase();

    // Memo markers — checked first; most unambiguous intent signals.
    let memo_kw = [
        "remind", "memo", "note:", "add a note", "calendar", "schedule",
        "appointment", "meeting", "deadline", "to-do", "todo", "remember",
    ];

    // Wellness markers — checked before shopping so "calories in oat milk"
    // is treated as nutrition advice rather than a purchase request.
    // Avoid product names like "protein powder" — those are Shopping.
    let wellness_kw = [
        "calorie", "calories", "workout", "exercise", "diet ", "nutrition",
        "vitamin", "supplement", "yoga", "meditation", "sleep", "stress",
        "mental health", "fitness",
    ];

    // Shopping markers — specific enough to avoid false positives.
    // "eco" is intentionally excluded (matches "Freeco"); use "eco-score" instead.
    let shopping_kw = [
        "buy ", "shop", "price", "product", "organic", "vegan", "food",
        "oat milk", "almond milk", "cheese", "olive oil", "bread", "meat",
        "fish", "vegetable", "fruit", "supermarket", "where can i find",
        "how much is", "cheapest", "best value", "recommend", "groceries",
        "ingredient", "eco-score", "nutri-score", "nutriscore", "barcode",
        "brand", "migros", "coop", "aldi", "lidl", "denner", "manor",
        "bio ", "fairtrade",
    ];

    if memo_kw.iter().any(|kw| lower.contains(kw)) {
        Intent::Memo
    } else if wellness_kw.iter().any(|kw| lower.contains(kw)) {
        Intent::Wellness
    } else if shopping_kw.iter().any(|kw| lower.contains(kw)) {
        Intent::Shopping
    } else {
        Intent::General
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shopping_queries_classified_correctly() {
        for query in &[
            "buy organic oat milk",
            "where can i find vegan cheese in Geneva",
            "best price for olive oil",
            "is this product eco-friendly",
            "Migros or Coop for groceries",
        ] {
            assert_eq!(
                classify_intent_heuristic(query),
                Intent::Shopping,
                "query {query:?} should be Shopping"
            );
        }
    }

    #[test]
    fn memo_queries_classified_correctly() {
        for query in &[
            "remind me to buy milk tomorrow",
            "add a note: pick up bread",
            "schedule meeting with team",
            "set a calendar appointment for 3pm",
        ] {
            assert_eq!(
                classify_intent_heuristic(query),
                Intent::Memo,
                "query {query:?} should be Memo"
            );
        }
    }

    #[test]
    fn wellness_queries_classified_correctly() {
        for query in &[
            "how many calories in oat milk",
            "give me a workout plan",
            "tips for better sleep",
            "what vitamins should I take",
        ] {
            assert_eq!(
                classify_intent_heuristic(query),
                Intent::Wellness,
                "query {query:?} should be Wellness"
            );
        }
    }

    #[test]
    fn general_queries_classified_correctly() {
        for query in &[
            "hello",
            "what is Freeco",
            "how does this work",
            "tell me a joke",
        ] {
            assert_eq!(
                classify_intent_heuristic(query),
                Intent::General,
                "query {query:?} should be General"
            );
        }
    }

    #[test]
    fn shopping_beats_wellness_keyword_overlap() {
        // "protein" is a wellness keyword but "buy protein powder" should be Shopping
        let intent = classify_intent_heuristic("buy protein powder at the store");
        assert_eq!(intent, Intent::Shopping);
    }
}
