use eframe::egui;
use shared::ThemeMode;

use crate::app::PersonalityApp;

impl PersonalityApp {
    pub(crate) fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.current_theme == Some(self.state.config.theme) {
            return;
        }
        self.current_theme = Some(self.state.config.theme);

        let visuals = match self.state.config.theme {
            ThemeMode::Light => {
                let mut light = egui::Visuals::light();
                light.panel_fill = egui::Color32::from_rgb(245, 244, 241);
                light.window_fill = egui::Color32::from_rgb(252, 250, 246);
                light.extreme_bg_color = egui::Color32::from_rgb(238, 236, 231);

                light.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(45, 44, 42);
                light.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(55, 54, 52);
                light.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(20, 20, 18);
                light.widgets.active.fg_stroke.color = egui::Color32::from_rgb(0, 0, 0);

                light.widgets.noninteractive.bg_stroke.color =
                    egui::Color32::from_rgb(222, 220, 215);
                light.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(212, 210, 205);

                light.widgets.inactive.bg_fill = egui::Color32::from_rgb(252, 251, 248);
                light.widgets.hovered.bg_fill = egui::Color32::from_rgb(236, 234, 229);
                light.widgets.active.bg_fill = egui::Color32::from_rgb(220, 218, 212);
                light
            }
            ThemeMode::Dark => egui::Visuals::dark(),
            ThemeMode::HighContrastDark => {
                let mut hc = egui::Visuals::dark();
                hc.panel_fill = egui::Color32::BLACK;
                hc.window_fill = egui::Color32::BLACK;
                hc.extreme_bg_color = egui::Color32::from_rgb(10, 10, 10);
                hc.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
                hc.widgets.inactive.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
                hc.widgets.hovered.fg_stroke =
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0));
                hc.widgets.active.fg_stroke =
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0));
                hc.widgets.noninteractive.bg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
                hc.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
                hc.widgets.hovered.bg_stroke =
                    egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 255, 0));
                hc.widgets.active.bg_stroke =
                    egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 255, 0));
                hc.widgets.inactive.bg_fill = egui::Color32::BLACK;
                hc.widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 30, 0);
                hc.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 0);
                hc
            }
            ThemeMode::HighContrastLight => {
                let mut hc = egui::Visuals::light();
                hc.panel_fill = egui::Color32::WHITE;
                hc.window_fill = egui::Color32::WHITE;
                hc.extreme_bg_color = egui::Color32::WHITE;
                hc.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);
                hc.widgets.inactive.fg_stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);
                hc.widgets.hovered.fg_stroke =
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 180));
                hc.widgets.active.fg_stroke =
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 220));
                hc.widgets.noninteractive.bg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
                hc.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
                hc.widgets.hovered.bg_stroke =
                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 0, 180));
                hc.widgets.active.bg_stroke =
                    egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 0, 220));
                hc.widgets.inactive.bg_fill = egui::Color32::WHITE;
                hc.widgets.hovered.bg_fill = egui::Color32::from_rgb(230, 235, 255);
                hc.widgets.active.bg_fill = egui::Color32::from_rgb(210, 220, 255);
                hc
            }
        };
        ctx.set_visuals(visuals);
    }
}
