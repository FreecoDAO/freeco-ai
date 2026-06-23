use std::collections::HashMap;
use std::sync::Arc;

use agent_core::{Agent, Capability};

/// Holds all registered [`Agent`] instances, keyed by their `id()`.
pub struct AgentRegistry {
    agents: HashMap<String, Arc<dyn Agent>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Register an agent.  The agent's `id()` is used as the lookup key.
    /// Registering a second agent with the same ID overwrites the first.
    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        let id = agent.id().to_string();
        tracing::info!(agent_id = %id, name = %agent.name(), "registered agent");
        self.agents.insert(id, agent);
    }

    /// Look up an agent by exact ID.
    pub fn get(&self, id: &str) -> Option<Arc<dyn Agent>> {
        self.agents.get(id).cloned()
    }

    /// Find the first agent that advertises the given capability.
    ///
    /// Iteration order is non-deterministic (HashMap); for stable routing
    /// prefer explicit agent IDs.
    pub fn find_by_capability(&self, cap: &Capability) -> Option<Arc<dyn Agent>> {
        self.agents
            .values()
            .find(|a| a.capabilities().contains(cap))
            .cloned()
    }

    /// All registered agent IDs (order non-deterministic).
    pub fn agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{AgentContext, AgentError, AgentResponse, Message};
    use async_trait::async_trait;

    struct DummyAgent {
        id: String,
        caps: Vec<Capability>,
    }

    #[async_trait]
    impl Agent for DummyAgent {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            "dummy"
        }
        fn capabilities(&self) -> Vec<Capability> {
            self.caps.clone()
        }
        async fn handle(&self, _ctx: &AgentContext, msg: Message) -> Result<AgentResponse, AgentError> {
            Ok(AgentResponse::text(msg.id, &self.id, "ok"))
        }
    }

    fn make_agent(id: &str, caps: Vec<Capability>) -> Arc<dyn Agent> {
        Arc::new(DummyAgent { id: id.into(), caps })
    }

    #[test]
    fn register_and_get() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("ag-1", vec![]));
        assert!(reg.get("ag-1").is_some());
        assert!(reg.get("ag-2").is_none());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn find_by_capability_returns_matching_agent() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("ag-a", vec![Capability::ProductResearch]));
        reg.register(make_agent("ag-b", vec![Capability::TaskRouting]));

        let found = reg.find_by_capability(&Capability::ProductResearch).unwrap();
        assert_eq!(found.id(), "ag-a");

        assert!(reg.find_by_capability(&Capability::Checkout).is_none());
    }

    #[test]
    fn overwriting_agent_replaces_old_one() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("ag-1", vec![]));
        reg.register(make_agent("ag-1", vec![Capability::GeneralQa]));
        let ag = reg.get("ag-1").unwrap();
        assert!(ag.capabilities().contains(&Capability::GeneralQa));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn agent_ids_lists_all() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("x", vec![]));
        reg.register(make_agent("y", vec![]));
        let mut ids = reg.agent_ids();
        ids.sort();
        assert_eq!(ids, ["x", "y"]);
    }
}
