use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::GatewayError;

/// NOVA food processing group (1 = unprocessed, 4 = ultra-processed).
pub type NovaGroup = u8;

/// Open Food Facts eco-score grade (A–E or Unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EcoScore {
    A,
    B,
    C,
    D,
    E,
    #[default]
    #[serde(other)]
    Unknown,
}

impl EcoScore {
    /// Numeric value for freeco_score calculation (0.0–1.0).
    pub fn as_score(&self) -> f32 {
        match self {
            EcoScore::A => 1.0,
            EcoScore::B => 0.75,
            EcoScore::C => 0.5,
            EcoScore::D => 0.25,
            EcoScore::E => 0.0,
            EcoScore::Unknown => 0.0,
        }
    }
}

/// Nutri-Score grade (A–E or Unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NutriScore {
    A,
    B,
    C,
    D,
    E,
    #[default]
    #[serde(other)]
    Unknown,
}

/// Product information returned by Open Food Facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub barcode: String,
    pub name: String,
    pub brands: Vec<String>,
    pub eco_score: EcoScore,
    pub nutri_score: NutriScore,
    pub nova_group: Option<NovaGroup>,
    pub is_vegan: Option<bool>,
    pub is_vegetarian: Option<bool>,
    pub is_organic: Option<bool>,
    pub origin_country: Option<String>,
    pub labels: Vec<String>,
    pub image_url: Option<String>,
}

// ── Raw API shapes ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OffSearchResponse {
    products: Vec<OffProductRaw>,
}

#[derive(Debug, Deserialize)]
struct OffBarcodeResponse {
    status: u8,
    product: Option<OffProductRaw>,
}

#[derive(Debug, Deserialize, Default)]
struct OffProductRaw {
    #[serde(default)]
    code: String,
    #[serde(default)]
    product_name: String,
    #[serde(default)]
    brands: String,
    #[serde(default)]
    ecoscore_grade: String,
    #[serde(default)]
    nutriscore_grade: String,
    #[serde(default)]
    nova_group: Option<NovaGroup>,
    #[serde(default)]
    labels_tags: Vec<String>,
    #[serde(default)]
    origins: String,
    #[serde(default)]
    image_front_url: Option<String>,
}

impl OffProductRaw {
    fn into_product_info(self) -> ProductInfo {
        let labels: Vec<String> = self.labels_tags.clone();
        let is_vegan = Some(
            labels
                .iter()
                .any(|l| l.contains("vegan") || l.contains("en:vegan")),
        );
        let is_vegetarian = Some(
            labels
                .iter()
                .any(|l| l.contains("vegetarian") || l.contains("en:vegetarian")),
        );
        let is_organic = Some(
            labels
                .iter()
                .any(|l| l.contains("organic") || l.contains("en:organic") || l.contains("bio")),
        );

        let eco_score = match self.ecoscore_grade.to_lowercase().as_str() {
            "a" => EcoScore::A,
            "b" => EcoScore::B,
            "c" => EcoScore::C,
            "d" => EcoScore::D,
            "e" => EcoScore::E,
            _ => EcoScore::Unknown,
        };

        let nutri_score = match self.nutriscore_grade.to_lowercase().as_str() {
            "a" => NutriScore::A,
            "b" => NutriScore::B,
            "c" => NutriScore::C,
            "d" => NutriScore::D,
            "e" => NutriScore::E,
            _ => NutriScore::Unknown,
        };

        let brands: Vec<String> = self
            .brands
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        ProductInfo {
            barcode: self.code,
            name: self.product_name,
            brands,
            eco_score,
            nutri_score,
            nova_group: self.nova_group,
            is_vegan,
            is_vegetarian,
            is_organic,
            origin_country: if self.origins.is_empty() {
                None
            } else {
                Some(self.origins)
            },
            labels,
            image_url: self.image_front_url,
        }
    }
}

/// HTTP client for the Open Food Facts API.
pub struct OpenFoodFactsClient {
    http: Client,
    base_url: String,
}

impl OpenFoodFactsClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            base_url: "https://world.openfoodfacts.org".into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Search products by name. Returns up to 20 results.
    pub async fn search(&self, query: &str) -> Result<Vec<ProductInfo>, GatewayError> {
        if query.trim().is_empty() {
            return Err(GatewayError::InvalidInput("search query is empty".into()));
        }

        let url = format!(
            "{}/cgi/search.pl?search_terms={}&search_simple=1&action=process&json=1&page_size=20&fields=code,product_name,brands,ecoscore_grade,nutriscore_grade,nova_group,labels_tags,origins,image_front_url",
            self.base_url,
            urlencoding::encode(query)
        );

        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(GatewayError::ApiError { status, message });
        }

        let raw: OffSearchResponse = resp
            .json()
            .await
            .map_err(|e| GatewayError::Deserialize(e.to_string()))?;

        Ok(raw
            .products
            .into_iter()
            .map(OffProductRaw::into_product_info)
            .collect())
    }

    /// Look up a single product by barcode (EAN-13 etc).
    pub async fn get_by_barcode(&self, barcode: &str) -> Result<Option<ProductInfo>, GatewayError> {
        if barcode.trim().is_empty() {
            return Err(GatewayError::InvalidInput("barcode is empty".into()));
        }

        let url = format!(
            "{}/api/v0/product/{}.json?fields=code,product_name,brands,ecoscore_grade,nutriscore_grade,nova_group,labels_tags,origins,image_front_url",
            self.base_url, barcode
        );

        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(GatewayError::ApiError { status, message });
        }

        let raw: OffBarcodeResponse = resp
            .json()
            .await
            .map_err(|e| GatewayError::Deserialize(e.to_string()))?;

        if raw.status == 0 {
            return Ok(None);
        }

        Ok(raw.product.map(OffProductRaw::into_product_info))
    }
}

impl Default for OpenFoodFactsClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── URL encoding helper (avoids extra dep) ──────────────────────────────────
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char);
                }
                b' ' => out.push('+'),
                other => {
                    out.push('%');
                    out.push_str(&format!("{:02X}", other));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn make_product_response(extra_fields: &str) -> String {
        format!(
            r#"{{
                "products": [{{
                    "code": "3017624010701",
                    "product_name": "Bio Oat Milk",
                    "brands": "Oatly",
                    "ecoscore_grade": "a",
                    "nutriscore_grade": "b",
                    "nova_group": 3,
                    "labels_tags": ["en:organic", "en:vegan", "en:fair-trade"],
                    "origins": "Switzerland",
                    "image_front_url": "https://images.openfoodfacts.org/oatmilk.jpg"
                    {}
                }}]
            }}"#,
            extra_fields
        )
    }

    #[tokio::test]
    async fn search_parses_product_correctly() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_product_response(""))
            .create_async()
            .await;

        let client = OpenFoodFactsClient::new().with_base_url(server.url());
        let results = client.search("oat milk").await.unwrap();

        assert_eq!(results.len(), 1);
        let p = &results[0];
        assert_eq!(p.barcode, "3017624010701");
        assert_eq!(p.name, "Bio Oat Milk");
        assert_eq!(p.brands, vec!["Oatly"]);
        assert_eq!(p.eco_score, EcoScore::A);
        assert_eq!(p.nutri_score, NutriScore::B);
        assert_eq!(p.nova_group, Some(3));
        assert_eq!(p.is_vegan, Some(true));
        assert_eq!(p.is_organic, Some(true));
        assert_eq!(p.origin_country, Some("Switzerland".into()));
    }

    #[tokio::test]
    async fn search_empty_query_returns_error() {
        let client = OpenFoodFactsClient::new();
        let err = client.search("").await.unwrap_err();
        assert!(matches!(err, GatewayError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn barcode_lookup_not_found() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": 0, "product": null}"#)
            .create_async()
            .await;

        let client = OpenFoodFactsClient::new().with_base_url(server.url());
        let result = client.get_by_barcode("0000000000000").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn barcode_empty_returns_error() {
        let client = OpenFoodFactsClient::new();
        let err = client.get_by_barcode("").await.unwrap_err();
        assert!(matches!(err, GatewayError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn eco_score_numeric_values() {
        assert!((EcoScore::A.as_score() - 1.0).abs() < f32::EPSILON);
        assert!((EcoScore::B.as_score() - 0.75).abs() < f32::EPSILON);
        assert!((EcoScore::C.as_score() - 0.5).abs() < f32::EPSILON);
        assert!((EcoScore::D.as_score() - 0.25).abs() < f32::EPSILON);
        assert!((EcoScore::E.as_score() - 0.0).abs() < f32::EPSILON);
        assert!((EcoScore::Unknown.as_score() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn non_organic_product_correctly_flagged() {
        let raw = OffProductRaw {
            code: "123".into(),
            product_name: "Regular Milk".into(),
            labels_tags: vec!["en:pasteurized".into()],
            ..Default::default()
        };
        let info = raw.into_product_info();
        assert_eq!(info.is_vegan, Some(false));
        assert_eq!(info.is_organic, Some(false));
    }
}
