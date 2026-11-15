//! Unified multi-tiered storage engine, persistence manager, PWA install bridge, and diagnostics.
//!
//! Core storage routing, diagnostics, download triggers, and JSON/Snapshot deserialization
//! are delegated to `spodeian_persist`, while retaining backwards-compatible RON fallback
//! for legacy desktop and `eframe::Storage` entries.

pub use spodeian_persist::{
    StorageBackend, StorageDiagnostics, StorageError, query_storage_diagnostics,
    request_persistent_storage, trigger_binary_download, trigger_pwa_install,
    trigger_text_download,
};

use tracing::{info, warn};

/// Primary dedicated storage key used across `eframe::Storage`, `localStorage`, and disk backups.
pub const DEDICATED_STORAGE_KEY: &str = "revisited_ipip_neo_state";

/// Dual-format deserializer for [`shared::AppState`], delegating JSON/VersionedSnapshot
/// parsing to `spodeian_persist::StateMigrator` with legacy RON fallback.
pub fn deserialize_app_state(content: &str) -> Result<shared::AppState, StorageError> {
    spodeian_persist::StateMigrator::deserialize_with_fallback(content, |trimmed| {
        ron::from_str::<shared::AppState>(trimmed).map_err(|e| e.to_string())
    })
}

fn finalize_loaded_state(mut state: shared::AppState) -> shared::AppState {
    if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
        state.questionnaire.show_results = true;
    }
    state.questionnaire.rebuild_cache();
    state
}

/// Multi-tiered loader for [`shared::AppState`].
/// Checks explicit `eframe::Storage` first (JSON & RON), followed by `spodeian_persist`
/// multi-tier keys and disk / `localStorage` fallback.
pub fn load_state_multi_tier(storage: Option<&dyn eframe::Storage>) -> Option<shared::AppState> {
    if let Some(storage) = storage {
        for &key in &[DEDICATED_STORAGE_KEY, eframe::APP_KEY] {
            if let Some(raw) = storage.get_string(key) {
                match deserialize_app_state(&raw) {
                    Ok(state) => {
                        info!("Restored assessment state from eframe storage [{}]", key);
                        return Some(finalize_loaded_state(state));
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse assessment state from eframe storage [{}]: {}",
                            key, e
                        );
                    }
                }
            }
        }

        if let Some(state) = eframe::get_value::<shared::AppState>(storage, eframe::APP_KEY) {
            info!("Restored assessment state from eframe get_value");
            return Some(finalize_loaded_state(state));
        }
    }

    let disk_filename = format!("{}.json", DEDICATED_STORAGE_KEY);
    let disk_path = std::path::Path::new(&disk_filename);
    if let Some(state) = spodeian_persist::load_state_multi_tier_keys::<shared::AppState>(
        None,
        &[DEDICATED_STORAGE_KEY, eframe::APP_KEY],
        Some(disk_path),
    ) {
        info!("Restored assessment state via spodeian_persist multi-tier loader");
        return Some(finalize_loaded_state(state));
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string(disk_path) {
        match deserialize_app_state(&content) {
            Ok(state) => {
                info!(
                    "Restored legacy RON state from disk file [{}]",
                    disk_filename
                );
                return Some(finalize_loaded_state(state));
            }
            Err(e) => {
                warn!(
                    "Failed to parse assessment state from disk file [{}]: {}",
                    disk_filename, e
                );
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(local_storage)) = window.local_storage() {
            for &key in &[DEDICATED_STORAGE_KEY, eframe::APP_KEY] {
                if let Ok(Some(content)) = local_storage.get_item(key) {
                    match deserialize_app_state(&content) {
                        Ok(state) => {
                            info!("Restored legacy RON state from localStorage [{}]", key);
                            return Some(finalize_loaded_state(state));
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse assessment state from localStorage [{}]: {}",
                                key, e
                            );
                        }
                    }
                }
            }
        }
    }

    None
}

/// Saves serialized state JSON via `spodeian_persist` preferential multi-tier routing.
pub fn save_state_multi_tier(key: &str, json_str: &str) -> Result<StorageBackend, StorageError> {
    let filename = format!("{}.json", key);
    spodeian_persist::save_state_multi_tier(
        key,
        json_str,
        "ipip_neo_cas_store",
        Some(std::path::Path::new(&filename)),
    )
}
