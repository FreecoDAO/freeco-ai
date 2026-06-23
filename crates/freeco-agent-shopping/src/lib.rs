//! # agent-shopping
//!
//! Freeco.AI Shopping Concierge agent.
//!
//! For each query it:
//! 1. Runs a Tavily web search for product offers
//! 2. Queries Open Food Facts for nutritional / eco-score data
//! 3. Finds nearby stores via Google Places (when a location is known)
//! 4. Synthesises a **three-tier recommendation** (Budget / Value / Luxury)
//!    using Google Gemma 4 26B via Novita AI
//!
//! Implements [`agent_core::Agent`] and compiles to both native and `wasm32`.

pub mod agent;
pub mod error;
pub mod research;

pub use agent::ShoppingAgent;
pub use error::ShoppingError;
