use eframe::egui;

use crate::app::PersonalityApp;
use crate::storage_manager::trigger_pwa_install;
use crate::types::ScreenConstraints;

impl PersonalityApp {
    pub(crate) fn calculate_estimated_time_remaining(&self) -> Option<String> {
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

    pub(crate) fn render_top_bar(&mut self, ui: &mut egui::Ui, constraints: &ScreenConstraints) {
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
            let btn_height = if is_touch { 44.0 } else { 32.0 };

            ui.horizontal(|ui| {
                ui.set_min_height(btn_height + 4.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                    ui.spacing_mut().interact_size.y = btn_height;

                    ui.menu_button(egui::RichText::new("Menu").strong(), |ui| {
                        ui.set_min_width(220.0);

                        ui.label(egui::RichText::new("Assessment Navigation").small().weak());

                        let results_text = if self.state.questionnaire.show_results {
                            "Return to Questions"
                        } else {
                            "View Assessment Results"
                        };
                        if ui.button(results_text).on_hover_text("Toggle between questionnaire answering view and hierarchical results report").clicked() {
                            self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                            self.persist_state();
                            ui.close();
                        }

                        if ui.button("Item Matrix Map").on_hover_text("Open interactive 221-item questionnaire matrix map").clicked() {
                            self.show_grid_dialog = true;
                            ui.close();
                        }

                        if self.is_viewing_shared_link
                            && ui.button("Return to Saved Answers").on_hover_text("Exit shared link view and restore locally saved assessment").clicked()
                        {
                            self.restore_saved_instance();
                            ui.close();
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Data & Management").small().weak());

                        if ui.button("Import Answers (CSV, JSON, BSON)").on_hover_text("Import assessment backup from CSV, JSON, or BSON file").clicked() {
                            self.show_import_dialog = true;
                            self.import_text_buffer.clear();
                            self.import_result_message = None;
                            ui.close();
                        }

                        if ui.button("Reset Assessment").on_hover_text("Clear all recorded responses and restart assessment from item #1").clicked() {
                            self.show_reset_dialog = true;
                            ui.close();
                        }

                        let storage_label = match self.storage_diag.is_persisted {
                            Some(true) => "Storage Status: Persistent",
                            Some(false) => "Storage Status: Ephemeral",
                            None => "Storage Status",
                        };
                        if ui.button(storage_label).on_hover_text("Inspect offline storage durability, persistence mode, quota, and backups").clicked() {
                            self.show_storage_modal = true;
                            ui.close();
                        }

                        if self.storage_diag.pwa_install_available && !self.storage_diag.is_pwa_installed
                            && ui.button("Install App (PWA)").on_hover_text("Install assessment as a standalone Progressive Web App for offline durability").clicked()
                        {
                            trigger_pwa_install();
                            ui.close();
                        }

                        ui.separator();
                        ui.label(egui::RichText::new("Settings & Info").small().weak());

                        let theme_label = format!("Theme: {}", self.state.config.theme.label());
                        if ui.button(theme_label).on_hover_text("Cycle visual themes between Dark, Light, and High Contrast modes").clicked() {
                            self.state.config.theme = self.state.config.theme.next();
                            self.persist_state();
                            ui.close();
                        }

                        if ui.button("Help & Shortcuts").on_hover_text("View instructions, psychometric background, keyboard shortcuts, and data privacy").clicked() {
                            self.show_help_dialog = true;
                            ui.close();
                        }

                        if ui.button("Research Paper (DOI)").on_hover_text("Open published research paper in European Journal of Personality (DOI)").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://doi.org/10.1177/08902070251352590"));
                            ui.close();
                        }

                        if ui.button("GitHub Repository").on_hover_text("Open open-source repository and codebase on GitHub").clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab("https://github.com/Spodeian/Revisited-IPIP-NEO"));
                            ui.close();
                        }

                        ui.separator();
                        if ui.button("Hide Header Bar").on_hover_text("Collapse top navigation header to maximize questionnaire workspace").clicked() {
                            self.hide_header = true;
                            ui.close();
                        }
                    });

                    let results_btn_text = if self.state.questionnaire.show_results {
                        "Questions"
                    } else {
                        "Results"
                    };
                    let mut res_btn = egui::Button::new(egui::RichText::new(results_btn_text).strong());
                    if is_touch {
                        res_btn = res_btn.min_size(egui::vec2(0.0, btn_height));
                    }
                    if ui.add(res_btn).on_hover_text("Toggle between questionnaire answering view and hierarchical results report").clicked() {
                        self.state.questionnaire.show_results = !self.state.questionnaire.show_results;
                        self.persist_state();
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                        let title_size = if avail_w < 480.0 { 16.0 } else if avail_w < 780.0 { 18.0 } else { 20.0 };
                        ui.label(egui::RichText::new(title_text).size(title_size).strong())
                            .on_hover_text("Revisited IPIP-NEO Personality Assessment based on Taxonomic Graph Analysis (Samo et al., 2026)");

                        if let Some(save_t) = self.last_save_time
                            && current_time - save_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(70, 180, 90), "Saved")
                                .on_hover_text("Assessment responses save automatically to local storage");
                        }
                        if let Some(undo_t) = self.undo_notification_time
                            && current_time - undo_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(240, 160, 40), "Undone")
                                .on_hover_text("Reverted last response change (Ctrl+Z / Cmd+Z)");
                        }
                        if let Some(redo_t) = self.redo_notification_time
                            && current_time - redo_t < 2.5
                        {
                            ui.colored_label(egui::Color32::from_rgb(100, 180, 240), "Redone")
                                .on_hover_text("Restored reverted response change (Ctrl+Y / Cmd+Shift+Z)");
                        }
                        if avail_w > 520.0 && let Some(est) = self.calculate_estimated_time_remaining() {
                            ui.label(egui::RichText::new(format!("Est. remaining: {}", est)).small().weak())
                                .on_hover_text("Estimated time to complete remaining questions based on recent answering pace");
                        }
                    });
                });
            });

            ui.add_space(4.0);
        });
    }
}
