use eframe::egui;
use shared::{Aspect, Response};

use crate::app::PersonalityApp;
use crate::types::ScreenConstraints;

impl PersonalityApp {
    pub(crate) fn render_question_focus(&mut self, ui: &mut egui::Ui, constraints: &ScreenConstraints) {
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
                let top_space = if is_ultra_tight { 0.0 } else if is_tight_height { 4.0 } else if is_mobile_portrait { 8.0 } else { 20.0 };
                ui.add_space(top_space);

                ui.vertical_centered(|ui| {
                    let max_width = (avail_width - 8.0).min(700.0);
                    ui.set_max_width(max_width);

                    let framing_font_size = if is_ultra_tight { 15.0 } else if is_tight_height { 17.0 } else { 19.0 };
                    ui.label(
                        egui::RichText::new("Rate how accurately this statement describes you:")
                            .size(framing_font_size)
                            .strong()
                            .color(ui.visuals().hyperlink_color),
                    ).on_hover_text("Select the response option (1–5) that best reflects your typical behavior and self-perception");
                    ui.add_space(if is_ultra_tight { 4.0 } else { 10.0 });

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
                        (Response::StronglyDisagree, "Strongly Disagree", "1", "Strongly Disagree (-1.0 point) — Press '1'"),
                        (Response::Disagree, "Disagree", "2", "Disagree (-0.5 points) — Press '2'"),
                        (Response::Neutral, "Neutral", "3", "Neutral (0.0 points) — Press '3'"),
                        (Response::Agree, "Agree", "4", "Agree (+0.5 points) — Press '4'"),
                        (Response::StronglyAgree, "Strongly Agree", "5", "Strongly Agree (+1.0 point) — Press '5'"),
                    ];

                    let button_height = if is_ultra_tight { 34.0 } else if is_tight_height { 38.0 } else { 44.0 };
                    let button_text_size = if is_ultra_tight { 13.5 } else if is_tight_height { 14.5 } else if is_mobile_portrait { 15.5 } else { 16.0 };
                    let btn_width = (ui.available_width() - 8.0).min(340.0);

                    for (resp, text, shortcut, tooltip_desc) in responses {
                        let is_selected = q_response == Some(resp);

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
                            self.persist_state();
                        }
                        ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 6.0 });
                    }

                    let space_before_nav = if is_ultra_tight { 4.0 } else if is_tight_height { 6.0 } else if is_mobile_portrait { 15.0 } else { 25.0 };
                    ui.add_space(space_before_nav);
                    ui.separator();
                    ui.add_space(if is_ultra_tight { 2.0 } else if is_tight_height { 4.0 } else { 10.0 });

                    let curr_focus = curr_idx + 1;
                    let progress = self.state.questionnaire.completion_rate();
                    let progress_text = format!("Item #{} of {} ({:.0}%)", curr_focus, total, progress * 100.0);
                    let answered = self.state.questionnaire.answered_count();
                    let remaining = total.saturating_sub(answered);
                    let progress_hover_text = format!(
                        "Assessment progress: {}/{} answered ({:.1}%)\n{} remaining question{}",
                        answered, total, progress * 100.0, remaining, if remaining == 1 { "" } else { "s" }
                    );

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(if is_ultra_tight { 4.0 } else { 6.0 }, 6.0);

                        let btn_prev = if is_ultra_tight { "◀" } else { "◀ Prev" };
                        if ui.button(btn_prev).on_hover_text("Previous item in sequence (Left Arrow / Mouse Scroll Up)").clicked() {
                            self.state.questionnaire.navigate_previous();
                        }

                        let btn_prev_un = if is_ultra_tight { "⏪" } else { "⏪ Unanswered" };
                        if ui.button(btn_prev_un).on_hover_text("Jump backward to nearest unanswered question (Shift + Left Arrow)").clicked() {
                            self.state.questionnaire.navigate_previous_unanswered();
                        }

                        if self.state.questionnaire.can_undo() {
                            let btn_undo = if is_ultra_tight { "Undo" } else { "Undo" };
                            if ui.button(btn_undo).on_hover_text("Undo previous response change (Ctrl+Z / Cmd+Z)").clicked()
                                && self.state.questionnaire.undo()
                            {
                                let current_t = ui.input(|i| i.time);
                                self.undo_notification_time = Some(current_t);
                                self.last_save_time = Some(current_t);
                                self.persist_state();
                            }
                        }

                        if self.state.questionnaire.can_redo() {
                            let btn_redo = if is_ultra_tight { "Redo" } else { "Redo" };
                            if ui.button(btn_redo).on_hover_text("Redo reverted response change (Ctrl+Y / Cmd+Shift+Z)").clicked()
                                && self.state.questionnaire.redo()
                            {
                                let current_t = ui.input(|i| i.time);
                                self.redo_notification_time = Some(current_t);
                                self.last_save_time = Some(current_t);
                                self.persist_state();
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_skip = if is_ultra_tight { "Skip" } else { "Skip ⏭" };
                            if ui.button(btn_skip).on_hover_text("Skip question and defer to end of pending queue (Right Arrow / Mouse Scroll Down)").clicked() {
                                self.state.questionnaire.skip_current();
                            }

                            let btn_next_un = if is_ultra_tight { "Next Unanswered" } else { "Next Unanswered ⏩" };
                            if ui.button(btn_next_un).on_hover_text("Jump forward to nearest unanswered question (Shift + Right Arrow)").clicked() {
                                self.state.questionnaire.navigate_next_unanswered();
                            }

                            if q_response.is_some() {
                                let btn_clear = if is_ultra_tight { "Clear" } else { "Clear" };
                                if ui.button(btn_clear).on_hover_text("Clear recorded answer for this question and mark it unanswered").clicked() {
                                    self.is_viewing_shared_link = false;
                                    self.state.questionnaire.clear_response(curr_idx);
                                    let current_t = ui.input(|i| i.time);
                                    self.last_save_time = Some(current_t);
                                    self.persist_state();
                                }
                            }

                            let remaining_width = (ui.available_width() - 8.0).max(0.0);
                            if remaining_width > 20.0 {
                                let pb_resp = ui.add(
                                    egui::ProgressBar::new(progress)
                                        .text(progress_text.clone())
                                        .desired_width(remaining_width),
                                ).on_hover_text(&progress_hover_text);
                                let p_text = progress_text.clone();
                                pb_resp.widget_info(move || {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::ProgressIndicator,
                                        true,
                                        format!("Questionnaire assessment progress: {}", p_text),
                                    )
                                });
                            }
                        });
                    });
                });
            });

        let current_time = ui.input(|i| i.time);
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);

        if !is_tight_height
            && !is_mobile_portrait
            && !is_ultra_tight
            && ui.rect_contains_pointer(ui.max_rect())
        {
            if scroll_y.abs() > 1.0 {
                self.scroll_accumulator += scroll_y;
            }

            if current_time - self.last_scroll_time > 0.35 {
                if self.scroll_accumulator < -40.0 {
                    self.state.questionnaire.skip_current();
                    self.last_scroll_time = current_time;
                    self.scroll_accumulator = 0.0;
                } else if self.scroll_accumulator > 40.0 {
                    self.state.questionnaire.navigate_previous();
                    self.last_scroll_time = current_time;
                    self.scroll_accumulator = 0.0;
                }
            }

            if current_time - self.last_scroll_time > 0.5 && scroll_y.abs() < 1.0 {
                self.scroll_accumulator = 0.0;
            }
        } else {
            self.scroll_accumulator = 0.0;
        }

        if self.scroll_accumulator.abs() > 0.0 {
            ui.ctx().request_repaint();
        }
    }
}
