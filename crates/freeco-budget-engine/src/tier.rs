use serde::{Deserialize, Serialize};

/// Freeco.ai subscription tiers with associated token limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTier {
    /// 3 searches/day, limited tokens, no cart.
    #[default]
    Free,
    /// CHF 11/mo — unlimited searches, full concierge, base token pool.
    Concierge,
    /// CHF 22/mo — everything in Concierge + B2B agents, marketing team.
    Business,
}

impl SubscriptionTier {
    /// Monthly token limit for the subscription (excluding top-up packs).
    pub fn monthly_token_limit(&self) -> u64 {
        match self {
            SubscriptionTier::Free => 50_000,
            SubscriptionTier::Concierge => 2_000_000,
            SubscriptionTier::Business => 6_000_000,
        }
    }

    /// CHF price per month (in centimes, to avoid floats).
    pub fn price_chf_centimes(&self) -> u32 {
        match self {
            SubscriptionTier::Free => 0,
            SubscriptionTier::Concierge => 1_100,
            SubscriptionTier::Business => 2_200,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SubscriptionTier::Free => "Free",
            SubscriptionTier::Concierge => "Concierge",
            SubscriptionTier::Business => "Business",
        }
    }
}

/// A purchasable token top-up pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTopUp {
    /// Display name (e.g. "Starter Pack").
    pub name: &'static str,
    /// Price in CHF centimes.
    pub price_chf_centimes: u32,
    /// Number of tokens included.
    pub tokens: u64,
}

/// All available top-up packs.
pub const TOPUP_PACKS: &[TokenTopUp] = &[
    TokenTopUp {
        name: "Starter Pack",
        price_chf_centimes: 500,
        tokens: 500_000,
    },
    TokenTopUp {
        name: "Standard Pack",
        price_chf_centimes: 1_500,
        tokens: 2_000_000,
    },
    TokenTopUp {
        name: "Power Pack",
        price_chf_centimes: 4_000,
        tokens: 6_000_000,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_limit() {
        assert_eq!(SubscriptionTier::Free.monthly_token_limit(), 50_000);
    }

    #[test]
    fn concierge_tier_price() {
        assert_eq!(SubscriptionTier::Concierge.price_chf_centimes(), 1_100);
    }

    #[test]
    fn business_tier_limit() {
        assert_eq!(SubscriptionTier::Business.monthly_token_limit(), 6_000_000);
    }

    #[test]
    fn topup_packs_are_ordered_by_price() {
        let prices: Vec<u32> = TOPUP_PACKS.iter().map(|p| p.price_chf_centimes).collect();
        let mut sorted = prices.clone();
        sorted.sort_unstable();
        assert_eq!(prices, sorted);
    }

    #[test]
    fn topup_tokens_increase_with_price() {
        let tokens: Vec<u64> = TOPUP_PACKS.iter().map(|p| p.tokens).collect();
        for w in tokens.windows(2) {
            assert!(w[1] > w[0]);
        }
    }
}
