# Implementation Plan: Revisited IPIP-NEO Personality Assessment

## Overview
This project is a psychometrically robust personality test application built with Rust and `egui` (targeting Desktop and Web/WASM). It administers a 221-item questionnaire derived from `Distillined Key.csv` in an optimized order specified by `Optimized_Keys.csv`.

The sequence optimization methodology and factor weights are formally referenced in:
- **DOI Publication**: [https://doi.org/10.1177/08902070251352590](https://doi.org/10.1177/08902070251352590)

---

## 1. Domain & Psychometric Model

### 1.1 Construct Hierarchy
- **3 Meta-Traits**: Stability, Plasticity, Disinhibition
- **6 Traits**: Neuroticism, Sociability, Conscientiousness, Integrity, Openness to Experience, Impulsivity
- **28 Facets**: Anxiety, Gregariousness, Trust, Self-efficacy, Anger, Fairness, Orderliness, Dominance, Emotionality, Adventurousness, Determination, Excitement-seeking, Intellect, Attention-seeking, Cheerfulness, Liberalism, Artistic Interests, Empathy, Work Ethic, Cautiousness, Manipulativeness, Humility, Introspection, Honesty, Immoderation, Self-discipline, Recklessness, Calmness.

### 1.2 Scoring Mathematics
For each construct $A$:
- **Likert Responses**:
  - Strongly Disagree: $-1.0$ (Key `1`)
  - Disagree: $-0.5$ (Key `2`)
  - Neutral: $0.0$ (Key `3`)
  - Agree: $+0.5$ (Key `4`)
  - Strongly Agree: $+1.0$ (Key `5`)
- **Raw Score**:
  $$S_{\text{raw}} = \sum_{i \in \text{answered}} r_i \cdot w_i$$
- **Absolute Weight Sum**:
  $$W_{\text{abs}} = \sum_{i \in \text{answered}} |w_i|$$
- **Normalized Score**:
  $$\hat{S} = \frac{\sum r_i w_i}{\sum |w_i|} \in [-1.0, 1.0]$$
- **Standard Error ($SE$)**:
  $$SE = \frac{\sqrt{\sum_{i \in \text{answered}} w_i^2}}{\sum_{i \in \text{answered}} |w_i|}$$

### 1.3 5-Tier Classification
- **Very Low**: $[-1.00, -0.60)$
- **Low**: $[-0.60, -0.20)$
- **Average / Neutral**: $[-0.20, +0.20]$
- **High**: $(+0.20, +0.60]$
- **Very High**: $(+0.60, +1.00]$

---

## 2. Interaction & UX Flow

1. **Focused Single-Question Mode**:
   - One question presented prominently at center screen.
   - Large Likert response buttons with keyboard shortcuts `1`–`5`.
   - Selection immediately records response and auto-advances.
2. **Skipping & Dynamic Queue**:
   - Scrolling (mouse wheel / touchpad gesture) or pressing `Skip` / `Right` / `Down` defers the current unanswered question to the back of the queue.
   - Back navigation (`Left` / `Up` / `Previous`) allows reviewing and changing previous answers.
3. **Results Visibility & Side Panel**:
   - Hidden during test taking by default.
   - Automatically becomes available when all 221 questions are completed, or anytime on-demand by clicking "Show Results Early".
   - Rendered in a collapsible/resizable side panel alongside the test.
4. **Detailed Analytics Toggle**:
   - Summary view shows 5-tier classification pills/badges.
   - Expandable detailed table reveals Normalized Score, Raw Score, Weight Sums, and $SE$.
5. **Persistence & Reset**:
   - Progress saved continuously to `eframe::Storage` (localStorage on Web, JSON storage on Desktop).
   - "Reset Assessment" modal with confirmation to start fresh.
6. **Export Options**:
   - CSV export (all items + scores).
   - JSON export (structured machine-readable state).
   - Print/PDF trigger (`window.print()` on Web, printable dialog on Desktop).

## 3. Serverless Cloudflare Publishing

The application is fully prepared for one-click serverless deployment on **Cloudflare Pages**:
1. **Automated Pipeline**: The root folder includes `deploy_cloudflare.sh`, which automatically checks/installs Rust, adds the `wasm32-unknown-unknown` target, installs `trunk`, and builds the optimized web assets.
2. **Cloudflare Pages Build Settings**:
   - **Build Command**: `bash deploy_cloudflare.sh`
   - **Build Output Directory**: `crates/web/dist`
   - **Environment Variable**: `RUST_VERSION` = `stable`

---

## 4. Architecture & Crate Responsibilities

```
Revisited IPIP-NEO/
├── crates/
│   ├── shared/      # Core data models, CSV parsing, question queue, scoring math, exports
│   ├── app/         # egui UI components (Question Card, Progress Bar, Side Results, Modals)
│   ├── desktop/     # Native desktop runner (window configuration, storage)
│   └── web/         # WASM runner (HTML template, canvas integration, print stylesheets)
├── Distillined Key.csv # Source questionnaire items and factor weights
└── Optimized_Keys.csv  # Sequence optimization order
```

---

## 4. Implementation Steps

- [x] Create planning documentation (`PLAN.md`).
- [x] Implement embedded dataset loader & CSV parser in `crates/shared`.
- [x] Implement questionnaire state machine & scoring engine with $SE$ computation in `crates/shared`.
- [x] Implement CSV and JSON export generators in `crates/shared`.
- [x] Implement `egui` frontend in `crates/app`:
  - Top header (Progress, completion %, Show Results toggle, Reset, Theme).
  - Single question card with keyboard shortcuts (`1`–`5`, Arrows, Wheel scroll skip).
  - Side results panel with 5-tier badges and detailed $SE$ metrics.
  - Export dialogs.
- [x] Update `crates/desktop` and `crates/web` runners and `index.html`.
- [x] Replace obsolete task tests with psychometric, queuing, and persistence tests.
- [x] Validate with `cargo test --workspace` and `cargo clippy --workspace --all-targets`.
