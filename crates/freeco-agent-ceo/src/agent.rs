use std::sync::Arc;

use agent_core::{
    Agent, AgentContext, AgentError, AgentResponse, Capability, Message,
    MessageContent,
};
use async_trait::async_trait;

use crate::manus_client::ManusClient;
use crate::types::Directive;
use agent_core::message::Priority;

/// Freeco.AI CEO Agent — executive orchestration via Manus AI.
///
/// Accepts [`MessageContent::Directive`] and plain [`MessageContent::Text`].
/// Returns an `ExecutiveDecision` response with a summary, reasoning, and
/// prioritised action items (some of which may be delegated to other agents).
pub struct CeoAgent {
    id: String,
    manus: Arc<ManusClient>,
}

impl CeoAgent {
    /// Create a CEO agent.
    ///
    /// Pass `None` for `manus_api_key` to run in stub mode (no API call).
    pub fn new(id: impl Into<String>, manus_api_key: Option<String>) -> Self {
        Self {
            id: id.into(),
            manus: Arc::new(ManusClient::new(manus_api_key)),
        }
    }

    /// Override the Manus API base URL — used in tests.
    pub fn with_manus_base_url(mut self, url: impl Into<String>) -> Self {
        self.manus = Arc::new(
            ManusClient::new(Some("test-key".into())).with_base_url(url),
        );
        self
    }
}


#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Agent for CeoAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Freeco CEO Agent (Manus AI)"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::ExecutiveDecision]
    }

    async fn handle(
        &self,
        _ctx: &AgentContext,
        msg: Message,
    ) -> Result<AgentResponse, AgentError> {
        let directive = match &msg.content {
            MessageContent::Directive {
                instruction,
                priority,
            } => Directive {
                instruction: instruction.clone(),
                priority: priority.clone(),
                context: None,
            },
            MessageContent::Text(text) => Directive {
                instruction: text.clone(),
                priority: Priority::Normal,
                context: None,
            },
            other => {
                return Err(AgentError::UnsupportedMessage {
                    agent_id: self.id.clone(),
                    message_type: format!("{other:?}"),
                })
            }
        };

        tracing::info!(
            agent_id = %self.id,
            instruction = %directive.instruction,
            priority = ?directive.priority,
            "CEO directive received"
        );

        let decision = self
            .manus
            .submit_directive(&directive)
            .await
            .map_err(|e| AgentError::Internal(e.to_string()))?;

        let actions: Vec<String> = decision.actions.iter().map(|a| a.description.clone()).collect();
        let delegations: Vec<(String, String)> = decision
            .actions
            .iter()
            .filter_map(|a| {
                a.delegate_to
                    .as_ref()
                    .map(|d| (d.clone(), a.description.clone()))
            })
            .collect();

        Ok(AgentResponse::executive_decision(
            msg.id,
            &self.id,
            decision.summary,
            actions,
            delegations,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::ResponseContent;
    use mockito::Server;

    #[tokio::test]
    async fn stub_mode_returns_executive_decision() {
        let agent = CeoAgent::new("agent-ceo", None);
        let ctx = AgentContext::new("agent-ceo", "user-1");
        let msg = Message::directive("d-1", "agent-ceo", "launch new product line", Priority::High);
        let resp = agent.handle(&ctx, msg).await.unwrap();

        assert_eq!(resp.from_agent, "agent-ceo");
        assert!(matches!(
            resp.content,
            ResponseContent::ExecutiveDecision { .. }
        ));
        if let ResponseContent::ExecutiveDecision { summary, .. } = resp.content {
            assert!(summary.contains("launch new product line"));
        }
    }

    #[tokio::test]
    async fn plain_text_treated_as_normal_priority_directive() {
        let agent = CeoAgent::new("agent-ceo", None);
        let ctx = AgentContext::new("agent-ceo", "u");
        let msg = Message::user_text("m-1", "agent-ceo", "review Q3 budget");
        let resp = agent.handle(&ctx, msg).await.unwrap();
        assert!(matches!(resp.content, ResponseContent::ExecutiveDecision { .. }));
    }

    #[tokio::test]
    async fn live_mode_with_mock_api() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({
                "output": {
                    "summary": "Product line approved",
                    "reasoning": "Strong market demand",
                    "actions": [
                        {"description": "Brief marketing team", "delegate_to": null}
                    ]
                }
            }).to_string())
            .create_async()
            .await;

        let agent = CeoAgent::new("agent-ceo", None)
            .with_manus_base_url(server.url());

        let ctx = AgentContext::new("agent-ceo", "u");
        let msg = Message::directive("d-1", "agent-ceo", "approve product launch", Priority::Normal);
        let resp = agent.handle(&ctx, msg).await.unwrap();
        if let ResponseContent::ExecutiveDecision { summary, actions, .. } = resp.content {
            assert_eq!(summary, "Product line approved");
            assert_eq!(actions.len(), 1);
        } else {
            panic!("expected ExecutiveDecision");
        }
    }

    #[tokio::test]
    async fn unsupported_message_returns_error() {
        let agent = CeoAgent::new("agent-ceo", None);
        let ctx = AgentContext::new("agent-ceo", "u");
        let msg = Message {
            id: "m".into(),
            from: "user".into(),
            to: "agent-ceo".into(),
            role: agent_core::MessageRole::User,
            content: MessageContent::ShoppingQuery {
                query: "buy milk".into(),
                location: None,
                language: None,
            },
            timestamp_ms: 0,
        };
        let err = agent.handle(&ctx, msg).await.unwrap_err();
        assert!(matches!(err, AgentError::UnsupportedMessage { .. }));
    }
}
