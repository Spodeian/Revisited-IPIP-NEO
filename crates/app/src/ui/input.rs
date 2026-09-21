use eframe::egui;
use shared::Response;

use crate::app::PersonalityApp;

impl PersonalityApp {
    pub(crate) fn handle_keyboard_and_scroll(&mut self, ui: &mut egui::Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }

        let (current_time, cmd_or_ctrl, shift_held) = ui.input(|i| {
            (
                i.time,
                i.modifiers.command || i.modifiers.ctrl,
                i.modifiers.shift,
            )
        });

        let record_answer_timestamp =
            |timestamps: &mut std::collections::VecDeque<f64>, save_time: &mut Option<f64>| {
                timestamps.push_back(current_time);
                if timestamps.len() > 25 {
                    timestamps.pop_front();
                }
                *save_time = Some(current_time);
            };

        // Keyboard shortcuts for responses: 1-5
        if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.answer_question(
                self.state.questionnaire.current_focus_idx,
                Response::StronglyDisagree,
            );
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
            self.persist_state();
        } else if ui.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.answer_question(
                self.state.questionnaire.current_focus_idx,
                Response::Disagree,
            );
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
            self.persist_state();
        } else if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.answer_question(
                self.state.questionnaire.current_focus_idx,
                Response::Neutral,
            );
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
            self.persist_state();
        } else if ui.input(|i| i.key_pressed(egui::Key::Num4)) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .answer_question(self.state.questionnaire.current_focus_idx, Response::Agree);
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
            self.persist_state();
        } else if ui.input(|i| i.key_pressed(egui::Key::Num5)) {
            self.is_viewing_shared_link = false;
            self.state.questionnaire.answer_question(
                self.state.questionnaire.current_focus_idx,
                Response::StronglyAgree,
            );
            record_answer_timestamp(&mut self.answer_timestamps, &mut self.last_save_time);
            self.persist_state();
        }

        // Undo shortcut: Ctrl+Z / Cmd+Z
        if cmd_or_ctrl
            && !shift_held
            && ui.input(|i| i.key_pressed(egui::Key::Z))
            && self.state.questionnaire.undo()
        {
            self.undo_notification_time = Some(current_time);
            self.last_save_time = Some(current_time);
            self.persist_state();
        }

        // Redo shortcut: Ctrl+Y / Cmd+Y OR Ctrl+Shift+Z / Cmd+Shift+Z
        if cmd_or_ctrl
            && ((!shift_held && ui.input(|i| i.key_pressed(egui::Key::Y)))
                || (shift_held && ui.input(|i| i.key_pressed(egui::Key::Z))))
            && self.state.questionnaire.redo()
        {
            self.redo_notification_time = Some(current_time);
            self.last_save_time = Some(current_time);
            self.persist_state();
        }

        // Delete / Backspace shortcut: Clear recorded answer
        if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            self.is_viewing_shared_link = false;
            self.state
                .questionnaire
                .clear_response(self.state.questionnaire.current_focus_idx);
            self.last_save_time = Some(current_time);
            self.persist_state();
        }

        // Escape key to dismiss dialogs or close results screen
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
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
                self.persist_state();
            }
        }

        // Navigation shortcuts
        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowUp)) {
            if shift_held {
                self.state.questionnaire.navigate_previous_unanswered();
            } else {
                self.state.questionnaire.navigate_previous();
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowDown))
        {
            if shift_held {
                self.state.questionnaire.navigate_next_unanswered();
            } else {
                self.state.questionnaire.skip_current();
            }
        }
    }
}
