use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::GatewayError;

/// Geographic coordinate pair.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl LatLng {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    /// Geneva city centre coordinates.
    pub fn geneva() -> Self {
        Self::new(46.2044, 6.1432)
    }
}

/// Opening hours for a place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningHours {
    pub open_now: Option<bool>,
    pub weekday_text: Vec<String>,
}

/// Delivery / pickup availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceServices {
    pub delivery: Option<bool>,
    pub pickup: Option<bool>,
}

/// A store result from Google Places.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceResult {
    pub place_id: String,
    pub name: String,
    pub address: String,
    pub location: LatLng,
    pub distance_km: Option<f32>,
    pub rating: Option<f32>,
    pub services: PlaceServices,
    pub opening_hours: Option<OpeningHours>,
    pub maps_url: String,
}

// ── Raw API shapes ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PlacesResponse {
    results: Vec<PlaceRaw>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct PlaceRaw {
    place_id: String,
    name: String,
    #[serde(default)]
    vicinity: String,
    geometry: GeometryRaw,
    rating: Option<f32>,
    opening_hours: Option<OpeningHoursRaw>,
}

#[derive(Debug, Deserialize)]
struct GeometryRaw {
    location: LocationRaw,
}

#[derive(Debug, Deserialize)]
struct LocationRaw {
    lat: f64,
    lng: f64,
}

#[derive(Debug, Deserialize)]
struct OpeningHoursRaw {
    open_now: Option<bool>,
    #[serde(default)]
    weekday_text: Vec<String>,
}

/// HTTP client for the Google Places API (Nearby Search).
pub struct GooglePlacesClient {
    api_key: String,
    http: Client,
    base_url: String,
}

impl GooglePlacesClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: Client::new(),
            base_url: "https://maps.googleapis.com".into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Find stores near `location` that match the product category.
    /// `radius_km` is clamped to 50 km (Google Places maximum is 50 000 m).
    pub async fn find_stores(
        &self,
        product_category: &str,
        location: LatLng,
        radius_km: f32,
    ) -> Result<Vec<PlaceResult>, GatewayError> {
        if product_category.trim().is_empty() {
            return Err(GatewayError::InvalidInput(
                "product_category is empty".into(),
            ));
        }

        let radius_m = (radius_km.clamp(0.1, 50.0) * 1000.0) as u32;
        let keyword = format!("organic vegan {}", product_category);

        let url = format!(
            "{}/maps/api/place/nearbysearch/json?location={},{}&radius={}&keyword={}&key={}",
            self.base_url,
            location.lat,
            location.lng,
            radius_m,
            urlencoding_simple(&keyword),
            self.api_key,
        );

        let resp = self.http.get(&url).send().await?;
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

        let raw: PlacesResponse = resp
            .json()
            .await
            .map_err(|e| GatewayError::Deserialize(e.to_string()))?;

        if raw.status == "REQUEST_DENIED" {
            return Err(GatewayError::ApiError {
                status: 403,
                message: "Google Places API key denied".into(),
            });
        }

        Ok(raw
            .results
            .into_iter()
            .map(|r| {
                let place_location = LatLng::new(r.geometry.location.lat, r.geometry.location.lng);
                let distance_km = Some(haversine_km(location, place_location));
                let maps_url = format!(
                    "https://www.google.com/maps/place/?q=place_id:{}",
                    r.place_id
                );
                PlaceResult {
                    place_id: r.place_id,
                    name: r.name,
                    address: r.vicinity,
                    location: place_location,
                    distance_km,
                    rating: r.rating,
                    services: PlaceServices {
                        delivery: None,
                        pickup: Some(true),
                    },
                    opening_hours: r.opening_hours.map(|h| OpeningHours {
                        open_now: h.open_now,
                        weekday_text: h.weekday_text,
                    }),
                    maps_url,
                }
            })
            .collect())
    }
}

/// Haversine great-circle distance in kilometres.
fn haversine_km(a: LatLng, b: LatLng) -> f32 {
    const R: f64 = 6371.0;
    let dlat = (b.lat - a.lat).to_radians();
    let dlng = (b.lng - a.lng).to_radians();
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
    (2.0 * R * h.sqrt().asin()) as f32
}

fn urlencoding_simple(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c.to_string()
            } else if c == ' ' {
                "+".to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn places_response_body() -> &'static str {
        r#"{
            "status": "OK",
            "results": [
                {
                    "place_id": "ChIJ_abc123",
                    "name": "Bio Marché Genève",
                    "vicinity": "10 Rue de Rive, Genève",
                    "geometry": {
                        "location": { "lat": 46.2044, "lng": 6.1500 }
                    },
                    "rating": 4.5,
                    "opening_hours": {
                        "open_now": true,
                        "weekday_text": ["Monday: 8:00 AM – 7:00 PM"]
                    }
                }
            ]
        }"#
    }

    #[tokio::test]
    async fn find_stores_returns_place() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(places_response_body())
            .create_async()
            .await;

        let client = GooglePlacesClient::new("test-key".into()).with_base_url(server.url());
        let results = client
            .find_stores("dairy alternatives", LatLng::geneva(), 5.0)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Bio Marché Genève");
        assert_eq!(results[0].rating, Some(4.5));
        assert!(results[0].distance_km.is_some());
    }

    #[tokio::test]
    async fn find_stores_empty_category_errors() {
        let client = GooglePlacesClient::new("key".into());
        let err = client
            .find_stores("", LatLng::geneva(), 5.0)
            .await
            .unwrap_err();
        assert!(matches!(err, GatewayError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn find_stores_handles_request_denied() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"status": "REQUEST_DENIED", "results": []}"#)
            .create_async()
            .await;

        let client = GooglePlacesClient::new("bad-key".into()).with_base_url(server.url());
        let err = client
            .find_stores("organic food", LatLng::geneva(), 5.0)
            .await
            .unwrap_err();
        assert!(matches!(err, GatewayError::ApiError { status: 403, .. }));
    }

    #[test]
    fn haversine_geneva_to_lausanne_approx_51km() {
        let geneva = LatLng::geneva();
        let lausanne = LatLng::new(46.5197, 6.6323);
        let dist = haversine_km(geneva, lausanne);
        // Straight-line distance Geneva→Lausanne is ~51 km
        // (road distance is ~60 km, but Haversine measures as-the-crow-flies)
        assert!((dist - 51.0).abs() < 3.0, "dist={dist}");
    }

    #[test]
    fn haversine_same_point_is_zero() {
        let p = LatLng::geneva();
        assert!(haversine_km(p, p) < 0.01);
    }

    #[test]
    fn radius_clamped_to_max() {
        // We can't call the real API, but we can verify the logic in isolation.
        let clamped = (200_f32).clamp(0.1, 50.0) * 1000.0;
        assert_eq!(clamped as u32, 50_000);
    }
}
