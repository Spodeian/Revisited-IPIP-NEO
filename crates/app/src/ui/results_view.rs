use eframe::egui;
use shared::{
    Aspect, Facet, MetaTrait, ScoreTier, Trait, encode_responses_to_url_code,
    export_to_compressed_bson, export_to_csv, export_to_json, export_to_printable_html,
    export_to_svg,
};

use crate::app::PersonalityApp;
use crate::storage_manager::{trigger_binary_download, trigger_text_download};
use crate::types::ExportFormat;

impl PersonalityApp {
    pub(crate) fn render_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("📊  Assessment Results");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕  Close").on_hover_text("Close results panel and return to questions (Escape)").clicked() {
                        self.state.questionnaire.show_results = false;
                        self.persist_state();
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
                if ui.button("↩  Return to Saved Assessment").on_hover_text("Exit shared link view and restore locally saved assessment").clicked() {
                    self.restore_saved_instance();
                }
            }

            ui.add_space(4.0);
            if ui.checkbox(&mut self.state.questionnaire.show_detailed_stats, "Show Detailed Metrics & SE")
                .on_hover_text("Toggle raw score sums, absolute item weights, sample counts, and Standard Error (SE) values")
                .changed()
            {
                self.persist_state();
            }

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("🔗  Share Link").on_hover_text("Copy shareable results URL to clipboard without affecting recipients' saved progress").clicked() {
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

                egui::ComboBox::from_id_salt("export_format_dropdown")
                    .selected_text(format!("📄 {}", self.selected_export_format.label()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Csv, "📊 CSV File");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Json, "⚙ JSON File");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Bson, "📦 Compressed BSON (.bson)");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Svg, "🎨 SVG Vector Graphic");
                        ui.selectable_value(&mut self.selected_export_format, ExportFormat::Html, "🖨 HTML Report");
                    });

                if ui.button("💾  Download File").on_hover_text("Export results and download selected file format to your device").clicked() {
                    match self.selected_export_format {
                        ExportFormat::Csv => {
                            let csv_content = export_to_csv(&self.state.questionnaire);
                            trigger_text_download("ipip_neo_tga_results.csv", &csv_content, "text/csv;charset=utf-8");
                        }
                        ExportFormat::Json => {
                            let json_content = export_to_json(&self.state.questionnaire);
                            trigger_text_download("ipip_neo_tga_results.json", &json_content, "application/json;charset=utf-8");
                        }
                        ExportFormat::Bson => {
                            if let Ok(bytes) = export_to_compressed_bson(&self.state.questionnaire) {
                                trigger_binary_download("ipip_neo_tga_results.bson", &bytes, "application/octet-stream");
                            }
                        }
                        ExportFormat::Svg => {
                            let svg_content = export_to_svg(&self.state.questionnaire);
                            trigger_text_download("ipip_neo_tga_results.svg", &svg_content, "image/svg+xml;charset=utf-8");
                        }
                        ExportFormat::Html => {
                            let html_content = export_to_printable_html(&self.state.questionnaire);
                            trigger_text_download("ipip_neo_tga_report.html", &html_content, "text/html;charset=utf-8");
                        }
                    }
                }
            });

            if let Some(t) = self.share_link_copied_time
                && ui.input(|i| i.time) - t < 3.0
            {
                ui.label(egui::RichText::new("✓  Share link copied to clipboard").color(egui::Color32::from_rgb(80, 180, 90)).strong());
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Accessible Results Table (3σ / 2σ / 1σ intervals)
            ui.collapsing("Accessible Results Table (Screen Reader View)", |ui| {
                ui.label(egui::RichText::new("Flat, linear summary of all traits, domains, and facets for assistive technology navigation").small().weak());
                ui.add_space(4.0);
                egui::Grid::new("accessible_results_summary_grid")
                    .striped(true)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Construct").strong());
                        ui.label(egui::RichText::new("Tier").strong());
                        ui.label(egui::RichText::new("Score").strong());
                        ui.label(egui::RichText::new("CI").strong());
                        ui.label(egui::RichText::new("Progress").strong());
                        ui.end_row();

                        for &meta in &MetaTrait::ALL {
                            let acc = self.state.questionnaire.meta_trait_acc.get(&meta).copied().unwrap_or_default();
                            let score_str = acc.normalized_score().map(|s| format!("{:+.2}", s)).unwrap_or_else(|| "N/A".to_string());
                            let tier_str = acc.tier().map(|t| t.label()).unwrap_or("N/A");
                            let se = acc.standard_error().unwrap_or(0.0);
                            let ci_str = if let Some(s) = acc.normalized_score() {
                                format!("[{:+.2}, {:+.2}]", (s - se * 3.0).clamp(-1.0, 1.0), (s + se * 3.0).clamp(-1.0, 1.0))
                            } else {
                                "N/A".to_string()
                            };
                            ui.label(egui::RichText::new(format!("Meta: {}", meta.display_name())).strong());
                            ui.label(tier_str);
                            ui.label(score_str);
                            ui.label(ci_str);
                            ui.label(format!("{}/{}", acc.answered_count, acc.total_items));
                            ui.end_row();

                            for trait_item in meta.child_traits() {
                                let d_acc = self.state.questionnaire.trait_acc.get(&trait_item).copied().unwrap_or_default();
                                let d_score_str = d_acc.normalized_score().map(|s| format!("{:+.2}", s)).unwrap_or_else(|| "N/A".to_string());
                                let d_tier_str = d_acc.tier().map(|t| t.label()).unwrap_or("N/A");
                                let d_se = d_acc.standard_error().unwrap_or(0.0);
                                let d_ci_str = if let Some(s) = d_acc.normalized_score() {
                                    format!("[{:+.2}, {:+.2}]", (s - d_se * 2.0).clamp(-1.0, 1.0), (s + d_se * 2.0).clamp(-1.0, 1.0))
                                } else {
                                    "N/A".to_string()
                                };
                                ui.label(format!("  Trait: {}", trait_item.display_name()));
                                ui.label(d_tier_str);
                                ui.label(d_score_str);
                                ui.label(d_ci_str);
                                ui.label(format!("{}/{}", d_acc.answered_count, d_acc.total_items));
                                ui.end_row();

                                for facet in trait_item.child_facets() {
                                    let f_acc = self.state.questionnaire.facet_acc.get(&facet).copied().unwrap_or_default();
                                    let f_score_str = f_acc.normalized_score().map(|s| format!("{:+.2}", s)).unwrap_or_else(|| "N/A".to_string());
                                    let f_tier_str = f_acc.tier().map(|t| t.label()).unwrap_or("N/A");
                                    let f_se = f_acc.standard_error().unwrap_or(0.0);
                                    let f_ci_str = if let Some(s) = f_acc.normalized_score() {
                                        format!("[{:+.2}, {:+.2}]", (s - f_se * 1.0).clamp(-1.0, 1.0), (s + f_se * 1.0).clamp(-1.0, 1.0))
                                    } else {
                                        "N/A".to_string()
                                    };
                                    ui.label(format!("    Facet: {}", facet.display_name()));
                                    ui.label(f_tier_str);
                                    ui.label(f_score_str);
                                    ui.label(f_ci_str);
                                    ui.label(format!("{}/{}", f_acc.answered_count, f_acc.total_items));
                                    ui.end_row();
                                }
                            }
                        }
                    });
            });
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

    pub(crate) fn render_meta_trait_node(&self, ui: &mut egui::Ui, meta: MetaTrait) {
        let acc = self
            .state
            .questionnaire
            .meta_trait_acc
            .get(&meta)
            .copied()
            .unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;
        let id = ui.make_persistent_id(meta.display_name());
        let collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        collapsing
            .show_header(ui, |ui| {
                let label_resp =
                    ui.label(egui::RichText::new(meta.display_name()).strong().size(15.0));
                label_resp.on_hover_ui(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Meta-Trait: {}", meta.display_name()))
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(meta.description());
                    ui.add_space(4.0);
                    let children = meta
                        .child_traits()
                        .iter()
                        .map(|t| t.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    ui.label(
                        egui::RichText::new(format!("Subordinate Traits: {}", children))
                            .small()
                            .weak(),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_construct_badge_row(ui, &acc, show_detailed, 3.0, "3σ");
                });
            })
            .body(|ui| {
                for trait_item in meta.child_traits() {
                    self.render_trait_node(ui, trait_item);
                }
            });
    }

    pub(crate) fn render_trait_node(&self, ui: &mut egui::Ui, trait_item: Trait) {
        let acc = self
            .state
            .questionnaire
            .trait_acc
            .get(&trait_item)
            .copied()
            .unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;
        let id = ui.make_persistent_id(trait_item.display_name());
        let collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        collapsing
            .show_header(ui, |ui| {
                let label_resp =
                    ui.label(egui::RichText::new(trait_item.display_name()).size(14.0));
                label_resp.on_hover_ui(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Trait: {}", trait_item.display_name()))
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(trait_item.description());
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "Parent Meta-Trait: {}",
                            trait_item.parent_meta_trait().display_name()
                        ))
                        .small()
                        .weak(),
                    );
                    let children = trait_item
                        .child_facets()
                        .iter()
                        .map(|f| f.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    ui.label(
                        egui::RichText::new(format!("Subordinate Facets: {}", children))
                            .small()
                            .weak(),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_construct_badge_row(ui, &acc, show_detailed, 2.0, "2σ");
                });
            })
            .body(|ui| {
                for facet in trait_item.child_facets() {
                    self.render_facet_row(ui, facet);
                }
            });
    }

    pub(crate) fn render_facet_row(&self, ui: &mut egui::Ui, facet: Facet) {
        let acc = self
            .state
            .questionnaire
            .facet_acc
            .get(&facet)
            .copied()
            .unwrap_or_default();
        let show_detailed = self.state.questionnaire.show_detailed_stats;

        ui.horizontal(|ui| {
            let label_resp = ui.label(facet.display_name());
            label_resp.on_hover_ui(|ui| {
                ui.label(egui::RichText::new(format!("Facet: {}", facet.display_name())).strong());
                ui.add_space(2.0);
                ui.label(facet.description());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Parent Trait: {} (under {})",
                        facet.parent_trait().display_name(),
                        facet.parent_trait().parent_meta_trait().display_name()
                    ))
                    .small()
                    .weak(),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_construct_badge_row(ui, &acc, show_detailed, 1.0, "1σ");
            });
        });
    }

    pub(crate) fn render_score_gauge(
        ui: &mut egui::Ui,
        norm_score: f32,
        se: f32,
        tier_color: egui::Color32,
        width: f32,
        ci_mult: f32,
        ci_label: &str,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, 14.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let is_dark = ui.visuals().dark_mode;
            let track_bg = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25)
            };

            painter.rect_filled(rect, 3.0, track_bg);

            let center_x = rect.left() + rect.width() * 0.5;
            let tick_color = if is_dark {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
            };
            painter.line_segment(
                [
                    egui::pos2(center_x, rect.top() + 1.0),
                    egui::pos2(center_x, rect.bottom() - 1.0),
                ],
                egui::Stroke::new(1.0, tick_color),
            );

            let score_to_x = |s: f32| -> f32 {
                let norm = ((s.clamp(-1.0, 1.0) + 1.0) / 2.0).clamp(0.0, 1.0);
                rect.left() + norm * rect.width()
            };

            let score_x = score_to_x(norm_score);
            let center_y = rect.center().y;

            let error_span = se * ci_mult;
            let ci_min = (norm_score - error_span).clamp(-1.0, 1.0);
            let ci_max = (norm_score + error_span).clamp(-1.0, 1.0);
            let left_ci_x = score_to_x(ci_min);
            let right_ci_x = score_to_x(ci_max);

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

            painter.line_segment(
                [
                    egui::pos2(left_ci_x, center_y - 4.0),
                    egui::pos2(left_ci_x, center_y + 4.0),
                ],
                egui::Stroke::new(1.0, tier_color),
            );
            painter.line_segment(
                [
                    egui::pos2(right_ci_x, center_y - 4.0),
                    egui::pos2(right_ci_x, center_y + 4.0),
                ],
                egui::Stroke::new(1.0, tier_color),
            );

            painter.circle_filled(egui::pos2(score_x, center_y), 4.5, tier_color);
            painter.circle_stroke(
                egui::pos2(score_x, center_y),
                4.5,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        }

        let ci_min = (norm_score - se * ci_mult).clamp(-1.0, 1.0);
        let ci_max = (norm_score + se * ci_mult).clamp(-1.0, 1.0);
        let ci_label_str = ci_label.to_string();
        response.widget_info(move || {
            egui::WidgetInfo::labeled(
                egui::WidgetType::ProgressIndicator,
                true,
                format!(
                    "Score gauge: normalized score {:+.2}, standard error {:.2}, {} confidence interval [{:+.2}, {:+.2}]",
                    norm_score, se, ci_label_str, ci_min, ci_max
                ),
            )
        });
        response.on_hover_ui(|ui| {
            ui.label(egui::RichText::new(format!("Normalized Score: {:+.2}", norm_score)).strong());
            ui.label(format!("Standard Error (SE): {:.2}", se));
            ui.label(format!(
                "Confidence Interval (±{}): [{:+.2}, {:+.2}]",
                ci_label, ci_min, ci_max
            ));
        });
    }

    pub(crate) fn render_construct_badge_row(
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

            let tier_badge_resp =
                ui.colored_label(tier_color, egui::RichText::new(tier.label()).strong());
            let tier_label_str = tier.label().to_string();
            tier_badge_resp.widget_info(move || {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Other,
                    true,
                    format!("Construct tier classification: {}", tier_label_str),
                )
            });
            tier_badge_resp.on_hover_ui(|ui| {
                ui.label(egui::RichText::new(format!("Classification: {}", tier.label())).strong());
                ui.label(format!(
                    "Normalized Score: {:+.2} (scale: -1.0 to +1.0)",
                    norm_score
                ));
                if let Some(se_val) = acc.standard_error() {
                    ui.label(format!("Standard Error (SE): {:.3}", se_val));
                }
                ui.label(format!(
                    "Progress: {}/{} items answered",
                    acc.answered_count, acc.total_items
                ));
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
}
