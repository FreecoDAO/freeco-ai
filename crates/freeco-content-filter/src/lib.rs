//! # freeco-content-filter
//!
//! Content moderation and safety layer for FreEco Kids.
//!
//! Provides a fast, zero-token keyword/pattern pre-filter that runs *before*
//! any message reaches the LLM, and a post-filter that validates responses
//! before delivery to the child.
//!
//! ## Design
//!
//! Three-layer defence:
//! 1. **Keyword pre-filter** (this crate) — fast regex scan, zero API cost
//! 2. **System prompt persona** — LLM-level guardrails (in agent.toml)
//! 3. **Post-filter** — validates LLM output before showing to child
//!
//! All filtering is configurable via [`FilterConfig`] and respects the
//! `[parental_controls]` section in the agent configuration.

mod filter;
mod wordlists;

pub use filter::{ContentFilter, FilterAction, FilterConfig, FilterResult, Severity};
