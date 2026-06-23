use thiserror::Error;

/// Errors that the [`crate::runtime::OpenFangRuntime`] may return.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("agent '{0}' not found in registry")]
    AgentNotFound(String),

    #[error("budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("agent returned an error: {0}")]
    AgentError(String),

    #[error("routing loop detected (exceeded max depth {max_depth})")]
    RoutingLoop { max_depth: usize },

    #[error("runtime internal error: {0}")]
    Internal(String),
}
