pub mod storage_manager;
pub use storage_manager::*;

use eframe::egui;
use shared::{
    encode_responses_to_url_code, export_to_compressed_bson, export_to_csv, export_to_json,
    export_to_printable_html, export_to_svg, import_responses_from_bson, import_responses_from_csv,
    import_responses_from_json, AppState, Aspect, Facet, MetaTrait, Response, ScoreTier, ThemeMode,
    Trait,
};
use tracing::{info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub struct ScreenConstraints {
    pub is_mobile: bool,
    pub is_mobile_portrait: bool,
    pub is_tight_height: bool,
    pub is_ultra_tight: bool,
}

impl ScreenConstraints {
    pub fn compute(ui: &egui::Ui) -> Self {
        let avail_w = ui.available_width();
        let avail_h = ui.available_height();

        Self {
            is_mobile: avail_w < 800.0,
            is_mobile_portrait: avail_w < 650.0,
            is_tight_height: avail_h < 530.0 || avail_w < 350.0,
            is_ultra_tight: avail_w < 330.0 || avail_h < 490.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ExportFormat {
    #[default]
    Csv,
    Json,
    Bson,
    Svg,
    Html,
}

impl ExportFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV File",
            Self::Json => "JSON File",
            Self::Bson => "Compressed BSON (.bson)",
            Self::Svg => "SVG Vector Graphic",
            Self::Html => "HTML Report",
        }
    }
}

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
    pub answer_timestamps: std::collections::VecDeque<f64>,
    pub last_save_time: Option<f64>,
    pub undo_notification_time: Option<f64>,
    pub redo_notification_time: Option<f64>,
    pub storage_diag: StorageDiagnostics,
    pub show_storage_modal: bool,
    pub dismissed_ephemeral_warning: bool,
    pub dismissed_quota_warning: bool,
    pub dismissed_combined_warning: bool,
    pub last_diag_poll_time: f64,
}

fn trigger_file_download(filename: &str, content: &str, _mime_type: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let blob_parts = js_sys::Array::new();
                blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
                let blob_props = web_sys::BlobPropertyBag::new();
                blob_props.set_type(_mime_type);
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
        match std::fs::write(filename, content) {
            Ok(()) => info!("Successfully exported file: {}", filename),
            Err(e) => tracing::error!("Failed to write export file '{}': {}", filename, e),
        }
    }
}

impl PersonalityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Initializing Revisited IPIP-NEO Personality Assessment...");

        let mut state = if let Some(storage) = cc.storage {
            match eframe::get_value::<AppState>(storage, eframe::APP_KEY) {
                Some(s) => {
                    info!("Loaded personality assessment state from storage.");
                    s
                }
                None => {
                    warn!("No existing saved assessment found, initializing fresh.");
                    AppState::default()
                }
            }
        } else {
            AppState::default()
        };

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
                        if let Ok(_count) = shared::decode_responses_from_url_code(&mut shared_state, c) {
                            info!("Loaded shared results from URL hash with {} answers", _count);
                            let shared_responses = shared_state.current_responses_snapshot();
                            state.questionnaire.load_snapshot_with_undo(shared_responses, true, "Friend's Shared Link Loaded");
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
            answer_timestamps: std::collections::VecDeque::new(),
            last_save_time: None,
            undo_notification_time: None,
            redo_notification_time: None,
            storage_diag: query_storage_diagnostics(),
            show_storage_modal: false,
            dismissed_ephemeral_warning: false,
            dismissed_quota_warning: false,
            dismissed_combined_warning: false,
            last_diag_poll_time: 0.0,
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

    pub fn open_export_dialog(&mut self, format: ExportFormat) {
        if format == ExportFormat::Bson {
            if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                use base64::{engine::general_purpose, Engine as _};
                self.export_text_buffer = general_purpose::STANDARD.encode(&bytes);
                trigger_binary_download("ipip_neo_assessment_backup.bson", &bytes, "application/octet-stream");
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
        // 1. Try BSON binary directly (e.g. from .bson file)
        if let Ok(count) = import_responses_from_bson(&mut self.state.questionnaire, bytes) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.rebuild_cache();
            return Ok(count);
        }

        // 2. Try UTF-8 string parse (JSON, CSV, or Base64 BSON)
        if let Ok(text) = std::str::from_utf8(bytes) {
            let trimmed = text.trim();
            if trimmed.starts_with('{') {
                if let Ok(count) = import_responses_from_json(&mut self.state.questionnaire, trimmed) {
                    self.is_viewing_shared_link = false;
                    self.state.questionnaire.rebuild_cache();
                    return Ok(count);
                }
            } else if trimmed.contains('#') || trimmed.contains(',') || trimmed.to_lowercase().contains("item_id") {
                if let Ok(count) = import_responses_from_csv(&mut self.state.questionnaire, trimmed) {
                    self.is_viewing_shared_link = false;
                    self.state.questionnaire.rebuild_cache();
                    return Ok(count);
                }
            } else {
                // Attempt Base64 BSON decode
                use base64::{engine::general_purpose, Engine as _};
                if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(trimmed) {
                    if let Ok(count) = import_responses_from_bson(&mut self.state.questionnaire, &decoded_bytes) {
                        self.is_viewing_shared_link = false;
                        self.state.questionnaire.rebuild_cache();
                        return Ok(count);
                    }
                }
            }
        }

        Err(format!("Could not parse '{}'. Supported formats: .bson, .json, .csv", filename))
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.current_theme == Some(self.state.config.theme) {
            return;
        }
        self.current_theme = Some(self.state.config.theme);

        let visuals = match self.state.config.theme {
            ThemeMode::Light => {
                let mut light = egui::Visuals::light();

                // Soothing neutral/warm light backgrounds instead of blinding pure white
                light.panel_fill = egui::Color32::from_rgb(245, 244, 241); // Soft warm grey
                light.window_fill = egui::Color32::from_rgb(252, 250, 246); // Warm off-white
                light.extreme_bg_color = egui::Color32::from_rgb(238, 236, 231); // Insets background

                // Soft charcoal for high-contrast, comfortable reading without harsh black
                light.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(45, 44, 42);
                light.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(55, 54, 52);
                light.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(20, 20, 18);
                light.widgets.active.fg_stroke.color = egui::Color32::from_rgb(0, 0, 0);

                // Muted border strokes to reduce visual clutter
                light.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(222, 220, 215);
                light.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(212, 210, 205);

                // Subtle buttons background
                light.widgets.inactive.bg_fill = egui::Color32::from_rgb(252, 251, 248);
                light.widgets.hovered.bg_fill = egui::Color32::from_rgb(236, 234, 229);
                light.widgets.active.bg_fill = egui::Color32::from_rgb(220, 218, 212);

                light
            }
            ThemeMode::Dark => egui::Visuals::dark(),
        };
        ctx.set_visuals(visuals);
    }

    fn handle_keyboard_and_scroll(&mut self, ui: &mut egui::Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }

        let input = ui.input(|i| i.clone());
        let current_time = input.time;

        let record_answer_timestamp = |timestamps: &mut std::collections::VecDeque<f64>, save_time: &mut Option<f64>| {
            timestamps.push_back(current_time);
            if timestamps.len() > 25 {
                timestamps.pop_front();
            }
            *save_time = Some(current_time);
        };

        // Keyboard shortcuts for responses: 1-5
        if input.key_pressed(egui::Key::Num1) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::StronglyDisagree);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
        } else if input.key_pressed(egui::Key::Num2) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Disagree);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
        } else if input.key_pressed(egui::Key::Num3) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Neutral);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
        } else if input.key_pressed(egui::Key::Num4) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Agree);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
        } else if input.key_pressed(egui::Key::Num5) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::StronglyAgree);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
        }

        // Undo shortcut: Ctrl+Z / Cmd+Z (without Shift)
        if (input.modifiers.command || input.modifiers.ctrl)
            && input.key_pressed(egui::Key::Z)
            && !input.modifiers.shift
            && self.state.questionnaire.undo()
        {
            self.undo_notification_time = Some(current_time);
            self.last_save_time = Some(current_time);
        }

        // Redo shortcut: Ctrl+Y / Cmd+Y OR Ctrl+Shift+Z / Cmd+Shift+Z
        if (((input.modifiers.command || input.modifiers.ctrl) && input.key_pressed(egui::Key::Y))
            || ((input.modifiers.command || input.modifiers.ctrl) && input.modifiers.shift && input.key_pressed(egui::Key::Z)))
            && self.state.questionnaire.redo()
        {
            self.redo_notification_time = Some(current_time);
            self.last_save_time = Some(current_time);
        }

        // Navigation shortcuts:
        let shift_held = input.modifiers.shift;

        // Escape key to dismiss dialogs or close results screen
        if input.key_pressed(egui::Key::Escape) {
            if self.show_grid_dialog {
                self.show_grid_dialog = false;
            } else if self.show_help_dialog {
                self.show_help_dialog = false;
            } else if self.show_reset_dialog {
                self.show_reset_dialog = false;
            } else if self.show_import_dialog {
                self.show_import_dialog = false;
                self.import_text_buffer.clear();
                self.import_result_message = None;
            } else if self.show_export_dialog.is_some() {
                self.show_export_dialog = None;
                self.export_text_buffer.clear();
            } else if self.state.questionnaire.show_results {
                self.state.questionnaire.show_results = false;
            }
        }

        // Left/Up arrow
        if input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::ArrowUp) {
            if shift_held {
                self.state.questionnaire.navigate_previous_unanswered();
            } else {
                self.state.questionnaire.navigate_previous();
            }
        }
        // Right/Down arrow / Tab
        if input.key_pressed(egui::Key::ArrowRight) || input.key_pressed(egui::Key::ArrowDown) {
            if shift_held {
                self.state.questionnaire.navigate_next_unanswered();
            } else {
                self.state.questionnaire.skip_current();
            }
        }
    }

    fn calculate_estimated_time_remaining(&self) -> Option<String> {
        let unanswered = self.state.questionnaire.unanswered_count();
        if unanswered == 0 {
            return None;
        }
        if self.answer_timestamps.len() >= 3 {
            let first = *self.answer_timestamps.front()?;
            let last = *self.answer_timestamps.back()?;
            let elapsed = last - first;
            let count = self.answer_timestamps.len() - 1;
            if count > 0 && elapsed > 0.5 {
                let sec_per_item = (elapsed / count as f64).clamp(1.0, 30.0);
                let remaining_secs = (unanswered as f64 * sec_per_item).round() as u64;
                let minutes = remaining_secs / 60;
                let seconds = remaining_secs % 60;
                if minutes > 0 {
                    return Some(format!("~{}m {}s", minutes, seconds));
                } else {
                    return Some(format!("~{}s", seconds));
                }
            }
        }
        None
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui, constraints: &ScreenConstraints) {
        let current_time = ui.input(|i| i.time);

        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.add_space(4.0);

            let avail_w = ui.available_width();
            let title_text = if avail_w < 480.0 {
                "IPIP-NEO"
            } else if avail_w < 780.0 {
                "Revisited IPIP-NEO (TGA)"
            } else {
                "Revisited IPIP-NEO Personality Assessment"
            };

            let is_touch = constraints.is_mobile;
            let btn_height = if is_touch { 34.0 } else { 26.0 };

            ui.horizontal(|ui| {
                ui.set_min_height(btn_height + 4.0);

                // Right-aligned cluster: Quick Results toggle + Dropdown Menu
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                    ui.spacing_mut().interact_size.y = btn_height;

                    // Dropdown menu button in top right (styled with matching bold typography and height)
                    ui.menu_button(egui::RichText::new("☰ Menu").strong(), |ui| {
                        ui.set_min_width(220.0);

                        ui.label(egui::RichText::new("Assessment Navigation").small().weak());

                        let results_text = if self.state.questionnaire.show_results {
                            "❓ Return to Questions"
                        } else {
                            "📊 View Assessment Results"
                        };
                        if ui.button(results_text).on_hover_text("Toggle between questionnaire answering view and hierarchical results report").clicked() {
                            self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                            ui.close();
                        }

                        if ui.button("🗺 Item Matrix Map").on_hover_text("Open interactive 221-item questionnaire matrix map").clicked() {
                            self.show_grid_dialog = true;
                            ui.close();
                        }

                        if self.is_viewing_shared_link
                            && ui.button("↩ My Saved Answers").on_hover_text("Exit shared link view and restore your local saved answers").clicked()
                        {
                            self.restore_saved_instance();
                            ui.close();
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Data & Management").small().weak());

                        if ui.button("📥 Import Answers (CSV/JSON/BSON)").on_hover_text("Import previously exported CSV, JSON, or BSON backup to resume testing").clicked() {
                            self.show_import_dialog = true;
                            self.import_text_buffer.clear();
                            self.import_result_message = None;
                            ui.close();
                        }

                        if ui.button("🔄 Reset Assessment").on_hover_text("Clear all recorded responses and restart assessment from item #1").clicked() {
                            self.show_reset_dialog = true;
                            ui.close();
                        }

                        let storage_label = match self.storage_diag.is_persisted {
                            Some(true) => "💾 Storage Status: Persistent",
                            Some(false) => "💾 Storage Status: Ephemeral",
                            None => "💾 Storage Status",
                        };
                        if ui.button(storage_label).on_hover_text("Inspect offline storage durability, persistence status, quota, and backups").clicked() {
                            self.show_storage_modal = true;
                            ui.close();
                        }

                        if self.storage_diag.pwa_install_available && !self.storage_diag.is_pwa_installed
                            && ui.button("📲 Install App (PWA)").on_hover_text("Install assessment as a standalone Progressive Web App for permanent offline durability").clicked()
                        {
                            trigger_pwa_install();
                            ui.close();
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Settings & Info").small().weak());

                        let theme_label = match self.state.config.theme {
                            ThemeMode::Light => "🌙 Dark Mode",
                            ThemeMode::Dark => "☀ Light Mode",
                        };
                        if ui.button(theme_label).on_hover_text("Switch between dark and light visual themes").clicked() {
                            self.state.config.theme = match self.state.config.theme {
                                ThemeMode::Light => ThemeMode::Dark,
                                ThemeMode::Dark => ThemeMode::Light,
                            };
                            ui.close();
                        }

                        if ui.button("ℹ Help & Shortcuts").on_hover_text("View instructions, psychometric background, keyboard shortcuts, and data privacy").clicked() {
                            self.show_help_dialog = true;
                            ui.close();
                        }

                        if ui.button("📄 Research Paper (DOI)").on_hover_text("Read published paper: 'Revisiting the IPIP-NEO personality hierarchy with taxonomic graph analysis' (doi:10.1177/08902070251352590)").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://doi.org/10.1177/08902070251352590"));
                            ui.close();
                        }

                        if ui.button("🐙 GitHub Repository").on_hover_text("View open-source repository and Rust codebase on GitHub").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/Spodeian/Revisited-IPIP-NEO"));
                            ui.close();
                        }

                        ui.separator();
                        if ui.button("▲ Hide Header Bar").on_hover_text("Collapse top navigation header to maximize questionnaire focus area").clicked() {
                            self.hide_header = true;
                            ui.close();
                        }
                    });

                    // Quick Results toggle button right beside Menu
                    let results_btn_text = if self.state.questionnaire.show_results {
                        "❓ Questions"
                    } else {
                        "📊 Results"
                    };
                    let mut res_btn = egui::Button::new(egui::RichText::new(results_btn_text).strong());
                    if is_touch {
                        res_btn = res_btn.min_size(egui::vec2(0.0, btn_height));
                    }
                    if ui.add(res_btn).on_hover_text("Toggle between questionnaire answering view and hierarchical results report").clicked() {
                        self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                    }

                    // Left side: Title and status indicators in remaining horizontal space
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                        let title_size = if avail_w < 480.0 { 16.0 } else if avail_w < 780.0 { 18.0 } else { 20.0 };
                        ui.label(egui::RichText::new(title_text).size(title_size).strong())
                            .on_hover_text("Revisited IPIP-NEO Personality Assessment based on Taxonomic Graph Analysis (Samo et al., 2026)");

                        // Dynamic Status Badges
                        if let Some(save_t) = self.last_save_time
                            && current_time - save_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(70, 180, 90), "✔ Saved")
                                .on_hover_text("Your assessment responses are saved automatically to local storage");
                        }
                        if let Some(undo_t) = self.undo_notification_time
                            && current_time - undo_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(240, 160, 40), "↩ Undone")
                                .on_hover_text("Previous answer change was undone (Ctrl+Z / Cmd+Z)");
                        }
                        if let Some(redo_t) = self.redo_notification_time
                            && current_time - redo_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(100, 180, 240), "↪ Redone")
                                .on_hover_text("Previous answer change was restored (Ctrl+Y / Cmd+Shift+Z)");
                        }
                        if avail_w > 520.0 && let Some(est) = self.calculate_estimated_time_remaining() {
                            ui.label(egui::RichText::new(format!("⏱ {}", est)).small().weak())
                                .on_hover_text("Estimated time to complete remaining questions based on your average answering speed");
                        }
                    });
                });
            });

            ui.add_space(4.0);
        });
    }

    fn render_question_focus(&mut self, ui: &mut egui::Ui, constraints: &ScreenConstraints) {
        let total = self.state.questionnaire.total_questions();
        if total == 0 {
            ui.centered_and_justified(|ui| {
                ui.label("No questions loaded.");
            });
            return;
        }

        let curr_idx = self.state.questionnaire.current_focus_idx;
        let (_q_id, q_text, q_response) = {
            let q = match self.state.questionnaire.questions.get(curr_idx) {
                Some(q) => q,
                None => return,
            };
            (q.id, q.text.clone(), q.response)
        };

        let is_mobile_portrait = constraints.is_mobile_portrait;
        let is_tight_height = constraints.is_tight_height;
        let is_ultra_tight = constraints.is_ultra_tight;
        let avail_width = ui.available_width();

        let _scroll_response = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Scale spacing based on screen constraints
                let top_space = if is_ultra_tight { 0.0 } else if is_tight_height { 4.0 } else if is_mobile_portrait { 8.0 } else { 20.0 };
                ui.add_space(top_space);

                // Centered question card container
                ui.vertical_centered(|ui| {
                    // Limit width strictly to fit screens perfectly (with scrollbar space)
                    let max_width = (avail_width - 8.0).min(700.0);
                    ui.set_max_width(max_width);

                    // Prominent Framing Instruction
                    let framing_font_size = if is_ultra_tight { 15.0 } else if is_tight_height { 17.0 } else { 19.0 };
                    ui.label(
                        egui::RichText::new("Rate how accurately this statement describes you:")
                            .size(framing_font_size)
                            .strong()
                            .color(ui.visuals().hyperlink_color),
                    ).on_hover_text("Select the response option (1–5) that best reflects your typical behavior and self-perception");
                    ui.add_space(if is_ultra_tight { 4.0 } else { 10.0 });

                    // Question Statement Box (scaled down for small screens)
                    let card_padding = if is_ultra_tight { 6.0 } else if is_tight_height { 12.0 } else if is_mobile_portrait { 16.0 } else { 24.0 };
                    let font_size = if is_ultra_tight { 16.0 } else if is_tight_height { 18.0 } else if is_mobile_portrait { 21.0 } else { 26.0 };

                    let card_frame = egui::Frame::group(ui.style())
                        .inner_margin(card_padding)
                        .corner_radius(8.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(&q_text)
                                        .size(font_size)
                                        .strong(),
                                );
                            });
                        });

                    if let Some(curr_q) = self.state.questionnaire.questions.get(curr_idx) {
                        card_frame.response.on_hover_ui(|ui| {
                            ui.label(egui::RichText::new(format!("Item #{} of {}", curr_q.id, total)).strong());
                            ui.add_space(2.0);
                            ui.label(format!("\"{}\"", curr_q.text));
                            ui.add_space(4.0);
                            ui.label(format!("• Facet: {} ({:+.3} weight)", curr_q.facet.category.display_name(), curr_q.facet.weight));
                            ui.label(format!("• Trait: {} ({:+.3} weight)", curr_q.r#trait.category.display_name(), curr_q.r#trait.weight));
                            ui.label(format!("• Meta-Trait: {} ({:+.3} weight)", curr_q.meta_trait.category.display_name(), curr_q.meta_trait.weight));
                            if let Some(r) = curr_q.response {
                                ui.label(format!("• Current Response: {} ({:+.1})", r.label(), r.to_score()));
                            } else {
                                ui.label("• Status: Unanswered");
                            }
                        });
                    }

                    let space_after_card = if is_ultra_tight { 4.0 } else if is_tight_height { 6.0 } else if is_mobile_portrait { 15.0 } else { 30.0 };
                    ui.add_space(space_after_card);

                    let responses = [
                        (Response::StronglyDisagree, "Strongly Disagree", "1", "Strongly Disagree (-1.0 point)\nShortcut: Press '1' on keyboard"),
                        (Response::Disagree, "Disagree", "2", "Disagree (-0.5 points)\nShortcut: Press '2' on keyboard"),
                        (Response::Neutral, "Neutral", "3", "Neutral (0.0 points)\nShortcut: Press '3' on keyboard"),
                        (Response::Agree, "Agree", "4", "Agree (+0.5 points)\nShortcut: Press '4' on keyboard"),
                        (Response::StronglyAgree, "Strongly Agree", "5", "Strongly Agree (+1.0 point)\nShortcut: Press '5' on keyboard"),
                    ];

                    // Clean vertical stack for Likert buttons across all orientations
                    let button_height = if is_ultra_tight { 26.0 } else if is_tight_height { 32.0 } else if is_mobile_portrait { 36.0 } else { 42.0 };
                    let button_text_size = if is_ultra_tight { 13.5 } else if is_tight_height { 14.5 } else if is_mobile_portrait { 15.5 } else { 16.0 };
                    let btn_width = (ui.available_width() - 8.0).min(340.0);

                    for (resp, text, shortcut, tooltip_desc) in responses {
                        let is_selected = q_response == Some(resp);

                        // Remove keyboard shortcut hints on mobile touchscreens to save space
                        let button_text = if constraints.is_mobile {
                            text.to_string()
                        } else {
                            format!("[{}]  {}", shortcut, text)
                        };

                        let mut rich_text = egui::RichText::new(button_text).size(button_text_size);
                        if is_selected {
                            rich_text = rich_text.strong();
                        }

                        let btn = egui::Button::new(rich_text)
                            .min_size(egui::vec2(btn_width, button_height))
                            .selected(is_selected);

                        if ui.add(btn).on_hover_text(tooltip_desc).clicked() {
                            self.is_viewing_shared_link = false;
                            self.state.questionnaire.answer_question(curr_idx, resp);
                            let current_t = ui.input(|i| i.time);
                            self.answer_timestamps.push_back(current_t);
                            if self.answer_timestamps.len() > 25 {
                                self.answer_timestamps.pop_front();
                            }
                            self.last_save_time = Some(current_t);
                        }
                        ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 6.0 });
                    }

                    let space_before_nav = if is_ultra_tight { 4.0 } else if is_tight_height { 6.0 } else if is_mobile_portrait { 15.0 } else { 25.0 };
                    ui.add_space(space_before_nav);
                    ui.separator();
                    ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 10.0 });

                    // Navigation and Progress Row (Progress bar in between Left and Right button clusters)
                    let curr_focus = curr_idx + 1;
                    let progress = self.state.questionnaire.completion_rate();
                    let progress_text = format!("Item #{} of {} ({:.0}%)", curr_focus, total, progress * 100.0);
                    let answered = self.state.questionnaire.answered_count();
                    let remaining = total.saturating_sub(answered);
                    let progress_hover_text = format!(
                        "Assessment Progress: {}/{} answered ({:.1}%)\n{} remaining question{}",
                        answered, total, progress * 100.0, remaining, if remaining == 1 { "" } else { "s" }
                    );

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(if is_ultra_tight { 4.0 } else { 6.0 }, 6.0);

                        // Left Action Cluster
                        let btn_prev = if is_ultra_tight { "◀" } else { "◀ Prev" };
                        if ui.button(btn_prev).on_hover_text("Previous item in sequence (Left Arrow / Mouse Scroll Up)").clicked() {
                            self.state.questionnaire.navigate_previous();
                        }

                        let btn_prev_un = if is_ultra_tight { "⏪" } else { "⏪ Unanswered" };
                        if ui.button(btn_prev_un).on_hover_text("Jump backward to nearest unanswered question (Shift + Left Arrow)").clicked() {
                            self.state.questionnaire.navigate_previous_unanswered();
                        }

                        if self.state.questionnaire.can_undo() {
                            let btn_undo = if is_ultra_tight { "↩" } else { "↩ Undo" };
                            if ui.button(btn_undo).on_hover_text("Undo previous response change (Ctrl+Z / Cmd+Z)").clicked()
                                && self.state.questionnaire.undo()
                            {
                                let current_t = ui.input(|i| i.time);
                                self.undo_notification_time = Some(current_t);
                                self.last_save_time = Some(current_t);
                            }
                        }

                        if self.state.questionnaire.can_redo() {
                            let btn_redo = if is_ultra_tight { "↪" } else { "↪ Redo" };
                            if ui.button(btn_redo).on_hover_text("Redo reverted response change (Ctrl+Y / Cmd+Shift+Z)").clicked()
                                && self.state.questionnaire.redo()
                            {
                                let current_t = ui.input(|i| i.time);
                                self.redo_notification_time = Some(current_t);
                                self.last_save_time = Some(current_t);
                            }
                        }

                        // Right Action Cluster + Center Expanding Progress Bar
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_skip = if is_ultra_tight { "⏭" } else { "Skip ⏭" };
                            if ui.button(btn_skip).on_hover_text("Skip question and defer to end of pending queue (Right Arrow / Mouse Scroll Down)").clicked() {
                                self.state.questionnaire.skip_current();
                            }

                            let btn_next_un = if is_ultra_tight { "⏩" } else { "Next Unanswered ⏩" };
                            if ui.button(btn_next_un).on_hover_text("Jump forward to nearest unanswered question (Shift + Right Arrow)").clicked() {
                                self.state.questionnaire.navigate_next_unanswered();
                            }

                            if q_response.is_some() {
                                let btn_clear = if is_ultra_tight { "❌" } else { "❌ Clear" };
                                if ui.button(btn_clear).on_hover_text("Clear recorded answer for this question and mark it unanswered").clicked() {
                                    self.is_viewing_shared_link = false;
                                    self.state.questionnaire.clear_response(curr_idx);
                                    let current_t = ui.input(|i| i.time);
                                    self.last_save_time = Some(current_t);
                                }
                            }

                            // Center Progress Bar filling remaining horizontal space in between
                            let remaining_width = (ui.available_width() - 8.0).max(0.0);
                            if remaining_width > 20.0 {
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .text(progress_text.clone())
                                        .desired_width(remaining_width),
                                ).on_hover_text(&progress_hover_text);
                            }
                        });
                    });
                });
            });

        // Throttled and debounced scroll detection for Desktop "Scroll to Skip"
        let current_time = ui.input(|i| i.time);
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);

        if !is_tight_height && !is_mobile_portrait && !is_ultra_tight && ui.rect_contains_pointer(ui.max_rect()) {
            if scroll_y.abs() > 1.0 {
                self.scroll_accumulator += scroll_y;
            }

            // Cooldown of 350ms and delta accumulation threshold of 40.0
            if current_time - self.last_scroll_time > 0.35 {
                if self.scroll_accumulator < -40.0 {
                    // Scrolling down -> skip/defer forwards
                    self.state.questionnaire.skip_current();
                    self.last_scroll_time = current_time;
                    self.scroll_accumulator = 0.0;
                } else if self.scroll_accumulator > 40.0 {
                    // Scrolling up -> navigate backwards
                    self.state.questionnaire.navigate_previous();
                    self.last_scroll_time = current_time;
                    self.scroll_accumulator = 0.0;
                }
            }

            // Reset accumulator if user stopped scrolling for half a second
            if current_time - self.last_scroll_time > 0.5 && scroll_y.abs() < 1.0 {
                self.scroll_accumulator = 0.0;
            }
        } else {
            self.scroll_accumulator = 0.0;
        }

        // Fix UI Repaint Freeze on Mouse Scroll: Ensure egui continues rendering frame cycles
        // to reset the scroll cooldown state even if mouse movement stops.
        if self.scroll_accumulator.abs() > 0.0 {
            ui.ctx().request_repaint();
        }
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Assessment Results");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").on_hover_text("Close results panel and return to questions (Escape)").clicked() {
                        self.state.questionnaire.show_results = false;
                    }
                });
            });

            ui.add_space(6.0);
            let answered = self.state.questionnaire.answered_count();
            let total = self.state.questionnaire.total_questions();
            let pct = self.state.questionnaire.completion_rate() * 100.0;
            ui.label(format!("Completed: {}/{} ({:.1}%)", answered, total, pct))
                .on_hover_text("Total items answered out of the 221-item questionnaire");

            if self.is_viewing_shared_link {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Viewing shared results link. Your own saved answers are preserved unless you answer or modify questions.")
                        .color(egui::Color32::from_rgb(147, 197, 253))
                        .small(),
                );
                ui.add_space(2.0);
                if ui.button("↩ Return to My Saved Assessment").on_hover_text("Exit shared link view and restore your local saved answers").clicked() {
                    self.restore_saved_instance();
                }
            }

            ui.add_space(4.0);
            ui.checkbox(&mut self.state.questionnaire.show_detailed_stats, "Show Detailed Metrics & SE")
                .on_hover_text("Toggle raw score sums, absolute item weights, sample counts, and Standard Error (SE) values");

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Share Link").on_hover_text("Copy shareable results URL to clipboard without affecting recipients' saved progress").clicked() {
                    let code = encode_responses_to_url_code(&self.state.questionnaire);

                    let full_url = {
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let loc = window.location();
                                let origin = loc.origin().unwrap_or_else(|_| "".to_string());
                                let pathname = loc.pathname().unwrap_or_else(|_| "".to_string());
                                format!("{}{}/#r={}", origin, pathname.trim_end_matches('/'), code)
                            } else {
                                format!("https://tga-ipip-neo.spodeian.trade/#r={}", code)
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            format!("https://tga-ipip-neo.spodeian.trade/#r={}", code)
                        }
                    };

                    ui.ctx().copy_text(full_url);
                    self.share_link_copied_time = Some(ui.input(|i| i.time));
                }

                ui.separator();

                // Consolidated Dropdown & Single Action Button
                egui::ComboBox::from_id_salt("export_format_dropdown")
                    .selected_text(self.selected_export_format.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Csv, "CSV File");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Json, "JSON File");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Bson, "Compressed BSON (.bson)");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Svg, "SVG Vector Graphic");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Html, "HTML Report");
                    });

                if ui.button("Download File").on_hover_text("Export results and download selected file format to your device").clicked() {
                    match self.selected_export_format {
                        ExportFormat::Csv => {
                            let csv_content = export_to_csv(&self.state.questionnaire);
                            trigger_file_download("ipip_neo_tga_results.csv", &csv_content, "text/csv;charset=utf-8");
                        }
                        ExportFormat::Json => {
                            let json_content = export_to_json(&self.state.questionnaire);
                            trigger_file_download("ipip_neo_tga_results.json", &json_content, "application/json;charset=utf-8");
                        }
                        ExportFormat::Bson => {
                            if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                                trigger_binary_download("ipip_neo_tga_results.bson", &bytes, "application/octet-stream");
                            }
                        }
                        ExportFormat::Svg => {
                            let svg_content = export_to_svg(&self.state.questionnaire);
                            trigger_file_download("ipip_neo_tga_results.svg", &svg_content, "image/svg+xml;charset=utf-8");
                        }
                        ExportFormat::Html => {
                            let html_content = export_to_printable_html(&self.state.questionnaire);
                            trigger_file_download("ipip_neo_tga_report.html", &html_content, "text/html;charset=utf-8");
                        }
                    }
                }
            });

            if let Some(t) = self.share_link_copied_time
                && ui.input(|i| i.time) - t < 3.0
            {
                ui.label(egui::RichText::new("Share link copied to clipboard!").color(egui::Color32::from_rgb(80, 180, 90)).strong());
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Construct Hierarchy Tree
            egui::ScrollArea::vertical().show(ui, |ui| {
                for &meta in &MetaTrait::ALL {
                    self.render_meta_trait_node(ui, meta);
                    ui.add_space(8.0);
                }
            });
        });
    }

    fn render_meta_trait_node(&self, ui: &mut egui::Ui, meta: MetaTrait) {
        let acc = self.state.questionnaire.meta_trait_acc.get(&meta).copied().unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;
        let id = ui.make_persistent_id(meta.display_name());
        let collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        collapsing
            .show_header(ui, |ui| {
                let label_resp = ui.label(egui::RichText::new(meta.display_name()).strong().size(15.0));
                label_resp.on_hover_ui(|ui| {
                    ui.label(egui::RichText::new(format!("Meta-Trait: {}", meta.display_name())).strong());
                    ui.add_space(2.0);
                    ui.label(meta.description());
                    ui.add_space(4.0);
                    let children = meta.child_traits().iter().map(|t| t.display_name()).collect::<Vec<_>>().join(", ");
                    ui.label(egui::RichText::new(format!("Subordinate Traits: {}", children)).small().weak());
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_construct_badge_row(
                        ui,
                        &acc,
                        show_detailed,
                        std::f32::consts::E,
                        "e",
                    );
                });
            })
            .body(|ui| {
                for trait_item in meta.child_traits() {
                    self.render_trait_node(ui, trait_item);
                }
            });
    }

    fn render_trait_node(&self, ui: &mut egui::Ui, trait_item: Trait) {
        let acc = self.state.questionnaire.trait_acc.get(&trait_item).copied().unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;
        let id = ui.make_persistent_id(trait_item.display_name());
        let collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        collapsing
            .show_header(ui, |ui| {
                let label_resp = ui.label(egui::RichText::new(trait_item.display_name()).size(14.0));
                label_resp.on_hover_ui(|ui| {
                    ui.label(egui::RichText::new(format!("Trait: {}", trait_item.display_name())).strong());
                    ui.add_space(2.0);
                    ui.label(trait_item.description());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("Parent Meta-Trait: {}", trait_item.parent_meta_trait().display_name())).small().weak());
                    let children = trait_item.child_facets().iter().map(|f| f.display_name()).collect::<Vec<_>>().join(", ");
                    ui.label(egui::RichText::new(format!("Subordinate Facets: {}", children)).small().weak());
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_construct_badge_row(
                        ui,
                        &acc,
                        show_detailed,
                        std::f32::consts::E / 2.0,
                        "e/2",
                    );
                });
            })
            .body(|ui| {
                for facet in trait_item.child_facets() {
                    self.render_facet_row(ui, facet);
                }
            });
    }

    fn render_facet_row(&self, ui: &mut egui::Ui, facet: Facet) {
        let acc = self.state.questionnaire.facet_acc.get(&facet).copied().unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;

        ui.horizontal(|ui| {
            let label_resp = ui.label(facet.display_name());
            label_resp.on_hover_ui(|ui| {
                ui.label(egui::RichText::new(format!("Facet: {}", facet.display_name())).strong());
                ui.add_space(2.0);
                ui.label(facet.description());
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("Parent Trait: {} (under {})", facet.parent_trait().display_name(), facet.parent_trait().parent_meta_trait().display_name())).small().weak());
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_construct_badge_row(
                    ui,
                    &acc,
                    show_detailed,
                    std::f32::consts::E / 4.0,
                    "e/4",
                );
            });
        });
    }

    fn render_score_gauge(
        ui: &mut egui::Ui,
        norm_score: f32,
        se: f32,
        tier_color: egui::Color32,
        width: f32,
        ci_mult: f32,
        ci_label: &str,
    ) {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 14.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let is_dark = ui.visuals().dark_mode;
            let track_bg = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25)
            };

            // Track background [-1.0, 1.0]
            painter.rect_filled(rect, 3.0, track_bg);

            // Center zero tick
            let center_x = rect.left() + rect.width() * 0.5;
            let tick_color = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
            };
            painter.line_segment(
                [egui::pos2(center_x, rect.top() + 1.0), egui::pos2(center_x, rect.bottom() - 1.0)],
                egui::Stroke::new(1.0, tick_color),
            );

            // Scale score from [-1.0, 1.0] to [rect.left(), rect.right()]
            let score_to_x = |s: f32| -> f32 {
                let norm = ((s.clamp(-1.0, 1.0) + 1.0) / 2.0).clamp(0.0, 1.0);
                rect.left() + norm * rect.width()
            };

            let score_x = score_to_x(norm_score);
            let center_y = rect.center().y;

            // Error bracket scaled by construct hierarchy multiplier, strictly clamped to [-1.0, 1.0]
            let error_span = se * ci_mult;
            let ci_min = (norm_score - error_span).clamp(-1.0, 1.0);
            let ci_max = (norm_score + error_span).clamp(-1.0, 1.0);
            let left_ci_x = score_to_x(ci_min);
            let right_ci_x = score_to_x(ci_max);

            // CI error band (semi-transparent)
            let band_color = egui::Color32::from_rgba_unmultiplied(
                tier_color.r(),
                tier_color.g(),
                tier_color.b(),
                80,
            );
            let ci_rect = egui::Rect::from_min_max(
                egui::pos2(left_ci_x, center_y - 3.0),
                egui::pos2(right_ci_x, center_y + 3.0),
            );
            painter.rect_filled(ci_rect, 2.0, band_color);

            // Error bar caps / strokes
            painter.line_segment(
                [egui::pos2(left_ci_x, center_y - 4.0), egui::pos2(left_ci_x, center_y + 4.0)],
                egui::Stroke::new(1.0, tier_color),
            );
            painter.line_segment(
                [egui::pos2(right_ci_x, center_y - 4.0), egui::pos2(right_ci_x, center_y + 4.0)],
                egui::Stroke::new(1.0, tier_color),
            );

            // Score point dot
            painter.circle_filled(egui::pos2(score_x, center_y), 4.5, tier_color);
            painter.circle_stroke(
                egui::pos2(score_x, center_y),
                4.5,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        }

        let ci_min = (norm_score - se * ci_mult).clamp(-1.0, 1.0);
        let ci_max = (norm_score + se * ci_mult).clamp(-1.0, 1.0);
        response.on_hover_ui(|ui| {
            ui.label(egui::RichText::new(format!("Normalized Score: {:+.2}", norm_score)).strong());
            ui.label(format!("Standard Error (SE): {:.2}", se));
            ui.label(format!("Confidence Interval (±{}×SE): [{:+.2}, {:+.2}]", ci_label, ci_min, ci_max));
        });
    }

    fn render_construct_badge_row(
        &self,
        ui: &mut egui::Ui,
        acc: &shared::ScoreAccumulator,
        show_detailed: bool,
        ci_mult: f32,
        ci_label: &str,
    ) {
        if let Some(norm_score) = acc.normalized_score() {
            let tier = acc.tier().unwrap_or(ScoreTier::Average);
            let tier_color = match tier {
                ScoreTier::VeryLow => egui::Color32::from_rgb(220, 70, 70),
                ScoreTier::Low => egui::Color32::from_rgb(230, 140, 50),
                ScoreTier::Average => egui::Color32::from_rgb(140, 140, 150),
                ScoreTier::High => egui::Color32::from_rgb(70, 170, 90),
                ScoreTier::VeryHigh => egui::Color32::from_rgb(30, 140, 220),
            };

            let se = acc.standard_error().unwrap_or(0.0);
            Self::render_score_gauge(ui, norm_score, se, tier_color, 80.0, ci_mult, ci_label);

            let tier_badge_resp = ui.colored_label(tier_color, egui::RichText::new(tier.label()).strong());
            tier_badge_resp.on_hover_ui(|ui| {
                ui.label(egui::RichText::new(format!("Classification: {}", tier.label())).strong());
                ui.label(format!("Normalized Score: {:+.2} (scale: -1.0 to +1.0)", norm_score));
                if let Some(se_val) = acc.standard_error() {
                    ui.label(format!("Standard Error (SE): {:.3}", se_val));
                }
                ui.label(format!("Progress: {}/{} items answered", acc.answered_count, acc.total_items));
            });

            if show_detailed {
                ui.label(
                    egui::RichText::new(format!(
                        "score: {:.2} (SE: {:.2}, raw: {:.1}, n: {})",
                        norm_score, se, acc.raw_score, acc.answered_count
                    ))
                    .small()
                    .weak(),
                ).on_hover_text(format!(
                    "Detailed Metrics:\n• Normalized Score: {:.4}\n• Standard Error: {:.4}\n• Raw Sum: {:.2}\n• Absolute Weight Sum: {:.2}\n• Answered Items: {} / {}",
                    norm_score, se, acc.raw_score, acc.total_abs_weight, acc.answered_count, acc.total_items
                ));
            }
        } else {
            ui.label(egui::RichText::new("No items yet").weak().small())
                .on_hover_text("No questions for this construct have been answered yet");
        }
    }

    fn render_dialogs(&mut self, ui: &mut egui::Ui) {
        if self.show_grid_dialog {
            self.render_grid_dialog(ui);
        }
        if self.show_help_dialog {
            self.render_help_dialog(ui);
        }
        if self.show_reset_dialog {
            self.render_reset_dialog(ui);
        }
        if self.show_storage_modal {
            self.render_storage_modal(ui);
        }
        if self.show_export_dialog.is_some() {
            self.render_export_dialog(ui);
        }
        if self.show_import_dialog {
            self.render_import_dialog(ui);
        }
    }

    fn render_storage_modal(&mut self, ui: &mut egui::Ui) {
        let mut open = true;
        egui::Window::new("Storage & Offline Persistence Diagnostics")
            .open(&mut open)
            .default_size(egui::vec2(540.0, 480.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.heading("Storage Durability & Quota");
                ui.add_space(4.0);

                let is_persisted = self.storage_diag.is_persisted;
                match is_persisted {
                    Some(true) => {
                        ui.horizontal(|ui| {
                            ui.label("• Persistence Mode:");
                            ui.colored_label(egui::Color32::from_rgb(80, 180, 90), "Persistent (Immune to eviction)");
                        });
                        ui.label("The browser has granted persistent storage. Your responses and assessments are protected against automatic browser cache clearing.");
                    }
                    Some(false) => {
                        ui.horizontal(|ui| {
                            ui.label("• Persistence Mode:");
                            ui.colored_label(egui::Color32::from_rgb(230, 140, 50), "Best-Effort / Ephemeral");
                        });
                        ui.label("Storage permission has not been granted. Browsers (especially Safari ITP after 7 days or Chrome under disk pressure) may evict site data.");
                        ui.add_space(4.0);
                        if ui.button("Request Persistent Storage Permission")
                            .on_hover_text("Ask your browser for persistent storage permissions to prevent eviction")
                            .clicked()
                        {
                            request_persistent_storage();
                        }
                    }
                    None => {
                        ui.horizontal(|ui| {
                            ui.label("• Persistence Mode:");
                            ui.colored_label(egui::Color32::GRAY, "Checking Browser API...");
                        });
                        if ui.button("Request Persistent Storage")
                            .on_hover_text("Ask your browser for persistent storage permissions to prevent eviction")
                            .clicked()
                        {
                            request_persistent_storage();
                        }
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Active Storage Tier & Compaction");
                ui.add_space(4.0);
                ui.label(format!("• Current Engine: {}", self.storage_diag.backend.label()));
                ui.label(format!("• Undo/Redo Depth: {} undo actions, {} redo actions", self.state.questionnaire.undo_stack.len(), self.state.questionnaire.redo_stack.len()));
                if self.storage_diag.quota_exceeded {
                    ui.colored_label(egui::Color32::from_rgb(220, 70, 70), "localStorage quota reached; undo history was compacted. Consider saving a .bson backup.");
                } else {
                    ui.label("Storage engine uses LocalStorage with automatic recency-density undo compaction.");
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Progressive Web App (PWA)");
                ui.add_space(4.0);
                if self.storage_diag.is_pwa_installed {
                    ui.colored_label(egui::Color32::from_rgb(80, 180, 90), "App is installed as standalone PWA.");
                } else if self.storage_diag.pwa_install_available {
                    ui.label("Installing this assessment to your home screen or desktop grants permanent storage status.");
                    if ui.button("Install Assessment App")
                        .on_hover_text("Install assessment as a standalone Progressive Web App for permanent offline durability")
                        .clicked()
                    {
                        trigger_pwa_install();
                    }
                } else {
                    ui.label("PWA offline capabilities are active. You can also add this page to your home screen via browser options.");
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Manual Backups & Compressed BSON");
                ui.add_space(4.0);
                ui.label("Download a standalone compressed binary backup of your assessment to store on disk or transfer across devices:");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Download .bson Backup").on_hover_text("Download compact, binary compressed assessment state").clicked()
                        && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                    {
                        trigger_binary_download("ipip_neo_assessment_backup.bson", &bytes, "application/octet-stream");
                    }
                    if ui.button("Import Saved File").on_hover_text("Open import window to restore saved assessment data").clicked() {
                        self.show_import_dialog = true;
                    }
                });
            });

        if !open {
            self.show_storage_modal = false;
        }
    }

    fn render_warning_banners(&mut self, ui: &mut egui::Ui) {
        let is_ephemeral = self.storage_diag.is_persisted == Some(false);
        let quota_exceeded = self.storage_diag.quota_exceeded;

        // Combined Warning: Both Ephemeral & Quota Exceeded
        if is_ephemeral && quota_exceeded && !self.dismissed_combined_warning {
            egui::Frame::group(ui.style())
                .fill(if ui.visuals().dark_mode { egui::Color32::from_rgb(60, 20, 20) } else { egui::Color32::from_rgb(255, 230, 230) })
                .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 50, 50)))
                .inner_margin(8.0)
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("Storage Alert:").strong().color(egui::Color32::from_rgb(220, 50, 50)));
                        ui.label("Storage is ephemeral and quota is constrained. To prevent data loss:");
                        if ui.button("Request Persistence")
                            .on_hover_text("Ask your browser for persistent storage permissions to prevent eviction")
                            .clicked()
                        {
                            request_persistent_storage();
                        }
                        if self.storage_diag.pwa_install_available && !self.storage_diag.is_pwa_installed && ui.button("Install App").on_hover_text("Install application for permanent offline storage").clicked() {
                            trigger_pwa_install();
                        }
                        if ui.button("Save .bson Backup").on_hover_text("Download compressed binary backup of your assessment").clicked()
                            && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                        {
                            trigger_binary_download("ipip_neo_assessment_backup.bson", &bytes, "application/octet-stream");
                        }
                        if ui.small_button("Dismiss").on_hover_text("Dismiss this warning banner").clicked() {
                            self.dismissed_combined_warning = true;
                        }
                    });
                });
            ui.add_space(6.0);
        } else if is_ephemeral && !self.dismissed_ephemeral_warning {
            // Ephemeral Only Warning Banner
            egui::Frame::group(ui.style())
                .fill(if ui.visuals().dark_mode { egui::Color32::from_rgb(45, 35, 15) } else { egui::Color32::from_rgb(255, 248, 225) })
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 140, 50)))
                .inner_margin(8.0)
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("Storage is ephemeral:").strong().color(egui::Color32::from_rgb(230, 140, 50)));
                        ui.label("Browser may evict data under disk pressure.");
                        if ui.button("Request Persistence")
                            .on_hover_text("Ask your browser for persistent storage permissions to prevent eviction")
                            .clicked()
                        {
                            request_persistent_storage();
                        }
                        if self.storage_diag.pwa_install_available && !self.storage_diag.is_pwa_installed && ui.button("Install App").on_hover_text("Install application for permanent offline storage").clicked() {
                            trigger_pwa_install();
                        }
                        if ui.small_button("Dismiss").on_hover_text("Dismiss this warning banner").clicked() {
                            self.dismissed_ephemeral_warning = true;
                        }
                    });
                });
            ui.add_space(6.0);
        } else if quota_exceeded && !self.dismissed_quota_warning {
            // Quota Exceeded Warning Banner
            egui::Frame::group(ui.style())
                .fill(if ui.visuals().dark_mode { egui::Color32::from_rgb(45, 35, 15) } else { egui::Color32::from_rgb(255, 248, 225) })
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 140, 50)))
                .inner_margin(8.0)
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new("Quota Warning:").strong().color(egui::Color32::from_rgb(230, 140, 50)));
                        ui.label("LocalStorage limit reached; undo history compacted to conserve space.");
                        if ui.button("Save .bson Backup").on_hover_text("Download compressed binary backup of your assessment").clicked()
                            && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                        {
                            trigger_binary_download("ipip_neo_assessment_backup.bson", &bytes, "application/octet-stream");
                        }
                        if ui.small_button("Dismiss").on_hover_text("Dismiss this warning banner").clicked() {
                            self.dismissed_quota_warning = true;
                        }
                    });
                });
            ui.add_space(6.0);
        }
    }

    fn render_grid_dialog(&mut self, ui: &mut egui::Ui) {
        let mut open = true;
        let win_w = (ui.available_width() - 24.0).clamp(320.0, 580.0);
        let win_h = (ui.available_height() - 32.0).clamp(380.0, 540.0);

        egui::Window::new("Question Item Matrix (221 Items)")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size(egui::vec2(win_w, win_h))
            .min_width(300.0)
            .min_height(340.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Click any item to jump directly to that question.").small().weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let answered = self.state.questionnaire.answered_count();
                        let total = self.state.questionnaire.total_questions();
                        ui.label(egui::RichText::new(format!("{}/{} Answered", answered, total)).strong())
                            .on_hover_text("Total questions answered out of 221");
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        let curr_idx = self.state.questionnaire.current_focus_idx;

                        for (idx, q) in self.state.questionnaire.questions.iter().enumerate() {
                            let is_curr = idx == curr_idx;
                            let (bg_color, text_color, status_text) = match q.response {
                                Some(Response::StronglyAgree) => (
                                    egui::Color32::from_rgb(34, 139, 34),
                                    egui::Color32::WHITE,
                                    "Strongly Agree",
                                ),
                                Some(Response::Agree) => (
                                    egui::Color32::from_rgb(70, 170, 90),
                                    egui::Color32::WHITE,
                                    "Agree",
                                ),
                                Some(Response::Neutral) => (
                                    egui::Color32::from_rgb(140, 140, 150),
                                    egui::Color32::WHITE,
                                    "Neutral",
                                ),
                                Some(Response::Disagree) => (
                                    egui::Color32::from_rgb(230, 140, 50),
                                    egui::Color32::WHITE,
                                    "Disagree",
                                ),
                                Some(Response::StronglyDisagree) => (
                                    egui::Color32::from_rgb(220, 70, 70),
                                    egui::Color32::WHITE,
                                    "Strongly Disagree",
                                ),
                                None => {
                                    if ui.visuals().dark_mode {
                                        (egui::Color32::from_rgb(50, 50, 55), egui::Color32::LIGHT_GRAY, "Unanswered")
                                    } else {
                                        (egui::Color32::from_rgb(220, 220, 225), egui::Color32::DARK_GRAY, "Unanswered")
                                    }
                                }
                            };

                            let mut btn_text = egui::RichText::new(format!("{}", q.id)).size(11.0).color(text_color);
                            if is_curr {
                                btn_text = btn_text.strong();
                            }

                            let mut btn = egui::Button::new(btn_text)
                                .fill(bg_color)
                                .min_size(egui::vec2(28.0, 24.0));

                            if is_curr {
                                btn = btn.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(30, 140, 240)));
                            }

                            let tooltip = format!(
                                "#{}. {}\nFacet: {} | Trait: {} | Meta-Trait: {}\nStatus: {}",
                                q.id,
                                q.text,
                                q.facet.category.display_name(),
                                q.facet.category.parent_trait().display_name(),
                                q.facet.category.parent_trait().parent_meta_trait().display_name(),
                                status_text
                            );

                            if ui.add(btn).on_hover_text(tooltip).clicked() {
                                self.state.questionnaire.current_focus_idx = idx;
                                self.show_grid_dialog = false;
                            }
                        }
                    });
                });
                ui.add_space(8.0);
                if ui.button("Close").on_hover_text("Close item matrix map (Escape)").clicked() {
                    self.show_grid_dialog = false;
                }
            });

        if !open {
            self.show_grid_dialog = false;
        }
    }

    fn render_help_dialog(&mut self, ui: &mut egui::Ui) {
        let mut open = true;
        let win_w = (ui.available_width() - 24.0).clamp(340.0, 640.0);
        let win_h = (ui.available_height() - 32.0).clamp(480.0, 750.0);

        egui::Window::new("Help & Information")
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size(egui::vec2(win_w, win_h))
            .min_width(320.0)
            .min_height(420.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Revisited IPIP-NEO");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                        .strong()
                                        .color(ui.visuals().hyperlink_color),
                                );
                            });
                        });
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.heading("Estimated Time");
                        ui.add_space(4.0);
                        ui.label("~10–15 minutes (221 items). Take your time to answer honestly without overthinking.");

                        if self.is_viewing_shared_link {
                            ui.add_space(6.0);
                            egui::Frame::group(ui.style())
                                .inner_margin(8.0)
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Currently viewing shared link.").weak());
                                        if ui.button("Return to My Saved Assessment").on_hover_text("Exit shared link view and restore your local saved answers").clicked() {
                                            self.restore_saved_instance();
                                            self.show_help_dialog = false;
                                        }
                                    });
                                });
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Taxonomic Graph Analysis (TGA) Methodology
                        ui.heading("Taxonomic Graph Analysis (TGA)");
                        ui.add_space(4.0);
                        ui.label(
                            "This assessment implements Taxonomic Graph Analysis (TGA), a mathematically and statistically rigorous psychometric framework based on the IPIP-NEO:",
                        );
                        ui.label("• Empirical Factor Loadings: TGA statistically models the continuous loadings (w_i) of each question across all construct levels.");
                        ui.label("• Optimized Item Distillation: Trimmed redundant questions from the original 300-item inventory down to the distilled 221-item set.");
                        ui.label("• Dimensional Mapping: Statistically models topological connections across 3 Meta-Traits, 6 Traits, and 28 Facets.");
                        ui.label("• Dynamic Sequencing: Questions are ordered to achieve rapid convergence and minimize cumulative standard error.");

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Scoring & Confidence Intervals
                        ui.heading("Scoring & Confidence Intervals");
                        ui.add_space(4.0);
                        ui.label(
                            "Construct scores are normalized to [-1.0, +1.0]. Standard Error (SE) is projected to the normalized interval using discrete Likert response dispersion (σ = 0.5):",
                        );
                        ui.label(
                            egui::RichText::new("  SE = (√(Σ w_i²) / Σ |w_i|) × 0.5")
                                .monospace()
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            "Visual score error bars apply hierarchical confidence interval multipliers based on Euler's constant (e ≈ 2.718) and are strictly bounded on [-1.0, +1.0]:",
                        );
                        ui.label("• Meta-Traits (Global Factors): ±e × SE (≈ ±2.72 × SE)");
                        ui.label("• Traits (Broad Domains): ±(e / 2) × SE (≈ ±1.36 × SE)");
                        ui.label("• Facets (Specific Aspects): ±(e / 4) × SE (≈ ±0.68 × SE)");

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Keyboard Shortcuts & Navigation
                        ui.heading("Navigation & Keyboard Shortcuts");
                        ui.add_space(4.0);
                        ui.label("• 1, 2, 3, 4, 5: Select response (Strongly Disagree to Strongly Agree)");
                        ui.label("• Left / Up Arrow: Navigate to previous question");
                        ui.label("• Right / Down Arrow: Skip question (defers to back of queue)");
                        ui.label("• Shift + Left Arrow: Jump to previous unanswered question");
                        ui.label("• Shift + Right Arrow: Jump to next unanswered question");
                        ui.label("• Ctrl+Z / Cmd+Z: Undo previous response change");
                        ui.label("• Item Map: Open interactive 221-item matrix map");
                        ui.label("• Escape: Close dialogs or return from results screen");
                        ui.label("• Mouse Scroll: Scroll to skip / navigate questions (Desktop)");

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Privacy & Data Safety
                        ui.heading("Privacy & Data Safety");
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("100% Client-Side: No data ever leaves your device.")
                                .color(egui::Color32::from_rgb(80, 160, 90))
                                .strong(),
                        );
                        ui.label(
                            "This application executes entirely in your browser using WebAssembly. Responses, scores, and exports are never transmitted to any external server.",
                        );
                        ui.label(
                            egui::RichText::new("100% Offline Capable: PWA service worker caches all static assets for full offline use.")
                                .italics()
                                .small(),
                        );

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Academic Reference & Source Code
                        ui.heading("Academic Reference & Source");
                        ui.add_space(4.0);
                        ui.label("Samo, A., Garrido, L. E., Abad, F. J., Golino, H., McAbee, S. T., & Christensen, A. P. (2026). Revisiting the IPIP-NEO personality hierarchy with taxonomic graph analysis. European Journal of Personality, 40(2), 369–390.");
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label("Published Article:");
                            ui.hyperlink_to(
                                "doi:10.1177/08902070251352590",
                                "https://doi.org/10.1177/08902070251352590",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Open Science Framework (OSF):");
                            ui.hyperlink_to(
                                "osf.io/hwpa9",
                                "https://osf.io/hwpa9",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Source Code:");
                            ui.hyperlink_to(
                                "GitHub Repository",
                                "https://github.com/Spodeian/Revisited-IPIP-NEO",
                            );
                        });
                    });
            });
        if !open {
            self.show_help_dialog = false;
        }
    }

    fn render_reset_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Reset Assessment?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to clear all responses and start over?");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes, Reset").on_hover_text("Clear all answers, reset queue, and restart the questionnaire").clicked() {
                        self.is_viewing_shared_link = false;
                        self.state.reset_questionnaire();
                        self.show_reset_dialog = false;
                    }
                    if ui.button("Cancel").on_hover_text("Keep existing answers and return to questionnaire").clicked() {
                        self.show_reset_dialog = false;
                    }
                });
            });
    }

    fn render_export_dialog(&mut self, ui: &mut egui::Ui) {
        let export_format = match self.show_export_dialog {
            Some(fmt) => fmt,
            None => return,
        };

        // Cache export text buffer if it's empty (e.g. if dialog state was set directly)
        if self.export_text_buffer.is_empty() {
            if export_format == ExportFormat::Bson {
                if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                    use base64::{engine::general_purpose, Engine as _};
                    self.export_text_buffer = general_purpose::STANDARD.encode(&bytes);
                }
            } else {
                self.export_text_buffer = match export_format {
                    ExportFormat::Csv => export_to_csv(&self.state.questionnaire),
                    ExportFormat::Json => export_to_json(&self.state.questionnaire),
                    ExportFormat::Bson => unreachable!(),
                    ExportFormat::Svg => export_to_svg(&self.state.questionnaire),
                    ExportFormat::Html => export_to_printable_html(&self.state.questionnaire),
                };
            }
        }

        let title = match export_format {
            ExportFormat::Csv => "Export CSV",
            ExportFormat::Json => "Export JSON",
            ExportFormat::Bson => "Export Compressed BSON Binary",
            ExportFormat::Svg => "Export SVG Vector Graphic",
            ExportFormat::Html => "Printable Report (HTML/PDF)",
        };

        let mut open = true;
        egui::Window::new(title)
            .open(&mut open)
            .default_size(egui::vec2(600.0, 450.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if export_format == ExportFormat::Bson
                        && ui.button("Download .bson File").on_hover_text("Download compressed binary backup to your device").clicked()
                        && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                    {
                        trigger_binary_download("ipip_neo_assessment_backup.bson", &bytes, "application/octet-stream");
                    }
                    if ui.button("Copy to Clipboard").on_hover_text("Copy formatted export data directly to clipboard").clicked() {
                        ui.ctx().copy_text(self.export_text_buffer.clone());
                        self.export_copied_notification = Some(ui.input(|i| i.time));
                    }
                    if let Some(t) = self.export_copied_notification
                        && ui.input(|i| i.time) - t < 3.0
                    {
                        ui.label(egui::RichText::new("Copied to clipboard!").color(egui::Color32::GREEN));
                    }
                });

                ui.add_space(8.0);
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.export_text_buffer)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );
                });
            });

        if !open {
            self.show_export_dialog = None;
            self.export_text_buffer.clear();
        }
    }

    fn render_import_dialog(&mut self, ui: &mut egui::Ui) {
        let mut open = true;
        egui::Window::new("📥 Import Assessment Backup")
            .open(&mut open)
            .default_size(egui::vec2(520.0, 420.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label("Restore your assessment progress from a previously exported backup file (.bson, .json, .csv):");
                ui.add_space(8.0);

                // Option 1: Native File Picker Button
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        let file_btn = egui::Button::new(
                            egui::RichText::new("📁 Browse & Open Backup File...")
                                .size(14.0)
                                .strong(),
                        ).min_size(egui::vec2(280.0, 34.0));

                        if ui.add(file_btn).on_hover_text("Open file chooser to select your .bson, .json, or .csv backup").clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Assessment Backup (.bson, .json, .csv)", &["bson", "json", "csv"])
                                    .pick_file()
                                {
                                    if let Ok(bytes) = std::fs::read(&path) {
                                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                                        match self.import_from_bytes(&bytes, filename) {
                                            Ok(count) => {
                                                let current_t = ui.input(|i| i.time);
                                                self.last_save_time = Some(current_t);
                                                self.import_result_message = Some(Ok(format!("Successfully imported {} answers from '{}'!", count, filename)));
                                            }
                                            Err(e) => {
                                                self.import_result_message = Some(Err(e));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("💡 Tip: You can also drag and drop your .bson or .json file directly onto the app window!").small().weak());
                        ui.add_space(4.0);
                    });
                });

                ui.add_space(8.0);

                // Option 2: Paste Raw Text / Base64 Content
                ui.collapsing("Or Paste Raw CSV, JSON, or Base64 BSON Text", |ui| {
                    ui.add_space(4.0);
                    egui::ScrollArea::both()
                        .max_height(160.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.import_text_buffer)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("Paste CSV text, JSON text, or Base64 BSON string here...")
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(6),
                            );
                        });

                    ui.add_space(6.0);
                    if ui.button("Apply Pasted Text").on_hover_text("Parse pasted data and restore answers").clicked() {
                        let input = self.import_text_buffer.trim().as_bytes().to_vec();
                        if input.is_empty() {
                            self.import_result_message = Some(Err("Pasted text is empty.".to_string()));
                        } else {
                            match self.import_from_bytes(&input, "pasted text") {
                                Ok(count) => {
                                    let current_t = ui.input(|i| i.time);
                                    self.last_save_time = Some(current_t);
                                    self.import_result_message = Some(Ok(format!("Successfully imported {} answers from pasted text!", count)));
                                }
                                Err(e) => {
                                    self.import_result_message = Some(Err(e));
                                }
                            }
                        }
                    }
                });

                ui.add_space(8.0);
                if let Some(ref result) = self.import_result_message {
                    match result {
                        Ok(msg) => {
                            ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(80, 180, 90)).strong());
                        }
                        Err(msg) => {
                            ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(220, 70, 70)).strong());
                        }
                    }
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.show_import_dialog = false;
                        self.import_text_buffer.clear();
                        self.import_result_message = None;
                    }
                });
            });

        if !open {
            self.show_import_dialog = false;
            self.import_text_buffer.clear();
            self.import_result_message = None;
        }
    }
}


impl eframe::App for PersonalityApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Do not overwrite user's saved local answers if they are only viewing a shared link
        if !self.is_viewing_shared_link {
            eframe::set_value(storage, eframe::APP_KEY, &self.state);
            if let Ok(json_str) = serde_json::to_string(&self.state) {
                match save_state_multi_tier(eframe::APP_KEY, &json_str) {
                    Ok(backend) => {
                        self.storage_diag.backend = backend;
                        self.storage_diag.quota_exceeded = false;
                    }
                    Err(_) => {
                        // Space ran out: compact undo history and retry saving
                        self.state.questionnaire.compact_history(30);
                        if let Ok(compacted_json) = serde_json::to_string(&self.state) {
                            if let Ok(backend) = save_state_multi_tier(eframe::APP_KEY, &compacted_json) {
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
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Periodic storage diagnostics poll every 2.0s
        let current_time = ui.input(|i| i.time);
        if current_time - self.last_diag_poll_time > 2.0 {
            let fresh_diag = query_storage_diagnostics();
            self.storage_diag.is_persisted = fresh_diag.is_persisted;
            self.storage_diag.pwa_install_available = fresh_diag.pwa_install_available;
            self.storage_diag.is_pwa_installed = fresh_diag.is_pwa_installed;
            self.last_diag_poll_time = current_time;
        }

        self.apply_theme(ui.ctx());
        self.handle_keyboard_and_scroll(ui);

        // Handle drag & drop files anywhere onto the window
        let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for file in dropped_files {
                let mut loaded_bytes = None;
                let mut name = String::new();

                if let Ok(bytes) = file.bytes() {
                    loaded_bytes = Some(bytes);
                }

                let path = file.path();
                if !path.as_os_str().is_empty() {
                    if let Some(n) = path.file_name().and_then(|n| n.to_str()) {
                        name = n.to_string();
                    }
                    if loaded_bytes.is_none() {
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Ok(b) = std::fs::read(path) {
                            loaded_bytes = Some(b);
                        }
                    }
                }

                if name.is_empty() {
                    name = "backup_file".to_string();
                }

                if let Some(bytes) = loaded_bytes {
                    self.show_import_dialog = true;
                    match self.import_from_bytes(&bytes, &name) {
                        Ok(count) => {
                            let current_t = ui.input(|i| i.time);
                            self.last_save_time = Some(current_t);
                            self.import_result_message = Some(Ok(format!("Successfully imported {} answers from '{}'!", count, name)));
                        }
                        Err(e) => {
                            self.import_result_message = Some(Err(e));
                        }
                    }
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

        egui::CentralPanel::default().frame(central_frame).show(ui, |ui| {
            if self.hide_header {
                // Render subtle unhide button floating at top center when header is collapsed
                ui.vertical_centered(|ui| {
                    let expand_btn = egui::Button::new(egui::RichText::new("Show Header").size(11.0).weak())
                        .min_size(egui::vec2(120.0, 22.0));
                    if ui.add(expand_btn).on_hover_text("Show top navigation header").clicked() {
                        self.hide_header = false;
                    }
                });
                ui.add_space(6.0);
            }

            // Render warning banners (persistence / quota / combined)
            self.render_warning_banners(ui);

            if self.state.questionnaire.show_results && !show_results_side_panel {
                // Viewport is under 900px: render results full-screen inside CentralPanel
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    if ui.button("◀ Return to Questions").clicked() {
                        self.state.questionnaire.show_results = false;
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    self.render_results_panel(ui);
                });
            } else {
                // Render Question Focus Card
                self.render_question_focus(ui, &constraints);
            }
        });

        self.render_dialogs(ui);
    }
}
