//! UI Implementations and View Controller logic for Revisited IPIP-NEO.

use eframe::egui;
use shared::{
    export_to_csv, export_to_json, export_to_printable_html, export_to_svg,
    import_responses_from_csv, import_responses_from_json, AppState, Aspect, Facet, MetaTrait,
    Response, ScoreTier, ThemeMode, Trait,
};
use tracing::{info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Default)]
pub struct PersonalityApp {
    pub state: AppState,
    pub show_reset_dialog: bool,
    pub show_help_dialog: bool,
    pub show_import_dialog: bool,
    pub import_text_buffer: String,
    pub import_result_message: Option<Result<String, String>>,
    pub show_export_dialog: Option<ExportType>,
    pub export_copied_notification: Option<f64>,
    pub last_scroll_time: f64,
    pub scroll_accumulator: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExportType {
    Csv,
    Json,
    PrintableHtml,
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
        let _ = std::fs::write(filename, content);
    }
}

impl PersonalityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Initializing Revisited IPIP-NEO Personality Assessment...");

        let mut state = if let Some(storage) = cc.storage {
            match eframe::get_value::<AppState>(storage, eframe::APP_KEY) {
                Some(mut s) => {
                    info!("Loaded personality assessment state from storage.");
                    s.questionnaire.rebuild_cache();
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

        Self {
            state,
            show_reset_dialog: false,
            show_help_dialog: false,
            show_import_dialog: false,
            import_text_buffer: String::new(),
            import_result_message: None,
            show_export_dialog: None,
            export_copied_notification: None,
            last_scroll_time: 0.0,
            scroll_accumulator: 0.0,
        }
    }

    fn handle_keyboard_and_scroll(&mut self, ui: &mut egui::Ui) {
        let input = ui.input(|i| i.clone());

        // Keyboard shortcuts for responses: 1-5
        if input.key_pressed(egui::Key::Num1) {
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::StronglyDisagree);
        } else if input.key_pressed(egui::Key::Num2) {
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Disagree);
        } else if input.key_pressed(egui::Key::Num3) {
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Neutral);
        } else if input.key_pressed(egui::Key::Num4) {
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Agree);
        } else if input.key_pressed(egui::Key::Num5) {
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::StronglyAgree);
        }

        // Navigation shortcuts:
        let shift_held = input.modifiers.shift;

        // Escape key to dismiss dialogs or close results screen
        if input.key_pressed(egui::Key::Escape) {
            if self.show_help_dialog {
                self.show_help_dialog = false;
            } else if self.show_reset_dialog {
                self.show_reset_dialog = false;
            } else if self.show_import_dialog {
                self.show_import_dialog = false;
                self.import_text_buffer.clear();
                self.import_result_message = None;
            } else if self.show_export_dialog.is_some() {
                self.show_export_dialog = None;
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

        // Mouse scroll detection has been moved specifically to the question focus view
        // to prevent scrolling results from skipping questions, and to respect directions.
    }
}

impl eframe::App for PersonalityApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Theme visual style
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
        ui.ctx().set_visuals(visuals);

        self.handle_keyboard_and_scroll(ui);

        // Top Navigation Bar
        egui::Panel::top("top_panel").show(ui, |ui| {
            let width = ui.available_width();
            let is_mobile = width < 800.0;
            ui.add_space(4.0);

            let title_text = if is_mobile { "IPIP-NEO (TGA)" } else { "Revisited IPIP-NEO Personality Assessment" };
            let header_row_height = if is_mobile { 44.0 } else { 32.0 };

            ui.horizontal(|ui| {
                ui.set_height(header_row_height);

                if is_mobile {
                    ui.label(egui::RichText::new(title_text).size(18.0).strong());
                } else {
                    ui.heading(title_text);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_mobile {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Mobile larger touch-target buttons (min 44x44px standard for easy thumb tapping)
                        let results_btn_text = if self.state.questionnaire.show_results { "📝 Questions" } else { "📊 Results" };
                        let res_btn = egui::Button::new(egui::RichText::new(results_btn_text).size(14.0).strong())
                            .min_size(egui::vec2(96.0, 44.0));
                        if ui.add(res_btn).on_hover_text("Toggle assessment results").clicked() {
                            self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                        }

                        let reset_btn = egui::Button::new(egui::RichText::new("🔄").size(20.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(reset_btn).on_hover_text("Reset test and clear all answers").clicked() {
                            self.show_reset_dialog = true;
                        }

                        let gh_btn = egui::Button::new(egui::RichText::new("🐙").size(20.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(gh_btn).on_hover_text("View source on GitHub").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/Spodeian/Revisited-IPIP-NEO"));
                        }

                        let doi_btn = egui::Button::new(egui::RichText::new("📖").size(20.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(doi_btn).on_hover_text("Read the research").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://doi.org/10.1177/08902070251352590"));
                        }

                        let help_btn = egui::Button::new(egui::RichText::new("❓").size(20.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(help_btn).on_hover_text("Help, shortcuts & privacy").clicked() {
                            self.show_help_dialog = true;
                        }

                        let theme_icon = match self.state.config.theme {
                            ThemeMode::Light => "🌙",
                            ThemeMode::Dark => "☀️",
                        };
                        let theme_btn = egui::Button::new(egui::RichText::new(theme_icon).size(20.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(theme_btn).on_hover_text("Toggle dark / light theme").clicked() {
                            self.state.config.theme = match self.state.config.theme {
                                ThemeMode::Light => ThemeMode::Dark,
                                ThemeMode::Dark => ThemeMode::Light,
                            };
                        }
                    } else {
                        // Desktop layout
                        let theme_icon = match self.state.config.theme {
                            ThemeMode::Light => "🌙 Dark",
                            ThemeMode::Dark => "☀️ Light",
                        };
                        if ui.button(theme_icon).on_hover_text("Toggle dark / light theme").clicked() {
                            self.state.config.theme = match self.state.config.theme {
                                ThemeMode::Light => ThemeMode::Dark,
                                ThemeMode::Dark => ThemeMode::Light,
                            };
                        }

                        // Help Icon
                        if ui.button("❓").on_hover_text("Help, shortcuts & privacy").clicked() {
                            self.show_help_dialog = true;
                        }

                        // Import Button
                        if ui.button("📥 Import").on_hover_text("Import CSV or JSON answers to resume your assessment").clicked() {
                            self.show_import_dialog = true;
                            self.import_text_buffer.clear();
                            self.import_result_message = None;
                        }

                        // Research DOI Icon
                        if ui.button("📖").on_hover_text("Read the research").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://doi.org/10.1177/08902070251352590"));
                        }

                        // GitHub Icon (Icon only)
                        if ui.button("🐙").on_hover_text("View source on GitHub").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/Spodeian/Revisited-IPIP-NEO"));
                        }

                        // Import Button (Mobile)
                        let import_btn = egui::Button::new(egui::RichText::new("📥").size(18.0))
                            .min_size(egui::vec2(38.0, 38.0));
                        if ui.add(import_btn).on_hover_text("Import CSV or JSON answers to resume assessment").clicked() {
                            self.show_import_dialog = true;
                            self.import_text_buffer.clear();
                            self.import_result_message = None;
                        }

                        // Reset button
                        if ui.button("🔄 Reset").on_hover_text("Reset test and clear all answers").clicked() {
                            self.show_reset_dialog = true;
                        }

                        // Results / Questions Toggle
                        let results_btn_text = if self.state.questionnaire.show_results {
                            "📊 Hide Results"
                        } else {
                            "📊 Show Results"
                        };
                        if ui.button(results_btn_text).on_hover_text("Toggle assessment results").clicked() {
                            self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                        }
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Side Results Panel (Only render on Desktop/Wide viewports)
        let is_mobile = ui.available_width() < 800.0;

        if self.state.questionnaire.show_results && !is_mobile {
            egui::Panel::right("results_panel")
                .min_size(380.0)
                .default_size(420.0)
                .show(ui, |ui| {
                    self.render_results_panel(ui);
                });
        }

        // Main Central View Routing
        let is_tiny = ui.available_width() < 350.0 || ui.available_height() < 500.0;
        let mut central_frame = egui::Frame::central_panel(ui.style());
        if is_tiny {
            central_frame.inner_margin = egui::Margin::same(4);
        }

        egui::CentralPanel::default().frame(central_frame).show(ui, |ui| {
            if is_mobile && self.state.questionnaire.show_results {
                // Mobile View: Render results screen full-screen inside CentralPanel
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(10.0);
                    if ui.button("📝 Return to Questions").clicked() {
                        self.state.questionnaire.show_results = false;
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    self.render_results_panel(ui);
                });
            } else {
                // Desktop View or Mobile Questions View: Render Question Focus Card
                self.render_question_focus(ui);
            }
        });

        // Dialogs
        self.render_dialogs(ui);
    }
}

impl PersonalityApp {
    fn render_question_focus(&mut self, ui: &mut egui::Ui) {
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

        // Measure available space to detect viewport constraints
        let avail_height = ui.available_height();
        let avail_width = ui.available_width();

        let is_mobile_portrait = avail_width < 650.0;
        let is_tight_height = avail_height < 530.0 || avail_width < 350.0;
        let is_ultra_tight = avail_width < 330.0 || avail_height < 490.0;

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
                    let framing_font_size = if is_ultra_tight { 14.0 } else if is_tight_height { 16.0 } else { 19.0 };
                    ui.label(
                        egui::RichText::new("Rate how accurately this statement describes you:")
                            .size(framing_font_size)
                            .strong()
                            .color(ui.visuals().hyperlink_color),
                    );
                    ui.add_space(if is_ultra_tight { 4.0 } else { 10.0 });

                    // Question Statement Box (scaled down for small screens)
                    let card_padding = if is_ultra_tight { 6.0 } else if is_tight_height { 12.0 } else if is_mobile_portrait { 16.0 } else { 24.0 };
                    let font_size = if is_ultra_tight { 14.0 } else if is_tight_height { 17.0 } else if is_mobile_portrait { 20.0 } else { 26.0 };

                    egui::Frame::group(ui.style())
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

                    let space_after_card = if is_ultra_tight { 4.0 } else if is_tight_height { 6.0 } else if is_mobile_portrait { 15.0 } else { 30.0 };
                    ui.add_space(space_after_card);

                    let responses = [
                        (Response::StronglyDisagree, "Strongly Disagree", "1"),
                        (Response::Disagree, "Disagree", "2"),
                        (Response::Neutral, "Neutral", "3"),
                        (Response::Agree, "Agree", "4"),
                        (Response::StronglyAgree, "Strongly Agree", "5"),
                    ];

                    // Clean vertical stack for Likert buttons across all orientations
                    let button_height = if is_ultra_tight { 26.0 } else if is_tight_height { 32.0 } else if is_mobile_portrait { 36.0 } else { 42.0 };
                    let button_text_size = if is_ultra_tight { 12.0 } else if is_tight_height { 13.0 } else if is_mobile_portrait { 14.0 } else { 16.0 };
                    let btn_width = (ui.available_width() - 8.0).min(340.0);

                    for (resp, text, shortcut) in responses {
                        let is_selected = q_response == Some(resp);

                        // Remove keyboard shortcut hints on ultra-tight touchscreens to save space
                        let button_text = if is_ultra_tight {
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

                        if ui.add(btn).clicked() {
                            self.state.questionnaire.answer_question(curr_idx, resp);
                        }
                        ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 6.0 });
                    }

                    let space_before_nav = if is_ultra_tight { 4.0 } else if is_tight_height { 6.0 } else if is_mobile_portrait { 15.0 } else { 25.0 };
                    ui.add_space(space_before_nav);
                    ui.separator();
                    ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 10.0 });

                    // Navigation and Progress Row: Left buttons, Center Progress Bar (Item #x of 221), Right buttons
                    let curr_focus = curr_idx + 1;
                    let progress = self.state.questionnaire.completion_rate();
                    let progress_text = format!("Item #{} of {} ({:.0}%)", curr_focus, total, progress * 100.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = if is_ultra_tight { 4.0 } else { 6.0 };

                        // Left Action Cluster
                        let btn_prev = if is_ultra_tight { "◀" } else { "◀ Prev" };
                        if ui.button(btn_prev).on_hover_text("Previous item (Left Arrow)").clicked() {
                            self.state.questionnaire.navigate_previous();
                        }

                        let btn_prev_un = if is_ultra_tight { "⏪" } else { "⏪ Unanswered" };
                        if ui.button(btn_prev_un).on_hover_text("Jump to previous unanswered (Shift + Left Arrow)").clicked() {
                            self.state.questionnaire.navigate_previous_unanswered();
                        }

                        // Right Action Cluster + Center Expanding Progress Bar
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_skip = if is_ultra_tight { "⏭" } else { "Skip ⏭" };
                            if ui.button(btn_skip).on_hover_text("Skip question and defer to end of queue (Right Arrow / Scroll)").clicked() {
                                self.state.questionnaire.skip_current();
                            }

                            let btn_next_un = if is_ultra_tight { "⏩" } else { "Next Unanswered ⏩" };
                            if ui.button(btn_next_un).on_hover_text("Jump to next unanswered (Shift + Right Arrow)").clicked() {
                                self.state.questionnaire.navigate_next_unanswered();
                            }

                            if q_response.is_some() {
                                let btn_clear = if is_ultra_tight { "🗑" } else { "🗑 Clear" };
                                if ui.button(btn_clear).on_hover_text("Clear recorded answer").clicked() {
                                    self.state.questionnaire.clear_response(curr_idx);
                                }
                            }

                            // Center Progress Bar filling remaining horizontal space
                            let remaining_width = ui.available_width() - 8.0;
                            if remaining_width > 40.0 {
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .text(progress_text)
                                        .desired_width(remaining_width),
                                );
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
    }

    fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Assessment Results");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").clicked() {
                        self.state.questionnaire.show_results = false;
                    }
                });
            });

            ui.add_space(6.0);
            let answered = self.state.questionnaire.answered_count();
            let total = self.state.questionnaire.total_questions();
            let pct = self.state.questionnaire.completion_rate() * 100.0;
            ui.label(format!("Completed: {}/{} ({:.1}%)", answered, total, pct));

            ui.add_space(4.0);
            ui.checkbox(&mut self.state.questionnaire.show_detailed_stats, "Show Detailed Metrics & SE");

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("📄 Export CSV").on_hover_text("Immediately download full results and responses as a CSV file").clicked() {
                    let csv_content = export_to_csv(&self.state.questionnaire);
                    trigger_file_download("ipip_neo_tga_results.csv", &csv_content, "text/csv;charset=utf-8");
                }
                if ui.button("{ } Export JSON").on_hover_text("Immediately download full results and responses as a JSON file").clicked() {
                    let json_content = export_to_json(&self.state.questionnaire);
                    trigger_file_download("ipip_neo_tga_results.json", &json_content, "application/json;charset=utf-8");
                }
                if ui.button("🖼 Export SVG").on_hover_text("Download high-resolution vector SVG graphic of your results hierarchy").clicked() {
                    let svg_content = export_to_svg(&self.state.questionnaire);
                    trigger_file_download("ipip_neo_tga_results.svg", &svg_content, "image/svg+xml;charset=utf-8");
                }
                if ui.button("📋 Save PDF / Print").on_hover_text("Open formatted hierarchical report for printing or saving to PDF").clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let html_content = export_to_printable_html(&self.state.questionnaire);
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(new_win)) = window.open_with_url_and_target("", "_blank") {
                                if let Some(doc) = new_win.document() {
                                    if let Some(doc_element) = doc.document_element() {
                                        // Overwrite the entire <html> element cleanly to preserve <head> styles and <body>
                                        doc_element.set_inner_html(&html_content);
                                        let _ = new_win.print();
                                    }
                                }
                            }
                        }
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // On desktop, fallback to showing the HTML in the dialog
                        self.show_export_dialog = Some(ExportType::PrintableHtml);
                    }
                }
            });

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

        egui::CollapsingHeader::new(egui::RichText::new(meta.display_name()).strong().size(15.0))
            .default_open(true)
            .show(ui, |ui| {
                self.render_construct_badge_row(ui, &acc, show_detailed);

                // Child Traits
                for trait_item in meta.child_traits() {
                    self.render_trait_node(ui, trait_item);
                }
            });
    }

    fn render_trait_node(&self, ui: &mut egui::Ui, trait_item: Trait) {
        let acc = self.state.questionnaire.trait_acc.get(&trait_item).copied().unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;

        egui::CollapsingHeader::new(egui::RichText::new(trait_item.display_name()).size(14.0))
            .default_open(true)
            .show(ui, |ui| {
                self.render_construct_badge_row(ui, &acc, show_detailed);

                // Child Facets
                for facet in trait_item.child_facets() {
                    self.render_facet_row(ui, facet);
                }
            });
    }

    fn render_facet_row(&self, ui: &mut egui::Ui, facet: Facet) {
        let acc = self.state.questionnaire.facet_acc.get(&facet).copied().unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;

        ui.horizontal(|ui| {
            ui.label(facet.display_name());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_construct_badge_row(ui, &acc, show_detailed);
            });
        });
    }

    fn render_construct_badge_row(&self, ui: &mut egui::Ui, acc: &shared::ScoreAccumulator, show_detailed: bool) {
        if let Some(norm_score) = acc.normalized_score() {
            let tier = acc.tier().unwrap_or(ScoreTier::Average);
            let tier_color = match tier {
                ScoreTier::VeryLow => egui::Color32::from_rgb(220, 70, 70),
                ScoreTier::Low => egui::Color32::from_rgb(230, 140, 50),
                ScoreTier::Average => egui::Color32::from_rgb(140, 140, 150),
                ScoreTier::High => egui::Color32::from_rgb(70, 170, 90),
                ScoreTier::VeryHigh => egui::Color32::from_rgb(30, 140, 220),
            };

            ui.colored_label(tier_color, egui::RichText::new(tier.label()).strong());

            if show_detailed {
                let se = acc.standard_error().unwrap_or(0.0);
                ui.label(
                    egui::RichText::new(format!(
                        "score: {:.2} (SE: {:.2}, raw: {:.1}, n: {})",
                        norm_score, se, acc.raw_score, acc.answered_count
                    ))
                    .small()
                    .weak(),
                );
            }
        } else {
            ui.label(egui::RichText::new("No items yet").weak().small());
        }
    }

    fn render_dialogs(&mut self, ui: &mut egui::Ui) {
        // Help & Shortcuts Dialog (Docked to bottom-left, expanding upward & outward with generous sizing)
        if self.show_help_dialog {
            let mut open = true;
            let win_w = (ui.available_width() - 24.0).clamp(320.0, 480.0);
            let win_h = (ui.available_height() - 32.0).clamp(380.0, 460.0);

            egui::Window::new("❓ Help & Information")
                .open(&mut open)
                .resizable(true)
                .collapsible(true)
                .default_size(egui::vec2(win_w, win_h))
                .min_width(300.0)
                .min_height(340.0)
                .max_width((ui.available_width() - 16.0).min(520.0))
                .max_height(ui.available_height() - 20.0)
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                        ui.heading("Estimated Time");
                        ui.add_space(4.0);
                        ui.label("⏱ ~10–15 minutes (221 items). Take your time to answer honestly without overthinking.");

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.heading("Keyboard Shortcuts & Navigation");
                        ui.add_space(4.0);
                        ui.label("• 1, 2, 3, 4, 5: Select response (Strongly Disagree to Strongly Agree)");
                        ui.label("• Left / Up Arrow: Navigate to previous question");
                        ui.label("• Right / Down Arrow: Skip question (defers to back of queue)");
                        ui.label("• Shift + Left Arrow: Jump to previous unanswered question");
                        ui.label("• Shift + Right Arrow: Jump to next unanswered question");
                        ui.label("• Escape: Close open menus or return from results");
                        ui.label("• Mouse Scroll: Scroll to skip / navigate questions (Desktop)");

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.heading("Privacy & Data Safety");
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("🔒 Privacy: 100% Local. No data ever leaves your device.")
                                .color(egui::Color32::from_rgb(80, 160, 90))
                                .strong(),
                        );
                        ui.label(
                            "This application runs entirely in your local browser using client-side WebAssembly. Your responses, scores, and exports are never transmitted to any server or external database.",
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("⚡ 100% Offline Capable: Once loaded, you can complete the entire assessment and export results even without an internet connection.")
                                .italics()
                                .small(),
                        );

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.heading("Psychometric Methodology");
                        ui.add_space(4.0);
                        ui.label(
                            "This assessment implements the Trait-Group-Aspect (TGA) model based on the IPIP-NEO, optimizing question sequences for minimal standard error.",
                        );
                        ui.horizontal(|ui| {
                            ui.label("Reference:");
                            ui.hyperlink_to(
                                "doi:10.1177/08902070251352590",
                                "https://doi.org/10.1177/08902070251352590",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Source code:");
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

        // Reset Confirmation Dialog
        if self.show_reset_dialog {
            egui::Window::new("Reset Assessment?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.label("Are you sure you want to clear all responses and start over?");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes, Reset").clicked() {
                            self.state.reset_questionnaire();
                            self.show_reset_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_reset_dialog = false;
                        }
                    });
                });
        }

        // Export Dialog
        if let Some(export_type) = self.show_export_dialog {
            let title = match export_type {
                ExportType::Csv => "Export CSV",
                ExportType::Json => "Export JSON",
                ExportType::PrintableHtml => "Printable Report (HTML/PDF)",
            };

            let content = match export_type {
                ExportType::Csv => export_to_csv(&self.state.questionnaire),
                ExportType::Json => export_to_json(&self.state.questionnaire),
                ExportType::PrintableHtml => export_to_printable_html(&self.state.questionnaire),
            };

            let mut open = true;
            egui::Window::new(title)
                .open(&mut open)
                .default_size(egui::vec2(600.0, 450.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy to Clipboard").clicked() {
                            ui.ctx().copy_text(content.clone());
                            self.export_copied_notification = Some(ui.input(|i| i.time));
                        }
                        if let Some(t) = self.export_copied_notification {
                            if ui.input(|i| i.time) - t < 3.0 {
                                ui.label(egui::RichText::new("✓ Copied to clipboard!").color(egui::Color32::GREEN));
                            }
                        }
                    });

                    ui.add_space(8.0);
                    egui::ScrollArea::both().show(ui, |ui| {
                        let mut display_content = content;
                        ui.add(
                            egui::TextEdit::multiline(&mut display_content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .lock_focus(true)
                                .desired_width(f32::INFINITY),
                        );
                    });
                });

            if !open {
                self.show_export_dialog = None;
            }
        }

        // Import Progress Dialog
        if self.show_import_dialog {
            let mut open = true;
            egui::Window::new("📥 Import Saved Progress")
                .open(&mut open)
                .default_size(egui::vec2(500.0, 400.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.label("Paste the contents of your exported CSV or JSON file below to restore your answers and resume the assessment:");
                    ui.add_space(8.0);

                    egui::ScrollArea::both()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.import_text_buffer)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("Paste CSV or JSON content here...")
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(12),
                            );
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("▶ Apply and Resume").clicked() {
                            let input = self.import_text_buffer.trim();
                            if input.is_empty() {
                                self.import_result_message = Some(Err("Input is empty.".to_string()));
                            } else if input.starts_with('{') {
                                // Attempt JSON parse
                                match import_responses_from_json(&mut self.state.questionnaire, input) {
                                    Ok(count) => {
                                        self.import_result_message = Some(Ok(format!("Successfully imported {} answers!", count)));
                                    }
                                    Err(e) => {
                                        self.import_result_message = Some(Err(e.to_string()));
                                    }
                                }
                            } else {
                                // Attempt CSV parse
                                match import_responses_from_csv(&mut self.state.questionnaire, input) {
                                    Ok(count) => {
                                        self.import_result_message = Some(Ok(format!("Successfully imported {} answers!", count)));
                                    }
                                    Err(e) => {
                                        self.import_result_message = Some(Err(e.to_string()));
                                    }
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_import_dialog = false;
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
                });

            if !open {
                self.show_import_dialog = false;
            }
        }
    }
}
