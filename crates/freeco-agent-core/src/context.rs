use serde::{Deserialize, Serialize};

/// Per-request context passed to every `Agent::handle()` invocation.
///
/// This struct is intentionally WASM-safe: no OS handles, no raw pointers,
/// no `Send + Sync` requirements.  Budget enforcement happens in the **native
/// runtime** *before* calling `handle()`, and the serialised state is
/// forwarded here for informational use by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// The agent being invoked (matches `Agent::id()`).
    pub agent_id: String,

    /// The end-user on whose behalf this call is made.
    pub user_id: String,

    /// Opaque session token linking requests in a conversation.
    pub session_id: String,

    /// Tokens remaining for this user in the current billing period.
    /// Agents should use this to decide how verbose to be.
    pub tokens_remaining: u64,

    /// Subscription tier display name: `"Free"`, `"Concierge"`, `"Business"`.
    pub tier: String,

    /// User's location (city / country) for geo-aware recommendations.
    pub location: Option<String>,

    /// BCP-47 language tag of the user's preferred language (e.g. `"en"`, `"fr"`).
    pub language: String,
}

impl AgentContext {
    /// Create a minimal context for the given agent / user pair.
    pub fn new(agent_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            user_id: user_id.into(),
            session_id: String::new(),
            tokens_remaining: u64::MAX,
            tier: "Free".into(),
            location: None,
            language: "en".into(),
        }
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = session_id.into();
        self
    }

    pub fn with_tokens(mut self, remaining: u64, tier: impl Into<String>) -> Self {
        self.tokens_remaining = remaining;
        self.tier = tier.into();
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context_has_max_tokens() {
        let ctx = AgentContext::new("agent-shopping", "user-1");
        assert_eq!(ctx.tokens_remaining, u64::MAX);
        assert_eq!(ctx.tier, "Free");
        assert_eq!(ctx.language, "en");
        assert!(ctx.location.is_none());
    }

    #[test]
    fn builder_methods_set_fields() {
        let ctx = AgentContext::new("ag", "u1")
            .with_session("sess-42")
            .with_tokens(500_000, "Concierge")
            .with_location("Geneva, Switzerland")
            .with_language("fr");

        assert_eq!(ctx.session_id, "sess-42");
        assert_eq!(ctx.tokens_remaining, 500_000);
        assert_eq!(ctx.tier, "Concierge");
        assert_eq!(ctx.location.as_deref(), Some("Geneva, Switzerland"));
        assert_eq!(ctx.language, "fr");
    }

    #[test]
    fn context_roundtrips_through_json() {
        let ctx = AgentContext::new("ag", "u").with_location("Zurich");
        let json = serde_json::to_string(&ctx).unwrap();
        let back: AgentContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.location.as_deref(), Some("Zurich"));
    }
}
