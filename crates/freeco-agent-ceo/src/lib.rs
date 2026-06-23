//! # agent-ceo
//!
//! Freeco.AI CEO Agent — native executive routing using OpenFang kernel handles.
//!
//! Implements [`agent_core::Agent`] and compiles to both native and `wasm32`.

pub mod agent;
pub mod error;
pub mod types;

pub use agent::CeoAgent;
pub use error::CeoError;
pub use types::{Action, CeoDecision, Directive};
