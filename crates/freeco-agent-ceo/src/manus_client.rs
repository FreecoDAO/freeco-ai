use reqwest::Client;
use serde::{Deserialize, Serialize};

use agent_core::message::Priority;

use crate::error::CeoError;
use crate::types::{Action, CeoDecision, Directive};

/// HTTP client bridging to the Manus AI MCP API.
///
/// When no API key is configured, [`submit_directive`] returns a deterministic
/// stub so the rest of the system works end-to-end without a live Manus account.
pub struct ManusClient {
    api_key: Option<String>,
    http: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct ManusRequest<'a> {
    tool: &'static str,
    input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct ManusResponse {
    output: Option<serde_json::Value>,
    error: Option<String>,
}

impl ManusClient {
    /// Create a client.  Pass `None` for `api_key` to use stub mode.
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            http: Client::new(),
            base_url: "https://api.manus.im/v1/mcp".into(),
        }
    }

    /// Override the base URL — used in tests.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Whether this client is configured with a real API key.
    pub fn is_live(&self) -> bool {
        self.api_key.is_some()
    }

    /// Submit a directive and return a [`CeoDecision`].
    ///
    /// Falls back to [`stub_decision`] when no API key is present.
    pub async fn submit_directive(&self, directive: &Directive) -> Result<CeoDecision, CeoError> {
        match &self.api_key {
            Some(key) => self.call_manus_api(key.clone(), directive).await,
            None => {
                tracing::info!("Manus API key not set — returning stub CEO decision");
                Ok(self.stub_decision(directive))
            }
        }
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    async fn call_manus_api(&self, key: String, directive: &Directive) -> Result<CeoDecision, CeoError> {
        let req = ManusRequest {
            tool: "ceo_directive",
            input: serde_json::json!({
                "instruction": directive.instruction,
                "priority":    directive.priority,
                "context":     directive.context,
            }),
            session_id: None,
        };

        let resp = self
            .http
            .post(&self.base_url)
            .bearer_auth(&key)
            .json(&req)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CeoError::ApiError(format!("HTTP {status}: {body}")));
        }

        let raw: ManusResponse = resp
            .json()
            .await
            .map_err(|e| CeoError::ParseError(e.to_string()))?;

        if let Some(err) = raw.error {
            return Err(CeoError::ApiError(err));
        }

        let output = raw.output.unwrap_or_default();
        let summary = output["summary"]
            .as_str()
            .unwrap_or("Decision made by Manus AI")
            .to_string();
        let reasoning = output["reasoning"].as_str().unwrap_or("").to_string();
        let actions = output["actions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|a| Action {
                        description: a["description"].as_str().unwrap_or("").to_string(),
                        delegate_to: a["delegate_to"].as_str().map(str::to_string),
                        priority: Priority::Normal,
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CeoDecision {
            summary,
            reasoning,
            actions,
        })
    }

    /// Deterministic stub — no API call, no key required.
    fn stub_decision(&self, directive: &Directive) -> CeoDecision {
        CeoDecision {
            summary: format!("Acknowledged: {}", directive.instruction),
            reasoning: "Manus AI API key not configured — stub response generated locally.".into(),
            actions: vec![
                Action {
                    description: format!(
                        "Proceed with: {}",
                        directive.instruction
                    ),
                    delegate_to: None,
                    priority: directive.priority.clone(),
                },
                Action {
                    description: "Configure Manus AI API key to enable live CEO decisions."
                        .into(),
                    delegate_to: None,
                    priority: Priority::Low,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn directive(instruction: &str) -> Directive {
        Directive {
            instruction: instruction.into(),
            priority: Priority::Normal,
            context: None,
        }
    }

    #[tokio::test]
    async fn stub_mode_returns_acknowledged_summary() {
        let client = ManusClient::new(None);
        assert!(!client.is_live());
        let dec = client.submit_directive(&directive("expand to Paris")).await.unwrap();
        assert!(dec.summary.contains("expand to Paris"));
        assert!(!dec.actions.is_empty());
    }

    #[tokio::test]
    async fn live_mode_parses_api_response() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({
                "output": {
                    "summary": "Approved expansion plan",
                    "reasoning": "Market conditions are favourable",
                    "actions": [
                        {"description": "Hire Paris team lead", "delegate_to": "agent-hr"}
                    ]
                }
            }).to_string())
            .create_async()
            .await;

        let client = ManusClient::new(Some("test-key".into())).with_base_url(server.url());
        assert!(client.is_live());
        let dec = client.submit_directive(&directive("expand to Paris")).await.unwrap();
        assert_eq!(dec.summary, "Approved expansion plan");
        assert_eq!(dec.actions.len(), 1);
        assert_eq!(dec.actions[0].delegate_to.as_deref(), Some("agent-hr"));
    }

    #[tokio::test]
    async fn api_error_response_returns_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(403)
            .with_body("Forbidden")
            .create_async()
            .await;

        let client = ManusClient::new(Some("bad-key".into())).with_base_url(server.url());
        let err = client.submit_directive(&directive("anything")).await.unwrap_err();
        assert!(matches!(err, CeoError::ApiError(ref s) if s.contains("403")));
    }

    #[tokio::test]
    async fn manus_error_field_becomes_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "rate limit exceeded"}"#)
            .create_async()
            .await;

        let client = ManusClient::new(Some("key".into())).with_base_url(server.url());
        let err = client.submit_directive(&directive("x")).await.unwrap_err();
        assert!(matches!(err, CeoError::ApiError(ref s) if s.contains("rate limit")));
    }
}
