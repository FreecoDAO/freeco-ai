//! # agent-ceo
//!
//! Freeco.AI CEO Agent — thin bridge to the Manus AI MCP API.
//!
//! Without a Manus API key the agent returns deterministic stub responses so
//! the full agent graph can be tested end-to-end without external deps.
//!
//! Implements [`agent_core::Agent`] and compiles to both native and `wasm32`.

pub mod agent;
pub mod error;
pub mod manus_client;
pub mod types;

pub use agent::CeoAgent;
pub use error::CeoError;
pub use manus_client::ManusClient;
pub use types::{Action, CeoDecision, Directive};
