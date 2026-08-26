# Revisited IPIP-NEO Personality Assessment

A modular, production-ready psychometric application built with Rust and `egui` targeting Native Desktop and Web (WASM) platforms. It administers a comprehensive 221-item questionnaire using a mathematically optimized standard error reduction sequence.

## Academic Reference
This implementation is based on the Taxonomic Graph Analysis (TGA) methodology detailed in the following publication:
- **Article**: Samo, A., Garrido, L. E., Abad, F. J., Golino, H., McAbee, S. T., & Christensen, A. P. (2026). *Revisiting the IPIP-NEO personality hierarchy with taxonomic graph analysis*. European Journal of Personality, 40(2), 369–390.
- **DOI**: [10.1177/08902070251352590](https://doi.org/10.1177/08902070251352590)
- **Open Science Framework (OSF)**: [https://osf.io/hwpa9](https://osf.io/hwpa9)

## Licensing & Attribution
This project is dual-licensed to fully respect academic research rights and software author contributions:

**Standard Non-Commercial License**: [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC-BY-NC-SA 4.0)](LICENSE).
   - Allows copying, modifying, and redistributing the codebase for non-commercial purposes, provided downstream edits are shared under the same license.
**Commercial use**: Any commercial use requires separate licensing for the [library](https://github.com/Spodeian/Revisited-IPIP-NEO) and the research ([Data](Distilled%20Key.csv)/[Research](https://doi.org/10.1177/08902070251352590)).

## Structure
- `crates/shared`: Core data models, CSV loaders, dynamic queue machine, scoring engine, and data exporters (CSV, JSON, HTML).
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
Then navigate to [localhost:8080](http://localhost:8080).

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

## Cloudflare Pages Deployment Configuration Reference
When setting up your automated deployment in the **Cloudflare Pages Dashboard**, use these exact fields:

### 1. Build Settings
- **Framework Preset**: `None` (or `Custom`)
- **Build Command**: `bash deploy_cloudflare.sh`
- **Build Output Directory**: `crates/web/dist`
- **Root Directory**: *(Leave empty or use `/`)*

### 2. Environment Variables (Advanced)
Configure these three key-value pairs in the dashboard:
- **`RUST_VERSION`**: `stable` (Installs the latest stable compiler toolchain)
- **`CARGO_HOME`**: `/opt/buildhome/.cargo` (Caches cargo registry assets to speed up builds 5x!)

---

## Testing Serverless Builds with Wrangler
You can run a local emulation of Cloudflare's serverless edge environment using **Wrangler**:

1. **Install Wrangler globally**:
   ```bash
   npm install -g wrangler
   ```
2. **Compile the production assets**:
   ```bash
   trunk build --release
   ```
3. **Run the Pages emulation server**:
   ```bash
   wrangler pages dev
   ```
   *(This reads [wrangler.toml](wrangler.toml) and hosts your compiled assessment perfectly on [localhost:8788](http://localhost:8788))*
