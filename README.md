# Revisited IPIP-NEO Personality Assessment

A modular, production-ready psychometric application built with Rust and `egui` targeting Native Desktop and Web (WASM) platforms. It administers a comprehensive 221-item questionnaire using a mathematically optimized standard error reduction sequence.

## Academic & Psychometric Reference
This implementation is based on the sequence optimization methodology detailed in the following publication:
- **Reference URL**: [https://doi.org/10.1177/08902070251352590](https://doi.org/10.1177/08902070251352590)

## Structure
- `crates/shared`: Core psychometric data models, CSV loaders, dynamic queue machine, scoring engine, and data exporters (CSV, JSON, HTML).
- `crates/app`: `eframe` GUI implementation (focused single-question dashboard, keyboard shortcuts, scroll skip deferral, real-time side-by-side results, and export panels).
- `crates/desktop`: Native desktop application runner.
- `crates/web`: `wasm-bindgen` runner for serverless and browser deployment.

## Key Features
- **Dynamic Queue Mechanics**: Skips and defers questions (via mouse scroll/arrow keys) to the back of the queue, ensuring skipped items reappear at the end.
- **Robust Scoring Engine**: Computes exact raw scores, normalized construct averages, absolute weights, and standard error ($SE$) values for 3 Meta-Traits, 6 Traits, and 28 Facets.
- **Instant Data Exporting**: Provides native tools for copying or downloading reports as raw CSV, structured JSON, or printable HTML/PDF.
- **Cross-Platform State Persistence**: Saves in-progress tests automatically across Native Desktop restarts and Web browser refreshes using `eframe::Storage`.

## Build Requirements
- Rust (latest stable)
- For Web builds: `trunk` (`cargo install trunk`) and the wasm target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

## Development & Production Guides

### 1. Desktop Development
Run the native desktop application:
```bash
cargo run -p desktop
```

### 2. Web Development (Local Serve)
Serve the application locally in your browser:
```bash
trunk serve
```
Then navigate to `http://127.0.0.1:8080`.

### 3. Running Verification & Tests
Ensure the code passes clippy static analysis and unit/integration tests:
```bash
cargo clippy --workspace --all-targets
cargo test --workspace
```

### 4. Build for Production Deployment
- **Native Desktop**:
  ```bash
  cargo build -p desktop --release
  ```
- **Web (Serverless / Static Hosting)**:
  - You can deploy the compiled web application directly to Cloudflare Pages (Serverless static CDN) by configuring the build command to use our automated deploy pipeline script:
    ```bash
    bash deploy_cloudflare.sh
    ```
    Configure the Cloudflare Pages **Build Output Directory** to `crates/web/dist` and set the Environment Variable `RUST_VERSION` to `stable`.
  - Alternatively, compile locally using `trunk build --release` and upload the static directory.
