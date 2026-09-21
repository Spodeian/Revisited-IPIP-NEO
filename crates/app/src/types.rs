use eframe::egui;

/// Responsive layout constraints computed against the active UI viewport.
pub struct ScreenConstraints {
    pub is_mobile: bool,
    pub is_mobile_portrait: bool,
    pub is_tight_height: bool,
    pub is_ultra_tight: bool,
}

impl ScreenConstraints {
    pub fn compute(ui: &egui::Ui) -> Self {
        let avail_w = ui.available_width();
        let avail_h = ui.available_height();

        Self {
            is_mobile: avail_w < 800.0,
            is_mobile_portrait: avail_w < 650.0,
            is_tight_height: avail_h < 530.0 || avail_w < 350.0,
            is_ultra_tight: avail_w < 330.0 || avail_h < 490.0,
        }
    }
}

/// Data interchange formats supported for assessment exports.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ExportFormat {
    #[default]
    Csv,
    Json,
    Bson,
    Svg,
    Html,
}

impl ExportFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV File",
            Self::Json => "JSON File",
            Self::Bson => "Compressed BSON (.bson)",
            Self::Svg => "SVG Vector Graphic",
            Self::Html => "HTML Report",
        }
    }
}
