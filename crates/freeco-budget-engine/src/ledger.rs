use std::path::Path;

use rusqlite::{params, Connection};

use crate::{error::BudgetError, tier::SubscriptionTier};

/// SQLite-backed persistent token ledger.
///
/// Stores per-user subscription tiers, monthly usage totals, and top-up
/// balances. The database is created automatically on first open.
pub struct BudgetLedger {
    conn: Connection,
}

impl BudgetLedger {
    /// Open (or create) the database at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, BudgetError> {
        let conn = Connection::open(path)?;
        let ledger = Self { conn };
        ledger.migrate()?;
        Ok(ledger)
    }

    /// Create an in-memory database (useful for tests).
    pub fn in_memory() -> Result<Self, BudgetError> {
        let conn = Connection::open_in_memory()?;
        let ledger = Self { conn };
        ledger.migrate()?;
        Ok(ledger)
    }

    /// Run schema migrations (idempotent).
    fn migrate(&self) -> Result<(), BudgetError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS user_tiers (
                user_id     TEXT PRIMARY KEY,
                tier        TEXT NOT NULL DEFAULT 'free',
                topup_tokens INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );

            CREATE TABLE IF NOT EXISTS token_usage (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id    TEXT NOT NULL,
                user_id     TEXT NOT NULL,
                model       TEXT NOT NULL,
                tokens      INTEGER NOT NULL,
                recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
                period      TEXT NOT NULL DEFAULT (strftime('%Y-%m', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_usage_user_period
                ON token_usage(user_id, period);
            ",
        )?;
        Ok(())
    }

    /// Ensure a user row exists, creating it with Free tier if absent.
    fn ensure_user(&self, user_id: &str) -> Result<(), BudgetError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO user_tiers (user_id, tier) VALUES (?1, 'free')",
            params![user_id],
        )?;
        Ok(())
    }

    /// Get the subscription tier for a user. Defaults to Free if not found.
    pub fn get_tier(&self, user_id: &str) -> Result<SubscriptionTier, BudgetError> {
        self.ensure_user(user_id)?;
        let tier_str: String = self.conn.query_row(
            "SELECT tier FROM user_tiers WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(parse_tier(&tier_str))
    }

    /// Set (or update) the subscription tier for a user.
    pub fn set_tier(&self, user_id: &str, tier: SubscriptionTier) -> Result<(), BudgetError> {
        self.ensure_user(user_id)?;
        self.conn.execute(
            "UPDATE user_tiers SET tier = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE user_id = ?2",
            params![tier_to_str(&tier), user_id],
        )?;
        Ok(())
    }

    /// Add top-up tokens to a user's balance.
    pub fn add_topup_tokens(&self, user_id: &str, tokens: u64) -> Result<(), BudgetError> {
        self.ensure_user(user_id)?;
        self.conn.execute(
            "UPDATE user_tiers SET topup_tokens = topup_tokens + ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE user_id = ?2",
            params![tokens as i64, user_id],
        )?;
        Ok(())
    }

    /// Get remaining top-up tokens for a user.
    pub fn topup_tokens_remaining(&self, user_id: &str) -> Result<u64, BudgetError> {
        self.ensure_user(user_id)?;
        let topup: i64 = self.conn.query_row(
            "SELECT topup_tokens FROM user_tiers WHERE user_id = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        Ok(topup.max(0) as u64)
    }

    /// Record token usage for an agent run.
    pub fn record(
        &self,
        agent_id: &str,
        user_id: &str,
        model: &str,
        tokens: u64,
    ) -> Result<(), BudgetError> {
        if tokens == 0 {
            return Ok(());
        }
        self.ensure_user(user_id)?;
        self.conn.execute(
            "INSERT INTO token_usage (agent_id, user_id, model, tokens) VALUES (?1, ?2, ?3, ?4)",
            params![agent_id, user_id, model, tokens as i64],
        )?;
        Ok(())
    }

    /// Total tokens consumed by a user in the current calendar month.
    pub fn tokens_used_this_period(&self, user_id: &str) -> Result<u64, BudgetError> {
        let period = current_period();
        let total: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(tokens), 0) FROM token_usage WHERE user_id = ?1 AND period = ?2",
                params![user_id, period],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(total.max(0) as u64)
    }
}

fn tier_to_str(tier: &SubscriptionTier) -> &'static str {
    match tier {
        SubscriptionTier::Free => "free",
        SubscriptionTier::Concierge => "concierge",
        SubscriptionTier::Business => "business",
    }
}

fn parse_tier(s: &str) -> SubscriptionTier {
    match s {
        "concierge" => SubscriptionTier::Concierge,
        "business" => SubscriptionTier::Business,
        _ => SubscriptionTier::Free,
    }
}

fn current_period() -> String {
    // Returns "YYYY-MM" matching the SQLite DEFAULT expression
    let now = chrono::Utc::now();
    format!("{}", now.format("%Y-%m"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> BudgetLedger {
        BudgetLedger::in_memory().unwrap()
    }

    #[test]
    fn new_user_defaults_to_free() {
        let l = fresh();
        let tier = l.get_tier("user-new").unwrap();
        assert_eq!(tier, SubscriptionTier::Free);
    }

    #[test]
    fn set_and_get_tier() {
        let l = fresh();
        l.set_tier("user-1", SubscriptionTier::Concierge).unwrap();
        assert_eq!(l.get_tier("user-1").unwrap(), SubscriptionTier::Concierge);
    }

    #[test]
    fn record_and_sum_tokens() {
        let l = fresh();
        l.record("agent-a", "user-1", "gemma-4", 1_000).unwrap();
        l.record("agent-a", "user-1", "gemma-4", 500).unwrap();
        l.record("agent-b", "user-1", "gemma-4", 250).unwrap();
        assert_eq!(l.tokens_used_this_period("user-1").unwrap(), 1_750);
    }

    #[test]
    fn tokens_isolated_per_user() {
        let l = fresh();
        l.record("agent-a", "user-1", "gemma-4", 5_000).unwrap();
        l.record("agent-a", "user-2", "gemma-4", 1_000).unwrap();
        assert_eq!(l.tokens_used_this_period("user-1").unwrap(), 5_000);
        assert_eq!(l.tokens_used_this_period("user-2").unwrap(), 1_000);
    }

    #[test]
    fn zero_tokens_not_recorded() {
        let l = fresh();
        l.record("agent-a", "user-1", "gemma-4", 0).unwrap();
        assert_eq!(l.tokens_used_this_period("user-1").unwrap(), 0);
    }

    #[test]
    fn topup_adds_and_remains() {
        let l = fresh();
        l.add_topup_tokens("user-1", 500_000).unwrap();
        assert_eq!(l.topup_tokens_remaining("user-1").unwrap(), 500_000);
        l.add_topup_tokens("user-1", 2_000_000).unwrap();
        assert_eq!(l.topup_tokens_remaining("user-1").unwrap(), 2_500_000);
    }

    #[test]
    fn new_user_has_zero_topup() {
        let l = fresh();
        assert_eq!(l.topup_tokens_remaining("brand-new-user").unwrap(), 0);
    }

    #[test]
    fn fresh_user_has_zero_usage() {
        let l = fresh();
        assert_eq!(l.tokens_used_this_period("unused-user").unwrap(), 0);
    }
}
