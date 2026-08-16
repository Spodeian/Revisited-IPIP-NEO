//! Web entrypoint and runner for Revisited IPIP-NEO.

#[cfg(target_arch = "wasm32")]
use app::PersonalityApp;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), JsValue> {
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::WebRunner::new()
        .start(
            canvas_id,
            web_options,
            Box::new(|cc| Ok(Box::new(PersonalityApp::new(cc)))),
        )
        .await?;
    Ok(())
}
