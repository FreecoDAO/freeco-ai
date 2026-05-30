//! Learning event types and data structures.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The category of a captured learning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningType {
    /// The agent made a mistake and was corrected.
    Correction,
    /// The agent encountered a topic it had no knowledge of.
    KnowledgeGap,
    /// A pattern that worked well and should be repeated.
    BestPractice,
    /// A runtime or API error with optional resolution.
    Error,
    /// A capability the agent or user wishes existed.
    FeatureRequest,
    /// A security or privacy concern observed during execution.
    SecurityObservation,
}

impl LearningType {
    /// Returns the human-readable label used in Markdown logs.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Correction => "CORRECTION",
            Self::KnowledgeGap => "KNOWLEDGE GAP",
            Self::BestPractice => "BEST PRACTICE",
            Self::Error => "ERROR",
            Self::FeatureRequest => "FEATURE REQUEST",
            Self::SecurityObservation => "SECURITY OBSERVATION",
        }
    }

    /// Returns the target log filename for this learning type.
    pub fn log_file(&self) -> &'static str {
        match self {
            Self::Error => "ERRORS.md",
            Self::FeatureRequest => "FEATURE_REQUESTS.md",
            Self::SecurityObservation => "SECURITY.md",
            _ => "LEARNINGS.md",
        }
    }
}

/// A single captured learning or error event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Unique identifier for this event.
    pub id: String,
    /// ISO 8601 timestamp of when this event was captured.
    pub timestamp: String,
    /// Unix epoch seconds (for sorting and scoring).
    pub timestamp_unix: i64,
    /// The category of this learning.
    pub learning_type: LearningType,
    /// The context in which this learning occurred (e.g., tool name, agent name).
    pub context: String,
    /// The content of the learning — what was observed or learned.
    pub content: String,
    /// Optional resolution or corrective action taken.
    pub resolution: Option<String>,
    /// How many times this same learning has been observed (recurrence counter).
    pub recurrence: u32,
    /// Impact score 0–10 assigned by the agent or heuristic.
    pub impact: u8,
}

impl LearningEvent {
    /// Create a new learning event with default recurrence and impact.
    pub fn new(
        learning_type: LearningType,
        context: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("L-{}", Uuid::new_v4().simple()),
            timestamp: now.to_rfc3339(),
            timestamp_unix: now.timestamp(),
            learning_type,
            context: context.into(),
            content: content.into(),
            resolution: None,
            recurrence: 1,
            impact: 5,
        }
    }

    /// Render this event as a Markdown section for log files.
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "\n### [{label}] `{id}` — {ts}\n\
             **Context:** `{ctx}`  \n\
             **Details:** {content}  \n",
            label = self.learning_type.label(),
            id = self.id,
            ts = self.timestamp,
            ctx = self.context,
            content = self.content,
        );
        if let Some(res) = &self.resolution {
            md.push_str(&format!("**Resolution:** {res}  \n"));
        }
        md.push_str(&format!(
            "**Recurrence:** {}  **Impact:** {}/10\n",
            self.recurrence, self.impact
        ));
        md
    }
}
