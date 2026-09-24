# Revisited IPIP-NEO Development Roadmap

---

## Phase 1: Core TGA Engine & Assessment UI [COMPLETE]
- [x] Decoupled crates (`shared`, `app`, `desktop`, `web`).
- [x] Full IPIP-NEO-120 item taxonomy with 6 primary factors and 3 superordinate meta-factors.
- [x] 100% Client-side WebAssembly execution with zero server telemetry.
- [x] Multi-tier JSON / RON persistence.

---

## Phase 2: Performance & Benchmarking [CURRENT]
- [x] Zero-warning Clippy enforcement.
- [ ] Criterion benchmark suite for TGA graph resolution and norming calculations (`benches/`).
- [ ] Compressed binary profile export (BSON/Zstd).

---

## Phase 3: Reporting & Visualizations
- [x] Spider / radar trait visualization in egui.
- [x] PDF / High-resolution PNG summary card exports.
- [ ] Longitudinal tracking of repeated assessments over time.
