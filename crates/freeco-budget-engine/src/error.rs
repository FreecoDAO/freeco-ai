use thiserror::Error;

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("budget exceeded: agent '{agent_id}' for user '{user_id}' has used {used} of {limit} tokens this period")]
    Exceeded {
        agent_id: String,
        user_id: String,
        used: u64,
        limit: u64,
    },

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("user '{0}' not found in budget ledger")]
    UserNotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}
