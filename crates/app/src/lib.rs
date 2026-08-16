//! UI Implementations and View Controller logic for Revisited IPIP-NEO.

use eframe::egui;
use shared::{
    export_to_csv, export_to_json, export_to_printable_html, AppState, Aspect, Facet, MetaTrait,
    Response, ScoreTier, ThemeMode, Trait,
};
use tracing::{info, warn};

#[derive(Default)]
pub struct PersonalityApp {
    pub state: AppState,
    pub show_reset_dialog: bool,
    pub show_export_dialog: Option<ExportType>,
    pub export_copied_notification: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExportType {
    Csv,
    Json,
    PrintableHtml,
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
            show_export_dialog: None,
            export_copied_notification: None,
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

        // Mouse scroll on question view to skip question
        if input.smooth_scroll_delta.y.abs() > 20.0 || input.smooth_scroll_delta.x.abs() > 20.0 {
            self.state.questionnaire.skip_current();
        }
    }
}

impl eframe::App for PersonalityApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Theme visual style
        let visuals = match self.state.config.theme {
            ThemeMode::Light => egui::Visuals::light(),
            ThemeMode::Dark => egui::Visuals::dark(),
        };
        ui.ctx().set_visuals(visuals);

        self.handle_keyboard_and_scroll(ui);

        // Top Navigation Bar
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Revisited IPIP-NEO Personality Assessment");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme toggle
                    let theme_icon = match self.state.config.theme {
                        ThemeMode::Light => "🌙 Dark",
                        ThemeMode::Dark => "☀️ Light",
                    };
                    if ui.button(theme_icon).clicked() {
                        self.state.config.theme = match self.state.config.theme {
                            ThemeMode::Light => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::Light,
                        };
                    }

                    // Reset button
                    if ui.button("🔄 Reset Test").clicked() {
                        self.show_reset_dialog = true;
                    }

                    // Results toggle
                    let results_btn_text = if self.state.questionnaire.show_results {
                        "📊 Hide Results"
                    } else {
                        "📊 Show Results"
                    };
                    if ui.button(results_btn_text).clicked() {
                        self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                    }

                    ui.separator();

                    // Progress bar
                    let answered = self.state.questionnaire.answered_count();
                    let total = self.state.questionnaire.total_questions();
                    let progress = self.state.questionnaire.completion_rate();
                    let progress_text = format!("Progress: {}/{} ({:.0}%)", answered, total, progress * 100.0);
                    ui.add(egui::ProgressBar::new(progress).text(progress_text).desired_width(220.0));
                });
            });
            ui.add_space(4.0);
        });

        // Side Results Panel (if enabled or completed)
        if self.state.questionnaire.show_results {
            egui::Panel::right("results_panel")
                .min_size(380.0)
                .default_size(420.0)
                .show(ui, |ui| {
                    self.render_results_panel(ui);
                });
        }

        // Main Questionnaire Area (Single Question Focus Mode)
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_question_focus(ui);
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
        let (q_id, q_text, q_response) = {
            let q = match self.state.questionnaire.questions.get(curr_idx) {
                Some(q) => q,
                None => return,
            };
            (q.id, q.text.clone(), q.response)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);

            // Centered question card container
            ui.vertical_centered(|ui| {
                ui.set_max_width(700.0);

                // Queue & Status indicator
                let pending_remaining = self.state.questionnaire.pending_queue.len();
                let status_text = if q_response.is_some() {
                    "✓ Answered (Reviewing / Modifying)"
                } else {
                    "Pending Answer"
                };

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Item #{} of {}", q_id, total))
                            .strong()
                            .color(ui.visuals().hyperlink_color),
                    );
                    ui.label(format!("•  {}", status_text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} remaining in queue", pending_remaining))
                                .italics()
                                .small(),
                        );
                    });
                });

                ui.add_space(25.0);

                // Clear Framing Instruction
                ui.label(
                    egui::RichText::new("Rate how accurately this statement describes you:")
                        .size(16.0)
                        .strong()
                        .color(ui.visuals().text_color()),
                );
                ui.add_space(10.0);

                // Question Statement Box
                egui::Frame::group(ui.style())
                    .inner_margin(24.0)
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(&q_text)
                                    .size(26.0)
                                    .strong(),
                            );
                        });
                    });

                ui.add_space(30.0);

                // Likert Response Options
                ui.label(
                    egui::RichText::new("How well does this describe you? (Select or press 1-5):")
                        .italics()
                );
                ui.add_space(10.0);

                let responses = [
                    (Response::StronglyDisagree, "Strongly Disagree", "1"),
                    (Response::Disagree, "Disagree", "2"),
                    (Response::Neutral, "Neutral", "3"),
                    (Response::Agree, "Agree", "4"),
                    (Response::StronglyAgree, "Strongly Agree", "5"),
                ];

                for (resp, text, shortcut) in responses {
                    let is_selected = q_response == Some(resp);
                    let button_text = format!("[{}]  {}", shortcut, text);

                    let mut rich_text = egui::RichText::new(button_text).size(16.0);
                    if is_selected {
                        rich_text = rich_text.strong();
                    }

                    let btn = egui::Button::new(rich_text)
                        .min_size(egui::vec2(340.0, 42.0))
                        .selected(is_selected);

                    if ui.add(btn).clicked() {
                        self.state.questionnaire.answer_question(curr_idx, resp);
                    }
                    ui.add_space(6.0);
                }

                ui.add_space(25.0);
                ui.separator();
                ui.add_space(10.0);

                // Navigation and Skip Actions
                ui.horizontal(|ui| {
                    if ui.button("◀ Prev (Left)").clicked() {
                        self.state.questionnaire.navigate_previous();
                    }

                    if ui.button("⏪ Prev Unanswered (Shift+Left)").clicked() {
                        self.state.questionnaire.navigate_previous_unanswered();
                    }

                    if q_response.is_some() && ui.button("🗑 Clear Answer").clicked() {
                        self.state.questionnaire.clear_response(curr_idx);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⏩ Next Unanswered (Shift+Right)").clicked() {
                            self.state.questionnaire.navigate_next_unanswered();
                        }
                        let skip_btn = ui.button("Skip / Defer ⏭ (Right)");
                        if skip_btn.clicked() {
                            self.state.questionnaire.skip_current();
                        }
                    });
                });

                ui.add_space(15.0);
                ui.label(
                    egui::RichText::new(
                        "Navigation: Use Left/Right arrow keys to move. Use Shift + Left/Right arrow keys to jump directly to unanswered questions. Scroll to skip.",
                    )
                    .small()
                    .weak(),
                );

                ui.add_space(35.0);
                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.weak("Reference Methodology:");
                    ui.hyperlink_to(
                        "doi:10.1177/08902070251352590",
                        "https://doi.org/10.1177/08902070251352590",
                    );
                });
            });
        });
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
                if ui.button("📄 Export CSV").clicked() {
                    self.show_export_dialog = Some(ExportType::Csv);
                }
                if ui.button("{ } Export JSON").clicked() {
                    self.show_export_dialog = Some(ExportType::Json);
                }
                if ui.button("🖨 Print / PDF").clicked() {
                    self.show_export_dialog = Some(ExportType::PrintableHtml);
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
    }
}
