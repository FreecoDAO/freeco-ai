use std::sync::Arc;

use agent_core::{
    Agent, AgentContext, AgentError, AgentResponse, Capability, Message, MessageContent,
    MessageRole,
};
use async_trait::async_trait;
use tool_gateway::{clients::llm::ChatMessage, LlmClient};

use crate::classifier::{classify_intent_heuristic, classify_intent_llm, Intent};

/// Freeco.AI CEO Secretary agent (OpenClaw).
///
/// Acts as the **supervisor** in the multi-agent graph:
/// - Shopping queries → forwarded to `shopping_agent_id`
/// - Wellness queries → forwarded to `shopping_agent_id` (nutrition context)
/// - Memo / calendar → handled inline with a brief LLM reply
/// - General Q&A     → handled inline with a brief LLM reply
pub struct SecretaryAgent {
    id: String,
    llm: Arc<LlmClient>,
    shopping_agent_id: String,
    /// Use the fast keyword heuristic instead of an LLM call for classification.
    /// Set to `true` to conserve tokens or for WASM deployments.
    use_heuristic: bool,
}

impl SecretaryAgent {
    /// Create a Secretary that uses the LLM for intent classification.
    pub fn new(
        id: impl Into<String>,
        llm: Arc<LlmClient>,
        shopping_agent_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            llm,
            shopping_agent_id: shopping_agent_id.into(),
            use_heuristic: false,
        }
    }

    /// Create a Secretary that uses only keyword heuristics (no LLM call for
    /// classification, zero token cost, fully deterministic).
    pub fn with_heuristic_only(
        id: impl Into<String>,
        llm: Arc<LlmClient>,
        shopping_agent_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            llm,
            shopping_agent_id: shopping_agent_id.into(),
            use_heuristic: true,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Agent for SecretaryAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Freeco CEO Secretary (OpenClaw)"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::TaskRouting,
            Capability::MemoCalendar,
            Capability::GeneralQa,
            Capability::WellnessCoach,
        ]
    }

    async fn handle(&self, ctx: &AgentContext, msg: Message) -> Result<AgentResponse, AgentError> {
        // Direct shopping queries bypass classification
        if let MessageContent::ShoppingQuery { .. } = &msg.content {
            return Ok(self.route_to_shopping(msg));
        }

        let text = match &msg.content {
            MessageContent::Text(t) => t.clone(),
            other => {
                return Err(AgentError::UnsupportedMessage {
                    agent_id: self.id.clone(),
                    message_type: format!("{other:?}"),
                })
            }
        };

        // Classify intent
        let intent = if self.use_heuristic {
            classify_intent_heuristic(&text)
        } else {
            classify_intent_llm(&text, self.llm.as_ref())
                .await
                .unwrap_or_else(|_| classify_intent_heuristic(&text))
        };

        tracing::info!(
            agent_id = %self.id,
            intent = ?intent,
            "classified user intent"
        );

        match intent {
            Intent::Shopping | Intent::Wellness => {
                // Build a ShoppingQuery for the downstream agent
                let fwd = Message {
                    id: format!("{}-fwd", msg.id),
                    from: self.id.clone(),
                    to: self.shopping_agent_id.clone(),
                    role: MessageRole::Agent,
                    content: MessageContent::ShoppingQuery {
                        query: text,
                        location: ctx.location.clone(),
                        language: Some(ctx.language.clone()),
                    },
                    timestamp_ms: msg.timestamp_ms,
                };
                Ok(AgentResponse::route(
                    &msg.id,
                    &self.id,
                    &self.shopping_agent_id,
                    "shopping / wellness intent",
                    fwd,
                ))
            }
            Intent::Memo | Intent::General => {
                // Handle inline with a brief LLM reply
                let system = if intent == Intent::Memo {
                    "You are Freeco.AI CEO Secretary. Help with memo and calendar requests. Be concise."
                } else {
                    "You are Freeco.AI CEO Secretary for Swiss sustainable living. Answer briefly and helpfully."
                };
                let messages = vec![ChatMessage::system(system), ChatMessage::user(&text)];
                let resp = self
                    .llm
                    .chat(messages, 256)
                    .await
                    .map_err(|e| AgentError::LlmError(e.to_string()))?;
                Ok(AgentResponse::text(&msg.id, &self.id, resp.content))
            }
        }
    }
}

impl SecretaryAgent {
    fn route_to_shopping(&self, msg: Message) -> AgentResponse {
        AgentResponse::route(
            &msg.id,
            &self.id,
            &self.shopping_agent_id,
            "direct shopping query",
            Message {
                id: format!("{}-fwd", msg.id),
                from: self.id.clone(),
                to: self.shopping_agent_id.clone(),
                ..msg
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{Priority, ResponseContent};
    use mockito::Server;

    fn make_secretary_heuristic(shopping_agent_id: &str, llm_url: &str) -> SecretaryAgent {
        let llm = Arc::new(LlmClient::new("test-key").with_base_url(llm_url));
        SecretaryAgent::with_heuristic_only("agent-secretary", llm, shopping_agent_id)
    }

    fn ctx() -> AgentContext {
        AgentContext::new("agent-secretary", "user-1")
            .with_location("Geneva")
            .with_language("en")
    }

    #[tokio::test]
    async fn shopping_text_is_routed_to_shopping_agent() {
        let llm_srv = Server::new_async().await;
        // LLM should not be called (heuristic mode)
        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message::user_text("m-1", "agent-secretary", "buy organic oat milk");
        let resp = secretary.handle(&ctx(), msg).await.unwrap();
        match resp.content {
            ResponseContent::RouteToAgent {
                target_agent_id, ..
            } => {
                assert_eq!(target_agent_id, "agent-shopping");
            }
            _ => panic!("expected RouteToAgent, got {:?}", resp.content),
        }
        // No LLM call should have been made
        drop(llm_srv);
    }

    #[tokio::test]
    async fn direct_shopping_query_is_routed() {
        let llm_srv = Server::new_async().await;
        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message::shopping_query("q-1", "agent-secretary", "vegan cheese", None);
        let resp = secretary.handle(&ctx(), msg).await.unwrap();
        assert!(matches!(resp.content, ResponseContent::RouteToAgent { .. }));
    }

    #[tokio::test]
    async fn general_query_is_answered_inline() {
        let mut llm_srv = Server::new_async().await;
        let _mock = llm_srv
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices":[{"message":{"content":"Freeco is a sustainable concierge."},"finish_reason":"stop"}],"usage":{"total_tokens":20}}"#)
            .create_async()
            .await;

        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message::user_text("m-1", "agent-secretary", "what is Freeco");
        let resp = secretary.handle(&ctx(), msg).await.unwrap();
        match resp.content {
            ResponseContent::Text(t) => assert!(t.contains("Freeco")),
            _ => panic!("expected Text, got {:?}", resp.content),
        }
    }

    #[tokio::test]
    async fn wellness_query_routes_to_shopping_agent() {
        let llm_srv = Server::new_async().await;
        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message::user_text("m-1", "agent-secretary", "best vitamins for energy");
        let resp = secretary.handle(&ctx(), msg).await.unwrap();
        assert!(matches!(resp.content, ResponseContent::RouteToAgent { .. }));
    }

    #[tokio::test]
    async fn unsupported_message_type_returns_error() {
        let llm_srv = Server::new_async().await;
        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message {
            id: "m-1".into(),
            from: "ceo".into(),
            to: "agent-secretary".into(),
            role: MessageRole::Agent,
            content: MessageContent::Directive {
                instruction: "do something".into(),
                priority: Priority::Normal,
            },
            timestamp_ms: 0,
        };
        let err = secretary.handle(&ctx(), msg).await.unwrap_err();
        assert!(matches!(err, AgentError::UnsupportedMessage { .. }));
    }

    #[tokio::test]
    async fn forwarded_shopping_message_preserves_location() {
        let llm_srv = Server::new_async().await;
        let secretary = make_secretary_heuristic("agent-shopping", &llm_srv.url());
        let msg = Message::user_text("m-1", "agent-secretary", "where can I buy oat milk");
        let ctx = AgentContext::new("agent-secretary", "u").with_location("Zurich");
        let resp = secretary.handle(&ctx, msg).await.unwrap();
        if let ResponseContent::RouteToAgent {
            forward_message, ..
        } = resp.content
        {
            if let MessageContent::ShoppingQuery { location, .. } = forward_message.content {
                assert_eq!(location.as_deref(), Some("Zurich"));
            } else {
                panic!("forwarded message should be ShoppingQuery");
            }
        } else {
            panic!("expected RouteToAgent");
        }
    }
}
