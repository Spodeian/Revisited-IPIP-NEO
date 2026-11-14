# Revisited IPIP-NEO Development Roadmap

---

## Phase 1: Core TGA Engine & Assessment UI [COMPLETE]
- [x] Decoupled crates (`shared`, `app`, `desktop`, `web`).
- [x] Full IPIP-NEO-221 item taxonomy with 28 facets, 6 primary traits, and 3 superordinate meta-traits (Samo et al., 2026).
- [x] 100% Client-side WebAssembly execution with zero server telemetry.
- [x] Multi-tier JSON / RON persistence.

---

## Phase 2: Performance & Benchmarking [COMPLETE]
- [x] Zero-warning Clippy enforcement (`-D warnings`).
- [x] Criterion benchmark suite for TGA graph resolution, norming, exports, and URL codecs (`crates/shared/benches/scoring_benchmarks.rs`).
- [x] Compressed binary profile export (`spodeian-export` deflate BSON & schema-v2 JSON versioning).
- [x] Shared ecosystem integration (`spodeian-persist`, `spodeian-export`, `spodeian-ui` command palette) & TGA `NormSet` priors.

---

## Phase 3: Reporting, Accessibility & Longitudinal Tracking [COMPLETE]
- [x] Hierarchical trait/facet dashboard & CI score gauges in `egui`.
- [x] Printable HTML, vector SVG, and privacy-preserving `ShareableSummaryReport` JSON (raw item answers stripped).
- [x] Longitudinal profile snapshots & two-session delta comparison (`AppState::profile_history`).
- [x] Plain-language construct narratives, Full (221) vs. Rapid (112) facet-balanced protocol, and Dyslexia-Friendly Reading Mode.

---

## Phase 4: Mobile Android Deployment (`cargo-ndk`) [ACTIVE]
- [x] Multi-target toolchain support (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`).
- [x] Native `cdylib` output with `android_main` activity entry point.
- [x] Automated compilation scripts (`scripts/build-android.ps1`, `scripts/build-android.sh`).
- [ ] Mobile touch slider responsiveness, Android SafeArea insets, and haptic feedback.
