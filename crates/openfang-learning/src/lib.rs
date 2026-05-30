//! # openfang-learning
//!
//! Self-improving learning loop for OpenFang agents, inspired by the Hermes Agent
//! self-improvement architecture.
//!
//! ## Overview
//!
//! This crate provides a persistent, structured learning loop that allows OpenFang
//! agents to:
//!
//! - **Capture** corrections, knowledge gaps, errors, and best practices during task execution
//! - **Store** learnings in structured Markdown logs and optionally in the OpenFang memory substrate
//! - **Promote** high-value learnings to core agent memory files (SOUL.md, AGENTS.md, TOOLS.md)
//! - **Replay** past learnings to seed new agent sessions with accumulated knowledge
//! - **Score** learnings by recurrence and impact to prioritise promotion
//!
//! ## Architecture
//!
//! ```text
//! Agent Task Execution
//!        │
//!        ▼
//! ┌─────────────────┐    capture()    ┌──────────────────────┐
//! │  LearningLoop   │ ─────────────► │  .learnings/          │
//! │  (this crate)   │                │  ├── LEARNINGS.md      │
//! └─────────────────┘                │  ├── ERRORS.md         │
//!        │                           │  └── FEATURE_REQUESTS.md│
//!        │ promote_to_core_memory()  └──────────────────────┘
//!        ▼
//! ┌─────────────────┐
//! │  Core Memory    │
//! │  ├── SOUL.md    │  ← Agent identity & values
//! │  ├── AGENTS.md  │  ← Multi-agent knowledge
//! │  └── TOOLS.md   │  ← Tool usage patterns
//! └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use openfang_learning::{LearningLoop, LearningType};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut loop_ = LearningLoop::new("/path/to/agent/home").await?;
//!
//!     // Capture a correction after a failed tool call
//!     loop_.capture(LearningType::Correction,
//!         "web_search",
//!         "Tavily requires a non-empty query string; empty queries return 422").await?;
//!
//!     // Capture an error with resolution
//!     loop_.capture_error(
//!         "novita_api",
//!         "HTTP 403 NOT_ENOUGH_BALANCE",
//!         Some("Top up Novita AI account at https://novita.ai/console".into())).await?;
//!
//!     // Promote a high-value learning to core memory
//!     loop_.promote_to_core_memory("SOUL.md",
//!         "Always verify API balance before starting long research tasks.").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod event;
pub mod loop_manager;
pub mod promotion;
pub mod replay;
pub mod scoring;

pub use event::{LearningEvent, LearningType};
pub use loop_manager::LearningLoop;
pub use promotion::PromotionPolicy;
pub use scoring::LearningScore;
