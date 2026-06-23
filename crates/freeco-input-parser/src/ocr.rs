use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::Deserialize;

use crate::error::ParseError;

/// HTTP client for the Google Cloud Vision API (OCR / text detection).
pub struct OcrClient {
    api_key: String,
    http: Client,
    base_url: String,
}

impl OcrClient {
    pub fn new(api_key: String, http: Client) -> Self {
        Self {
            api_key,
            http,
            base_url: "https://vision.googleapis.com".into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Extract text from raw image bytes using Google Vision OCR.
    pub async fn extract_text_from_bytes(&self, bytes: &[u8]) -> Result<String, ParseError> {
        if bytes.is_empty() {
            return Err(ParseError::OcrFailed("image is empty".into()));
        }
        let b64 = BASE64.encode(bytes);
        self.extract_text_from_base64(&b64).await
    }

    /// Extract text from a base64-encoded image string.
    pub async fn extract_text_from_base64(&self, b64: &str) -> Result<String, ParseError> {
        if b64.is_empty() {
            return Err(ParseError::OcrFailed("base64 image is empty".into()));
        }

        // Validate base64 (will catch obviously corrupt data early)
        BASE64
            .decode(b64)
            .map_err(|e| ParseError::Base64Decode(e.to_string()))?;

        let url = format!(
            "{}/v1/images:annotate?key={}",
            self.base_url, self.api_key
        );

        let body = serde_json::json!({
            "requests": [{
                "image": { "content": b64 },
                "features": [{ "type": "TEXT_DETECTION", "maxResults": 1 }]
            }]
        });

        let resp = self.http.post(&url).json(&body).send().await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(ParseError::OcrFailed(format!("HTTP {status}: {message}")));
        }

        let vision_resp: VisionResponse = resp
            .json()
            .await
            .map_err(|e| ParseError::OcrFailed(e.to_string()))?;

        let text = vision_resp
            .responses
            .into_iter()
            .next()
            .and_then(|r| r.full_text_annotation)
            .map(|a| a.text)
            .unwrap_or_default();

        Ok(text)
    }
}

// ── Vision API response shapes ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VisionResponse {
    responses: Vec<VisionAnnotateResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VisionAnnotateResponse {
    full_text_annotation: Option<FullTextAnnotation>,
}

#[derive(Debug, Deserialize)]
struct FullTextAnnotation {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn make_vision_response(text: &str) -> String {
        serde_json::json!({
            "responses": [{
                "fullTextAnnotation": {
                    "text": text
                }
            }]
        })
        .to_string()
    }

    /// Minimal valid 1×1 white PNG, base64-encoded.
    fn tiny_png_b64() -> String {
        BASE64.encode(
            b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\
              \x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\x0f\x00\
              \x00\x01\x01\x00\x05\x18\xd8N\x00\x00\x00\x00IEND\xaeB`\x82",
        )
    }

    #[tokio::test]
    async fn extracts_text_from_base64() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_vision_response(
                "oat milk\nvegan cheese\nolive oil",
            ))
            .create_async()
            .await;

        let client = OcrClient::new("test-key".into(), Client::new()).with_base_url(server.url());
        let text = client.extract_text_from_base64(&tiny_png_b64()).await.unwrap();
        assert_eq!(text, "oat milk\nvegan cheese\nolive oil");
    }

    #[tokio::test]
    async fn empty_base64_returns_error() {
        let client = OcrClient::new("key".into(), Client::new());
        let err = client.extract_text_from_base64("").await.unwrap_err();
        assert!(matches!(err, ParseError::OcrFailed(_)));
    }

    #[tokio::test]
    async fn empty_bytes_returns_error() {
        let client = OcrClient::new("key".into(), Client::new());
        let err = client.extract_text_from_bytes(&[]).await.unwrap_err();
        assert!(matches!(err, ParseError::OcrFailed(_)));
    }

    #[tokio::test]
    async fn invalid_base64_returns_error() {
        let client = OcrClient::new("key".into(), Client::new());
        let err = client.extract_text_from_base64("!!!not-valid-base64!!!").await.unwrap_err();
        assert!(matches!(err, ParseError::Base64Decode(_)));
    }

    #[tokio::test]
    async fn vision_api_error_propagated() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(403)
            .with_body("API key invalid")
            .create_async()
            .await;

        let client = OcrClient::new("bad-key".into(), Client::new()).with_base_url(server.url());
        let err = client.extract_text_from_base64(&tiny_png_b64()).await.unwrap_err();
        assert!(matches!(err, ParseError::OcrFailed(_)));
    }

    #[tokio::test]
    async fn no_text_in_image_returns_empty_string() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"responses": [{}]}"#)
            .create_async()
            .await;

        let client = OcrClient::new("key".into(), Client::new()).with_base_url(server.url());
        let text = client.extract_text_from_base64(&tiny_png_b64()).await.unwrap();
        assert_eq!(text, "");
    }
}
