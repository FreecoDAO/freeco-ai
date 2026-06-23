use serde::{Deserialize, Serialize};

use crate::{message::Message, time_util::now_ms};

/// A single product recommendation within a tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedProduct {
    pub name: String,
    pub brand: Option<String>,
    /// Price in CHF.
    pub price_chf: Option<f32>,
    /// Store or retailer name.
    pub store: Option<String>,
    /// Product page URL.
    pub url: Option<String>,
    /// Nutri-Score grade (`"A"` … `"E"` or `"Unknown"`).
    pub nutri_score: Option<String>,
    /// Eco-Score grade (`"A"` … `"E"` or `"Unknown"`).
    pub eco_score: Option<String>,
    /// Why this product was recommended.
    pub reason: String,
}

/// A three-tier shopping recommendation (Budget / Value / Luxury).
///
/// Produced by the Shopping Concierge agent combining Tavily search results,
/// Open Food Facts nutritional data, and a Gemma 4 26B synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingRecommendation {
    /// The original user query.
    pub query: String,
    /// Location used for geo-aware results.
    pub location: Option<String>,
    /// Lowest price option.
    pub budget: Option<RecommendedProduct>,
    /// Best value-for-money option.
    pub value: Option<RecommendedProduct>,
    /// Premium / sustainable option.
    pub luxury: Option<RecommendedProduct>,
    /// Two-sentence LLM synthesis summary.
    pub analysis: String,
    /// Total tokens consumed generating this response.
    pub tokens_used: u64,
}

/// Typed payload returned by `Agent::handle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ResponseContent {
    /// Plain text answer.
    Text(String),

    /// Three-tier product recommendation.
    ShoppingRecommendation(ShoppingRecommendation),

    /// Agent requests the runtime forward this message to another agent.
    /// The runtime resolves the route transparently (up to `max_route_depth`).
    RouteToAgent {
        target_agent_id: String,
        reason: String,
        /// The message that should be forwarded.
        forward_message: Box<Message>,
    },

    /// Work was delegated — the result will arrive asynchronously.
    Delegated {
        delegate_agent_id: String,
        task_id: String,
    },

    /// CEO executive decision with action items.
    ExecutiveDecision {
        summary: String,
        actions: Vec<String>,
        /// `(agent_id, task_description)` pairs.
        delegations: Vec<(String, String)>,
    },

    /// Non-fatal error surfaced in the response rather than as `Err`.
    SoftError(String),
}

/// The value returned by `Agent::handle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// ID of the message this is replying to.
    pub in_reply_to: String,
    /// ID of the agent that produced this response.
    pub from_agent: String,
    /// Typed response payload.
    pub content: ResponseContent,
    /// Milliseconds since Unix epoch.
    pub timestamp_ms: i64,
}

impl AgentResponse {
    /// Convenience: wrap a plain text reply.
    pub fn text(
        in_reply_to: impl Into<String>,
        from_agent: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            in_reply_to: in_reply_to.into(),
            from_agent: from_agent.into(),
            content: ResponseContent::Text(text.into()),
            timestamp_ms: now_ms(),
        }
    }

    /// Convenience: wrap a shopping recommendation.
    pub fn shopping(
        in_reply_to: impl Into<String>,
        from_agent: impl Into<String>,
        rec: ShoppingRecommendation,
    ) -> Self {
        Self {
            in_reply_to: in_reply_to.into(),
            from_agent: from_agent.into(),
            content: ResponseContent::ShoppingRecommendation(rec),
            timestamp_ms: now_ms(),
        }
    }

    /// Convenience: request forwarding to another agent.
    pub fn route(
        in_reply_to: impl Into<String>,
        from_agent: impl Into<String>,
        target: impl Into<String>,
        reason: impl Into<String>,
        msg: Message,
    ) -> Self {
        Self {
            in_reply_to: in_reply_to.into(),
            from_agent: from_agent.into(),
            content: ResponseContent::RouteToAgent {
                target_agent_id: target.into(),
                reason: reason.into(),
                forward_message: Box::new(msg),
            },
            timestamp_ms: now_ms(),
        }
    }

    /// Convenience: wrap a CEO executive decision.
    pub fn executive_decision(
        in_reply_to: impl Into<String>,
        from_agent: impl Into<String>,
        summary: impl Into<String>,
        actions: Vec<String>,
        delegations: Vec<(String, String)>,
    ) -> Self {
        Self {
            in_reply_to: in_reply_to.into(),
            from_agent: from_agent.into(),
            content: ResponseContent::ExecutiveDecision {
                summary: summary.into(),
                actions,
                delegations,
            },
            timestamp_ms: now_ms(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_response_has_correct_fields() {
        let r = AgentResponse::text("msg-1", "agent-secretary", "Hello!");
        assert_eq!(r.in_reply_to, "msg-1");
        assert_eq!(r.from_agent, "agent-secretary");
        assert!(matches!(r.content, ResponseContent::Text(_)));
    }

    #[test]
    fn shopping_recommendation_roundtrips_through_json() {
        let rec = ShoppingRecommendation {
            query: "oat milk".into(),
            location: Some("Geneva".into()),
            budget: Some(RecommendedProduct {
                name: "Alnatura Oat Milk".into(),
                brand: Some("Alnatura".into()),
                price_chf: Some(1.95),
                store: Some("Coop".into()),
                url: None,
                nutri_score: Some("A".into()),
                eco_score: Some("B".into()),
                reason: "Lowest price in Geneva Coop stores.".into(),
            }),
            value: None,
            luxury: None,
            analysis: "Alnatura offers excellent value.".into(),
            tokens_used: 512,
        };
        let resp = AgentResponse::shopping("q-1", "agent-shopping", rec);
        let json = serde_json::to_string(&resp).unwrap();
        let back: AgentResponse = serde_json::from_str(&json).unwrap();
        match back.content {
            ResponseContent::ShoppingRecommendation(r) => {
                assert_eq!(r.query, "oat milk");
                assert_eq!(r.tokens_used, 512);
                assert!(r.budget.is_some());
            }
            _ => panic!("wrong content type"),
        }
    }

    #[test]
    fn route_response_wraps_message() {
        let fwd = Message::user_text("fwd-1", "agent-shopping", "buy oat milk");
        let resp = AgentResponse::route("msg-1", "agent-secretary", "agent-shopping", "shopping intent", fwd);
        match resp.content {
            ResponseContent::RouteToAgent { target_agent_id, reason, .. } => {
                assert_eq!(target_agent_id, "agent-shopping");
                assert_eq!(reason, "shopping intent");
            }
            _ => panic!("wrong content type"),
        }
    }
}
