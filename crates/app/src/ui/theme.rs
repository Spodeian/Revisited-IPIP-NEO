use eframe::egui;

use crate::app::PersonalityApp;

impl PersonalityApp {
    pub(crate) fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.current_theme == Some(self.state.config.theme) {
            return;
        }
        self.current_theme = Some(self.state.config.theme);
        spodeian_ui::apply_theme(ctx, self.state.config.theme);
    }
}
