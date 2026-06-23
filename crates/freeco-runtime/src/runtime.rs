use std::sync::{Arc, Mutex};

use agent_core::{
    Agent, AgentContext, AgentResponse, Capability, LearningRecord, LearningStore, Message,
    Outcome, ResponseContent,
};
use budget_engine::BudgetEngine;

use crate::{error::RuntimeError, registry::AgentRegistry};

/// The top-level OpenFang multi-agent orchestrator.
///
/// - Enforces token budgets **before** every agent call.
/// - Follows `RouteToAgent` responses automatically (Supervisor pattern).
/// - Accumulates [`LearningRecord`]s for the self-improvement loop.
pub struct OpenFangRuntime {
    registry: Arc<Mutex<AgentRegistry>>,
    budget: Arc<BudgetEngine>,
    learning: Arc<Mutex<LearningStore>>,
    /// Maximum hop count before treating a routing chain as a loop.
    max_route_depth: usize,
}

impl OpenFangRuntime {
    /// Create a runtime backed by the given budget engine.
    pub fn new(budget: BudgetEngine) -> Self {
        Self {
            registry: Arc::new(Mutex::new(AgentRegistry::new())),
            budget: Arc::new(budget),
            learning: Arc::new(Mutex::new(LearningStore::new())),
            max_route_depth: 4,
        }
    }

    /// Register an agent so the runtime can dispatch messages to it.
    pub fn register(&self, agent: Arc<dyn Agent>) {
        self.registry.lock().unwrap().register(agent);
    }

    /// Dispatch a message to the agent named in `msg.to`.
    ///
    /// Budget is checked before calling `handle()`; `RouteToAgent` responses
    /// are followed in a loop (up to `max_route_depth` hops) so the
    /// Secretary can forward to specialist agents transparently.
    ///
    /// # Arguments
    /// - `msg` – The message to dispatch.
    /// - `user_id` – The user on whose behalf the call is made.
    /// - `session_id` – Opaque conversation session token.
    /// - `tier` – Subscription tier name ("Free", "Concierge", "Business").
    pub async fn dispatch(
        &self,
        msg: Message,
        user_id: &str,
        session_id: &str,
        tier: &str,
    ) -> Result<AgentResponse, RuntimeError> {
        let tokens_remaining = self.budget.tokens_remaining(user_id).unwrap_or(0);

        let mut current_msg = msg;
        let mut depth: usize = 0;

        loop {
            if depth > self.max_route_depth {
                return Err(RuntimeError::RoutingLoop {
                    max_depth: self.max_route_depth,
                });
            }

            // Budget check
            self.budget
                .check_budget(&current_msg.to, user_id)
                .map_err(|e| RuntimeError::BudgetExceeded(e.to_string()))?;

            // Resolve the target agent
            let agent = {
                let reg = self.registry.lock().unwrap();
                reg.get(&current_msg.to)
                    .ok_or_else(|| RuntimeError::AgentNotFound(current_msg.to.clone()))?
            };

            // Build context
            let ctx = AgentContext::new(&current_msg.to, user_id)
                .with_session(session_id)
                .with_tokens(tokens_remaining, tier);

            let start = std::time::Instant::now();
            let resp = agent
                .handle(&ctx, current_msg.clone())
                .await
                .map_err(|e| RuntimeError::AgentError(e.to_string()))?;
            let duration_ms = start.elapsed().as_millis() as u64;

            tracing::debug!(
                agent_id = %resp.from_agent,
                in_reply_to = %resp.in_reply_to,
                duration_ms,
                "agent responded"
            );

            // Account for tokens used by shopping recommendations
            let tokens_used = if let ResponseContent::ShoppingRecommendation(ref rec) = resp.content {
                rec.tokens_used
            } else {
                0
            };

            if tokens_used > 0 {
                let _ = self.budget.record_tokens(
                    &resp.from_agent,
                    user_id,
                    tool_gateway::DEFAULT_MODEL,
                    tokens_used,
                );
            }

            // Record a learning event
            {
                let mut store = self.learning.lock().unwrap();
                let outcome = match &resp.content {
                    ResponseContent::SoftError(_) => Outcome::Failure,
                    _ => Outcome::Success,
                };
                store.push(
                    LearningRecord::new(&resp.from_agent, user_id, session_id)
                        .with_input(&current_msg.id)
                        .with_decision(format!("{:?}", std::mem::discriminant(&resp.content)))
                        .with_outcome(outcome)
                        .with_tokens(tokens_used)
                        .with_duration(duration_ms),
                );
            }

            // Follow RouteToAgent — loop instead of recurse
            if let ResponseContent::RouteToAgent {
                ref forward_message, ..
            } = resp.content
            {
                current_msg = *forward_message.clone();
                depth += 1;
                continue;
            }

            return Ok(resp);
        }
    }

    /// Find the ID of the first registered agent that has `cap`.
    pub fn find_agent_for_capability(&self, cap: &Capability) -> Option<String> {
        self.registry
            .lock()
            .unwrap()
            .find_by_capability(cap)
            .map(|a| a.id().to_string())
    }

    /// Drain and return all accumulated learning records.
    ///
    /// The caller is responsible for persisting these to the learning database.
    pub fn drain_learning_records(&self) -> Vec<LearningRecord> {
        self.learning.lock().unwrap().drain()
    }

    /// All registered agent IDs.
    pub fn agent_ids(&self) -> Vec<String> {
        self.registry.lock().unwrap().agent_ids()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{AgentError, MessageContent, MessageRole};
    use async_trait::async_trait;

    // ── Test double agents ────────────────────────────────────────────────────

    struct EchoAgent(String);

    #[async_trait]
    impl Agent for EchoAgent {
        fn id(&self) -> &str {
            &self.0
        }
        fn name(&self) -> &str {
            "echo"
        }
        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability::GeneralQa]
        }
        async fn handle(
            &self,
            _ctx: &AgentContext,
            msg: Message,
        ) -> Result<AgentResponse, AgentError> {
            let text = match &msg.content {
                MessageContent::Text(t) => format!("echo: {t}"),
                _ => "echo: (non-text)".into(),
            };
            Ok(AgentResponse::text(msg.id, &self.0, text))
        }
    }

    /// An agent that always routes to a target agent.
    struct RouterAgent {
        id: String,
        target: String,
    }

    #[async_trait]
    impl Agent for RouterAgent {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "router"
        }
        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability::TaskRouting]
        }
        async fn handle(
            &self,
            _ctx: &AgentContext,
            msg: Message,
        ) -> Result<AgentResponse, AgentError> {
            let fwd = Message {
                id: format!("{}-fwd", msg.id),
                from: self.id.clone(),
                to: self.target.clone(),
                role: MessageRole::Agent,
                content: msg.content.clone(),
                timestamp_ms: 0,
            };
            Ok(AgentResponse::route(
                msg.id,
                &self.id,
                &self.target,
                "routing test",
                fwd,
            ))
        }
    }

    fn make_runtime() -> OpenFangRuntime {
        OpenFangRuntime::new(BudgetEngine::in_memory().unwrap())
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn dispatch_to_echo_agent() {
        let rt = make_runtime();
        rt.register(Arc::new(EchoAgent("ag-echo".into())));

        let msg = Message::user_text("m-1", "ag-echo", "hello world");
        let resp = rt.dispatch(msg, "user-1", "sess-1", "Free").await.unwrap();

        assert_eq!(resp.from_agent, "ag-echo");
        assert!(matches!(resp.content, ResponseContent::Text(ref t) if t == "echo: hello world"));
    }

    #[tokio::test]
    async fn dispatch_unknown_agent_returns_error() {
        let rt = make_runtime();
        let msg = Message::user_text("m-1", "no-such-agent", "hello");
        let err = rt.dispatch(msg, "u", "s", "Free").await.unwrap_err();
        assert!(matches!(err, RuntimeError::AgentNotFound(_)));
    }

    #[tokio::test]
    async fn routing_follows_route_to_agent() {
        let rt = make_runtime();
        rt.register(Arc::new(RouterAgent {
            id: "ag-router".into(),
            target: "ag-echo".into(),
        }));
        rt.register(Arc::new(EchoAgent("ag-echo".into())));

        let msg = Message::user_text("m-1", "ag-router", "test routing");
        let resp = rt.dispatch(msg, "u", "s", "Free").await.unwrap();

        // Should have been transparently routed to the echo agent
        assert_eq!(resp.from_agent, "ag-echo");
        assert!(matches!(resp.content, ResponseContent::Text(ref t) if t.contains("echo:")));
    }

    #[tokio::test]
    async fn routing_loop_detected() {
        let rt = OpenFangRuntime {
            registry: Arc::new(Mutex::new(AgentRegistry::new())),
            budget: Arc::new(BudgetEngine::in_memory().unwrap()),
            learning: Arc::new(Mutex::new(LearningStore::new())),
            max_route_depth: 1, // very low to trigger loop detection quickly
        };

        // ag-a → ag-b → ag-a (loop)
        rt.register(Arc::new(RouterAgent {
            id: "ag-a".into(),
            target: "ag-b".into(),
        }));
        rt.register(Arc::new(RouterAgent {
            id: "ag-b".into(),
            target: "ag-a".into(),
        }));

        let msg = Message::user_text("m-1", "ag-a", "loop me");
        let err = rt.dispatch(msg, "u", "s", "Free").await.unwrap_err();
        assert!(matches!(err, RuntimeError::RoutingLoop { .. }));
    }

    #[tokio::test]
    async fn learning_records_are_drained() {
        let rt = make_runtime();
        rt.register(Arc::new(EchoAgent("ag-echo".into())));

        let msg = Message::user_text("m-1", "ag-echo", "hi");
        let _ = rt.dispatch(msg, "u", "s", "Free").await.unwrap();

        let records = rt.drain_learning_records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].agent_id, "ag-echo");

        // Drain again — should be empty
        assert!(rt.drain_learning_records().is_empty());
    }

    #[tokio::test]
    async fn find_agent_for_capability() {
        let rt = make_runtime();
        rt.register(Arc::new(EchoAgent("ag-echo".into())));

        let id = rt.find_agent_for_capability(&Capability::GeneralQa).unwrap();
        assert_eq!(id, "ag-echo");

        assert!(rt.find_agent_for_capability(&Capability::Checkout).is_none());
    }

    #[tokio::test]
    async fn multiple_agents_can_be_registered() {
        let rt = make_runtime();
        rt.register(Arc::new(EchoAgent("a1".into())));
        rt.register(Arc::new(EchoAgent("a2".into())));
        rt.register(Arc::new(EchoAgent("a3".into())));
        let mut ids = rt.agent_ids();
        ids.sort();
        assert_eq!(ids, ["a1", "a2", "a3"]);
    }
}
