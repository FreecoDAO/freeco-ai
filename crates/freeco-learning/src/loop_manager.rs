//! The main `LearningLoop` struct — the primary public API for self-improving agents.

use crate::event::{LearningEvent, LearningType};
use crate::promotion::PromotionPolicy;
use crate::scoring::LearningScore;
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

/// The central self-improving learning loop for an OpenFang agent.
///
/// Manages capture, storage, scoring, and promotion of learnings
/// to persistent memory files.
pub struct LearningLoop {
    /// Root directory for this agent (e.g., `~/.openfang/agents/<agent_id>/`)
    agent_dir: PathBuf,
    /// Directory where learning logs are stored.
    learnings_dir: PathBuf,
    /// Policy controlling when learnings are auto-promoted to core memory.
    pub promotion_policy: PromotionPolicy,
}

impl LearningLoop {
    /// Initialise a new `LearningLoop` for an agent at the given directory.
    ///
    /// Creates the `.learnings/` subdirectory if it does not exist.
    pub async fn new(agent_dir: impl Into<PathBuf>) -> Result<Self> {
        let agent_dir = agent_dir.into();
        let learnings_dir = agent_dir.join(".learnings");
        fs::create_dir_all(&learnings_dir).await?;

        info!("LearningLoop initialised at {:?}", learnings_dir);

        Ok(Self {
            agent_dir,
            learnings_dir,
            promotion_policy: PromotionPolicy::default(),
        })
    }

    /// Capture a learning event and persist it to the appropriate log file.
    ///
    /// If the promotion policy threshold is met, the learning is automatically
    /// promoted to the agent's core memory files.
    pub async fn capture(
        &mut self,
        learning_type: LearningType,
        context: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<LearningEvent> {
        let event = LearningEvent::new(learning_type, context, content);
        debug!(
            "Capturing learning: {} in context '{}'",
            event.id, event.context
        );

        self.append_to_log(&event).await?;

        // Auto-promote if the policy threshold is met
        let score = LearningScore::compute(&event);
        if score.should_promote(&self.promotion_policy) {
            info!(
                "Auto-promoting learning {} (score={:.1})",
                event.id, score.total
            );
            let target = self.promotion_policy.target_file(&event.learning_type);
            self.promote_to_core_memory(&target, &event.to_markdown())
                .await?;
        }

        Ok(event)
    }

    /// Capture an error event with an optional resolution.
    pub async fn capture_error(
        &mut self,
        context: impl Into<String>,
        error_msg: impl Into<String>,
        resolution: Option<String>,
    ) -> Result<LearningEvent> {
        let mut event = LearningEvent::new(LearningType::Error, context, error_msg);
        event.resolution = resolution;
        event.impact = 7; // Errors have higher default impact

        self.append_to_log(&event).await?;
        Ok(event)
    }

    /// Capture a security observation (always high impact, always promoted).
    pub async fn capture_security(
        &mut self,
        context: impl Into<String>,
        observation: impl Into<String>,
    ) -> Result<LearningEvent> {
        let mut event = LearningEvent::new(LearningType::SecurityObservation, context, observation);
        event.impact = 10;

        self.append_to_log(&event).await?;
        // Security observations are always promoted immediately
        self.promote_to_core_memory("SECURITY.md", &event.to_markdown())
            .await?;
        warn!("Security observation captured and promoted: {}", event.id);
        Ok(event)
    }

    /// Promote a learning directly to a core memory file.
    ///
    /// Core memory files (e.g., `SOUL.md`, `AGENTS.md`, `TOOLS.md`) are read
    /// by the agent at the start of every session to seed its context.
    pub async fn promote_to_core_memory(&self, target_filename: &str, content: &str) -> Result<()> {
        let file_path = self.agent_dir.join(target_filename);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        let entry = format!(
            "\n<!-- Promoted by openfang-learning at {} -->\n{}\n",
            chrono::Utc::now().to_rfc3339(),
            content
        );
        file.write_all(entry.as_bytes()).await?;
        info!("Promoted learning to {:?}", file_path);
        Ok(())
    }

    /// Load all learnings from the log files for replay into a new session.
    ///
    /// Returns a concatenated Markdown string suitable for injection into
    /// the agent's system prompt or memory substrate.
    pub async fn load_for_replay(&self) -> Result<String> {
        let mut combined = String::new();
        for filename in &[
            "LEARNINGS.md",
            "ERRORS.md",
            "FEATURE_REQUESTS.md",
            "SECURITY.md",
        ] {
            let path = self.learnings_dir.join(filename);
            if path.exists() {
                let content = fs::read_to_string(&path).await?;
                if !content.trim().is_empty() {
                    combined.push_str(&format!("\n## {filename}\n{content}\n"));
                }
            }
        }
        Ok(combined)
    }

    /// Internal: append a learning event to its target log file.
    async fn append_to_log(&self, event: &LearningEvent) -> Result<()> {
        let file_path = self.learnings_dir.join(event.learning_type.log_file());

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .await?;

        file.write_all(event.to_markdown().as_bytes()).await?;
        Ok(())
    }
}
