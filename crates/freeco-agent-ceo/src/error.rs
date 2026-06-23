use thiserror::Error;

/// Errors returned by the CEO agent and Manus client.
#[derive(Debug, Error)]
pub enum CeoError {
    #[error("Manus API error: {0}")]
    ApiError(String),

    #[error("response parse error: {0}")]
    ParseError(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
