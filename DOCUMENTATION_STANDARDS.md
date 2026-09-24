# Revisited IPIP-NEO Documentation Standards

**Psychometric Modeling, Taxonomic Graph Analysis (TGA), and Quality Enforcement**

---

## 1. Overview & Core Philosophy

Revisited IPIP-NEO is a scientifically robust, client-side psychometric assessment engine implementing Taxonomic Graph Analysis (TGA). All psychometric models, scoring functions, norming curves, and graphing components must be documented with academic rigor.

### The Five Pillars:
1. **Psychometric Grounding**: Every item, trait, facet, and meta-factor must cite its scientific foundations (Costa & McCrae, Goldberg, DeYoung, TGA papers) with exact scoring weights.
2. **Privacy by Design**: Client-side zero-server telemetry documentation; all state resides in browser local storage or IndexedDB.
3. **Mathematical Precision**: Use standard KaTeX notation for scoring functions, z-score normalization, percentile ranking, and graph distance metrics.
4. **Zero-Warning Hygiene**: `cargo doc --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` must compile with zero warnings.
5. **Strict Test Isolation**: All unit and integration tests must reside in dedicated test files under `tests/`; no inline tests inside production source files.

---

## 2. KaTeX Mathematical Notation

- **Z-Score Normalization**:
  $$z_i = \frac{X_i - \mu_i}{\sigma_i}$$
- **Percentile Conversion**:
  $$\text{Percentile} = \Phi(z_i) \times 100$$
- **TGA Graph Weights**: Trait graph node weights and edge distances.

---

## 3. Verification Checklist

Before opening PRs to `main`:
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] `cargo test --workspace` passes 100% of integration tests.
- [ ] No `DOCUMENTATION_STANDARDS.md` or internal roadmap files are included on `main`.
