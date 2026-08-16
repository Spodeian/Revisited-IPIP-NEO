use app::PersonalityApp;
use eframe::egui;
use eframe::NativeOptions;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> eframe::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,wgpu=warn,egui=warn")),
        )
        .init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Revisited IPIP-NEO Personality Assessment")
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Revisited IPIP-NEO",
        options,
        Box::new(|cc| Ok(Box::new(PersonalityApp::new(cc)))),
    )
}
