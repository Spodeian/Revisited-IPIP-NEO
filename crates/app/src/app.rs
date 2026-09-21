use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use eframe::egui;
use shared::{
    AppState, ThemeMode, export_to_compressed_bson, export_to_csv, export_to_json,
    export_to_printable_html, export_to_svg, import_responses_from_bson, import_responses_from_csv,
    import_responses_from_json,
};
use tracing::{info, warn};

use crate::storage_manager::{
    DEDICATED_STORAGE_KEY, StorageBackend, StorageDiagnostics, load_state_multi_tier,
    query_storage_diagnostics, save_state_multi_tier, trigger_binary_download,
};
use crate::types::{ExportFormat, ScreenConstraints};

#[derive(Default)]
pub struct PersonalityApp {
    pub state: AppState,
    pub saved_local_state: Option<AppState>,
    pub current_theme: Option<ThemeMode>,
    pub show_reset_dialog: bool,
    pub show_help_dialog: bool,
    pub show_grid_dialog: bool,
    pub show_import_dialog: bool,
    pub import_text_buffer: String,
    pub import_result_message: Option<Result<String, String>>,
    pub show_export_dialog: Option<ExportFormat>,
    pub export_text_buffer: String,
    pub export_copied_notification: Option<f64>,
    pub share_link_copied_time: Option<f64>,
    pub is_viewing_shared_link: bool,
    pub selected_export_format: ExportFormat,
    pub hide_header: bool,
    pub last_scroll_time: f64,
    pub scroll_accumulator: f32,
    pub answer_timestamps: VecDeque<f64>,
    pub last_save_time: Option<f64>,
    pub undo_notification_time: Option<f64>,
    pub redo_notification_time: Option<f64>,
    pub storage_diag: StorageDiagnostics,
    pub show_storage_modal: bool,
    pub dismissed_ephemeral_warning: bool,
    pub dismissed_quota_warning: bool,
    pub dismissed_combined_warning: bool,
    pub last_diag_poll_time: f64,
    pub pending_dropped_file: Arc<Mutex<Option<(Vec<u8>, String)>>>,
}

impl PersonalityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Initializing Revisited IPIP-NEO Personality Assessment...");

        let mut state = load_state_multi_tier(cc.storage).unwrap_or_else(|| {
            warn!("No existing saved assessment found, initializing fresh.");
            AppState::default()
        });

        if state.questionnaire.unanswered_count() == 0 && !state.questionnaire.questions.is_empty()
        {
            state.questionnaire.show_results = true;
        }
        state.questionnaire.rebuild_cache();
        let saved_local_state = Some(state.clone());

        #[allow(unused_mut)]
        let mut is_viewing_shared_link = false;
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    let hash_trimmed = hash.trim_start_matches('#');
                    let code = if let Some(stripped) = hash_trimmed.strip_prefix("r=") {
                        Some(stripped)
                    } else if let Some(stripped) = hash_trimmed.strip_prefix("code=") {
                        Some(stripped)
                    } else if !hash_trimmed.is_empty() && !hash_trimmed.contains('=') {
                        Some(hash_trimmed)
                    } else {
                        None
                    };

                    if let Some(c) = code {
                        let mut shared_state = shared::QuestionnaireState::from_embedded_data();
                        if let Ok(_count) =
                            shared::decode_responses_from_url_code(&mut shared_state, c)
                        {
                            info!(
                                "Loaded shared results from URL hash with {} answers",
                                _count
                            );
                            let shared_responses = shared_state.current_responses_snapshot();
                            state.questionnaire.load_snapshot_with_undo(
                                shared_responses,
                                true,
                                "Friend's Shared Link Loaded",
                            );
                            is_viewing_shared_link = true;
                        }
                    }
                }
            }
        }

        Self {
            state,
            saved_local_state,
            current_theme: None,
            show_reset_dialog: false,
            show_help_dialog: false,
            show_grid_dialog: false,
            show_import_dialog: false,
            import_text_buffer: String::new(),
            import_result_message: None,
            show_export_dialog: None,
            export_text_buffer: String::new(),
            export_copied_notification: None,
            share_link_copied_time: None,
            is_viewing_shared_link,
            selected_export_format: ExportFormat::default(),
            hide_header: false,
            last_scroll_time: 0.0,
            scroll_accumulator: 0.0,
            answer_timestamps: VecDeque::new(),
            last_save_time: None,
            undo_notification_time: None,
            redo_notification_time: None,
            storage_diag: query_storage_diagnostics(),
            show_storage_modal: false,
            dismissed_ephemeral_warning: false,
            dismissed_quota_warning: false,
            dismissed_combined_warning: false,
            last_diag_poll_time: 0.0,
            pending_dropped_file: Default::default(),
        }
    }

    pub fn restore_saved_instance(&mut self) {
        if let Some(ref saved) = self.saved_local_state {
            self.state = saved.clone();
        } else {
            self.state.reset_questionnaire();
        }
        self.state.questionnaire.rebuild_cache();
        self.is_viewing_shared_link = false;

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_hash("");
            }
        }
    }

    /// Immediately persist current state to storage tier.
    /// Skips writing when inspecting a shared guest link.
    pub fn persist_state(&mut self) {
        if self.is_viewing_shared_link {
            return;
        }

        if let Ok(json_str) = serde_json::to_string(&self.state) {
            match save_state_multi_tier(DEDICATED_STORAGE_KEY, &json_str) {
                Ok(backend) => {
                    self.storage_diag.backend = backend;
                    self.storage_diag.quota_exceeded = false;
                }
                Err(e) => {
                    warn!("Standard persistence failed ({}). Compacting history...", e);
                    self.state.questionnaire.compact_history(30);
                    if let Ok(compacted_json) = serde_json::to_string(&self.state) {
                        if let Ok(backend) =
                            save_state_multi_tier(DEDICATED_STORAGE_KEY, &compacted_json)
                        {
                            self.storage_diag.backend = backend;
                            self.storage_diag.quota_exceeded = false;
                        } else {
                            self.storage_diag.quota_exceeded = true;
                            self.storage_diag.backend = StorageBackend::MemoryOnly;
                        }
                    }
                }
            }
        }
    }

    pub fn open_export_dialog(&mut self, format: ExportFormat) {
        if format == ExportFormat::Bson {
            if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                use base64::{Engine as _, engine::general_purpose};
                self.export_text_buffer = general_purpose::STANDARD.encode(&bytes);
                trigger_binary_download(
                    "ipip_neo_assessment_backup.bson",
                    &bytes,
                    "application/octet-stream",
                );
            }
        } else {
            self.export_text_buffer = match format {
                ExportFormat::Csv => export_to_csv(&self.state.questionnaire),
                ExportFormat::Json => export_to_json(&self.state.questionnaire),
                ExportFormat::Bson => unreachable!(),
                ExportFormat::Svg => export_to_svg(&self.state.questionnaire),
                ExportFormat::Html => export_to_printable_html(&self.state.questionnaire),
            };
        }
        self.show_export_dialog = Some(format);
        self.export_copied_notification = None;
    }

    pub fn import_from_bytes(&mut self, bytes: &[u8], filename: &str) -> Result<usize, String> {
        if let Ok(count) = import_responses_from_bson(&mut self.state.questionnaire, bytes) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.rebuild_cache();
            self.persist_state();
            return Ok(count);
        }

        if let Ok(text) = std::str::from_utf8(bytes) {
            let trimmed = text.trim();
            if trimmed.starts_with('{') {
                if let Ok(count) =
                    import_responses_from_json(&mut self.state.questionnaire, trimmed)
                {
                    self.is_viewing_shared_link = false;
                    self.state.questionnaire.rebuild_cache();
                    self.persist_state();
                    return Ok(count);
                }
            } else if trimmed.contains('#')
                || trimmed.contains(',')
                || trimmed.to_lowercase().contains("item_id")
            {
                if let Ok(count) = import_responses_from_csv(&mut self.state.questionnaire, trimmed)
                {
                    self.is_viewing_shared_link = false;
                    self.state.questionnaire.rebuild_cache();
                    self.persist_state();
                    return Ok(count);
                }
            } else {
                use base64::{Engine as _, engine::general_purpose};
                if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(trimmed) {
                    if let Ok(count) =
                        import_responses_from_bson(&mut self.state.questionnaire, &decoded_bytes)
                    {
                        self.is_viewing_shared_link = false;
                        self.state.questionnaire.rebuild_cache();
                        self.persist_state();
                        return Ok(count);
                    }
                }
            }
        }

        Err(format!(
            "Could not parse '{}'. Supported formats: .bson, .json, .csv",
            filename
        ))
    }
}

impl eframe::App for PersonalityApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.is_viewing_shared_link {
            eframe::set_value(storage, eframe::APP_KEY, &self.state);

            if let Ok(json_str) = serde_json::to_string(&self.state) {
                storage.set_string(DEDICATED_STORAGE_KEY, json_str);
            }
            storage.flush();

            self.persist_state();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let current_time = ui.input(|i| i.time);
        if current_time - self.last_diag_poll_time > 2.0 {
            let fresh_diag = query_storage_diagnostics();
            self.storage_diag.is_persisted = fresh_diag.is_persisted;
            self.storage_diag.pwa_install_available = fresh_diag.pwa_install_available;
            self.storage_diag.is_pwa_installed = fresh_diag.is_pwa_installed;
            self.last_diag_poll_time = current_time;
        }

        self.apply_theme(ui.ctx());
        if self.state.config.theme.is_high_contrast() {
            ui.spacing_mut().interact_size = egui::vec2(44.0, 44.0);
            ui.spacing_mut().button_padding = egui::vec2(14.0, 10.0);
        } else {
            ui.spacing_mut().interact_size.y = ui.spacing_mut().interact_size.y.max(32.0);
            ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
        }
        self.handle_keyboard_and_scroll(ui);

        let pending_file = if let Ok(mut guard) = self.pending_dropped_file.lock() {
            guard.take()
        } else {
            None
        };

        if let Some((bytes, name)) = pending_file {
            self.show_import_dialog = true;
            match self.import_from_bytes(&bytes, &name) {
                Ok(count) => {
                    let current_t = ui.input(|i| i.time);
                    self.last_save_time = Some(current_t);
                    self.import_result_message = Some(Ok(format!(
                        "Successfully imported {} answers from '{}'",
                        count, name
                    )));
                }
                Err(e) => {
                    self.import_result_message = Some(Err(e));
                }
            }
        }

        let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for file in dropped_files {
                let path = file.path();
                let name = if !path.as_os_str().is_empty() {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("backup_file")
                        .to_string()
                } else {
                    "backup_file".to_string()
                };

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let mut loaded_bytes: Option<Vec<u8>> = None;
                    if let Ok(bytes) = file.bytes() {
                        loaded_bytes = Some(bytes);
                    } else if !path.as_os_str().is_empty() {
                        if let Ok(b) = std::fs::read(path) {
                            loaded_bytes = Some(b);
                        }
                    }

                    if let Some(bytes) = loaded_bytes {
                        self.show_import_dialog = true;
                        match self.import_from_bytes(&bytes, &name) {
                            Ok(count) => {
                                let current_t = ui.input(|i| i.time);
                                self.last_save_time = Some(current_t);
                                self.import_result_message = Some(Ok(format!(
                                    "Successfully imported {} answers from '{}'",
                                    count, name
                                )));
                            }
                            Err(e) => {
                                self.import_result_message = Some(Err(e));
                            }
                        }
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let pending = self.pending_dropped_file.clone();
                    let file_clone = file.clone();
                    let name_clone = name.clone();
                    let ctx = ui.ctx().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(bytes) = file_clone.bytes_async().await {
                            if let Ok(mut guard) = pending.lock() {
                                *guard = Some((bytes, name_clone));
                            }
                            ctx.request_repaint();
                        }
                    });
                }
            }
        }

        let constraints = ScreenConstraints::compute(ui);

        if !self.hide_header {
            self.render_top_bar(ui, &constraints);
        }

        let avail_w = ui.available_width();
        let show_results_side_panel = self.state.questionnaire.show_results && avail_w >= 900.0;

        if show_results_side_panel {
            egui::Panel::right("results_panel")
                .min_size(380.0)
                .default_size(420.0)
                .show(ui, |ui| {
                    self.render_results_panel(ui);
                });
        }

        let is_tiny = constraints.is_tight_height;
        let mut central_frame = egui::Frame::central_panel(ui.style());
        if is_tiny {
            central_frame.inner_margin = egui::Margin::same(4);
        }

        egui::CentralPanel::default()
            .frame(central_frame)
            .show(ui, |ui| {
                if self.hide_header {
                    ui.vertical_centered(|ui| {
                        let expand_btn =
                            egui::Button::new(egui::RichText::new("Show Header").size(11.0).weak())
                                .min_size(egui::vec2(120.0, 22.0));
                        if ui
                            .add(expand_btn)
                            .on_hover_text("Show top navigation header")
                            .clicked()
                        {
                            self.hide_header = false;
                        }
                    });
                    ui.add_space(6.0);
                }

                if self.state.questionnaire.show_results && !show_results_side_panel {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(8.0);
                        if ui.button("Return to Questions").clicked() {
                            self.state.questionnaire.show_results = false;
                            self.persist_state();
                        }
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        self.render_results_panel(ui);
                    });
                } else {
                    self.render_question_focus(ui, &constraints);
                }
            });

        self.render_dialogs(ui);
        self.render_warning_banners(ui.ctx());
    }
}
