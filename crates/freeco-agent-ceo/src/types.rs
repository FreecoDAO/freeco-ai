use serde::{Deserialize, Serialize};

use agent_core::message::Priority;

/// An executive directive submitted to the CEO agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub instruction: String,
    pub priority: Priority,
    /// Optional background context for the decision.
    pub context: Option<String>,
}

/// An action item within an executive decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub description: String,
    /// If set, this action is delegated to the named agent.
    pub delegate_to: Option<String>,
    pub priority: Priority,
}

/// The structured response returned by the CEO agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeoDecision {
    pub summary: String,
    pub reasoning: String,
    pub actions: Vec<Action>,
}
