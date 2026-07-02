use std::sync::Arc;

use agent_core::{
    Agent, AgentContext, AgentError, AgentResponse, Capability, Message, MessageContent,
};
use async_trait::async_trait;
use tool_gateway::{LlmClient, ToolGateway};

use crate::research::ProductResearcher;

/// Freeco.AI Shopping Concierge agent.
///
/// Accepts both [`MessageContent::ShoppingQuery`] and plain [`MessageContent::Text`]
/// (which is treated as an unstructured shopping request).
pub struct ShoppingAgent {
    id: String,
    researcher: ProductResearcher,
}

impl ShoppingAgent {
    /// Create a new Shopping Concierge.
    ///
    /// - `gateway` – permission-gated tool access (Tavily, OpenFoodFacts, GooglePlaces).
    /// - `llm`     – Novita AI / Gemma 4 26B client for recommendation synthesis.
    pub fn new(id: impl Into<String>, gateway: Arc<ToolGateway>, llm: Arc<LlmClient>) -> Self {
        Self {
            id: id.into(),
            researcher: ProductResearcher::new(gateway, llm),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Agent for ShoppingAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Freeco Shopping Concierge"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::ProductResearch,
            Capability::NutritionAnalysis,
            Capability::StoreLocator,
        ]
    }

    async fn handle(&self, ctx: &AgentContext, msg: Message) -> Result<AgentResponse, AgentError> {
        let (query, location) = match &msg.content {
            MessageContent::ShoppingQuery {
                query, location, ..
            } => (
                query.clone(),
                location.clone().or_else(|| ctx.location.clone()),
            ),
            MessageContent::Text(text) => (text.clone(), ctx.location.clone()),
            other => {
                return Err(AgentError::UnsupportedMessage {
                    agent_id: self.id.clone(),
                    message_type: format!("{other:?}"),
                })
            }
        };

        tracing::info!(
            agent_id = %self.id,
            query = %query,
            location = ?location,
            "shopping query received"
        );

        let rec = self
            .researcher
            .research(&query, location.as_deref(), ctx)
            .await
            .map_err(|e| AgentError::ToolError(e.to_string()))?;

        Ok(AgentResponse::shopping(&msg.id, &self.id, rec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{MessageRole, Priority};
    use mockito::Server;
    use tool_gateway::{ToolName, ToolPermissionManifest};

    fn all_tools_manifest() -> ToolPermissionManifest {
        ToolPermissionManifest::new(vec![
            ToolName::TavilySearch,
            ToolName::OpenFoodFacts,
            ToolName::GooglePlaces,
        ])
    }

    fn make_agent(tavily_url: &str, off_url: &str, llm_url: &str) -> ShoppingAgent {
        let gateway = Arc::new(ToolGateway::new_for_testing(
            all_tools_manifest(),
            tool_gateway::TavilyClient::new("tk".into()).with_base_url(tavily_url),
            tool_gateway::OpenFoodFactsClient::new().with_base_url(off_url),
            tool_gateway::GooglePlacesClient::new("gk".into()),
        ));
        let llm = Arc::new(LlmClient::new("lk").with_base_url(llm_url));
        ShoppingAgent::new("agent-shopping", gateway, llm)
    }

    #[tokio::test]
    async fn handle_shopping_query_returns_recommendation() {
        let mut tavily_srv = Server::new_async().await;
        let mut off_srv = Server::new_async().await;
        let mut llm_srv = Server::new_async().await;

        let _t = tavily_srv
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results": [{"title": "Coop Oat Milk","url":"https://coop.ch","content":"1.95 CHF","score":0.9}]}"#)
            .create_async()
            .await;

        let _o = off_srv
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"products":[]}"#)
            .create_async()
            .await;

        let llm_body = serde_json::json!({
            "choices": [{
                "message": {"role":"assistant","content":"BUDGET|Coop Oat Milk|Coop|1.95|Coop Geneva|Cheapest option\nVALUE|Alnatura Oat|Alnatura|2.50|Migros|Best value\nLUXURY|Oatly Barista|Oatly|3.80|Bio-Markt|Premium\nANALYSIS|Good options available in Geneva."},
                "finish_reason":"stop"
            }],
            "usage": {"prompt_tokens":100,"completion_tokens":50,"total_tokens":150}
        }).to_string();

        let _l = llm_srv
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&llm_body)
            .create_async()
            .await;

        let agent = make_agent(&tavily_srv.url(), &off_srv.url(), &llm_srv.url());
        let ctx = AgentContext::new("agent-shopping", "user-1").with_location("Geneva");
        let msg = Message::shopping_query("q-1", "agent-shopping", "organic oat milk", None);

        let resp = agent.handle(&ctx, msg).await.unwrap();
        assert_eq!(resp.from_agent, "agent-shopping");
        assert_eq!(resp.in_reply_to, "q-1");
        assert!(matches!(
            resp.content,
            agent_core::ResponseContent::ShoppingRecommendation(_)
        ));
    }

    #[tokio::test]
    async fn handle_plain_text_treats_as_query() {
        let mut tavily_srv = Server::new_async().await;
        let mut off_srv = Server::new_async().await;
        let mut llm_srv = Server::new_async().await;

        let _t = tavily_srv
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[]}"#)
            .create_async()
            .await;
        let _o = off_srv
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"products":[]}"#)
            .create_async()
            .await;
        let _l = llm_srv.mock("POST", "/chat/completions").with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices":[{"message":{"content":"ANALYSIS|No results."},"finish_reason":"stop"}],"usage":{"total_tokens":10}}"#)
            .create_async().await;

        let agent = make_agent(&tavily_srv.url(), &off_srv.url(), &llm_srv.url());
        let ctx = AgentContext::new("agent-shopping", "user-1");
        let msg = Message::user_text("m-1", "agent-shopping", "vegan cheese please");
        let resp = agent.handle(&ctx, msg).await.unwrap();
        assert_eq!(resp.from_agent, "agent-shopping");
    }

    #[tokio::test]
    async fn handle_unsupported_message_returns_error() {
        let agent = make_agent("http://unused", "http://unused", "http://unused");
        let ctx = AgentContext::new("agent-shopping", "u");
        let msg = Message {
            id: "m-1".into(),
            from: "ceo".into(),
            to: "agent-shopping".into(),
            role: MessageRole::Agent,
            content: agent_core::MessageContent::Directive {
                instruction: "buy everything".into(),
                priority: Priority::High,
            },
            timestamp_ms: 0,
        };
        let err = agent.handle(&ctx, msg).await.unwrap_err();
        assert!(matches!(err, AgentError::UnsupportedMessage { .. }));
    }
}
