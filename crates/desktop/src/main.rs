//! Native desktop application runner for the Revisited IPIP-NEO Personality Assessment.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use app::PersonalityApp;
use eframe::NativeOptions;
use eframe::egui;
use spodeian_telemetry::init_default;

fn main() -> eframe::Result<()> {
    // Universal telemetry & logging initialization
    init_default();

    // Native window viewport configurations
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Revisited IPIP-NEO Personality Assessment")
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([300.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Revisited IPIP-NEO",
        options,
        Box::new(|cc| Ok(Box::new(PersonalityApp::new(cc)))),
    )
}
