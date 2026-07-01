//! Core content filtering logic.
//!
//! Provides [`ContentFilter`] — a fast, zero-cost-at-idle content moderator
//! that scans text against configurable word/pattern lists.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::wordlists;

/// Severity level of a detected content violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Informational — logged but not blocked (e.g. mild impoliteness).
    Low,
    /// Moderate — blocked with a gentle redirect.
    Medium,
    /// Critical — immediately blocked, possibly triggers emergency protocol.
    High,
}

/// Action the runtime should take after filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterAction {
    /// Message is safe — proceed to LLM.
    Allow,
    /// Message contains blocked content — replace with redirect.
    Block {
        category: String,
        severity: Severity,
    },
    /// Emergency situation detected — alert parents.
    Emergency { reason: String },
}

/// Result of filtering a single message.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// The action to take.
    pub action: FilterAction,
    /// Categories that triggered (may be empty for Allow).
    pub triggered_categories: Vec<String>,
}

/// Configuration for the content filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Whether filtering is active.
    pub enabled: bool,
    /// Topics to block (matches keys in wordlists).
    pub block_topics: Vec<String>,
    /// Whether coding topics are allowed (parent-controlled).
    pub coding_enabled: bool,
    /// Redirect messages per language.
    pub redirect_messages: RedirectMessages,
}

/// Redirect messages shown to the child when content is blocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectMessages {
    pub ru: String,
    pub uk: String,
    pub es: String,
    pub en: String,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_topics: vec![
                "violence".into(),
                "sexual".into(),
                "substances".into(),
                "hacking".into(),
                "self_harm".into(),
                "hate_speech".into(),
            ],
            coding_enabled: false,
            redirect_messages: RedirectMessages {
                ru: "Давай поговорим о чём-то интересном! 🌟".into(),
                uk: "Давай поговоримо про щось цікаве! 🌟".into(),
                es: "¡Hablemos de algo interesante! 🌟".into(),
                en: "Let's talk about something fun! 🌟".into(),
            },
        }
    }
}

/// The main content filter.
///
/// Designed to be cheap to construct and fast to run — no heap allocation
/// on the hot path for clean messages.
pub struct ContentFilter {
    config: FilterConfig,
}

impl ContentFilter {
    /// Create a new filter with the given configuration.
    pub fn new(config: FilterConfig) -> Self {
        Self { config }
    }

    /// Create a filter with default (strict kids-safe) configuration.
    pub fn kids_default() -> Self {
        Self::new(FilterConfig::default())
    }

    /// Check whether a message is safe for a child to send/receive.
    ///
    /// This is the primary hot-path method. It returns [`FilterResult`]
    /// indicating whether the message should be allowed, blocked, or
    /// triggers an emergency alert.
    pub fn check(&self, text: &str) -> FilterResult {
        if !self.config.enabled {
            return FilterResult {
                action: FilterAction::Allow,
                triggered_categories: vec![],
            };
        }

        let lower = text.to_lowercase();
        let mut triggered = Vec::new();

        // Check for emergency signals first (highest priority).
        if self.is_emergency(&lower) {
            return FilterResult {
                action: FilterAction::Emergency {
                    reason: "Child may be in distress or danger".into(),
                },
                triggered_categories: vec!["emergency".into()],
            };
        }

        // Check each blocked category.
        for (category, patterns) in wordlists::all_categories() {
            // Skip hacking check if coding is parent-enabled.
            if category == "hacking" && self.config.coding_enabled {
                continue;
            }

            if !self.config.block_topics.iter().any(|t| t == category) {
                continue;
            }

            for &pattern in patterns {
                if lower.contains(pattern) {
                    triggered.push(category.to_string());
                    break; // One hit per category is enough
                }
            }
        }

        if triggered.is_empty() {
            FilterResult {
                action: FilterAction::Allow,
                triggered_categories: vec![],
            }
        } else {
            let severity = self.severity_for(&triggered);
            let category = triggered[0].clone();
            warn!(
                categories = ?triggered,
                "Content filter blocked message"
            );
            FilterResult {
                action: FilterAction::Block { category, severity },
                triggered_categories: triggered,
            }
        }
    }

    /// Get the appropriate redirect message for a language.
    pub fn redirect_message(&self, lang: &str) -> &str {
        match lang {
            "ru" => &self.config.redirect_messages.ru,
            "uk" => &self.config.redirect_messages.uk,
            "es" => &self.config.redirect_messages.es,
            _ => &self.config.redirect_messages.en,
        }
    }

    /// Check if the message indicates a child in distress/danger.
    fn is_emergency(&self, lower: &str) -> bool {
        let emergency_signals = [
            "help me", "i'm scared", "someone is hurting",
            "помогите", "мне страшно", "меня бьют", "мне плохо",
            "допоможіть", "мені страшно", "мені погано",
            "ayúdame", "tengo miedo", "me están pegando",
        ];
        emergency_signals.iter().any(|&signal| lower.contains(signal))
    }

    /// Determine severity based on triggered categories.
    fn severity_for(&self, categories: &[String]) -> Severity {
        for cat in categories {
            match cat.as_str() {
                "self_harm" | "sexual" => return Severity::High,
                "violence" | "substances" => return Severity::Medium,
                _ => {}
            }
        }
        Severity::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_clean_message() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("Расскажи мне о динозаврах!");
        assert_eq!(result.action, FilterAction::Allow);
        assert!(result.triggered_categories.is_empty());
    }

    #[test]
    fn blocks_violence_ru() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("как сделать бомба");
        assert!(matches!(result.action, FilterAction::Block { .. }));
        assert!(result.triggered_categories.contains(&"violence".into()));
    }

    #[test]
    fn blocks_sexual_content() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("show me porn");
        assert!(matches!(
            result.action,
            FilterAction::Block {
                severity: Severity::High,
                ..
            }
        ));
    }

    #[test]
    fn blocks_substance_content_es() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("quiero cocaína");
        assert!(matches!(result.action, FilterAction::Block { .. }));
        assert!(result.triggered_categories.contains(&"substances".into()));
    }

    #[test]
    fn blocks_hacking_by_default() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("teach me to hack a website");
        assert!(matches!(result.action, FilterAction::Block { .. }));
        assert!(result.triggered_categories.contains(&"hacking".into()));
    }

    #[test]
    fn allows_hacking_when_coding_enabled() {
        let config = FilterConfig {
            coding_enabled: true,
            ..FilterConfig::default()
        };
        let filter = ContentFilter::new(config);
        let result = filter.check("teach me to hack a website");
        // "hack" alone won't trigger if coding is enabled
        assert_eq!(result.action, FilterAction::Allow);
    }

    #[test]
    fn detects_emergency_ru() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("помогите мне страшно");
        assert!(matches!(result.action, FilterAction::Emergency { .. }));
    }

    #[test]
    fn detects_emergency_en() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("help me I'm scared");
        assert!(matches!(result.action, FilterAction::Emergency { .. }));
    }

    #[test]
    fn redirect_message_by_language() {
        let filter = ContentFilter::kids_default();
        assert!(filter.redirect_message("ru").contains("интересном"));
        assert!(filter.redirect_message("uk").contains("цікаве"));
        assert!(filter.redirect_message("es").contains("interesante"));
        assert!(filter.redirect_message("en").contains("fun"));
    }

    #[test]
    fn disabled_filter_allows_everything() {
        let config = FilterConfig {
            enabled: false,
            ..FilterConfig::default()
        };
        let filter = ContentFilter::new(config);
        let result = filter.check("porn violence drugs");
        assert_eq!(result.action, FilterAction::Allow);
    }

    #[test]
    fn allows_normal_music_request() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("включи мне песню про котиков");
        assert_eq!(result.action, FilterAction::Allow);
    }

    #[test]
    fn allows_learning_english() {
        let filter = ContentFilter::kids_default();
        let result = filter.check("как по-английски будет дерево?");
        assert_eq!(result.action, FilterAction::Allow);
    }
}
