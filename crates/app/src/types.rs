pub use spodeian_ui::ScreenConstraints;


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
    pub fn icon(self) -> &'static str {
        match self {
            Self::Csv => "📊",
            Self::Json => "⚙",
            Self::Bson => "📦",
            Self::Svg => "🎨",
            Self::Html => "🖨",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV File",
            Self::Json => "JSON File",
            Self::Bson => "Compressed BSON (.bson)",
            Self::Svg => "SVG Vector Graphic",
            Self::Html => "HTML Report",
        }
    }

    pub fn label_with_icon(self) -> String {
        format!("{}  {}", self.icon(), self.label())
    }
}
