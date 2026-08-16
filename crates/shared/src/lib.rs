//! Shared business logic and domain psychometric models for Revisited IPIP-NEO.

use serde::{Deserialize, Serialize};

pub mod export;
pub mod questionnaire;

pub use export::*;
pub use questionnaire::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub theme: ThemeMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppState {
    #[serde(default)]
    pub config: AppConfig,
    #[serde(default)]
    pub questionnaire: QuestionnaireState,
}

impl AppState {
    /// Resets all questionnaire answers and state while retaining user config (e.g. theme).
    pub fn reset_questionnaire(&mut self) {
        self.questionnaire.reset();
    }
}
