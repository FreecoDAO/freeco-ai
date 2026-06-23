//! # tool-gateway
//!
//! The only path to external APIs for OpenFang agents.
//! All tool calls are validated against a [`ToolPermissionManifest`] before
//! being executed, ensuring agents can only access the tools they are explicitly
//! granted.

pub mod clients;
pub mod error;
pub mod events;
pub mod manifest;

use std::sync::Arc;

pub use clients::{
    google_places::{GooglePlacesClient, LatLng, PlaceResult},
    llm::{ChatMessage, ChatResponse, LlmClient, TokenUsage, DEFAULT_MODEL},
    open_food_facts::{EcoScore, NutriScore, OpenFoodFactsClient, ProductInfo},
    tavily::{SearchResult, TavilyClient},
};
pub use error::GatewayError;
pub use events::ToolCallEvent;
pub use manifest::{ToolName, ToolPermissionManifest};

/// Central gateway through which all agent tool calls must pass.
///
/// Validates each call against the agent's [`ToolPermissionManifest`] and
/// emits a [`ToolCallEvent`] for every call (allowed or denied).
#[derive(Clone)]
pub struct ToolGateway {
    manifest: ToolPermissionManifest,
    tavily: Arc<TavilyClient>,
    open_food_facts: Arc<OpenFoodFactsClient>,
    google_places: Arc<GooglePlacesClient>,
}

impl ToolGateway {
    /// Create a new gateway from a manifest and API keys.
    pub fn new(
        manifest: ToolPermissionManifest,
        tavily_api_key: impl Into<String>,
        google_api_key: impl Into<String>,
    ) -> Self {
        Self {
            manifest,
            tavily: Arc::new(TavilyClient::new(tavily_api_key.into())),
            open_food_facts: Arc::new(OpenFoodFactsClient::new()),
            google_places: Arc::new(GooglePlacesClient::new(google_api_key.into())),
        }
    }

    /// Create a gateway with pre-built client instances.
    ///
    /// This constructor is intended for testing: callers can supply clients
    /// whose base URLs point at mock servers.
    pub fn new_for_testing(
        manifest: ToolPermissionManifest,
        tavily: TavilyClient,
        open_food_facts: OpenFoodFactsClient,
        google_places: GooglePlacesClient,
    ) -> Self {
        Self {
            manifest,
            tavily: Arc::new(tavily),
            open_food_facts: Arc::new(open_food_facts),
            google_places: Arc::new(google_places),
        }
    }

    /// Run a Tavily web/shopping search.
    pub async fn tavily_search(
        &self,
        query: &str,
        location: &str,
    ) -> Result<Vec<SearchResult>, GatewayError> {
        self.check_permission(ToolName::TavilySearch)?;
        tracing::debug!(tool = "tavily_search", query, location, "calling tool");
        let results = self.tavily.search(query, location).await?;
        tracing::info!(
            tool = "tavily_search",
            results = results.len(),
            "tool call succeeded"
        );
        Ok(results)
    }

    /// Look up a product by name in the Open Food Facts database.
    pub async fn open_food_facts_search(
        &self,
        query: &str,
    ) -> Result<Vec<ProductInfo>, GatewayError> {
        self.check_permission(ToolName::OpenFoodFacts)?;
        tracing::debug!(tool = "open_food_facts", query, "calling tool");
        let results = self.open_food_facts.search(query).await?;
        tracing::info!(
            tool = "open_food_facts",
            results = results.len(),
            "tool call succeeded"
        );
        Ok(results)
    }

    /// Look up a single product by barcode in Open Food Facts.
    pub async fn open_food_facts_barcode(
        &self,
        barcode: &str,
    ) -> Result<Option<ProductInfo>, GatewayError> {
        self.check_permission(ToolName::OpenFoodFacts)?;
        tracing::debug!(tool = "open_food_facts_barcode", barcode, "calling tool");
        let result = self.open_food_facts.get_by_barcode(barcode).await?;
        Ok(result)
    }

    /// Find stores near a location that carry a product category.
    pub async fn google_places_stores(
        &self,
        product_category: &str,
        location: LatLng,
        radius_km: f32,
    ) -> Result<Vec<PlaceResult>, GatewayError> {
        self.check_permission(ToolName::GooglePlaces)?;
        tracing::debug!(
            tool = "google_places",
            product_category,
            radius_km,
            "calling tool"
        );
        let results = self
            .google_places
            .find_stores(product_category, location, radius_km)
            .await?;
        tracing::info!(
            tool = "google_places",
            results = results.len(),
            "tool call succeeded"
        );
        Ok(results)
    }

    fn check_permission(&self, tool: ToolName) -> Result<(), GatewayError> {
        if self.manifest.allows(&tool) {
            Ok(())
        } else {
            tracing::warn!(tool = %tool, "tool call denied by manifest");
            Err(GatewayError::ToolDenied { tool })
        }
    }
}
