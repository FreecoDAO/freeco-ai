use serde::{Deserialize, Serialize};

use crate::time_util::now_ms;

/// Whether an agent action had the desired outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    PartialSuccess,
    Failure,
    Unknown,
}

/// A single learning event emitted by an agent after handling a message.
///
/// The native [`openfang_runtime`] collects these and persists them to SQLite
/// for the **openfang-learning** self-improvement loop: analysing which
/// decisions lead to good outcomes lets the system gradually improve its
/// routing, prompt selection, and tier classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecord {
    pub agent_id: String,
    pub user_id: String,
    pub session_id: String,
    /// Short summary of the input that triggered this decision.
    pub input_summary: String,
    /// What the agent decided to do (route, answer, delegate, etc.).
    pub decision: String,
    /// Known outcome at time of recording.
    pub outcome: Outcome,
    /// Tokens consumed by this interaction.
    pub tokens_used: u64,
    /// Wall-clock time in milliseconds.
    pub duration_ms: u64,
    /// Arbitrary structured metadata (e.g. tier chosen, intent label).
    pub metadata: serde_json::Value,
    /// Recording timestamp (ms since epoch).
    pub timestamp_ms: i64,
}

impl LearningRecord {
    pub fn new(
        agent_id: impl Into<String>,
        user_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            user_id: user_id.into(),
            session_id: session_id.into(),
            input_summary: String::new(),
            decision: String::new(),
            outcome: Outcome::Unknown,
            tokens_used: 0,
            duration_ms: 0,
            metadata: serde_json::Value::Null,
            timestamp_ms: now_ms(),
        }
    }

    pub fn with_input(mut self, summary: impl Into<String>) -> Self {
        self.input_summary = summary.into();
        self
    }

    pub fn with_decision(mut self, decision: impl Into<String>) -> Self {
        self.decision = decision.into();
        self
    }

    pub fn with_outcome(mut self, outcome: Outcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn with_tokens(mut self, tokens: u64) -> Self {
        self.tokens_used = tokens;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn with_metadata(mut self, meta: serde_json::Value) -> Self {
        self.metadata = meta;
        self
    }
}

/// In-memory accumulator for learning records produced during a session.
///
/// The native runtime calls [`drain`] after every dispatch and persists
/// the records to SQLite. The WASM runtime may choose to send them over
/// a channel to the host page.
#[derive(Debug, Default)]
pub struct LearningStore {
    records: Vec<LearningRecord>,
}

impl LearningStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new record into the store.
    pub fn push(&mut self, record: LearningRecord) {
        self.records.push(record);
    }

    /// Drain all accumulated records, leaving the store empty.
    pub fn drain(&mut self) -> Vec<LearningRecord> {
        std::mem::take(&mut self.records)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_builder_sets_all_fields() {
        let rec = LearningRecord::new("agent-shopping", "user-1", "sess-A")
            .with_input("buy oat milk")
            .with_decision("product_research")
            .with_outcome(Outcome::Success)
            .with_tokens(512)
            .with_duration(340)
            .with_metadata(serde_json::json!({"tier": "value"}));

        assert_eq!(rec.agent_id, "agent-shopping");
        assert_eq!(rec.input_summary, "buy oat milk");
        assert_eq!(rec.decision, "product_research");
        assert_eq!(rec.outcome, Outcome::Success);
        assert_eq!(rec.tokens_used, 512);
        assert_eq!(rec.duration_ms, 340);
        assert_eq!(rec.metadata["tier"], "value");
    }

    #[test]
    fn store_drain_clears_records() {
        let mut store = LearningStore::new();
        store.push(LearningRecord::new("ag", "u", "s"));
        store.push(LearningRecord::new("ag", "u", "s"));
        assert_eq!(store.len(), 2);
        let drained = store.drain();
        assert_eq!(drained.len(), 2);
        assert!(store.is_empty());
    }

    #[test]
    fn empty_store_drain_returns_empty_vec() {
        let mut store = LearningStore::new();
        assert!(store.drain().is_empty());
    }

    #[test]
    fn record_roundtrips_through_json() {
        let rec = LearningRecord::new("ag", "u", "s")
            .with_outcome(Outcome::PartialSuccess);
        let json = serde_json::to_string(&rec).unwrap();
        let back: LearningRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.outcome, Outcome::PartialSuccess);
    }
}
