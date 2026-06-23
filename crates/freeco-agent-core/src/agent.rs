use crate::{AgentContext, AgentError, AgentResponse, Capability, Message};

// ── Platform-conditional Send + Sync supertrait ──────────────────────────────
//
// In `wasm32-unknown-unknown` there is no threading, so `Send + Sync` bounds
// are meaningless and would prevent single-threaded JS runtimes from using the
// trait.  On native we require them so agents can be held in `Arc<dyn Agent>`
// across tokio tasks.

#[cfg(not(target_arch = "wasm32"))]
pub trait AgentBounds: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync> AgentBounds for T {}

#[cfg(target_arch = "wasm32")]
pub trait AgentBounds {}
#[cfg(target_arch = "wasm32")]
impl<T> AgentBounds for T {}

// ── Core trait ───────────────────────────────────────────────────────────────

/// The central trait every OpenFang agent must implement.
///
/// On native targets it is `Send + Sync` so it can be stored in an
/// `Arc<dyn Agent>` and dispatched from async tokio tasks.
/// On `wasm32` the bounds are relaxed for single-threaded JS runtimes.
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait Agent: AgentBounds {
    /// Unique identifier for this agent instance (e.g. `"agent-shopping"`).
    fn id(&self) -> &str;

    /// Human-readable display name.
    fn name(&self) -> &str;

    /// The capabilities this agent advertises (used for routing).
    fn capabilities(&self) -> Vec<Capability>;

    /// Process an incoming message and produce a response.
    ///
    /// The `ctx` holds per-request budget state and user preferences.
    /// Returning `AgentResponse` with `ResponseContent::RouteToAgent` tells
    /// the runtime to transparently forward the request to another agent.
    async fn handle(
        &self,
        ctx: &AgentContext,
        msg: Message,
    ) -> Result<AgentResponse, AgentError>;
}
