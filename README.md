# Revisited IPIP-NEO Personality Assessment

[![CI & Test Suite](https://github.com/Spodeian/Revisited-IPIP-NEO/actions/workflows/ci.yml/badge.svg)](https://github.com/Spodeian/Revisited-IPIP-NEO/actions/workflows/ci.yml)
[![GitHub Pages](https://github.com/Spodeian/Revisited-IPIP-NEO/actions/workflows/static.yml/badge.svg)](https://github.com/Spodeian/Revisited-IPIP-NEO/actions/workflows/static.yml)
[![License: PolyForm Noncommercial 1.0.0](https://img.shields.io/badge/license-PolyForm%20Noncommercial%201.0.0-blue.svg)](LICENSE.md)
[![Rust: 2024 Edition](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![egui: 0.36](https://img.shields.io/badge/egui-0.36-blueviolet.svg)](https://github.com/emilk/egui)
[![Cloudflare Pages: v3](https://img.shields.io/badge/Cloudflare%20Pages-Build%20System%20v3-F38020.svg)](https://pages.cloudflare.com)

A modular, production-ready psychometric assessment application built with Rust and [`egui 0.36`](https://github.com/emilk/egui) targeting **Serverless Web (WASM / Cloudflare Pages / PWA)** and **Native Desktop (Windows, macOS, Linux)** platforms. It administers a comprehensive 221-item questionnaire using a mathematically optimized standard error reduction sequence.

---

## 🔬 Academic Reference

This implementation is based on the Taxonomic Graph Analysis (TGA) methodology detailed in the following publication:
- **Article**: Samo, A., Garrido, L. E., Abad, F. J., Golino, H., McAbee, S. T., & Christensen, A. P. (2026). *Revisiting the IPIP-NEO personality hierarchy with taxonomic graph analysis*. European Journal of Personality, 40(2), 369–390.
- **DOI**: [10.1177/08902070251352590](https://doi.org/10.1177/08902070251352590)
- **Open Science Framework (OSF)**: [https://osf.io/hwpa9](https://osf.io/hwpa9)

---

## 🏛️ Architectural Structure

The workspace is organized into four decoupled, single-responsibility crates:

```mermaid
graph TD
    Shared["crates/shared<br/>(Psychometric Models, Scoring Engine, Queue Machine, Datasets)"]
    App["crates/app<br/>(egui UI, Assessment Dashboard, Live Scoring Radar, Modals, Persistence)"]
    Desktop["crates/desktop<br/>(Native Desktop Executable via eframe / Winit)"]
    Web["crates/web<br/>(Static WebAssembly Entrypoint, PWA Service Worker, Trunk Assets)"]

    App --> Shared
    Desktop --> App
    Desktop --> Shared
    Web --> App
    Web --> Shared
```

- **[`crates/shared`](crates/shared)**: Core psychometric data models, CSV loaders, dynamic queue machine, scoring engine, and multi-format exporters (CSV, JSON, HTML, compressed BSON).
- **[`crates/app`](crates/app)**: `eframe` GUI implementation (focused single-question dashboard, keyboard shortcuts, scroll skip deferral, real-time side-by-side results, export panels, and multi-tier state management).
- **[`crates/desktop`](crates/desktop)**: Native desktop application runner.
- **[`crates/web`](crates/web)**: WebAssembly client entrypoint (`wasm-bindgen`), HTML5 shell, Trunk asset bundling, and PWA service worker.

---

## ✨ Key Features

- **🎯 Dynamic Queue Mechanics & Rapid Protocol**: Skips and defers questions (via mouse scroll or arrow keys) to the back of the queue, and supports switching non-destructively between the **Full Battery (221 items)** and a facet-balanced **Rapid Protocol (112 items)** covering all 28 facets.
- **📊 Mathematically Robust TGA Scoring, Norming & Narratives**: Computes raw scores, normalized construct averages, absolute weights, standard error ($SE$) values, synthetic theoretical Gaussian prior percentiles (`NormSet`), and plain-language construct narratives for **3 Meta-Traits**, **6 Traits**, and **28 Facets**.
- **📈 Longitudinal History & Delta Comparison**: Saves up to 50 local profile snapshots (`AppState::profile_history`) to compare baseline vs. current scores ($\Delta$) across sessions without sending data off-device.
- **⌘ Command Palette & Accessibility**: Quick `Ctrl+K` / `Cmd+K` Command Palette (`spodeian-ui`), WCAG 2.1 Light/Dark/High-Contrast themes, and an optional **Dyslexia-Friendly Reading Mode** for enlarged typography and generous touch/reading spacing.
- **💾 Resilient Multi-Tier State Persistence (`spodeian-persist`)**:
  - **Dedicated Key Storage**: Persists state under dedicated key `revisited_ipip_neo_state` to prevent namespace collisions, with `StateMigrator` dual-format JSON and legacy RON fallback.
  - **Active State Persistence & Diagnostics**: Dispatches immediate storage saves on every answer, protocol change, snapshot, or import, with browser durability and quota inspection.
- **📦 Instant & Privacy-Preserving Data Exporting (`spodeian-export`)**: Built-in exports for schema-v2 JSON, **Shareable Summary JSON** (raw item answers stripped), CSV, printable HTML (with Methods, Norming & Limitations), vector SVG, and compressed BSON.
- **📶 Immutable Serverless PWA Caching**: Hybrid caching strategy (**Network-First** for `index.html` to guarantee atomic releases; **Cache-First** for immutable, content-hashed `.wasm`, `.js`, and `.css` assets) with offline fallback.
- **🛡️ SRI Minification Immunity**: Configured with `data-integrity="none"` to allow aggressive post-build asset minification (HTML, CSS, JS) without SRI hash mismatches.

---

## 🛠️ Build Requirements

- **Rust Toolchain**: Automatically managed via [`rust-toolchain.toml`](rust-toolchain.toml) (installs stable with `wasm32-unknown-unknown`).
- **Trunk Bundler**:
  ```bash
  cargo install trunk
  ```
- **Node.js 24 LTS**: (Required for asset minification pipelines, Wrangler edge previews, and Cloudflare Pages compatibility; managed via [`.node-version`](.node-version) / [`.nvmrc`](.nvmrc)):
  ```bash
  nvm use # or fnm use
  ```
- *(Optional)* **wasm-opt** (Binaryen v122+) for release binary size optimization.

---

## 🚀 Development Quickstart

### 1. Run Web App Locally (Trunk)
```bash
trunk serve
```
Open [http://localhost:8080](http://localhost:8080) in your browser. Live reloading is automatically enabled.

### 2. Run Native Desktop App
```bash
cargo run -p desktop
```

### 3. Run Test Suite & Benchmarks
```bash
# Workspace unit, integration, and property-based tests
cargo test --workspace

# Or with cargo-nextest (faster, parallel execution)
cargo nextest run --workspace

# Criterion psychometric scoring & export benchmarks
cargo bench -p shared
```

### 4. Run Static Analysis & Linter
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 📦 Production Builds & Deployment

### Static Serverless WASM Bundle (Trunk)
```bash
trunk build --release
```
The optimized output assets (`index.html`, `.wasm`, `.js`, `.css`, `_headers`, `_redirects`, `sw.js`) will be located in `crates/web/dist/`.

### Cloudflare Pages (Build System v3)

Deploy directly to Cloudflare Pages using the automated build script:

```bash
bash deploy.sh
```

#### Cloudflare Pages Dashboard Settings:
- **Build System Version**: `v3` (2024/2026 build image)
- **Framework Preset**: `None` (or `Custom`)
- **Build Command**: `bash deploy.sh`
- **Build Output Directory**: `crates/web/dist`
- **Environment Variables**:
  - `NODE_VERSION`: `24`
  - `RUST_VERSION`: `stable` (Optional if `rust-toolchain.toml` is present)
  - `CARGO_HOME`: `/opt/buildhome/.cargo`

#### Local Preview with Wrangler:
```bash
trunk build --release
npx wrangler pages dev crates/web/dist
```

### GitHub Pages Deployment

The repository includes an automated GitHub Actions workflow in [`.github/workflows/static.yml`](.github/workflows/static.yml) that builds, minifies, and deploys the static WASM application to GitHub Pages whenever changes are pushed to `main`.

### Native Desktop Binary
```bash
cargo build -p desktop --release
```
The compiled release executable will be located in `target/release/`.

---

## 📄 Licensing & Attribution

This repository combines three things under different terms:

- **The software** (the Rust/WASM code, queue engine and UI) is licensed under
  the [PolyForm Noncommercial License 1.0.0](LICENSE.md). You may use, change
  and share it for any noncommercial purpose. For a commercial license,
  contact [Spodeian](https://github.com/Spodeian). Earlier releases keep the
  license they were published under. This covers the software only, not the
  items or the research materials below.
- **The questionnaire items** come from the
  [International Personality Item Pool](https://ipip.ori.org) and are in the
  public domain.
- **The scoring keys and weights** (`Distilled Key.csv`, `Optimized_Keys.csv`)
  are derived from the materials of Samo et al. (2026) on the
  [Open Science Framework](https://osf.io/hwpa9), published under
  [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Any reuse must
  credit that work; see [ATTRIBUTION.md](ATTRIBUTION.md).
