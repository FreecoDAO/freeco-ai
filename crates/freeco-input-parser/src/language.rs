use serde::{Deserialize, Serialize};

/// Supported interface languages for the Freeco.ai Geneva concierge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    French,
}

impl Language {
    /// Heuristic language detection from a short text string.
    ///
    /// Checks for common French function words/patterns. Good enough for
    /// EN/FR shopping list items; replace with a proper library for production.
    pub fn detect(text: &str) -> Self {
        let lower = text.to_lowercase();
        let french_markers = [
            " de ", " du ", " des ", " le ", " la ", " les ", " un ", " une ",
            " et ", " ou ", " avec ", " pour ", " dans ", " sur ", " au ",
            "d'", "l'", "j'", "n'", "c'", "qu'",
            "lait", "fromage", "huile", "farine", "beurre", "pain",
            "légumes", "fruits", "eau", "jus", "café", "thé",
        ];
        let french_hits = french_markers
            .iter()
            .filter(|&&m| lower.contains(m))
            .count();
        if french_hits >= 2 {
            Language::French
        } else {
            Language::English
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_english() {
        assert_eq!(Language::detect("buy organic oat milk"), Language::English);
    }

    #[test]
    fn detects_french_with_markers() {
        assert_eq!(
            Language::detect("du lait d'avoine et du fromage vegan"),
            Language::French
        );
    }

    #[test]
    fn detects_french_food_words() {
        assert_eq!(Language::detect("lait avoine fromage bio"), Language::French);
    }

    #[test]
    fn empty_string_defaults_to_english() {
        assert_eq!(Language::detect(""), Language::English);
    }

    #[test]
    fn single_french_marker_stays_english() {
        // Only one marker → not enough confidence
        assert_eq!(Language::detect("le milk"), Language::English);
    }
}
