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

// ---------------------------------------------------------------------------
// Browser persistence: the whole state is one IndexedDB record, loaded once at startup
// and saved in the background (`spodeian_persist::WebStateStore`). Earlier versions
// saved large states only to IndexedDB but never read them back, so a reload restored an
// older local storage copy.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_STORE: std::cell::RefCell<Option<spodeian_persist::WebStateStore>> =
        const { std::cell::RefCell::new(None) };
}

/// Starts loading the saved state from IndexedDB (browser only). The result arrives
/// through [`take_loaded_web_state`]; `ctx` is repainted when it does.
pub fn open_web_store(ctx: &eframe::egui::Context) {
    #[cfg(target_arch = "wasm32")]
    {
        let ctx = ctx.clone();
        let legacy = spodeian_persist::LegacyLocations {
            local_storage_keys: vec![DEDICATED_STORAGE_KEY.into(), eframe::APP_KEY.into()],
            databases: vec![],
        };
        let store = spodeian_persist::WebStateStore::open(
            "revisited-ipip-neo-state",
            DEDICATED_STORAGE_KEY,
            legacy,
            move || ctx.request_repaint(),
        );
        WEB_STORE.with(|s| *s.borrow_mut() = Some(store));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ctx;
}

/// The state loaded from IndexedDB, once: `Some(None)` means nothing was saved there yet
/// (save the current state to move it in). Always `None` on native.
pub fn take_loaded_web_state() -> Option<Option<shared::AppState>> {
    #[cfg(target_arch = "wasm32")]
    return WEB_STORE
        .with(|s| s.borrow().as_ref().and_then(|store| store.take_loaded()))
        .map(|saved| {
            saved.and_then(|json| {
                deserialize_app_state(&json)
                    .map_err(|e| tracing::warn!("Ignoring unreadable saved state: {e}"))
                    .ok()
            })
        });
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// The result of the most recent finished browser save, once: `Ok(())` when IndexedDB
/// confirmed it. Always `None` on native.
pub fn take_web_save_outcome() -> Option<Result<(), String>> {
    #[cfg(target_arch = "wasm32")]
    return WEB_STORE
        .with(|s| s.borrow().as_ref().and_then(|store| store.take_outcome()))
        .map(|outcome| match outcome {
            spodeian_persist::SaveOutcome::Saved => Ok(()),
            spodeian_persist::SaveOutcome::Failed(message) => Err(message),
            other => Err(format!("{other:?}")),
        });
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// Queues `json` to be written to IndexedDB. Returns `false` (doing nothing) on native or
/// before [`open_web_store`] ran, so the caller can fall back to its native save.
pub fn queue_web_save(json: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    return WEB_STORE.with(|s| {
        s.borrow().as_ref().is_some_and(|store| {
            store.queue_save(json.to_string());
            true
        })
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = json;
        false
    }
}
