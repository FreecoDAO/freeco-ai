use thiserror::Error;

/// Errors that the shopping agent and product researcher may return.
#[derive(Debug, Error)]
pub enum ShoppingError {
    #[error("LLM synthesis failed: {0}")]
    LlmError(String),

    #[error("tool call failed: {0}")]
    ToolError(String),

    #[error("no results found for query '{0}'")]
    NoResults(String),
}
