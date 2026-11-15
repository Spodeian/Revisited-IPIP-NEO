# Changelog

All notable changes to `Revisited IPIP-NEO` are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and Semantic Versioning.

## [Unreleased]

### Added
- Versioned `FullAssessmentReport` (`REPORT_SCHEMA_VERSION = 2` with v1 serde fallback) and privacy-preserving `ShareableSummaryReport` (`export_shareable_summary_json`).
- Swappable `NormParameters` / `NormSet` priors (`z_score`, `percentile_estimate`) and `construct_narrative` helpers.
- Facet-balanced `AdministrationMode::Rapid112` (112 items covering all 28 facets) toggleable without losing responses.
- Longitudinal `profile_history` (up to `MAX_PROFILE_HISTORY = 50`) and two-profile comparison view.
- `Ctrl+K` / `Cmd+K` `CommandPaletteWidget` integration (`spodeian-ui`) and `Dyslexia-Friendly Reading` toggle.
- Criterion benchmarks (`scoring_benchmarks.rs`) and `proptest` queue-machine verification (`39` tests).

### Changed
- Migrated multi-tier persistence to `spodeian-persist` and JSON/BSON export to `spodeian-export`.
- Clarified export labels to "printable HTML and SVG" and "compressed BSON (deflate)".
