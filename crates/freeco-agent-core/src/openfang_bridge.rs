use crate::{
    Agent, AgentContext, AgentError, AgentResponse, Message, MessageContent, MessageRole,
    ResponseContent,
};

/// Convert an OpenFang message into a Freeco agent-core message.
pub fn from_openfang_message(msg: &openfang_types::message::Message, to: impl Into<String>) -> Message {
    let role = match msg.role {
        openfang_types::message::Role::System => MessageRole::System,
        openfang_types::message::Role::User => MessageRole::User,
        openfang_types::message::Role::Assistant => MessageRole::Agent,
    };
    Message {
        id: msg.msg_id.clone(),
        from: format!("{:?}", msg.role).to_lowercase(),
        to: to.into(),
        role,
        content: MessageContent::Text(msg.content.text_content()),
        timestamp_ms: crate::time_util::now_ms(),
    }
}

/// Convert a Freeco agent-core response into an OpenFang assistant message.
pub fn to_openfang_message(resp: &AgentResponse) -> openfang_types::message::Message {
    let text = match &resp.content {
        ResponseContent::Text(t) => t.clone(),
        ResponseContent::SoftError(e) => format!("error: {e}"),
        ResponseContent::ExecutiveDecision { summary, actions, .. } => {
            format!("{summary}\n\n{}", actions.join("\n"))
        }
        other => serde_json::to_string(other).unwrap_or_else(|_| "unsupported response".to_string()),
    };
    openfang_types::message::Message::assistant(text)
}

/// Adapter trait that lets Freeco agents run as first-class OpenFang message handlers.
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait OpenFangAgentBridge: Agent {
    async fn handle_openfang(
        &self,
        ctx: &AgentContext,
        msg: &openfang_types::message::Message,
    ) -> Result<openfang_types::message::Message, AgentError> {
        let converted = from_openfang_message(msg, ctx.agent_id.clone());
        let out = self.handle(ctx, converted).await?;
        Ok(to_openfang_message(&out))
    }
}

impl<T: Agent + ?Sized> OpenFangAgentBridge for T {}
