use std::fmt;

use serde::{Deserialize, Serialize};

/// Every tool that the gateway can call.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    TavilySearch,
    OpenFoodFacts,
    GooglePlaces,
    GoogleVision,
    UcpClient,
    WebScraper,
}

impl fmt::Display for ToolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ToolName::TavilySearch => "tavily_search",
            ToolName::OpenFoodFacts => "open_food_facts",
            ToolName::GooglePlaces => "google_places",
            ToolName::GoogleVision => "google_vision",
            ToolName::UcpClient => "ucp_client",
            ToolName::WebScraper => "web_scraper",
        };
        write!(f, "{}", s)
    }
}

/// Declares which tools an agent is permitted to call.
///
/// Loaded from the agent's TOML manifest `tools_allowed` array.
///
/// ```toml
/// [[team.agents]]
/// name = "product-researcher"
/// tools_allowed = ["tavily_search", "open_food_facts", "google_places"]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolPermissionManifest {
    pub tools_allowed: Vec<ToolName>,
}

impl ToolPermissionManifest {
    pub fn new(tools: Vec<ToolName>) -> Self {
        Self {
            tools_allowed: tools,
        }
    }

    /// Returns `true` if the given tool is in the allow-list.
    pub fn allows(&self, tool: &ToolName) -> bool {
        self.tools_allowed.contains(tool)
    }

    /// Parse a manifest from a TOML string (subset of the agent manifest).
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_listed_tool() {
        let m = ToolPermissionManifest::new(vec![ToolName::TavilySearch]);
        assert!(m.allows(&ToolName::TavilySearch));
        assert!(!m.allows(&ToolName::GooglePlaces));
    }

    #[test]
    fn empty_manifest_denies_all() {
        let m = ToolPermissionManifest::default();
        assert!(!m.allows(&ToolName::TavilySearch));
        assert!(!m.allows(&ToolName::OpenFoodFacts));
    }

    #[test]
    fn parse_from_toml() {
        let toml = r#"tools_allowed = ["tavily_search", "open_food_facts"]"#;
        let m = ToolPermissionManifest::from_toml(toml).unwrap();
        assert!(m.allows(&ToolName::TavilySearch));
        assert!(m.allows(&ToolName::OpenFoodFacts));
        assert!(!m.allows(&ToolName::GooglePlaces));
    }

    #[test]
    fn display_tool_name() {
        assert_eq!(ToolName::TavilySearch.to_string(), "tavily_search");
        assert_eq!(ToolName::OpenFoodFacts.to_string(), "open_food_facts");
        assert_eq!(ToolName::GooglePlaces.to_string(), "google_places");
    }
}
