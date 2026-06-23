//! # agent-secretary
//!
//! Freeco.AI CEO Secretary agent (OpenClaw).
//!
//! The Secretary is the **primary entry point** for user requests.  It:
//! 1. Classifies incoming text using LLM (with a fast keyword heuristic
//!    fallback for low-token / offline scenarios).
//! 2. Routes shopping intents to `agent-shopping`.
//! 3. Handles memo / calendar requests directly.
//! 4. Answers general questions via a brief LLM reply.
//!
//! Implements [`agent_core::Agent`] and compiles to both native and `wasm32`.

pub mod agent;
pub mod classifier;
pub mod error;

pub use agent::SecretaryAgent;
pub use classifier::{classify_intent_heuristic, Intent};
pub use error::SecretaryError;
