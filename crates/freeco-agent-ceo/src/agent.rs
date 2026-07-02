use std::sync::Arc;

use agent_core::{
    Agent, AgentContext, AgentError, AgentResponse, Capability, Message, MessageContent,
};
use async_trait::async_trait;
use openfang_runtime::kernel_handle::KernelHandle;

use crate::types::Directive;
use agent_core::message::Priority;

/// Freeco.AI CEO Agent — native executive routing via OpenFang kernel handle.
pub struct CeoAgent {
    id: String,
    kernel: Option<Arc<dyn KernelHandle>>,
}

impl CeoAgent {
    /// Create a CEO agent.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kernel: None,
        }
    }

    /// Attach a kernel handle to enable native task posting and discovery.
    pub fn with_kernel_handle(mut self, kernel: Arc<dyn KernelHandle>) -> Self {
        self.kernel = Some(kernel);
        self
    }

    fn infer_delegations(&self, instruction: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let lowered = instruction.to_lowercase();
        if lowered.contains("shop") || lowered.contains("buy") || lowered.contains("product") {
            out.push((
                "shopping".to_string(),
                "Prepare a 3-tier recommendation with sustainability analysis".to_string(),
            ));
        }
        if lowered.contains("route") || lowered.contains("classify") || lowered.contains("intent") {
            out.push((
                "secretary".to_string(),
                "Classify intent and route to the right specialist".to_string(),
            ));
        }
        if out.is_empty() {
            out.push((
                "secretary".to_string(),
                "Triage directive and propose delegation plan".to_string(),
            ));
        }
        out
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Agent for CeoAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Freeco CEO Agent"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::ExecutiveDecision]
    }

    async fn handle(&self, _ctx: &AgentContext, msg: Message) -> Result<AgentResponse, AgentError> {
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

        let delegations = self.infer_delegations(&directive.instruction);
        let mut actions = Vec::new();
        actions.push(format!("Priority set to {:?}", directive.priority));
        actions.push("Compile execution plan and checkpoints".to_string());
        for (target, task) in &delegations {
            actions.push(format!("Delegate to {target}: {task}"));
        }

        if let Some(kernel) = &self.kernel {
            for (target, task) in &delegations {
                let _ = kernel
                    .task_post(
                        &format!("CEO delegation → {target}"),
                        task,
                        Some(target),
                        Some(&self.id),
                    )
                    .await;
            }
        }

        Ok(AgentResponse::executive_decision(
            msg.id,
            &self.id,
            format!("Executive directive accepted: {}", directive.instruction),
            actions,
            delegations,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::ResponseContent;

    #[tokio::test]
    async fn returns_executive_decision_for_directive() {
        let agent = CeoAgent::new("agent-ceo");
        let ctx = AgentContext::new("agent-ceo", "user-1");
        let msg = Message::directive(
            "d-1",
            "agent-ceo",
            "launch new product line",
            Priority::High,
        );
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
        let agent = CeoAgent::new("agent-ceo");
        let ctx = AgentContext::new("agent-ceo", "u");
        let msg = Message::user_text("m-1", "agent-ceo", "review Q3 budget");
        let resp = agent.handle(&ctx, msg).await.unwrap();
        assert!(matches!(
            resp.content,
            ResponseContent::ExecutiveDecision { .. }
        ));
    }

    #[tokio::test]
    async fn unsupported_message_returns_error() {
        let agent = CeoAgent::new("agent-ceo");
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
