use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::GatewayError;

/// A single search result returned by the Tavily API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f32,
    #[serde(default)]
    pub raw_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResultRaw>,
}

#[derive(Debug, Deserialize)]
struct TavilyResultRaw {
    title: String,
    url: String,
    content: String,
    score: f32,
    raw_content: Option<String>,
}

/// HTTP client for the Tavily Search API.
pub struct TavilyClient {
    api_key: String,
    http: Client,
    base_url: String,
}

impl TavilyClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: Client::new(),
            base_url: "https://api.tavily.com".into(),
        }
    }

    /// Override the base URL — used in tests to point at a mock server.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Search for products, appending the location as a search qualifier.
    pub async fn search(
        &self,
        query: &str,
        location: &str,
    ) -> Result<Vec<SearchResult>, GatewayError> {
        let augmented_query = if location.is_empty() {
            query.to_string()
        } else {
            format!("{} {}", query, location)
        };

        let body = serde_json::json!({
            "api_key": self.api_key,
            "query": augmented_query,
            "search_depth": "basic",
            "include_answer": false,
            "max_results": 10,
        });

        let resp = self
            .http
            .post(format!("{}/search", self.base_url))
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

        let raw: TavilyResponse = resp
            .json()
            .await
            .map_err(|e| GatewayError::Deserialize(e.to_string()))?;

        Ok(raw
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                content: r.content,
                score: r.score,
                raw_content: r.raw_content,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn search_returns_results() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "results": [
                        {
                            "title": "Organic Oat Milk - Migros",
                            "url": "https://www.migros.ch/product/oat-milk",
                            "content": "Bio oat milk available in Geneva stores",
                            "score": 0.95
                        }
                    ]
                }"#,
            )
            .create_async()
            .await;

        let client = TavilyClient::new("test-key".into()).with_base_url(server.url());
        let results = client.search("oat milk organic", "Geneva").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Organic Oat Milk - Migros");
        assert!((results[0].score - 0.95).abs() < f32::EPSILON);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn search_appends_location_to_query() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results": []}"#)
            .create_async()
            .await;

        let client = TavilyClient::new("test-key".into()).with_base_url(server.url());
        let results = client.search("vegan cheese", "Geneva Switzerland").await.unwrap();

        assert!(results.is_empty());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn search_handles_rate_limit() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/search")
            .with_status(429)
            .create_async()
            .await;

        let client = TavilyClient::new("test-key".into()).with_base_url(server.url());
        let err = client.search("anything", "").await.unwrap_err();

        assert!(matches!(err, GatewayError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn search_handles_api_error() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/search")
            .with_status(401)
            .with_body("Unauthorized")
            .create_async()
            .await;

        let client = TavilyClient::new("bad-key".into()).with_base_url(server.url());
        let err = client.search("anything", "").await.unwrap_err();

        assert!(matches!(err, GatewayError::ApiError { status: 401, .. }));
    }

    #[tokio::test]
    async fn search_without_location() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results": []}"#)
            .create_async()
            .await;

        let client = TavilyClient::new("test-key".into()).with_base_url(server.url());
        // Empty location should still work
        let results = client.search("organic oat milk", "").await.unwrap();
        assert!(results.is_empty());
    }
}
