//! # budget-engine
//!
//! Rust-native token budget enforcement for OpenFang agents.
//!
//! Rust owns the money. No agent subprocess can bypass or miscount its own
//! budget. Every LLM call is metered here before being forwarded, and every
//! token consumed is recorded in a local SQLite ledger.
//!
//! # Usage
//!
//! ```no_run
//! use budget_engine::{BudgetEngine, SubscriptionTier};
//!
//! let engine = BudgetEngine::open("/var/lib/openfang/budget.db").unwrap();
//!
//! // Check before calling the LLM
//! engine.check_budget("agent-shopping", "user-42").unwrap();
//!
//! // Record after the LLM responds
//! engine.record_tokens("agent-shopping", "user-42", "gemma-4-26b", 1_024).unwrap();
//! ```

pub mod counter;
pub mod enforcer;
pub mod error;
pub mod ledger;
pub mod tier;

pub use counter::TokenCounter;
pub use enforcer::BudgetEnforcer;
pub use error::BudgetError;
pub use ledger::BudgetLedger;
pub use tier::{SubscriptionTier, TokenTopUp, TOPUP_PACKS};

use std::path::Path;

/// Unified entry point that wires together the ledger, counter, and enforcer.
pub struct BudgetEngine {
    ledger: BudgetLedger,
    enforcer: BudgetEnforcer,
}

impl BudgetEngine {
    /// Open (or create) the SQLite budget database at `db_path`.
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self, BudgetError> {
        let ledger = BudgetLedger::open(db_path)?;
        let enforcer = BudgetEnforcer::default();
        Ok(Self { ledger, enforcer })
    }

    /// In-memory engine — useful for tests.
    pub fn in_memory() -> Result<Self, BudgetError> {
        let ledger = BudgetLedger::in_memory()?;
        let enforcer = BudgetEnforcer::default();
        Ok(Self { ledger, enforcer })
    }

    /// Check whether `agent_id` / `user_id` still has budget for the current
    /// billing period.  Returns `Err(BudgetError::Exceeded)` when the limit is
    /// hit.
    pub fn check_budget(&self, agent_id: &str, user_id: &str) -> Result<(), BudgetError> {
        let tier = self.ledger.get_tier(user_id)?;
        let used = self.ledger.tokens_used_this_period(user_id)?;
        self.enforcer.check(agent_id, user_id, used, tier)
    }

    /// Record `tokens` consumed by `agent_id` on behalf of `user_id`.
    pub fn record_tokens(
        &self,
        agent_id: &str,
        user_id: &str,
        model: &str,
        tokens: u64,
    ) -> Result<(), BudgetError> {
        self.ledger.record(agent_id, user_id, model, tokens)
    }

    /// Set (or update) the subscription tier for a user.
    pub fn set_tier(&self, user_id: &str, tier: SubscriptionTier) -> Result<(), BudgetError> {
        self.ledger.set_tier(user_id, tier)
    }

    /// Apply a token top-up purchase to a user's account.
    pub fn apply_topup(&self, user_id: &str, topup: &TokenTopUp) -> Result<(), BudgetError> {
        self.ledger.add_topup_tokens(user_id, topup.tokens)
    }

    /// How many tokens remain for `user_id` this billing period.
    pub fn tokens_remaining(&self, user_id: &str) -> Result<u64, BudgetError> {
        let tier = self.ledger.get_tier(user_id)?;
        let used = self.ledger.tokens_used_this_period(user_id)?;
        let limit = tier.monthly_token_limit();
        let topup = self.ledger.topup_tokens_remaining(user_id)?;
        let total_limit = limit + topup;
        Ok(total_limit.saturating_sub(used))
    }
}
