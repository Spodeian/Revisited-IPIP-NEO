use eframe::egui;
use shared::{
    Aspect, Response, export_to_compressed_bson, export_to_csv, export_to_json,
    export_to_printable_html, export_to_svg,
};

use crate::app::PersonalityApp;
use crate::storage_manager::{
    request_persistent_storage, trigger_binary_download, trigger_pwa_install,
};
use crate::types::ExportFormat;

impl PersonalityApp {
    pub(crate) fn render_dialogs(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn render_storage_modal(&mut self, ui: &mut egui::Ui) {
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
                        ui.label("Responses and assessment progress are protected against automatic storage clearing.");
                    }
                    Some(false) => {
                        ui.horizontal(|ui| {
                            ui.label("• Persistence Mode:");
                            ui.colored_label(egui::Color32::from_rgb(230, 140, 50), "Best-Effort / Ephemeral");
                        });
                        ui.label("Storage permission has not been granted. Browsers may evict site data under disk pressure.");
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
                            ui.colored_label(egui::Color32::GRAY, "Checking Storage Engine...");
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
                    ui.colored_label(egui::Color32::from_rgb(220, 70, 70), "Storage quota reached; undo history was compacted. Consider saving a .bson backup.");
                } else {
                    ui.label("Storage engine uses persistent disk caching with automatic recency-density undo compaction.");
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

    pub(crate) fn render_warning_banners(&mut self, ctx: &egui::Context) {
        let is_ephemeral = self.storage_diag.is_persisted == Some(false);
        let quota_exceeded = self.storage_diag.quota_exceeded;

        let show_combined = is_ephemeral && quota_exceeded && !self.dismissed_combined_warning;
        let show_ephemeral = is_ephemeral && !self.dismissed_ephemeral_warning;
        let show_quota = quota_exceeded && !self.dismissed_quota_warning;

        if show_combined || show_ephemeral || show_quota {
            let (msg, fill_color, stroke_color, text_color) = if show_combined {
                (
                    "Storage Warning: Running in ephemeral storage and quota is constrained.",
                    egui::Color32::from_rgba_premultiplied(35, 20, 20, 245),
                    egui::Color32::from_rgb(240, 80, 80),
                    egui::Color32::from_rgb(255, 120, 120),
                )
            } else if show_ephemeral {
                (
                    "Ephemeral Storage: Browser may clear local data under storage pressure.",
                    egui::Color32::from_rgba_premultiplied(35, 28, 15, 245),
                    egui::Color32::from_rgb(220, 160, 30),
                    egui::Color32::from_rgb(255, 200, 80),
                )
            } else {
                (
                    "Quota Warning: Storage limit reached; compacted to conserve space.",
                    egui::Color32::from_rgba_premultiplied(35, 28, 15, 245),
                    egui::Color32::from_rgb(220, 160, 30),
                    egui::Color32::from_rgb(255, 200, 80),
                )
            };

            egui::Area::new(egui::Id::new("storage_warning_banner_area"))
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(fill_color)
                        .stroke(egui::Stroke::new(1.0_f32, stroke_color))
                        .corner_radius(8)
                        .inner_margin(egui::Margin::symmetric(16, 10))
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(msg).strong().color(text_color));
                                ui.add_space(6.0);
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center)
                                        .with_main_align(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .button("Save .bson Backup")
                                            .on_hover_text("Download compressed binary backup of your assessment")
                                            .clicked()
                                            && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                                        {
                                            trigger_binary_download(
                                                "ipip_neo_assessment_backup.bson",
                                                &bytes,
                                                "application/octet-stream",
                                            );
                                        }
                                        if ui
                                            .button("Request Persistence")
                                            .on_hover_text("Ask your browser for persistent storage permissions to prevent eviction")
                                            .clicked()
                                        {
                                            request_persistent_storage();
                                        }
                                        if ui
                                            .button("Dismiss")
                                            .on_hover_text("Dismiss this warning banner")
                                            .clicked()
                                        {
                                            if show_combined {
                                                self.dismissed_combined_warning = true;
                                            } else if show_ephemeral {
                                                self.dismissed_ephemeral_warning = true;
                                            } else {
                                                self.dismissed_quota_warning = true;
                                            }
                                        }
                                    },
                                );
                            });
                        });
                });
        }
    }

    pub(crate) fn render_grid_dialog(&mut self, ui: &mut egui::Ui) {
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
                    ui.label(
                        egui::RichText::new("Click any item to jump directly to that question")
                            .small()
                            .weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let answered = self.state.questionnaire.answered_count();
                        let total = self.state.questionnaire.total_questions();
                        ui.label(
                            egui::RichText::new(format!("{}/{} Answered", answered, total))
                                .strong(),
                        )
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
                                        (
                                            egui::Color32::from_rgb(50, 50, 55),
                                            egui::Color32::LIGHT_GRAY,
                                            "Unanswered",
                                        )
                                    } else {
                                        (
                                            egui::Color32::from_rgb(220, 220, 225),
                                            egui::Color32::DARK_GRAY,
                                            "Unanswered",
                                        )
                                    }
                                }
                            };

                            let mut btn_text = egui::RichText::new(format!("{}", q.id))
                                .size(11.0)
                                .color(text_color);
                            if is_curr {
                                btn_text = btn_text.strong();
                            }

                            let mut btn = egui::Button::new(btn_text)
                                .fill(bg_color)
                                .min_size(egui::vec2(28.0, 24.0));

                            if is_curr {
                                btn = btn.stroke(egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgb(30, 140, 240),
                                ));
                            }

                            let tooltip = format!(
                                "#{}. {}\nFacet: {} | Trait: {} | Meta-Trait: {}\nStatus: {}",
                                q.id,
                                q.text,
                                q.facet.category.display_name(),
                                q.facet.category.parent_trait().display_name(),
                                q.facet
                                    .category
                                    .parent_trait()
                                    .parent_meta_trait()
                                    .display_name(),
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
                if ui
                    .button("Close")
                    .on_hover_text("Close item matrix map (Escape)")
                    .clicked()
                {
                    self.show_grid_dialog = false;
                }
            });

        if !open {
            self.show_grid_dialog = false;
        }
    }

    pub(crate) fn render_help_dialog(&mut self, ui: &mut egui::Ui) {
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
                                        if ui.button("Return to Saved Assessment").on_hover_text("Exit shared link view and restore locally saved assessment").clicked() {
                                            self.restore_saved_instance();
                                            self.show_help_dialog = false;
                                        }
                                    });
                                });
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.heading("Taxonomic Graph Analysis (TGA)");
                        ui.add_space(4.0);
                        ui.label(
                            "This assessment implements Taxonomic Graph Analysis (TGA), a mathematically and statistically rigorous psychometric framework based on the IPIP-NEO:",
                        );
                        ui.label("• Empirical Factor Loadings: TGA statistically models continuous loadings (w_i) of each question across all construct levels.");
                        ui.label("• Optimized Item Distillation: Redundant questions were pruned from the original 300-item inventory down to the distilled 221-item set.");
                        ui.label("• Dimensional Mapping: Statistically models topological connections across 3 Meta-Traits, 6 Traits, and 28 Facets.");
                        ui.label("• Dynamic Sequencing: Questions are ordered to achieve rapid convergence and minimize cumulative standard error.");

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

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
                            "Visual score error bars apply hierarchical standard error confidence intervals (3σ, 2σ, 1σ) strictly bounded on [-1.0, +1.0]:",
                        );
                        ui.label("• Meta-Traits (Global Factors): ±3σ = ±3.0 × SE (~99.73% coverage)");
                        ui.label("• Traits (Broad Domains): ±2σ = ±2.0 × SE (~95.45% coverage)");
                        ui.label("• Facets (Specific Aspects): ±1σ = ±1.0 × SE (~68.27% coverage)");

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

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

                        ui.heading("Privacy & Data Safety");
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Client-Side Only: No assessment responses or scores leave your device.")
                                .color(egui::Color32::from_rgb(80, 160, 90))
                                .strong(),
                        );
                        ui.label(
                            "This application executes entirely in your browser using WebAssembly. Responses, scores, and exports are never transmitted to any external server.",
                        );
                        ui.label(
                            egui::RichText::new("Offline Capable: PWA service worker caches static assets for complete offline operation.")
                                .italics()
                                .small(),
                        );

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

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

    pub(crate) fn render_reset_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Reset Assessment")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to clear all responses and restart from item #1?");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("Yes, Reset")
                        .on_hover_text(
                            "Clear all answers, reset queue, and restart the questionnaire",
                        )
                        .clicked()
                    {
                        self.is_viewing_shared_link = false;
                        self.state.reset_questionnaire();
                        self.show_reset_dialog = false;
                        self.persist_state();
                    }
                    if ui
                        .button("Cancel")
                        .on_hover_text("Keep existing answers and return to questionnaire")
                        .clicked()
                    {
                        self.show_reset_dialog = false;
                    }
                });
            });
    }

    pub(crate) fn render_export_dialog(&mut self, ui: &mut egui::Ui) {
        let export_format = match self.show_export_dialog {
            Some(fmt) => fmt,
            None => return,
        };

        if self.export_text_buffer.is_empty() {
            if export_format == ExportFormat::Bson {
                if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                    use base64::{Engine as _, engine::general_purpose};
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
                        && ui
                            .button("Download .bson File")
                            .on_hover_text("Download compressed binary backup to your device")
                            .clicked()
                        && let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire)
                    {
                        trigger_binary_download(
                            "ipip_neo_assessment_backup.bson",
                            &bytes,
                            "application/octet-stream",
                        );
                    }
                    if ui
                        .button("Copy to Clipboard")
                        .on_hover_text("Copy formatted export data directly to clipboard")
                        .clicked()
                    {
                        ui.ctx().copy_text(self.export_text_buffer.clone());
                        self.export_copied_notification = Some(ui.input(|i| i.time));
                    }
                    if let Some(t) = self.export_copied_notification
                        && ui.input(|i| i.time) - t < 3.0
                    {
                        ui.label(
                            egui::RichText::new("Copied to clipboard").color(egui::Color32::GREEN),
                        );
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

    pub(crate) fn render_import_dialog(&mut self, ui: &mut egui::Ui) {
        let mut open = true;
        egui::Window::new("Import Assessment Backup")
            .open(&mut open)
            .default_size(egui::vec2(520.0, 420.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                ui.label("Restore your assessment progress from a previously exported backup file (.bson, .json, .csv):");
                ui.add_space(8.0);

                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        let file_btn = egui::Button::new(
                            egui::RichText::new("Browse Backup File...")
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
                                                self.import_result_message = Some(Ok(format!("Successfully imported {} answers from '{}'", count, filename)));
                                            }
                                            Err(e) => {
                                                self.import_result_message = Some(Err(e));
                                            }
                                        }
                                    }
                                }
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                let pending = self.pending_dropped_file.clone();
                                let ctx = ui.ctx().clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Some(file_handle) = rfd::AsyncFileDialog::new()
                                        .add_filter("Assessment Backup (.bson, .json, .csv)", &["bson", "json", "csv"])
                                        .pick_file()
                                        .await
                                    {
                                        let bytes = file_handle.read().await;
                                        let filename = file_handle.file_name();
                                        if let Ok(mut guard) = pending.lock() {
                                            *guard = Some((bytes, filename));
                                        }
                                        ctx.request_repaint();
                                    }
                                });
                            }
                        }

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Tip: You can also drag and drop your .bson, .json, or .csv file directly onto the app window").small().weak());
                        ui.add_space(4.0);
                    });
                });

                ui.add_space(8.0);

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
                            self.import_result_message = Some(Err("Pasted text is empty".to_string()));
                        } else {
                            match self.import_from_bytes(&input, "pasted text") {
                                Ok(count) => {
                                    let current_t = ui.input(|i| i.time);
                                    self.last_save_time = Some(current_t);
                                    self.import_result_message = Some(Ok(format!("Successfully imported {} answers from pasted text", count)));
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
