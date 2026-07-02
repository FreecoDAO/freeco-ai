//! Replay module — loads past learnings to seed new agent sessions.
//!
//! The replay system injects accumulated learnings into the agent's system
//! prompt or memory substrate at session start, enabling true persistent
//! self-improvement across sessions.

use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// Replay configuration controlling how much history is injected.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Maximum number of characters to inject from learning logs.
    /// Prevents context window overflow. Default: 4000 chars.
    pub max_chars: usize,
    /// Whether to include the ERRORS.md log in replay. Default: true.
    pub include_errors: bool,
    /// Whether to include the SECURITY.md log in replay. Default: true.
    pub include_security: bool,
    /// Whether to include FEATURE_REQUESTS.md in replay. Default: false.
    pub include_feature_requests: bool,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            max_chars: 4000,
            include_errors: true,
            include_security: true,
            include_feature_requests: false,
        }
    }
}

/// Load accumulated learnings from an agent directory for injection into a new session.
///
/// Returns a Markdown-formatted string suitable for inclusion in a system prompt
/// or memory substrate seed.
pub async fn load_session_context(
    agent_dir: &Path,
    config: &ReplayConfig,
) -> Result<String> {
    let learnings_dir = agent_dir.join(".learnings");
    let mut parts: Vec<String> = Vec::new();

    // Always include core learnings
    append_file(&learnings_dir, "LEARNINGS.md", "Past Learnings & Best Practices", &mut parts).await;

    if config.include_errors {
        append_file(&learnings_dir, "ERRORS.md", "Known Errors & Resolutions", &mut parts).await;
    }

    if config.include_security {
        append_file(&learnings_dir, "SECURITY.md", "Security Observations", &mut parts).await;
    }

    if config.include_feature_requests {
        append_file(&learnings_dir, "FEATURE_REQUESTS.md", "Feature Requests", &mut parts).await;
    }

    let combined = parts.join("\n");

    // Truncate to max_chars to avoid context overflow
    if combined.len() > config.max_chars {
        let truncated = &combined[..config.max_chars];
        // Find the last complete line
        let last_newline = truncated.rfind('\n').unwrap_or(config.max_chars);
        Ok(format!(
            "{}\n\n*[Learning history truncated at {} chars]*",
            &combined[..last_newline],
            config.max_chars
        ))
    } else {
        Ok(combined)
    }
}

async fn append_file(dir: &Path, filename: &str, heading: &str, parts: &mut Vec<String>) {
    let path = dir.join(filename);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path).await {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                parts.push(format!("## {heading}\n{trimmed}"));
            }
        }
    }
}
