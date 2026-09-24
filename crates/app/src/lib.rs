#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]

pub mod app;
pub mod storage_manager;
pub mod types;
pub mod ui;

pub use app::PersonalityApp;
pub use storage_manager::*;
pub use types::{ExportFormat, ScreenConstraints};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use eframe::NativeOptions;
    let mut options = NativeOptions::default();
    options.android_app = Some(app);
    eframe::run_native(
        "Revisited IPIP-NEO",
        options,
        Box::new(|cc| Ok(Box::new(PersonalityApp::new(cc)))),
    ).unwrap();
}
