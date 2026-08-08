//! Where a user actually gets an API key for each provider.
//!
//! The setup wizard lists every provider the runtime knows about — 42 of them
//! — but only had signup links for 13, hardcoded in the dashboard JavaScript.
//! Clicking any of the other 29 gave the user a card, a box demanding an API
//! key, and no indication of where such a key comes from. Someone who has
//! never used a model provider cannot recover from that, and there is nothing
//! in the interface that would teach them.
//!
//! Keeping the table here rather than in the front-end means the list lives
//! next to the provider registry it describes, and one answer serves the
//! wizard, the settings page, and anything added later.

/// A provider's cost model, as it matters to someone deciding what to click.
///
/// This is deliberately coarse. The point is not to quote prices — those
/// change and would go stale here — but to answer the one question a newcomer
/// has before signing up: will this ask me for a credit card?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cost {
    /// Usable with no payment method at all.
    Free,
    /// Has a genuine free tier; no card required to start.
    FreeTier,
    /// Requires credit or a card before the first call.
    Paid,
    /// Runs on the user's own machine. No account, no key.
    Local,
    /// Authenticates through a CLI or OAuth rather than a pasted key.
    NoKey,
}

impl Cost {
    pub fn label(self) -> &'static str {
        match self {
            Cost::Free => "Free",
            Cost::FreeTier => "Free tier",
            Cost::Paid => "Paid",
            Cost::Local => "Runs locally",
            Cost::NoKey => "No API key",
        }
    }
}

/// Signup page and one line of instruction for a provider.
pub struct Signup {
    pub url: &'static str,
    pub hint: &'static str,
    pub cost: Cost,
}

/// The recommended starting point for someone with no accounts anywhere.
///
/// OpenRouter is the only entry that is free, needs no card, and reaches
/// models from every other vendor with a single key — which is what makes it
/// the honest answer to "what should I pick first".
pub const RECOMMENDED_PROVIDER: &str = "openrouter";

/// Look up where to get a key for `provider_id`.
pub fn signup_for(provider_id: &str) -> Option<Signup> {
    let (url, hint, cost) = match provider_id {
        // ── The recommended free path ──
        "openrouter" => (
            "https://openrouter.ai/settings/keys",
            "Free, no credit card. Sign in with Google or GitHub, press Create Key, paste it here.",
            Cost::Free,
        ),

        // ── Free tiers: usable without paying ──
        "groq" => (
            "https://console.groq.com/keys",
            "Free tier with generous limits. Sign up, open API Keys, create one.",
            Cost::FreeTier,
        ),
        "gemini" => (
            "https://aistudio.google.com/apikey",
            "Free tier. Sign in with a Google account and press Create API key.",
            Cost::FreeTier,
        ),
        "cerebras" => (
            "https://cloud.cerebras.ai/platform/apikeys",
            "Free tier. Sign up, then create a key under API Keys.",
            Cost::FreeTier,
        ),
        "sambanova" => (
            "https://cloud.sambanova.ai/apis",
            "Free tier available. Sign up and generate a key.",
            Cost::FreeTier,
        ),
        "huggingface" => (
            "https://huggingface.co/settings/tokens",
            "Free account. Create an access token with Read permission.",
            Cost::FreeTier,
        ),
        "chutes" => (
            "https://chutes.ai/app/api",
            "Free tier available. Sign up and create an API key.",
            Cost::FreeTier,
        ),
        "nvidia" => (
            "https://build.nvidia.com/explore/discover",
            "Free credits on signup. Pick a model and press Get API Key.",
            Cost::FreeTier,
        ),
        "zhipu" | "zhipu_coding" => (
            "https://open.bigmodel.cn/usercenter/apikeys",
            "Free tier available. Register, then copy the key from API Keys.",
            Cost::FreeTier,
        ),
        "zai" | "zai_coding" => (
            "https://z.ai/manage-apikey/apikey-list",
            "Free tier available. Register, then create an API key.",
            Cost::FreeTier,
        ),
        "qwen" => (
            "https://bailian.console.alibabacloud.com/?tab=model#/api-key",
            "Free quota on signup. Create a key under API-KEY.",
            Cost::FreeTier,
        ),
        "minimax" => (
            "https://platform.minimax.io/user-center/basic-information/interface-key",
            "Free credits on signup. Copy the key from Interface Key.",
            Cost::FreeTier,
        ),
        "moonshot" | "kimi_coding" => (
            "https://platform.moonshot.ai/console/api-keys",
            "Free credits on signup. Create a key in the console.",
            Cost::FreeTier,
        ),
        "ai21" => (
            "https://studio.ai21.com/account/api-key",
            "Free trial credits. Sign up and copy the key from your account.",
            Cost::FreeTier,
        ),
        "venice" => (
            "https://venice.ai/settings/api",
            "Free tier available. Create a key in Settings, API.",
            Cost::FreeTier,
        ),
        "requesty" => (
            "https://app.requesty.ai/api-keys",
            "Free credits on signup. Create a key under API Keys.",
            Cost::FreeTier,
        ),

        // ── Paid: a card is needed before the first call ──
        "anthropic" => (
            "https://console.anthropic.com/settings/keys",
            "Requires credit. Add billing, then create a key in the Console.",
            Cost::Paid,
        ),
        "openai" => (
            "https://platform.openai.com/api-keys",
            "Requires credit. Add billing, then create a secret key.",
            Cost::Paid,
        ),
        "deepseek" => (
            "https://platform.deepseek.com/api_keys",
            "Paid, but very cheap. Top up a small amount, then create a key.",
            Cost::Paid,
        ),
        "mistral" => (
            "https://console.mistral.ai/api-keys",
            "Create a key in the Mistral Console (billing required).",
            Cost::Paid,
        ),
        "together" => (
            "https://api.together.xyz/settings/api-keys",
            "Create a key in Together AI settings.",
            Cost::Paid,
        ),
        "fireworks" => (
            "https://fireworks.ai/account/api-keys",
            "Create a key under Account, API Keys.",
            Cost::Paid,
        ),
        "perplexity" => (
            "https://www.perplexity.ai/settings/api",
            "Requires credit. Generate a key in Settings, API.",
            Cost::Paid,
        ),
        "cohere" => (
            "https://dashboard.cohere.com/api-keys",
            "Create a key in the Cohere dashboard.",
            Cost::Paid,
        ),
        "xai" => (
            "https://console.x.ai/",
            "Requires credit. Create a key in the xAI Console.",
            Cost::Paid,
        ),
        "replicate" => (
            "https://replicate.com/account/api-tokens",
            "Requires billing. Create an API token in your account.",
            Cost::Paid,
        ),
        "qianfan" => (
            "https://console.bce.baidu.com/iam/#/iam/apikey",
            "Create an API key in the Baidu Cloud console.",
            Cost::Paid,
        ),
        "volcengine" | "volcengine_coding" => (
            "https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey",
            "Create an API key in the Volcengine Ark console.",
            Cost::Paid,
        ),
        "bedrock" => (
            "https://console.aws.amazon.com/iam/home#/security_credentials",
            "Uses AWS credentials. Create an access key with Bedrock permissions.",
            Cost::Paid,
        ),
        "azure" => (
            "https://portal.azure.com/",
            "Create an Azure OpenAI resource, then copy Key 1 from Keys and Endpoint.",
            Cost::Paid,
        ),

        // ── Local: no account, no key ──
        "ollama" => (
            "https://ollama.com/download",
            "Runs on your machine. Install Ollama and it is detected automatically.",
            Cost::Local,
        ),
        "lmstudio" => (
            "https://lmstudio.ai/",
            "Runs on your machine. Install LM Studio and start its local server.",
            Cost::Local,
        ),
        "vllm" => (
            "https://docs.vllm.ai/en/latest/getting_started/installation.html",
            "Self-hosted. Point the base URL at your own vLLM server.",
            Cost::Local,
        ),
        "lemonade" => (
            "https://lemonade-server.ai/",
            "Runs on your machine. Install Lemonade Server and start it.",
            Cost::Local,
        ),

        // ── Authenticated through a CLI or OAuth, not a pasted key ──
        "claude-code" => (
            "https://docs.anthropic.com/en/docs/claude-code",
            "No API key. Run: npm install -g @anthropic-ai/claude-code, then claude auth.",
            Cost::NoKey,
        ),
        "codex" => (
            "https://developers.openai.com/codex/cli",
            "No API key. Install the Codex CLI and sign in with your ChatGPT account.",
            Cost::NoKey,
        ),
        "qwen-code" => (
            "https://github.com/QwenLM/qwen-code",
            "No API key. Install the Qwen Code CLI and sign in.",
            Cost::NoKey,
        ),
        "github-copilot" => (
            "https://github.com/settings/copilot",
            "No API key. Press Connect to sign in with GitHub — a Copilot subscription is required.",
            Cost::NoKey,
        ),

        _ => return None,
    };
    Some(Signup { url, hint, cost })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_provider_is_free_and_has_a_link() {
        let s = signup_for(RECOMMENDED_PROVIDER).expect("recommended provider must be covered");
        assert_eq!(s.cost, Cost::Free);
        assert!(s.url.starts_with("https://"));
    }

    /// The whole point of this module: a user who clicks any provider card must
    /// be told where the key comes from. Every provider the runtime ships must
    /// therefore be covered, and this fails if someone adds a provider without
    /// adding its signup page.
    #[test]
    fn every_shipped_provider_has_signup_guidance() {
        let catalog = openfang_runtime::model_catalog::ModelCatalog::new();
        let missing: Vec<&str> = catalog
            .list_providers()
            .iter()
            .filter(|p| signup_for(&p.id).is_none())
            .map(|p| p.id.as_str())
            .collect();
        assert!(
            missing.is_empty(),
            "providers with no signup guidance: {:?}",
            missing
        );
    }

    #[test]
    fn hints_are_actually_instructive() {
        let catalog = openfang_runtime::model_catalog::ModelCatalog::new();
        for p in catalog.list_providers() {
            if let Some(s) = signup_for(&p.id) {
                assert!(
                    s.hint.len() > 25,
                    "hint for {} is too short to help anyone: {:?}",
                    p.id,
                    s.hint
                );
                assert!(
                    s.url.starts_with("https://"),
                    "signup url for {} is not https: {}",
                    p.id,
                    s.url
                );
            }
        }
    }

    #[test]
    fn unknown_provider_has_no_guidance() {
        assert!(signup_for("not-a-real-provider").is_none());
    }
}
