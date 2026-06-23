//! # openfang-runtime
//!
//! The **native** multi-agent orchestrator for the OpenFang platform.
//!
//! Wires together:
//! - [`agent_core::Agent`] implementations via the [`AgentRegistry`]
//! - [`budget_engine::BudgetEngine`] for per-user token budget enforcement
//! - The [`openfang-learning`] self-improvement loop (learning record drain)
//!
//! ## Architecture
//!
//! ```text
//! User request
//!     │
//!     ▼
//! OpenFangRuntime::dispatch()
//!     │  checks budget
//!     │  finds agent in AgentRegistry
//!     │  calls Agent::handle(ctx, msg)
//!     │
//!     ├─► RouteToAgent → recursive dispatch (Supervisor pattern)
//!     │
//!     └─► AgentResponse returned to caller
//! ```
//!
//! This crate is **native-only** (`tokio`, `rusqlite`).
//! For WASM execution see the individual agent crates.

pub mod error;
pub mod registry;
pub mod runtime;

pub use error::RuntimeError;
pub use registry::AgentRegistry;
pub use runtime::OpenFangRuntime;
