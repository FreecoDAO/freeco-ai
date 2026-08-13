//! Scoring heuristics for deciding when a learning should be promoted to core memory.

use crate::event::{LearningEvent, LearningType};
use crate::promotion::PromotionPolicy;

/// A computed score for a learning event.
#[derive(Debug, Clone)]
pub struct LearningScore {
    /// Recurrence component (0–40 points).
    pub recurrence_score: f32,
    /// Impact component (0–50 points).
    pub impact_score: f32,
    /// Type bonus — security and corrections score higher (0–10 points).
    pub type_bonus: f32,
    /// Total composite score (0–100).
    pub total: f32,
}

impl LearningScore {
    /// Compute a score for the given learning event.
    pub fn compute(event: &LearningEvent) -> Self {
        // Recurrence: logarithmic scaling, max 40 points at recurrence=10+
        let recurrence_score = (event.recurrence as f32).ln_1p() / (10_f32).ln_1p() * 40.0;
        let recurrence_score = recurrence_score.min(40.0);

        // Impact: linear 0–10 → 0–50 points
        let impact_score = (event.impact as f32 / 10.0) * 50.0;

        // Type bonus
        let type_bonus = match event.learning_type {
            LearningType::SecurityObservation => 10.0,
            LearningType::Correction => 5.0,
            LearningType::Error => 4.0,
            LearningType::BestPractice => 3.0,
            LearningType::KnowledgeGap => 2.0,
            LearningType::FeatureRequest => 1.0,
        };

        let total = (recurrence_score + impact_score + type_bonus).min(100.0);

        Self {
            recurrence_score,
            impact_score,
            type_bonus,
            total,
        }
    }

    /// Returns true if this score exceeds the policy's promotion threshold.
    pub fn should_promote(&self, policy: &PromotionPolicy) -> bool {
        self.total >= policy.promotion_threshold
    }
}
