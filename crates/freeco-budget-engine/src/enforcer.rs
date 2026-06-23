use crate::{error::BudgetError, tier::SubscriptionTier};

/// Checks token usage against a user's subscription tier limits.
/// Stateless — all state lives in [`crate::ledger::BudgetLedger`].
#[derive(Debug, Default)]
pub struct BudgetEnforcer;

impl BudgetEnforcer {
    pub fn check(
        &self,
        agent_id: &str,
        user_id: &str,
        tokens_used: u64,
        tier: SubscriptionTier,
    ) -> Result<(), BudgetError> {
        let limit = tier.monthly_token_limit();
        if tokens_used >= limit {
            return Err(BudgetError::Exceeded {
                agent_id: agent_id.to_string(),
                user_id: user_id.to_string(),
                used: tokens_used,
                limit,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_limit_is_ok() {
        let e = BudgetEnforcer::default();
        assert!(e
            .check("agent-1", "user-1", 1_000, SubscriptionTier::Concierge)
            .is_ok());
    }

    #[test]
    fn exactly_at_limit_is_exceeded() {
        let e = BudgetEnforcer::default();
        let limit = SubscriptionTier::Free.monthly_token_limit();
        let err = e
            .check("agent-1", "user-1", limit, SubscriptionTier::Free)
            .unwrap_err();
        assert!(matches!(err, BudgetError::Exceeded { .. }));
    }

    #[test]
    fn over_limit_is_exceeded() {
        let e = BudgetEnforcer::default();
        let err = e
            .check("agent-x", "user-x", 999_999_999, SubscriptionTier::Business)
            .unwrap_err();
        assert!(matches!(
            err,
            BudgetError::Exceeded { used: 999_999_999, .. }
        ));
    }

    #[test]
    fn zero_tokens_is_ok() {
        let e = BudgetEnforcer::default();
        assert!(e
            .check("agent-1", "user-1", 0, SubscriptionTier::Free)
            .is_ok());
    }

    #[test]
    fn exceeded_error_contains_ids() {
        let e = BudgetEnforcer::default();
        let err = e
            .check("my-agent", "my-user", 100_000, SubscriptionTier::Free)
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("my-agent"));
        assert!(msg.contains("my-user"));
    }
}
