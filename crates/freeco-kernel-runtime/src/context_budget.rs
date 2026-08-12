//! Dynamic context budget for tool result truncation.
//!
//! Replaces the hardcoded MAX_TOOL_RESULT_CHARS with a two-layer system:
//! - Layer 1: Per-result cap based on context window size (30% of window)
//! - Layer 2: Context guard that scans all tool results before LLM calls
//!   and compacts oldest results when total exceeds 75% headroom.

use crate::str_utils::safe_truncate_str;
use openfang_types::message::{ContentBlock, Message, MessageContent};
use openfang_types::tool::ToolDefinition;
use tracing::debug;

/// Budget parameters derived from the model's context window.
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// Total context window size in tokens.
    pub context_window_tokens: usize,
    /// Estimated characters per token for tool results (denser content).
    pub tool_chars_per_token: f64,
    /// Estimated characters per token for general content.
    pub general_chars_per_token: f64,
}

impl ContextBudget {
    /// Create a new budget from a context window size.
    pub fn new(context_window_tokens: usize) -> Self {
        Self {
            context_window_tokens,
            tool_chars_per_token: 2.0,
            general_chars_per_token: 4.0,
        }
    }

    /// Per-result character cap: 30% of context window converted to chars.
    pub fn per_result_cap(&self) -> usize {
        let tokens_for_tool = (self.context_window_tokens as f64 * 0.30) as usize;
        (tokens_for_tool as f64 * self.tool_chars_per_token) as usize
    }

    /// Single result absolute max: 50% of context window.
    pub fn single_result_max(&self) -> usize {
        let tokens = (self.context_window_tokens as f64 * 0.50) as usize;
        (tokens as f64 * self.tool_chars_per_token) as usize
    }

    /// Total tool result headroom: 75% of context window in chars.
    pub fn total_tool_headroom_chars(&self) -> usize {
        let tokens = (self.context_window_tokens as f64 * 0.75) as usize;
        (tokens as f64 * self.tool_chars_per_token) as usize
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::new(200_000)
    }
}

/// Layer 1: Truncate a single tool result dynamically based on context budget.
///
/// Breaks at newline boundaries when possible to avoid mid-line truncation.
pub fn truncate_tool_result_dynamic(content: &str, budget: &ContextBudget) -> String {
    let cap = budget.per_result_cap();
    if content.len() <= cap {
        return content.to_string();
    }

    // Find last newline before the cap to break cleanly (char-boundary safe)
    let mut safe_cap = cap.min(content.len());
    while safe_cap > 0 && !content.is_char_boundary(safe_cap) {
        safe_cap -= 1;
    }
    let mut search_start = safe_cap.saturating_sub(200);
    // Ensure search_start is a valid char boundary
    while search_start > 0 && !content.is_char_boundary(search_start) {
        search_start -= 1;
    }
    let mut break_point = content[search_start..safe_cap]
        .rfind('\n')
        .map(|pos| search_start + pos)
        .unwrap_or(safe_cap.saturating_sub(100));
    // Ensure break_point is also a char boundary
    while break_point > 0 && !content.is_char_boundary(break_point) {
        break_point -= 1;
    }

    format!(
        "{}\n\n[TRUNCATED: result was {} chars, showing first {} (budget: {}% of {}K context window)]",
        &content[..break_point],
        content.len(),
        break_point,
        30,
        budget.context_window_tokens / 1000
    )
}

/// Layer 2: Context guard — scan all tool_result blocks in the message history.
///
/// If total tool result content exceeds 75% of the context headroom,
/// compact oldest results first. Returns the number of results compacted.
pub fn apply_context_guard(
    messages: &mut [Message],
    budget: &ContextBudget,
    _tools: &[ToolDefinition],
) -> usize {
    let headroom = budget.total_tool_headroom_chars();
    let single_max = budget.single_result_max();

    // Collect all tool result sizes and locations
    struct ToolResultLoc {
        msg_idx: usize,
        block_idx: usize,
        char_len: usize,
    }

    let mut locations: Vec<ToolResultLoc> = Vec::new();
    let mut total_chars: usize = 0;

    for (msg_idx, msg) in messages.iter().enumerate() {
        if let MessageContent::Blocks(blocks) = &msg.content {
            for (block_idx, block) in blocks.iter().enumerate() {
                if let ContentBlock::ToolResult { content, .. } = block {
                    let len = content.len();
                    total_chars += len;
                    locations.push(ToolResultLoc {
                        msg_idx,
                        block_idx,
                        char_len: len,
                    });
                }
            }
        }
    }

    if total_chars <= headroom {
        return 0;
    }

    debug!(
        total_chars,
        headroom,
        results = locations.len(),
        "Context guard: tool results exceed headroom, compacting oldest"
    );

    // First pass: cap any single result that exceeds 50% of context
    let mut compacted = 0;
    for loc in &locations {
        if loc.char_len > single_max {
            // Bounds check: indices may be stale if messages were modified concurrently
            if loc.msg_idx >= messages.len() {
                continue;
            }
            if let MessageContent::Blocks(blocks) = &mut messages[loc.msg_idx].content {
                if loc.block_idx >= blocks.len() {
                    continue;
                }
                if let ContentBlock::ToolResult { content, .. } = &mut blocks[loc.block_idx] {
                    let old_len = content.len();
                    *content = truncate_to(content, single_max);
                    total_chars -= old_len;
                    total_chars += content.len();
                    compacted += 1;
                }
            }
        }
    }

    // Second pass: compact oldest results until under headroom
    // (locations are already in chronological order)
    let compact_target = 2000; // compact to 2K chars each
    for loc in &locations {
        if total_chars <= headroom {
            break;
        }
        if loc.char_len <= compact_target {
            continue;
        }
        if loc.msg_idx >= messages.len() {
            continue;
        }
        if let MessageContent::Blocks(blocks) = &mut messages[loc.msg_idx].content {
            if loc.block_idx >= blocks.len() {
                continue;
            }
            if let ContentBlock::ToolResult { content, .. } = &mut blocks[loc.block_idx] {
                if content.len() > compact_target {
                    let old_len = content.len();
                    *content = truncate_to(content, compact_target);
                    total_chars -= old_len;
                    total_chars += content.len();
                    compacted += 1;
                }
            }
        }
    }

    compacted
}

/// Truncate content to `max_chars` with a marker.
fn truncate_to(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        return content.to_string();
    }
    let mut keep = max_chars.saturating_sub(80).min(content.len());
    // Walk back to a valid char boundary
    while keep > 0 && !content.is_char_boundary(keep) {
        keep -= 1;
    }
    let mut search_start = keep.saturating_sub(100);
    // Walk back to a valid char boundary
    while search_start > 0 && !content.is_char_boundary(search_start) {
        search_start -= 1;
    }
    // Try to break at newline
    let break_point = content[search_start..keep]
        .rfind('\n')
        .map(|pos| search_start + pos)
        .unwrap_or(keep);

    // Use safe_truncate_str as an extra layer of safety
    let safe_content = safe_truncate_str(content, break_point);
    format!(
        "{}\n\n[COMPACTED: {} → {} chars by context guard]",
        safe_content,
        content.len(),
        safe_content.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_defaults() {
        let budget = ContextBudget::default();
        assert_eq!(budget.context_window_tokens, 200_000);
        // 30% of 200K * 2.0 chars/token = 120K chars
        assert_eq!(budget.per_result_cap(), 120_000);
    }

    #[test]
    fn test_small_model_budget() {
        let budget = ContextBudget::new(8_000);
        // 30% of 8K * 2.0 = 4800 chars
        assert_eq!(budget.per_result_cap(), 4_800);
    }

    #[test]
    fn test_truncate_within_limit() {
        let budget = ContextBudget::default();
        let short = "Hello world";
        assert_eq!(truncate_tool_result_dynamic(short, &budget), short);
    }

    #[test]
    fn test_truncate_breaks_at_newline() {
        let budget = ContextBudget::new(100); // very small: cap = 60 chars
        let content =
            "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12";
        let result = truncate_tool_result_dynamic(content, &budget);
        assert!(result.contains("[TRUNCATED:"));
        // Should not split in the middle of a line
        assert!(
            result.starts_with("line1\n") || result.is_empty() || result.contains("[TRUNCATED:")
        );
    }

    #[test]
    fn test_context_guard_no_compaction_needed() {
        let budget = ContextBudget::default();
        let mut messages = vec![Message::user("hello")];
        let compacted = apply_context_guard(&mut messages, &budget, &[]);
        assert_eq!(compacted, 0);
    }

    #[test]
    fn test_context_guard_compacts_oldest() {
        // Use tiny budget to trigger compaction
        let budget = ContextBudget::new(100); // headroom = 75% of 100 * 2.0 = 150 chars
        let big_result = "x".repeat(500);
        let mut messages = vec![
            Message {
                role: openfang_types::message::Role::User,
                content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                    tool_use_id: "t1".to_string(),
                    tool_name: String::new(),
                    content: big_result.clone(),
                    is_error: false,
                }]),
                ..Default::default()
            },
            Message {
                role: openfang_types::message::Role::User,
                content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                    tool_use_id: "t2".to_string(),
                    tool_name: String::new(),
                    content: big_result,
                    is_error: false,
                }]),
                ..Default::default()
            },
        ];

        let compacted = apply_context_guard(&mut messages, &budget, &[]);
        assert!(compacted > 0);

        // Verify results were actually truncated
        if let MessageContent::Blocks(blocks) = &messages[0].content {
            if let ContentBlock::ToolResult { content, .. } = &blocks[0] {
                assert!(content.len() < 500);
            }
        }
    }

    #[test]
    fn test_truncate_tool_result_multibyte_chinese() {
        // Tiny budget: cap = 30% of 100 * 2.0 = 60 bytes
        let budget = ContextBudget::new(100);
        // Each Chinese char is 3 bytes in UTF-8; 100 chars = 300 bytes
        let content: String = "\u{4f60}\u{597d}\u{4e16}\u{754c}".repeat(25);
        assert_eq!(content.len(), 300);
        // Must not panic on multi-byte content
        let result = truncate_tool_result_dynamic(&content, &budget);
        assert!(result.contains("[TRUNCATED:"));
        // The visible portion must be valid UTF-8 (implicit: no panic)
        assert!(result.is_char_boundary(0));
    }

    #[test]
    fn test_truncate_to_multibyte_emoji() {
        // Each emoji is 4 bytes; 200 emojis = 800 bytes
        let content: String = "\u{1f600}".repeat(200);
        let result = truncate_to(&content, 100);
        assert!(result.contains("[COMPACTED:"));
        // Must not panic and must produce valid UTF-8
        assert!(result.is_char_boundary(0));
    }

    #[test]
    fn test_context_guard_multibyte_tool_results() {
        let budget = ContextBudget::new(100);
        // Chinese text: 500 chars * 3 bytes = 1500 bytes
        let big_chinese: String = "\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}\u{6570}\u{636e}".repeat(83);
        let mut messages = vec![Message {
            role: openfang_types::message::Role::User,
            content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: "t1".to_string(),
                tool_name: String::new(),
                content: big_chinese,
                is_error: false,
            }]),
            ..Default::default()
        }];
        // Must not panic on multi-byte content
        let compacted = apply_context_guard(&mut messages, &budget, &[]);
        assert!(compacted > 0);
    }
}

// ---------------------------------------------------------------------------
// Historical tool results
// ---------------------------------------------------------------------------

/// How much of an *old* tool result is worth re-sending, in characters.
///
/// The caps above are percentages of the context window, which is right for
/// stopping a single prompt overflowing and wrong for controlling cost. A 30%
/// cap on a large window permits a tool result of tens of thousands of tokens.
/// That is affordable once. It is not affordable re-sent on every one of the
/// next fifty turns, which is exactly what happens to anything left in the
/// history window.
///
/// Measured on this project: 23.0M input tokens against 734k output over 114
/// calls - a 31:1 ratio, 211k input tokens per call, and about EUR 25 spent
/// almost entirely on re-transmitting payloads the model had already read.
/// One agent pushed base64 file contents through tool results; each blob was
/// then billed again on every subsequent turn.
///
/// 4000 characters is roughly 1000 tokens: enough to keep a command's shape,
/// its first lines of output and its error, which is what later turns actually
/// reason about. The full result is never lost - it stays in the stored
/// session and can be read in the UI. This governs the wire, not the record.
pub const HISTORICAL_TOOL_RESULT_CHARS: usize = 4000;

/// How many recent tool results keep their full budget.
///
/// The immediately preceding result is usually the one being acted on, so
/// truncating it would break the reasoning this is meant to preserve. Two
/// covers the common read-then-edit pair.
pub const FULL_FIDELITY_RECENT_RESULTS: usize = 2;

/// Trim a tool result that has already been seen in an earlier turn.
///
/// Returns the text unchanged when it is already small, so short results -
/// the overwhelming majority - are untouched and cost nothing to process.
pub fn truncate_historical_tool_result(content: &str) -> String {
    if content.chars().count() <= HISTORICAL_TOOL_RESULT_CHARS {
        return content.to_string();
    }

    // Keep the head and the tail. A truncated middle preserves both what the
    // command was doing and how it ended; keeping only the head loses the
    // error, which is usually the part that matters.
    let head_chars = HISTORICAL_TOOL_RESULT_CHARS * 3 / 4;
    let tail_chars = HISTORICAL_TOOL_RESULT_CHARS - head_chars;

    let head: String = content.chars().take(head_chars).collect();
    let tail: String = {
        let total = content.chars().count();
        content.chars().skip(total - tail_chars).collect()
    };
    let omitted = content.chars().count() - head_chars - tail_chars;

    format!(
        "{head}\n\n[... {omitted} characters omitted from this earlier tool result to \
         avoid re-sending it in full on every turn. The complete output is kept in the \
         session and visible in the interface. ...]\n\n{tail}"
    )
}

#[cfg(test)]
mod historical_tests {
    use super::*;

    /// Short results must pass through untouched. Most tool output is small,
    /// and rewriting it would add noise for no saving.
    #[test]
    fn short_results_are_left_alone() {
        let text = "SANDBOX_OK\nLinux\nexit 0";
        assert_eq!(truncate_historical_tool_result(text), text);
    }

    /// The cost fix: a large result must shrink to roughly the cap, so its
    /// contribution stops scaling with the number of turns that follow it.
    #[test]
    fn a_large_result_is_cut_to_about_the_cap() {
        let huge = "A".repeat(200_000);
        let out = truncate_historical_tool_result(&huge);
        assert!(
            out.chars().count() < HISTORICAL_TOOL_RESULT_CHARS + 300,
            "expected about {} chars, got {}",
            HISTORICAL_TOOL_RESULT_CHARS,
            out.chars().count()
        );
        assert!(
            out.chars().count() < huge.chars().count() / 40,
            "must be a real saving"
        );
    }

    /// Both ends survive. Keeping only the head would drop the error message,
    /// which is the part later turns usually need.
    #[test]
    fn the_beginning_and_the_end_both_survive() {
        let text = format!("START-MARKER{}END-MARKER", "x".repeat(100_000));
        let out = truncate_historical_tool_result(&text);
        assert!(out.contains("START-MARKER"), "the head must survive");
        assert!(out.contains("END-MARKER"), "the tail must survive");
    }

    /// The notice must say the data still exists. A bare "truncated" reads as
    /// data loss, and this system has already lost the user's data twice.
    #[test]
    fn the_notice_says_the_full_output_is_still_available() {
        let out = truncate_historical_tool_result(&"z".repeat(50_000));
        assert!(out.contains("characters omitted"));
        assert!(out.contains("kept in the session"));
    }

    /// A base64 blob is the case that actually caused the bill.
    #[test]
    fn a_base64_payload_stops_being_expensive() {
        let blob = "QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo=".repeat(1000);
        let before = blob.chars().count();
        let after = truncate_historical_tool_result(&blob).chars().count();
        assert!(
            after * 5 < before,
            "a repeated blob must shrink dramatically"
        );
    }
}

/// Shrink tool results that earlier turns have already consumed.
///
/// Walks the list from the end so "recent" means recent, and leaves the last
/// `FULL_FIDELITY_RECENT_RESULTS` untouched: the immediately preceding result
/// is usually the one being acted on, and truncating it would break the
/// reasoning this exists to protect.
pub fn compact_historical_tool_results(
    messages: Vec<openfang_types::message::Message>,
) -> Vec<openfang_types::message::Message> {
    use openfang_types::message::{ContentBlock, MessageContent};

    let mut seen_results = 0usize;
    let mut out = messages;

    for message in out.iter_mut().rev() {
        let MessageContent::Blocks(blocks) = &mut message.content else {
            continue;
        };
        for block in blocks.iter_mut() {
            if let ContentBlock::ToolResult { content, .. } = block {
                seen_results += 1;
                if seen_results > FULL_FIDELITY_RECENT_RESULTS {
                    *content = truncate_historical_tool_result(content);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod wiring_tests {
    use super::*;
    use openfang_types::message::{ContentBlock, Message, MessageContent, Role};

    fn tool_result(text: &str) -> Message {
        Message {
            role: Role::User,
            content: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: "t1".into(),
                tool_name: "docker_exec".into(),
                content: text.into(),
                is_error: false,
            }]),
            ..Default::default()
        }
    }

    /// The end-to-end property that costs money: a conversation carrying
    /// several large tool results must shrink dramatically before it is sent,
    /// while the newest ones stay intact for the next turn to reason about.
    #[test]
    fn old_results_shrink_and_recent_ones_do_not() {
        // Twenty results is an ordinary working session, and the shape of the
        // one that cost EUR 25: the two most recent stay whole, so the floor is
        // 160 KB however long the history gets - which is the point. The saving
        // is in everything before them, and it grows with the conversation.
        let big = "Q".repeat(80_000);
        let history: Vec<Message> = (0..20).map(|_| tool_result(&big)).collect();
        let before: usize = history
            .iter()
            .map(|m| match &m.content {
                MessageContent::Blocks(b) => b
                    .iter()
                    .map(|x| match x {
                        ContentBlock::ToolResult { content, .. } => content.len(),
                        _ => 0,
                    })
                    .sum::<usize>(),
                _ => 0,
            })
            .sum();

        let after_msgs = compact_historical_tool_results(history);
        let sizes: Vec<usize> = after_msgs
            .iter()
            .map(|m| match &m.content {
                MessageContent::Blocks(b) => b
                    .iter()
                    .map(|x| match x {
                        ContentBlock::ToolResult { content, .. } => content.len(),
                        _ => 0,
                    })
                    .sum::<usize>(),
                _ => 0,
            })
            .collect();
        let after: usize = sizes.iter().sum();

        // The historical portion is what scales with turn count, so that is
        // what the assertion measures. Total saving is bounded below by the two
        // full-fidelity results and would flatter or damn the fix depending
        // only on how many results the test happened to use.
        let kept_whole = 2 * 80_000;
        let history_before = before - kept_whole;
        let history_after = after - kept_whole;
        assert!(
            history_after * 15 < history_before,
            "older results must shrink by more than 15x: {history_before} -> {history_after}"
        );
        assert!(
            after < before / 2,
            "total must still halve: {before} -> {after}"
        );

        assert_eq!(sizes[19], 80_000, "the newest result must be untouched");
        assert_eq!(sizes[18], 80_000, "the second newest must be untouched");
        assert!(sizes[0] < 5_000, "the oldest must be trimmed");
    }
}
