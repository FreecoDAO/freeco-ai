use serde::{Deserialize, Serialize};

use crate::manifest::ToolName;

/// Emitted for every tool call attempt — allowed or denied.
/// Used by the OpenFang event log and admin dashboard tracing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub tool: ToolName,
    pub agent_id: String,
    pub allowed: bool,
    pub input_summary: String,
    pub result_count: Option<usize>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub timestamp: String,
}
