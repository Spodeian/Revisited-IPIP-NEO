//! Unified multi-tiered storage engine, persistence manager, PWA install bridge, and diagnostics.

use serde::{Deserialize, Serialize};
use thiserror::Error;
#[allow(unused_imports)]
use tracing::{error, info, warn};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Strongly typed storage errors replacing ad-hoc string reporting.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage content is empty")]
    EmptyContent,

    #[error("Failed to deserialize assessment state: JSON ({json_err}), RON ({ron_err})")]
    Deserialization {
        json_err: serde_json::Error,
        ron_err: ron::error::SpannedError,
    },

    #[cfg(target_arch = "wasm32")]
    #[error("Web storage quota exceeded or access restricted: {0}")]
    QuotaExceeded(String),

    #[cfg(target_arch = "wasm32")]
    #[error("Browser window localStorage is unavailable in this context")]
    LocalStorageUnavailable,

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to write assessment state to desktop file '{path}': {source}")]
    DiskWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StorageBackend {
    #[default]
    LocalStorage,
    DiskFile,
    MemoryOnly,
}

impl StorageBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalStorage => "Local Storage (Fast Tier)",
            Self::DiskFile => "Local File System",
            Self::MemoryOnly => "In-Memory Only (Ephemeral)",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StorageDiagnostics {
    pub is_persisted: Option<bool>,
    pub pwa_install_available: bool,
    pub is_pwa_installed: bool,
    pub backend: StorageBackend,
    pub quota_exceeded: bool,
    pub usage_bytes: u64,
    pub quota_bytes: u64,
}

#[allow(unused_mut)]
pub fn query_storage_diagnostics() -> StorageDiagnostics {
    let mut diag = StorageDiagnostics::default();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(val) = js_sys::Reflect::get(
                &window,
                &wasm_bindgen::JsValue::from_str("__pwaInstallAvailable"),
            ) {
                diag.pwa_install_available = val.as_bool().unwrap_or(false);
            }
            if let Ok(val) =
                js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__pwaInstalled"))
            {
                diag.is_pwa_installed = val.as_bool().unwrap_or(false);
            }
            if let Ok(val) = js_sys::Reflect::get(
                &window,
                &wasm_bindgen::JsValue::from_str("__storagePersisted"),
            ) {
                if let Some(b) = val.as_bool() {
                    diag.is_persisted = Some(b);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        diag.backend = StorageBackend::DiskFile;
        diag.is_persisted = Some(true);
    }

    diag
}

/// Request persistent storage from the browser (immune to automatic eviction)
pub fn request_persistent_storage() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(func) = js_sys::Reflect::get(
                &window,
                &wasm_bindgen::JsValue::from_str("__requestPersistentStorage"),
            ) {
                if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(&window);
                    info!("Triggered persistent storage request from client");
                }
            }
        }
    }
}

/// Trigger the native PWA installation prompt
pub fn trigger_pwa_install() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(func) = js_sys::Reflect::get(
                &window,
                &wasm_bindgen::JsValue::from_str("__triggerPWAInstall"),
            ) {
                if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(&window);
                    info!("Triggered PWA install prompt from client");
                }
            }
        }
    }
}

pub const DEDICATED_STORAGE_KEY: &str = "revisited_ipip_neo_state";

/// Robust dual-format deserializer for AppState, attempting JSON first and falling back to RON.
pub fn deserialize_app_state(content: &str) -> Result<shared::AppState, StorageError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(StorageError::EmptyContent);
    }

    match serde_json::from_str::<shared::AppState>(trimmed) {
        Ok(state) => Ok(state),
        Err(json_err) => match ron::from_str::<shared::AppState>(trimmed) {
            Ok(state) => Ok(state),
            Err(ron_err) => Err(StorageError::Deserialization { json_err, ron_err }),
        },
    }
}

/// Multi-tiered loader for AppState.
/// Checks local disk file (on desktop), window.localStorage (on wasm32), and eframe::Storage.
pub fn load_state_multi_tier(storage: Option<&dyn eframe::Storage>) -> Option<shared::AppState> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let filename = format!("{}.json", DEDICATED_STORAGE_KEY);
        if let Ok(content) = std::fs::read_to_string(&filename) {
            match deserialize_app_state(&content) {
                Ok(mut state) => {
                    info!("Restored assessment state from disk file [{}]", filename);
                    if state.questionnaire.unanswered_count() == 0
                        && !state.questionnaire.questions.is_empty()
                    {
                        state.questionnaire.show_results = true;
                    }
                    state.questionnaire.rebuild_cache();
                    return Some(state);
                }
                Err(e) => {
                    warn!(
                        "Failed to parse assessment state from disk file [{}]: {}",
                        filename, e
                    );
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(local_storage)) = window.local_storage() {
                if let Ok(Some(content)) = local_storage.get_item(DEDICATED_STORAGE_KEY) {
                    match deserialize_app_state(&content) {
                        Ok(mut state) => {
                            info!(
                                "Restored assessment state from localStorage [{}]",
                                DEDICATED_STORAGE_KEY
                            );
                            if state.questionnaire.unanswered_count() == 0
                                && !state.questionnaire.questions.is_empty()
                            {
                                state.questionnaire.show_results = true;
                            }
                            state.questionnaire.rebuild_cache();
                            return Some(state);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse assessment state from localStorage [{}]: {}",
                                DEDICATED_STORAGE_KEY, e
                            );
                        }
                    }
                }

                if let Ok(Some(content)) = local_storage.get_item(eframe::APP_KEY) {
                    match deserialize_app_state(&content) {
                        Ok(mut state) => {
                            info!(
                                "Restored assessment state from localStorage [{}]",
                                eframe::APP_KEY
                            );
                            if state.questionnaire.unanswered_count() == 0
                                && !state.questionnaire.questions.is_empty()
                            {
                                state.questionnaire.show_results = true;
                            }
                            state.questionnaire.rebuild_cache();
                            return Some(state);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse assessment state from localStorage [{}]: {}",
                                eframe::APP_KEY,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    if let Some(storage) = storage {
        if let Some(raw) = storage.get_string(DEDICATED_STORAGE_KEY) {
            match deserialize_app_state(&raw) {
                Ok(mut state) => {
                    info!(
                        "Restored assessment state from eframe storage [{}]",
                        DEDICATED_STORAGE_KEY
                    );
                    if state.questionnaire.unanswered_count() == 0
                        && !state.questionnaire.questions.is_empty()
                    {
                        state.questionnaire.show_results = true;
                    }
                    state.questionnaire.rebuild_cache();
                    return Some(state);
                }
                Err(e) => {
                    warn!(
                        "Failed to parse assessment state from eframe storage [{}]: {}",
                        DEDICATED_STORAGE_KEY, e
                    );
                }
            }
        }

        if let Some(raw) = storage.get_string(eframe::APP_KEY) {
            match deserialize_app_state(&raw) {
                Ok(mut state) => {
                    info!(
                        "Restored assessment state from eframe storage [{}]",
                        eframe::APP_KEY
                    );
                    if state.questionnaire.unanswered_count() == 0
                        && !state.questionnaire.questions.is_empty()
                    {
                        state.questionnaire.show_results = true;
                    }
                    state.questionnaire.rebuild_cache();
                    return Some(state);
                }
                Err(e) => {
                    warn!(
                        "Failed to parse assessment state from eframe storage [{}]: {}",
                        eframe::APP_KEY,
                        e
                    );
                }
            }
        }

        if let Some(mut state) = eframe::get_value::<shared::AppState>(storage, eframe::APP_KEY) {
            info!("Restored assessment state from eframe get_value");
            if state.questionnaire.unanswered_count() == 0
                && !state.questionnaire.questions.is_empty()
            {
                state.questionnaire.show_results = true;
            }
            state.questionnaire.rebuild_cache();
            return Some(state);
        }
    }

    None
}

/// Save state using localStorage (WASM) or atomic file system write (desktop).
pub fn save_state_multi_tier(key: &str, json_str: &str) -> Result<StorageBackend, StorageError> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match storage.set_item(key, json_str) {
                    Ok(()) => return Ok(StorageBackend::LocalStorage),
                    Err(err) => {
                        warn!(
                            "Storage write failed with error {:?}; quota exceeded or storage restricted",
                            err
                        );
                        return Err(StorageError::QuotaExceeded(format!("{:?}", err)));
                    }
                }
            }
            return Err(StorageError::LocalStorageUnavailable);
        }
        Err(StorageError::LocalStorageUnavailable)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let filename = format!("{}.json", key);
        match std::fs::write(&filename, json_str) {
            Ok(()) => Ok(StorageBackend::DiskFile),
            Err(err) => {
                warn!(
                    "Failed to persist assessment state to file '{}': {}",
                    filename, err
                );
                Err(StorageError::DiskWrite {
                    path: filename,
                    source: err,
                })
            }
        }
    }
}

/// Trigger client-side text file download via Blob URL (WASM) or direct file write (desktop).
pub fn trigger_text_download(filename: &str, content: &str, mime_type: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
                let blob_props = web_sys::BlobPropertyBag::new();
                blob_props.set_type(mime_type);
                if let Ok(blob) =
                    web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_props)
                {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        if let Ok(element) = document.create_element("a") {
                            if let Ok(anchor) = element.dyn_into::<web_sys::HtmlAnchorElement>() {
                                anchor.set_href(&url);
                                anchor.set_download(filename);
                                anchor.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mime_type;
        match std::fs::write(filename, content) {
            Ok(()) => info!("Exported file successfully: {}", filename),
            Err(e) => error!("Failed to write export file '{}': {}", filename, e),
        }
    }
}

/// Trigger client-side binary file download (e.g. Compressed BSON) via Blob URL (WASM) or direct write (desktop).
pub fn trigger_binary_download(filename: &str, bytes: &[u8], mime_type: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let uint8_array = js_sys::Uint8Array::from(bytes);
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&uint8_array.buffer());
                let blob_props = web_sys::BlobPropertyBag::new();
                blob_props.set_type(mime_type);
                if let Ok(blob) =
                    web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_props)
                {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        if let Ok(element) = document.create_element("a") {
                            if let Ok(anchor) = element.dyn_into::<web_sys::HtmlAnchorElement>() {
                                anchor.set_href(&url);
                                anchor.set_download(filename);
                                anchor.click();
                                let _ = web_sys::Url::revoke_object_url(&url);
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mime_type;
        match std::fs::write(filename, bytes) {
            Ok(()) => info!("Exported binary file successfully: {}", filename),
            Err(e) => error!("Failed to write binary export file '{}': {}", filename, e),
        }
    }
}
