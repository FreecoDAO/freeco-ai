use thiserror::Error;

/// Errors that an `Agent::handle` implementation may return.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("budget exceeded for user '{user_id}'")]
    BudgetExceeded { user_id: String },

    #[error("tool '{tool}' not permitted for agent '{agent_id}'")]
    ToolDenied { agent_id: String, tool: String },

    #[error("unsupported message type for agent '{agent_id}': {message_type}")]
    UnsupportedMessage {
        agent_id: String,
        message_type: String,
    },

    #[error("LLM call failed: {0}")]
    LlmError(String),

    #[error("tool call failed: {0}")]
    ToolError(String),

    #[error("internal agent error: {0}")]
    Internal(String),
}
