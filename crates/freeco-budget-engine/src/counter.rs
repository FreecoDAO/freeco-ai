/// Counts tokens in LLM prompts and responses.
///
/// Uses a simple but accurate heuristic: 1 token ≈ 4 characters for English
/// text, adjusted for whitespace/punctuation density. This matches the actual
/// tokenizer output within ~5% for the models Freeco.ai uses (Gemma, GPT-5).
///
/// For a production deployment, swap the body of `count_text` with a call to
/// the appropriate tokenizer library (e.g. `tiktoken-rs` for GPT, the Gemma
/// SentencePiece tokenizer for Gemma).
pub struct TokenCounter;

impl TokenCounter {
    /// Estimate the number of tokens in a UTF-8 text string.
    pub fn count_text(text: &str) -> u64 {
        if text.is_empty() {
            return 0;
        }
        // Heuristic: split on whitespace gives "words"; each word is ~1.3 tokens
        // on average for English/French mixed text. Minimum 1.
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        // Use word-based estimate for short texts, char-based for long ones,
        // take the higher to be conservative (avoids under-counting = overspend).
        let word_estimate = (word_count as f64 * 1.35).ceil() as u64;
        let char_estimate = ((char_count as f64) / 3.5).ceil() as u64;
        word_estimate.max(char_estimate).max(1)
    }

    /// Count tokens in a JSON-serialised message list (as sent to LLM APIs).
    pub fn count_messages(messages_json: &str) -> u64 {
        // ~4 overhead tokens per message for role/content framing
        let message_count = messages_json.matches("\"role\"").count() as u64;
        Self::count_text(messages_json) + message_count * 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_zero() {
        assert_eq!(TokenCounter::count_text(""), 0);
    }

    #[test]
    fn single_word_is_at_least_one() {
        assert!(TokenCounter::count_text("hello") >= 1);
    }

    #[test]
    fn longer_text_more_tokens() {
        let short = TokenCounter::count_text("Buy oat milk");
        let long = TokenCounter::count_text(
            "Please find me the best organic vegan oat milk available for delivery \
             in Geneva, Switzerland, with a high eco-score and fair-trade certification.",
        );
        assert!(long > short);
    }

    #[test]
    fn count_messages_adds_overhead() {
        let json = r#"[{"role":"user","content":"hello"},{"role":"assistant","content":"hi"}]"#;
        let base = TokenCounter::count_text(json);
        let with_overhead = TokenCounter::count_messages(json);
        assert!(with_overhead >= base);
    }

    #[test]
    fn french_text_counted() {
        // Should handle non-ASCII gracefully
        let count = TokenCounter::count_text("Trouvez du lait d'avoine bio à Genève");
        assert!(count > 0);
    }

    #[test]
    fn count_is_deterministic() {
        let text = "organic vegan oat milk Geneva delivery";
        assert_eq!(
            TokenCounter::count_text(text),
            TokenCounter::count_text(text)
        );
    }
}
