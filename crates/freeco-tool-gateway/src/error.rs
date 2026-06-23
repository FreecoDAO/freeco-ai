use thiserror::Error;

use crate::manifest::ToolName;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("tool '{tool}' is not in the agent's permission manifest")]
    ToolDenied { tool: ToolName },

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API returned an error: {status} — {message}")]
    ApiError { status: u16, message: String },

    #[error("failed to deserialize API response: {0}")]
    Deserialize(String),

    #[error("rate limited by upstream API, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
