use thiserror::Error;

/// Errors that the Secretary agent may return.
#[derive(Debug, Error)]
pub enum SecretaryError {
    #[error("LLM classification failed: {0}")]
    LlmError(String),

    #[error("routing failed: {0}")]
    RoutingError(String),
}
