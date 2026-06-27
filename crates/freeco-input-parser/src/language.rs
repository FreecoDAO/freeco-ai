use serde::{Deserialize, Serialize};

/// Supported interface languages for Freeco.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    French,
    /// Russian — primary Freeco language.
    Russian,
    /// Ukrainian — second primary Freeco language.
    Ukrainian,
    /// Spanish — third primary Freeco language.
    Spanish,
}

impl Language {
    /// BCP-47 language tag for this language.
    pub fn bcp47(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
            Language::Russian => "ru",
            Language::Ukrainian => "uk",
            Language::Spanish => "es",
        }
    }

    /// Heuristic language detection from a short text string.
    ///
    /// Detection priority: Russian/Ukrainian (Cyrillic script) → Spanish →
    /// French → English (default).  Cyrillic detection is script-based and
    /// therefore high-confidence; romance language detection uses common
    /// function-word markers.  A minimum hit-count of 2 guards against
    /// accidental matches on loanwords.
    pub fn detect(text: &str) -> Self {
        // ── Cyrillic script detection ────────────────────────────────────────
        // Distinguish Ukrainian from Russian via language-exclusive characters.
        let cyrillic_count = text.chars().filter(|c| is_cyrillic(*c)).count();
        if cyrillic_count >= 2 {
            // Ukrainian-exclusive letters: і, ї, є, ґ (and their uppercase forms).
            let ukrainian_chars: usize = text
                .chars()
                .filter(|c| matches!(*c, 'і' | 'ї' | 'є' | 'ґ' | 'І' | 'Ї' | 'Є' | 'Ґ'))
                .count();
            if ukrainian_chars >= 1 {
                return Language::Ukrainian;
            }
            return Language::Russian;
        }

        let lower = text.to_lowercase();

        // ── Spanish detection ────────────────────────────────────────────────
        let spanish_markers = [
            " de ", " del ", " la ", " las ", " los ", " el ", " un ", " una ",
            " y ", " o ", " con ", " por ", " para ", " en ", " que ",
            "¿", "¡", "ñ", "leche", "queso", "aceite", "harina", "mantequilla",
            "pan", "verduras", "frutas", "agua", "jugo", "café", "té",
        ];
        let spanish_hits = spanish_markers
            .iter()
            .filter(|&&m| lower.contains(m))
            .count();
        if spanish_hits >= 2 {
            return Language::Spanish;
        }

        // ── French detection ─────────────────────────────────────────────────
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
            return Language::French;
        }

        Language::English
    }
}

/// Returns true for characters in the Cyrillic Unicode block.
fn is_cyrillic(c: char) -> bool {
    matches!(c,
        '\u{0400}'..='\u{04FF}' |  // Cyrillic
        '\u{0500}'..='\u{052F}' |  // Cyrillic Supplement
        '\u{2DE0}'..='\u{2DFF}' |  // Cyrillic Extended-A
        '\u{A640}'..='\u{A69F}'    // Cyrillic Extended-B
    )
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

    #[test]
    fn detects_russian_cyrillic() {
        assert_eq!(Language::detect("купи молоко и хлеб"), Language::Russian);
    }

    #[test]
    fn detects_ukrainian_by_exclusive_chars() {
        // "і" and "є" are Ukrainian-exclusive Cyrillic letters.
        assert_eq!(Language::detect("купи молоко і хліб"), Language::Ukrainian);
    }

    #[test]
    fn detects_spanish_with_markers() {
        assert_eq!(
            Language::detect("compra leche y queso para la familia"),
            Language::Spanish
        );
    }

    #[test]
    fn detects_spanish_special_chars() {
        assert_eq!(Language::detect("¿Qué tal? ¡Hola!"), Language::Spanish);
    }

    #[test]
    fn bcp47_tags_correct() {
        assert_eq!(Language::Russian.bcp47(), "ru");
        assert_eq!(Language::Ukrainian.bcp47(), "uk");
        assert_eq!(Language::Spanish.bcp47(), "es");
        assert_eq!(Language::French.bcp47(), "fr");
        assert_eq!(Language::English.bcp47(), "en");
    }

    #[test]
    fn single_cyrillic_char_not_enough() {
        // Only 1 Cyrillic character — not enough confidence.
        assert_eq!(Language::detect("buy молоко"), Language::English);
    }
}
