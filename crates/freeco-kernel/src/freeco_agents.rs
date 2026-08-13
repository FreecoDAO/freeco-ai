//! Native FreEco edition agents — dispatch for `freeco:*` agent modules.
//!
//! The `freeco-agent-*` crates implement the edition agents (CEO delegation
//! planning, Secretary intent routing, Shopping research) as native Rust
//! `Agent` implementations. This module makes them reachable from agent
//! manifests via the module ids:
//!
//! - `freeco:ceo`       — executive directives → delegation plans (no LLM)
//! - `freeco:secretary` — intent classification and routing
//! - `freeco:shopping`  — three-tier purchase research
//!
//! LLM-backed agents build their client from `FREECO_LLM_API_KEY` /
//! `FREECO_LLM_BASE_URL`, falling back to `NOVITA_API_KEY` (the freeco
//! stack's default provider). When no key is present the Secretary falls
//! back to its deterministic keyword heuristic and the Shopping agent
//! reports a soft error instead of failing the whole request.

use freeco_agent_ceo::CeoAgent;
use freeco_agent_core::{
    Agent, AgentContext, Message, MessageContent, MessageRole, ResponseContent,
};
use freeco_agent_secretary::SecretaryAgent;
use freeco_agent_shopping::ShoppingAgent;
use freeco_tool_gateway::{
    manifest::{ToolName, ToolPermissionManifest},
    LlmClient, ToolGateway,
};
use std::sync::Arc;

/// Agent id the Secretary routes shopping intents to. Matches the bundled
/// `freeco-shopping` edition template name.
const SHOPPING_AGENT_ID: &str = "freeco-shopping";

/// Returns true when the manifest module names a native freeco agent.
pub fn is_freeco_module(module: &str) -> bool {
    module.starts_with("freeco:")
}

/// Result of a native freeco agent invocation.
#[derive(Debug)]
pub struct FreecoAgentOutput {
    /// Final text rendered from the typed agent response.
    pub response: String,
}

fn llm_client() -> (Arc<LlmClient>, bool) {
    let key = std::env::var("FREECO_LLM_API_KEY")
        .or_else(|_| std::env::var("NOVITA_API_KEY"))
        .unwrap_or_default();
    let has_key = !key.is_empty();
    let mut client = LlmClient::new(key);
    if let Ok(url) = std::env::var("FREECO_LLM_BASE_URL") {
        if !url.is_empty() {
            client = client.with_base_url(url);
        }
    }
    (Arc::new(client), has_key)
}

fn tool_gateway() -> Arc<ToolGateway> {
    let manifest = ToolPermissionManifest::new(vec![
        ToolName::TavilySearch,
        ToolName::OpenFoodFacts,
        ToolName::GooglePlaces,
    ]);
    Arc::new(ToolGateway::new(
        manifest,
        std::env::var("TAVILY_API_KEY").unwrap_or_default(),
        std::env::var("GOOGLE_PLACES_API_KEY").unwrap_or_default(),
    ))
}

/// Build the requested native agent. Returns `None` for unknown module ids.
fn build_agent(module: &str, agent_name: &str) -> Option<Arc<dyn Agent>> {
    match module.strip_prefix("freeco:")? {
        "ceo" => Some(Arc::new(CeoAgent::new(agent_name))),
        "secretary" => {
            let (llm, has_key) = llm_client();
            Some(Arc::new(if has_key {
                SecretaryAgent::new(agent_name, llm, SHOPPING_AGENT_ID)
            } else {
                // No key — deterministic keyword routing, zero token cost.
                SecretaryAgent::with_heuristic_only(agent_name, llm, SHOPPING_AGENT_ID)
            }))
        }
        "shopping" => {
            let (llm, _) = llm_client();
            Some(Arc::new(ShoppingAgent::new(
                agent_name,
                tool_gateway(),
                llm,
            )))
        }
        _ => None,
    }
}

/// Render a typed freeco response as dashboard-friendly text.
fn render_response(content: &ResponseContent) -> String {
    match content {
        ResponseContent::Text(t) => t.clone(),
        ResponseContent::SoftError(e) => format!("I hit a problem: {e}"),
        ResponseContent::RouteToAgent {
            target_agent_id,
            reason,
            ..
        } => format!(
            "Routing this to `{target_agent_id}` ({reason}). Send your request to that agent to continue."
        ),
        other => {
            // Typed payloads (e.g. shopping recommendations) render as JSON so
            // no information is lost; the dashboard shows it in a code block.
            serde_json::to_string_pretty(other)
                .unwrap_or_else(|_| format!("{other:?}"))
        }
    }
}

/// Execute one message against a native freeco agent.
pub async fn execute(
    module: &str,
    agent_name: &str,
    user_message: &str,
    session_id: &str,
) -> Result<FreecoAgentOutput, String> {
    let agent = build_agent(module, agent_name)
        .ok_or_else(|| format!("unknown freeco module: {module}"))?;

    let ctx = AgentContext::new(agent_name, "local-user").with_session(session_id);
    let msg = Message {
        id: uuid::Uuid::new_v4().to_string(),
        from: "user".to_string(),
        to: agent_name.to_string(),
        role: MessageRole::User,
        content: MessageContent::Text(user_message.to_string()),
        timestamp_ms: chrono::Utc::now().timestamp_millis(),
    };

    let resp = agent
        .handle(&ctx, msg)
        .await
        .map_err(|e| format!("freeco agent error: {e}"))?;

    Ok(FreecoAgentOutput {
        response: render_response(&resp.content),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_freeco_modules() {
        assert!(is_freeco_module("freeco:ceo"));
        assert!(is_freeco_module("freeco:secretary"));
        assert!(!is_freeco_module("builtin:chat"));
        assert!(!is_freeco_module("wasm:foo.wasm"));
    }

    #[test]
    fn builds_all_known_agents() {
        assert!(build_agent("freeco:ceo", "ceo").is_some());
        assert!(build_agent("freeco:secretary", "sec").is_some());
        assert!(build_agent("freeco:shopping", "shop").is_some());
        assert!(build_agent("freeco:unknown", "x").is_none());
        assert!(build_agent("builtin:chat", "x").is_none());
    }

    #[tokio::test]
    async fn ceo_produces_delegation_plan() {
        let out = execute("freeco:ceo", "ceo", "Launch the spring campaign", "sess-1")
            .await
            .expect("ceo agent should handle a text directive");
        assert!(!out.response.is_empty());
    }

    #[tokio::test]
    async fn unknown_module_errors() {
        let err = execute("freeco:nope", "x", "hi", "s").await.unwrap_err();
        assert!(err.contains("unknown freeco module"));
    }
}
