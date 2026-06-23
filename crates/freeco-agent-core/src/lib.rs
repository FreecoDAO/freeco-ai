//! # agent-core
//!
//! The foundational abstraction layer for all OpenFang agents.
//!
//! Compiles to both **native** (via `rlib`) and **`wasm32-unknown-unknown`**
//! (via `cdylib`), enabling the same agent logic to run on:
//! - the server-side [`openfang-runtime`] orchestrator (native + tokio)
//! - the browser / edge via `wasm-pack` / Cloudflare Workers
//!
//! # Design
//!
//! Every agent implements [`Agent`].  Messages flow through the runtime as
//! [`Message`] values carrying a typed [`MessageContent`] payload.  Agents
//! return [`AgentResponse`] values; a `RouteToAgent` variant lets the
//! runtime transparently forward requests to specialist agents (the
//! Supervisor pattern used by the Secretary agent).
//!
//! The [`LearningStore`] collects outcome records during a session; the
//! native runtime drains and persists them for the self-improvement loop.

pub mod agent;
pub mod capability;
pub mod context;
pub mod error;
pub mod learning;
pub mod message;
pub mod openfang_bridge;
pub mod response;
mod time_util;

pub use agent::Agent;
pub use capability::Capability;
pub use context::AgentContext;
pub use error::AgentError;
pub use learning::{LearningRecord, LearningStore, Outcome};
pub use message::{Message, MessageContent, MessageRole, Priority};
pub use openfang_bridge::{from_openfang_message, to_openfang_message, OpenFangAgentBridge};
pub use response::{AgentResponse, RecommendedProduct, ResponseContent, ShoppingRecommendation};
