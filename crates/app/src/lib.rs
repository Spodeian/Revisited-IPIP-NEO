#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]

pub mod app;
pub mod storage_manager;
pub mod types;
pub mod ui;

pub use app::PersonalityApp;
pub use storage_manager::*;
pub use types::{ExportFormat, ScreenConstraints};
