import logging
import sys
import time
from typing import List
import numpy as np
import polars as pl

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("ItemOptimizer")


def _compute_se_vector(s_sq: np.ndarray, w_abs: np.ndarray, penalty: float = 1.5) -> np.ndarray:
    mask = w_abs == 0
    se = np.zeros_like(s_sq, dtype=np.float64)
    se[mask] = penalty
    se[~mask] = np.sqrt(s_sq[~mask]) / w_abs[~mask]
    return se


def export_optimized_keys(
    csv_path: str = "Distillined Key.csv",
    output_path: str = "Optimized_Keys.csv",
    key_column: str = "Label",
    alpha: float = 1.0,      # Weight for relative SE reduction
    gamma: float = 5.0,      # Sharpness for soft-max worst-case construct penalty
    epsilon: float = 1e-9,   # Scale for deterministic tie-breaker
) -> None:
    t_start = time.perf_counter()
    logger.info(f"Ingesting source file: '{csv_path}'")

    df = pl.read_csv(csv_path)

    labels = df[key_column].to_numpy()
    N = len(labels)

    unique_f, f_idx = np.unique(df["Facet"].to_numpy(), return_inverse=True)
    unique_t, t_idx = np.unique(df["Trait"].to_numpy(), return_inverse=True)
    unique_m, m_idx = np.unique(df["Meta-Trait"].to_numpy(), return_inverse=True)

    N_f, N_t, N_m = len(unique_f), len(unique_t), len(unique_m)

    wf_arr = df["Facet Weight"].cast(pl.Float64).to_numpy()
    wt_arr = df["Trait Weight"].cast(pl.Float64).to_numpy()
    wm_arr = df["Meta-Trait Weight"].cast(pl.Float64).to_numpy()

    wf_sq, wf_abs = wf_arr**2, np.abs(wf_arr)
    wt_sq, wt_abs = wt_arr**2, np.abs(wt_arr)
    wm_sq, wm_abs = wm_arr**2, np.abs(wm_arr)

    se_min_f = _compute_se_vector(np.bincount(f_idx, weights=wf_sq), np.bincount(f_idx, weights=wf_abs))
    se_min_t = _compute_se_vector(np.bincount(t_idx, weights=wt_sq), np.bincount(t_idx, weights=wt_abs))
    se_min_m = _compute_se_vector(np.bincount(m_idx, weights=wm_sq), np.bincount(m_idx, weights=wm_abs))

    # Pre-compute item-level micro tie-breaker vector
    item_tie_breaker = epsilon * (wf_abs + wt_abs + wm_abs) / (np.arange(N, dtype=np.float64) + 1.0)

    # State Initialization
    f_sq, f_abs = np.zeros(N_f, dtype=np.float64), np.zeros(N_f, dtype=np.float64)
    t_sq, t_abs = np.zeros(N_t, dtype=np.float64), np.zeros(N_t, dtype=np.float64)
    m_sq, m_abs = np.zeros(N_m, dtype=np.float64), np.zeros(N_m, dtype=np.float64)

    curr_se_f = np.full(N_f, 1.5, dtype=np.float64)
    curr_se_t = np.full(N_t, 1.5, dtype=np.float64)
    curr_se_m = np.full(N_m, 1.5, dtype=np.float64)

    curr_norm_f = curr_se_f / se_min_f
    curr_norm_t = curr_se_t / se_min_t
    curr_norm_m = curr_se_m / se_min_m

    rem_mask = np.ones(N, dtype=bool)
    optimized_path: List[int] = []

    logger.info("Executing sequence optimization with composite loss...")

    for step in range(N):
        rem = np.where(rem_mask)[0]
        fi, ti, mi = f_idx[rem], t_idx[rem], m_idx[rem]

        # 1. Compute new unnormalized SEs
        new_se_f = np.sqrt(f_sq[fi] + wf_sq[rem]) / (f_abs[fi] + wf_abs[rem])
        new_se_t = np.sqrt(t_sq[ti] + wt_sq[rem]) / (t_abs[ti] + wt_abs[rem])
        new_se_m = np.sqrt(m_sq[mi] + wm_sq[rem]) / (m_abs[mi] + wm_abs[rem])

        # 2. Relative SE reduction terms: (SE_old - SE_new) / SE_old
        rel_red_f = (curr_se_f[fi] - new_se_f) / curr_se_f[fi]
        rel_red_t = (curr_se_t[ti] - new_se_t) / curr_se_t[ti]
        rel_red_m = (curr_se_m[mi] - new_se_m) / curr_se_m[mi]
        total_rel_reduction = rel_red_f + rel_red_t + rel_red_m

        # 3. Updated Normalized SEs
        new_norm_f = new_se_f / se_min_f[fi]
        new_norm_t = new_se_t / se_min_t[ti]
        new_norm_m = new_se_m / se_min_m[mi]

        # 4. Soft-Max Imbalance Penalty across lagging constructs
        # Vectorized Log-Sum-Exp approximation for peak norm error across constructs
        lse_f = np.exp(gamma * new_norm_f)
        lse_t = np.exp(gamma * new_norm_t)
        lse_m = np.exp(gamma * new_norm_m)
        imbalance_penalty = (1.0 / gamma) * np.log(lse_f + lse_t + lse_m)

        # 5. Composite Cost Objective
        costs = imbalance_penalty - (alpha * total_rel_reduction) - item_tie_breaker[rem]

        # Select strict minimum
        best_local_idx = np.argmin(costs)
        cand = rem[best_local_idx]

        optimized_path.append(cand)
        rem_mask[cand] = False

        # State updates
        bfi, bti, bmi = f_idx[cand], t_idx[cand], m_idx[cand]
        f_sq[bfi] += wf_sq[cand]
        f_abs[bfi] += wf_abs[cand]
        t_sq[bti] += wt_sq[cand]
        t_abs[bti] += wt_abs[cand]
        m_sq[bmi] += wm_sq[cand]
        m_abs[bmi] += wm_abs[cand]

        curr_se_f[bfi] = new_se_f[best_local_idx]
        curr_se_t[bti] = new_se_t[best_local_idx]
        curr_se_m[bmi] = new_se_m[best_local_idx]

        curr_norm_f[bfi] = new_norm_f[best_local_idx]
        curr_norm_t[bti] = new_norm_t[best_local_idx]
        curr_norm_m[bmi] = new_norm_m[best_local_idx]

    col_names = [f"q_{i+1}" for i in range(N)]
    out_row = [labels[idx] for idx in optimized_path]
    out_df = pl.DataFrame([out_row], schema=col_names, orient="row")
    out_df.write_csv(output_path)

    t_elapsed = time.perf_counter() - t_start
    logger.info(f"Optimization completed deterministically in {t_elapsed:.4f}s. Saved to '{output_path}'.")


if __name__ == "__main__":
    export_optimized_keys()
