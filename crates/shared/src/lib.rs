//! Shared business logic and domain psychometric models for Revisited IPIP-NEO.

use serde::{Deserialize, Serialize};

pub mod export;
pub mod questionnaire;

pub use export::*;
pub use questionnaire::*;

pub use spodeian_tokens::ThemeMode;

/// Persistent user-configurable preferences stored across sessions.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub theme: ThemeMode,
    #[serde(default)]
    pub dyslexia_friendly: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            dyslexia_friendly: false,
        }
    }
}

/// Compact summary of a single construct's score captured in a saved profile snapshot.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SnapshotConstructScore {
    pub id: u8,
    pub name: String,
    pub normalized_score: Option<f32>,
    pub standard_error: Option<f32>,
    pub tier_label: Option<String>,
}

/// Maximum number of longitudinal profile snapshots retained in `AppState::profile_history`.
pub const MAX_PROFILE_HISTORY: usize = 50;

/// Longitudinal profile snapshot stored in `AppState::profile_history`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SavedProfileSnapshot {
    pub id: String,
    pub label: String,
    pub timestamp_label: String,
    pub answered_questions: usize,
    pub total_questions: usize,
    pub admin_mode: questionnaire::AdministrationMode,
    pub url_code: String,
    pub meta_scores: Vec<SnapshotConstructScore>,
    pub trait_scores: Vec<SnapshotConstructScore>,
    pub facet_scores: Vec<SnapshotConstructScore>,
}

/// Top-level persistent application state containing user config, questionnaire progress,
/// and longitudinal assessment snapshots.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppState {
    #[serde(default)]
    pub config: AppConfig,
    #[serde(default)]
    pub questionnaire: QuestionnaireState,
    #[serde(default)]
    pub profile_history: Vec<SavedProfileSnapshot>,
}

impl AppState {
    /// Resets all questionnaire answers and state while retaining user config and history.
    pub fn reset_questionnaire(&mut self) {
        self.questionnaire.reset();
    }

    /// Captures a snapshot of the current assessment state into `profile_history`
    /// (retained chronological order, bounded to `MAX_PROFILE_HISTORY` entries).
    pub fn record_profile_snapshot(
        &mut self,
        label: impl Into<String>,
        timestamp_label: impl Into<String>,
    ) -> SavedProfileSnapshot {
        let answered_questions = self.questionnaire.answered_count();
        let total_questions = self.questionnaire.active_total_count();
        let (meta_scores, trait_scores, facet_scores) =
            self.questionnaire.snapshot_construct_scores();
        let url_code = crate::export::encode_responses_to_url_code(&self.questionnaire);
        let next_seq = self
            .profile_history
            .iter()
            .filter_map(|s| {
                s.id.strip_prefix("snap-")
                    .and_then(|rest| rest.split('-').next())
                    .and_then(|n| n.parse::<usize>().ok())
            })
            .max()
            .unwrap_or(0)
            + 1;

        let snapshot = SavedProfileSnapshot {
            id: format!("snap-{}-{}", next_seq, answered_questions),
            label: label.into(),
            timestamp_label: timestamp_label.into(),
            answered_questions,
            total_questions,
            admin_mode: self.questionnaire.admin_mode,
            url_code,
            meta_scores,
            trait_scores,
            facet_scores,
        };

        self.profile_history.push(snapshot.clone());
        if self.profile_history.len() > MAX_PROFILE_HISTORY {
            let excess = self.profile_history.len() - MAX_PROFILE_HISTORY;
            self.profile_history.drain(0..excess);
        }

        snapshot
    }
}
