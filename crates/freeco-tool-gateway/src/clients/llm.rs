use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::GatewayError;

/// The default Novita AI model powering all Freeco agents.
pub const DEFAULT_MODEL: &str = "google/gemma-4-26b-a4b-it";

/// A single turn in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// `"system"`, `"user"`, or `"assistant"`.
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

/// Token usage reported by the API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Successful response from a chat completion call.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// The assistant's reply text.
    pub content: String,
    /// Model name echoed by the API.
    pub model: String,
    /// Token counts for budget accounting.
    pub usage: TokenUsage,
    /// `"stop"`, `"length"`, etc.
    pub finish_reason: String,
}

// ── Raw API shapes ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawChatResponse {
    model: Option<String>,
    choices: Vec<RawChoice>,
    usage: Option<RawUsage>,
}

#[derive(Debug, Deserialize)]
struct RawChoice {
    message: RawMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// OpenAI-compatible LLM client pointed at Novita AI.
///
/// Default model:    `google/gemma-4-26b-a4b-it`
/// Default base URL: `https://api.novita.ai/v3/openai`
pub struct LlmClient {
    api_key: String,
    http: Client,
    base_url: String,
    model: String,
}

impl LlmClient {
    /// Create a client for the Novita AI API.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: Client::new(),
            base_url: "https://api.novita.ai/v3/openai".into(),
            model: DEFAULT_MODEL.into(),
        }
    }

    /// Override the API base URL — used in tests to point at a mock server.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Override the model (e.g. for A/B testing).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// The model identifier this client is configured to use.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Send a chat completion request and return the assistant's reply.
    ///
    /// `max_tokens` caps the response length; use ≤ 256 for classification
    /// tasks and 1 024–2 048 for full recommendations.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
    ) -> Result<ChatResponse, GatewayError> {
        if messages.is_empty() {
            return Err(GatewayError::InvalidInput(
                "messages must not be empty".into(),
            ));
        }

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": 0.7,
        });

        let resp = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status == 429 {
            return Err(GatewayError::RateLimited {
                retry_after_secs: 60,
            });
        }
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(GatewayError::ApiError { status, message });
        }

        let raw: RawChatResponse = resp
            .json()
            .await
            .map_err(|e| GatewayError::Deserialize(e.to_string()))?;

        let choice = raw
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| GatewayError::Deserialize("no choices in LLM response".into()))?;

        let content = choice.message.content.unwrap_or_default();

        let usage = raw
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens.unwrap_or(0),
                completion_tokens: u.completion_tokens.unwrap_or(0),
                total_tokens: u.total_tokens.unwrap_or(0),
            })
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: raw.model.unwrap_or_else(|| self.model.clone()),
            usage,
            finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn ok_response(content: &str, prompt_tokens: u64, completion_tokens: u64) -> String {
        serde_json::json!({
            "model": "google/gemma-4-26b-a4b-it",
            "choices": [{
                "message": { "role": "assistant", "content": content },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn chat_returns_content() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ok_response("SHOPPING", 10, 1))
            .create_async()
            .await;

        let client = LlmClient::new("test-key").with_base_url(server.url());
        let resp = client
            .chat(vec![ChatMessage::user("classify this")], 10)
            .await
            .unwrap();

        assert_eq!(resp.content, "SHOPPING");
        assert_eq!(resp.finish_reason, "stop");
        assert_eq!(resp.usage.total_tokens, 11);
    }

    #[tokio::test]
    async fn chat_tracks_token_usage() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ok_response("Hello!", 500, 128))
            .create_async()
            .await;

        let client = LlmClient::new("key").with_base_url(server.url());
        let resp = client
            .chat(
                vec![ChatMessage::system("be helpful"), ChatMessage::user("hi")],
                256,
            )
            .await
            .unwrap();

        assert_eq!(resp.usage.prompt_tokens, 500);
        assert_eq!(resp.usage.completion_tokens, 128);
        assert_eq!(resp.usage.total_tokens, 628);
    }

    #[tokio::test]
    async fn chat_handles_rate_limit() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(429)
            .create_async()
            .await;

        let client = LlmClient::new("key").with_base_url(server.url());
        let err = client
            .chat(vec![ChatMessage::user("hi")], 64)
            .await
            .unwrap_err();

        assert!(matches!(err, GatewayError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn chat_handles_auth_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(401)
            .with_body("Unauthorized")
            .create_async()
            .await;

        let client = LlmClient::new("bad-key").with_base_url(server.url());
        let err = client
            .chat(vec![ChatMessage::user("hi")], 64)
            .await
            .unwrap_err();

        assert!(matches!(err, GatewayError::ApiError { status: 401, .. }));
    }

    #[tokio::test]
    async fn empty_messages_returns_error() {
        let client = LlmClient::new("key");
        let err = client.chat(vec![], 64).await.unwrap_err();
        assert!(matches!(err, GatewayError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn chat_message_constructors_set_role() {
        let sys = ChatMessage::system("be helpful");
        let usr = ChatMessage::user("hello");
        let ast = ChatMessage::assistant("hi there");
        assert_eq!(sys.role, "system");
        assert_eq!(usr.role, "user");
        assert_eq!(ast.role, "assistant");
    }

    #[tokio::test]
    async fn custom_model_is_sent_in_request() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ok_response("ok", 5, 1))
            .create_async()
            .await;

        let client = LlmClient::new("key")
            .with_base_url(server.url())
            .with_model("my-custom-model");

        assert_eq!(client.model(), "my-custom-model");
        let resp = client
            .chat(vec![ChatMessage::user("hi")], 32)
            .await
            .unwrap();
        assert_eq!(resp.content, "ok");
    }
}
