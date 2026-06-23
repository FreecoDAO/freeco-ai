use thiserror::Error;

/// Errors returned by the CEO agent.
#[derive(Debug, Error)]
pub enum CeoError {
    #[error("kernel operation failed: {0}")]
    Kernel(String),
}
