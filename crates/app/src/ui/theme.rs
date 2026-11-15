use eframe::egui;

use crate::app::PersonalityApp;

impl PersonalityApp {
    pub(crate) fn apply_theme(&mut self, ctx: &egui::Context) {
        let dyslexia = self.state.config.dyslexia_friendly;
        if self.current_theme == Some(self.state.config.theme)
            && self.current_dyslexia_mode == Some(dyslexia)
        {
            return;
        }
        self.current_theme = Some(self.state.config.theme);
        self.current_dyslexia_mode = Some(dyslexia);

        ctx.all_styles_mut(|style| {
            let defaults = egui::Style::default();
            style.text_styles = defaults.text_styles;
            style.spacing = defaults.spacing;
            if dyslexia {
                for font_id in style.text_styles.values_mut() {
                    font_id.size = (font_id.size * 1.12).round();
                }
                style.spacing.item_spacing = egui::vec2(10.0, 8.0);
                style.spacing.button_padding = egui::vec2(14.0, 10.0);
                style.spacing.indent = 22.0;
            }
        });
        spodeian_ui::apply_theme(ctx, self.state.config.theme);
    }
}
