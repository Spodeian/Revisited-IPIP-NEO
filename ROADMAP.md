# Revisited IPIP-NEO Development Roadmap

---

## Phase 1: Core TGA Engine & Assessment UI [COMPLETE]
- [x] Decoupled crates (`shared`, `app`, `desktop`, `web`).
- [x] Full IPIP-NEO-221 item taxonomy with 28 facets, 6 primary traits, and 3 superordinate meta-traits (Samo et al., 2026).
- [x] 100% Client-side WebAssembly execution with zero server telemetry.
- [x] Multi-tier JSON / RON persistence.

---

## Phase 2: Performance & Benchmarking [CURRENT]
- [x] Zero-warning Clippy enforcement.
- [ ] Criterion benchmark suite for TGA graph resolution and norming calculations (`benches/`).
- [x] Compressed binary profile export (compressed BSON via deflate / `miniz_oxide`).
- [ ] Samo et al. (2026) OSF reference fixture verification & shared `spodeian-persist` integration.

---

## Phase 3: Reporting & Visualizations [CURRENT]
- [x] Spider / radar trait visualization in egui.
- [x] Printable HTML and vector SVG summary card exports.
- [ ] Longitudinal tracking of repeated assessments over time.

---

## Phase 4: Mobile Android Deployment (`cargo-ndk`) [ACTIVE]
- [x] Multi-target toolchain support (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`).
- [x] Native `cdylib` output with `android_main` activity entry point.
- [x] Automated compilation scripts (`scripts/build-android.ps1`, `scripts/build-android.sh`).
- [ ] Mobile touch slider responsiveness, Android SafeArea insets, and haptic feedback.
