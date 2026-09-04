//! Unified multi-tiered storage engine, persistence manager, PWA install bridge, and diagnostics.

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use tracing::{error, info, warn};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StorageBackend {
    #[default]
    LocalStorage,
    MemoryOnly,
}

impl StorageBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalStorage => "Local Storage (Fast Tier)",
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
            // Check if PWA is installed or installable
            if let Ok(val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__pwaInstallAvailable")) {
                diag.pwa_install_available = val.as_bool().unwrap_or(false);
            }
            if let Ok(val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__pwaInstalled")) {
                diag.is_pwa_installed = val.as_bool().unwrap_or(false);
            }

            // Check persistence state
            if let Ok(val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__storagePersisted")) {
                if let Some(b) = val.as_bool() {
                    diag.is_persisted = Some(b);
                }
            }
        }
    }

    diag
}

/// Request persistent storage from the browser (immune to automatic eviction)
pub fn request_persistent_storage() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(func) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__requestPersistentStorage")) {
                if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(&window);
                    info!("Triggered __requestPersistentStorage from UI");
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
            if let Ok(func) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__triggerPWAInstall")) {
                if let Some(func) = func.dyn_ref::<js_sys::Function>() {
                    let _ = func.call0(&window);
                    info!("Triggered __triggerPWAInstall from UI");
                }
            }
        }
    }
}

pub const DEDICATED_STORAGE_KEY: &str = "revisited_ipip_neo_state";

/// Robust dual-format deserializer for AppState, attempting JSON first and falling back to RON.
pub fn deserialize_app_state(content: &str) -> Result<shared::AppState, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("Storage content is empty".to_string());
    }

    // 1. Attempt JSON deserialization
    match serde_json::from_str::<shared::AppState>(trimmed) {
        Ok(state) => Ok(state),
        Err(json_err) => {
            // 2. Attempt RON deserialization
            match ron::from_str::<shared::AppState>(trimmed) {
                Ok(state) => Ok(state),
                Err(ron_err) => Err(format!(
                    "Failed to deserialize AppState: JSON error: {}; RON error: {}",
                    json_err, ron_err
                )),
            }
        }
    }
}

/// Multi-tiered loader for AppState.
/// Checks window.localStorage (on wasm32) and eframe::Storage across both dedicated and legacy keys,
/// supporting both JSON and RON formats seamlessly with automatic cache rebuild.
pub fn load_state_multi_tier(storage: Option<&dyn eframe::Storage>) -> Option<shared::AppState> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(local_storage)) = window.local_storage() {
                // Tier 1: Check dedicated key in browser localStorage
                if let Ok(Some(content)) = local_storage.get_item(DEDICATED_STORAGE_KEY) {
                    match deserialize_app_state(&content) {
                        Ok(mut state) => {
                            info!("Successfully restored AppState from localStorage [{}]", DEDICATED_STORAGE_KEY);
                            if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
                                state.questionnaire.show_results = true;
                            }
                            state.questionnaire.rebuild_cache();
                            return Some(state);
                        }
                        Err(e) => {
                            warn!("Failed to parse AppState from localStorage [{}]: {}", DEDICATED_STORAGE_KEY, e);
                        }
                    }
                }

                // Tier 2: Check standard 'app' key in browser localStorage (fallback/legacy)
                if let Ok(Some(content)) = local_storage.get_item(eframe::APP_KEY) {
                    match deserialize_app_state(&content) {
                        Ok(mut state) => {
                            info!("Successfully restored AppState from localStorage [{}]", eframe::APP_KEY);
                            if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
                                state.questionnaire.show_results = true;
                            }
                            state.questionnaire.rebuild_cache();
                            return Some(state);
                        }
                        Err(e) => {
                            warn!("Failed to parse AppState from localStorage [{}]: {}", eframe::APP_KEY, e);
                        }
                    }
                }
            }
        }
    }

    // Tier 3: Check eframe::Storage
    if let Some(storage) = storage {
        // Check dedicated key in eframe storage
        if let Some(raw) = storage.get_string(DEDICATED_STORAGE_KEY) {
            match deserialize_app_state(&raw) {
                Ok(mut state) => {
                    info!("Successfully restored AppState from eframe::Storage [{}]", DEDICATED_STORAGE_KEY);
                    if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
                        state.questionnaire.show_results = true;
                    }
                    state.questionnaire.rebuild_cache();
                    return Some(state);
                }
                Err(e) => {
                    warn!("Failed to parse AppState from eframe::Storage [{}]: {}", DEDICATED_STORAGE_KEY, e);
                }
            }
        }

        // Check 'app' key string in eframe storage
        if let Some(raw) = storage.get_string(eframe::APP_KEY) {
            match deserialize_app_state(&raw) {
                Ok(mut state) => {
                    info!("Successfully restored AppState from eframe::Storage [{}]", eframe::APP_KEY);
                    if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
                        state.questionnaire.show_results = true;
                    }
                    state.questionnaire.rebuild_cache();
                    return Some(state);
                }
                Err(e) => {
                    warn!("Failed to parse AppState from eframe::Storage [{}]: {}", eframe::APP_KEY, e);
                }
            }
        }

        // Check native eframe::get_value (RON deserializer)
        if let Some(mut state) = eframe::get_value::<shared::AppState>(storage, eframe::APP_KEY) {
            info!("Successfully restored AppState from eframe::get_value (RON).");
            if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty() {
                state.questionnaire.show_results = true;
            }
            state.questionnaire.rebuild_cache();
            return Some(state);
        }
    }

    None
}

/// Save state using localStorage with fallback to in-memory ephemeral tier
pub fn save_state_multi_tier(key: &str, json_str: &str) -> Result<StorageBackend, String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                match storage.set_item(key, json_str) {
                    Ok(()) => {
                        return Ok(StorageBackend::LocalStorage);
                    }
                    Err(err) => {
                        warn!("localStorage.set_item failed with error {:?}. Quota exceeded or storage restricted.", err);
                        return Err(format!("localStorage quota exceeded: {:?}", err));
                    }
                }
            }
            return Err("localStorage unavailable in browser window.".to_string());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, json_str);
    }

    Ok(StorageBackend::MemoryOnly)
}

/// Trigger client-side text file download via Blob URL
pub fn trigger_text_download(filename: &str, content: &str, mime_type: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
                let blob_props = web_sys::BlobPropertyBag::new();
                blob_props.set_type(mime_type);
                if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_props) {
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
            Ok(()) => info!("Successfully exported file: {}", filename),
            Err(e) => error!("Failed to write export file '{}': {}", filename, e),
        }
    }
}

/// Trigger client-side binary file download (e.g. Compressed BSON) via Blob URL
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
                if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_props) {
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
            Ok(()) => info!("Successfully exported binary file: {}", filename),
            Err(e) => error!("Failed to write binary export file '{}': {}", filename, e),
        }
    }
}
