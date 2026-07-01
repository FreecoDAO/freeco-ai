use serde::{Deserialize, Serialize};

/// Advertised capability of an agent, used for dynamic routing.
///
/// The Secretary agent inspects this list to decide which specialist to
/// forward a user request to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Finds and compares products across price tiers (Budget / Value / Luxury).
    ProductResearch,
    /// Locates nearby stores and checks product availability.
    StoreLocator,
    /// Processes shopping checkouts via Google Universal Commerce Protocol.
    Checkout,
    /// Handles voice / speech input transcription and normalisation.
    VoiceInput,
    /// Classifies user intent and routes to specialist agents (Supervisor pattern).
    TaskRouting,
    /// CEO-level strategic decisions and company-wide directives.
    ExecutiveDecision,
    /// Memo creation and calendar event management.
    MemoCalendar,
    /// General question-answering powered by the LLM.
    GeneralQa,
    /// Nutritional and eco-score product analysis.
    NutritionAnalysis,
    /// Personalised wellness coaching recommendations.
    WellnessCoach,
    /// Child-safe conversational AI with content filtering.
    ChildSafeChat,
    /// Music playback control (YouTube Music deep-links, etc.).
    MusicPlayback,
    /// Real-time translation between supported languages.
    Translation,
    /// Parental control management (screen time, content gates, tracking).
    ParentalControl,
}

impl Capability {
    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Capability::ProductResearch => "Product Research",
            Capability::StoreLocator => "Store Locator",
            Capability::Checkout => "Checkout",
            Capability::VoiceInput => "Voice Input",
            Capability::TaskRouting => "Task Routing",
            Capability::ExecutiveDecision => "Executive Decision",
            Capability::MemoCalendar => "Memo & Calendar",
            Capability::GeneralQa => "General Q&A",
            Capability::NutritionAnalysis => "Nutrition Analysis",
            Capability::WellnessCoach => "Wellness Coach",
            Capability::ChildSafeChat => "Child-Safe Chat",
            Capability::MusicPlayback => "Music Playback",
            Capability::Translation => "Translation",
            Capability::ParentalControl => "Parental Control",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_are_non_empty() {
        let caps = [
            Capability::ProductResearch,
            Capability::StoreLocator,
            Capability::TaskRouting,
            Capability::ExecutiveDecision,
        ];
        for cap in &caps {
            assert!(!cap.display_name().is_empty(), "{cap:?} has empty display name");
        }
    }

    #[test]
    fn capability_roundtrips_through_json() {
        let cap = Capability::ProductResearch;
        let json = serde_json::to_string(&cap).unwrap();
        let back: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Capability::ProductResearch);
    }

    #[test]
    fn capabilities_are_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Capability::ProductResearch);
        set.insert(Capability::StoreLocator);
        set.insert(Capability::ProductResearch); // duplicate
        assert_eq!(set.len(), 2);
    }
}
