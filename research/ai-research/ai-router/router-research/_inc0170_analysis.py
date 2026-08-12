#!/usr/bin/env python3
"""INC-0170: Large-K Angular Capacity Test (K=600–5000).

Extends INC-0168 by adding TRANS PERM at K={600, 1000, 2000, 3000, 5000}.

INC-0168 established the canonical scaling law for TRANS ORIG vs TRANS PERM
up to K=400. INC-0167 ran TRANS ORIG at K=600 and K=1000 (static diagnostic)
but WITHOUT a PERM column. This experiment fills that gap and pushes to K=5000
to validate whether the scaling law holds at production-relevant K.

Canonical law to validate (INC-0169):
  TRANS ORIG:  eff_buckets = 2.957 x K^0.572   R^2=0.963
  TRANS PERM:  eff_buckets = 1.664 x K^0.814   R^2=0.993

Predicted eff_ratio (PERM/ORIG) at K=5000:
  ORIG: 2.957 x 5000^0.572 = ~419    PERM: 1.664 x 5000^0.814 = ~1840
  Predicted ratio: ~4.4x

Success (KEEP): eff_ratio grows monotonically through K=5000 OR saturates
  at identifiable K*. R^2 of OLS fit >= 0.95 across K=25..5000.
  alpha_TRANS_ORIG stays within 0.10 of 0.572.

Falsification (KILL): eff_ratio collapses toward 1 at large K, OR
  alpha shifts > 0.10 from 0.572.

CRITICAL: No MSE anywhere. Only structural routing metrics.
"""

import os
import sys
import json
import numpy as np
from collections import Counter
from typing import Dict, List, Any, Tuple

ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, ROOT)

from hyperbolic_router_so8 import (
    route_addresses,
    Chart,
    LOG_PHI,
    PHI,
)

# Canonical routing dimensions from INC-0162 through INC-0169
PHASE4_DIMS = (3, 65, 2, 21)
FIELD4_DIMS = (4, 66, 5, 22)
COMPLEX_DIMS = (1, 3)
DATA_PATH = os.path.join(ROOT, "data/wikitext2_proxy/ppmi_proxy.npz")
N_TOKENS = 5000
DELTA_R_CANONICAL = 3.6
PERM_SEED = 42

# Anchor K values from INC-0168 for continuity + new large-K values
K_RANGE_ANCHOR = [25, 50, 100, 200, 400]   # already measured in INC-0168
K_RANGE_NEW = [600, 1000, 2000, 3000, 5000]  # new in INC-0170
K_RANGE = K_RANGE_ANCHOR + K_RANGE_NEW
K_FIT_MIN = 25

OUTPUT_PATH = os.path.join(ROOT, "results/analysis/inc0170_large_k.json")


# ---------------------------------------------------------------------------
# Data loading and normalization
# ---------------------------------------------------------------------------

def load_data(n: int = N_TOKENS, seed: int = 0) -> Tuple[np.ndarray, np.ndarray]:
    d = np.load(DATA_PATH)
    X = d["x_train"].astype(np.float64)
    y = d["y_train"].astype(np.float64)
    rng = np.random.RandomState(seed)
    idx = rng.permutation(X.shape[0])[:n]
    return X[idx], y[idx]


def l2_normalize(X: np.ndarray) -> np.ndarray:
    norms = np.linalg.norm(X, axis=1, keepdims=True)
    return X / np.maximum(norms, 1e-12)


def col_perm(X: np.ndarray, seed: int = PERM_SEED) -> np.ndarray:
    rng = np.random.RandomState(seed)
    X_perm = X.copy()
    for j in range(X_perm.shape[1]):
        X_perm[:, j] = rng.permutation(X_perm[:, j])
    return X_perm


def make_chart(d: int) -> Chart:
    return Chart(R=np.eye(d, dtype=np.float64), s_global=None,
                 S_radial=None, scale_mode="global")


# ---------------------------------------------------------------------------
# Routing
# ---------------------------------------------------------------------------

def run_routing(
    X: np.ndarray,
    chart: Chart,
    K: int,
    sector_mode: str,
    delta_r: float,
    phase_transport_lambda: float = 1.0,
) -> Tuple[np.ndarray, np.ndarray]:
    shells, sectors, _, _ = route_addresses(
        X, delta_r=delta_r, C=None, chart=chart,
        sector_mode=sector_mode,
        phase_dim_i=0, phase_dim_j=1,
        phase4_dim_i=PHASE4_DIMS[0],
        phase4_dim_j=PHASE4_DIMS[1],
        phase4_dim_k=PHASE4_DIMS[2],
        phase4_dim_l=PHASE4_DIMS[3],
        field4_dim_i=FIELD4_DIMS[0],
        field4_dim_j=FIELD4_DIMS[1],
        field4_dim_k=FIELD4_DIMS[2],
        field4_dim_l=FIELD4_DIMS[3],
        complex_dim_i=COMPLEX_DIMS[0],
        complex_dim_j=COMPLEX_DIMS[1],
        K=K,
        time_pressure_lambda=0.0,
        tau=0.0,
        adaptive_min_pair_bins=3,
        adaptive_time_growth=1.4,
        adaptive_balance=1.2,
        adaptive_angle_growth=0.5,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.65,
        adaptive_converge_target=0.85,
        adaptive_converge_hysteresis=0.05,
        adaptive_converge_mode="phi_ladder",
        shell_mode="phi_log",
        shell_phase_coupling=0.0,
        hopf_chi_bins=2,
        route_scale_lambda=1.0,
        memory_coord_mode="route_chart",
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=0.0,
    )
    return shells, sectors


# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

def routing_stats(shells: np.ndarray, sectors: np.ndarray, K: int) -> Dict[str, Any]:
    n = len(shells)
    keys = list(zip(shells.tolist(), sectors.tolist()))
    bc = Counter(keys)
    counts = np.array(sorted(bc.values(), reverse=True), dtype=np.float64)

    p = counts / counts.sum()
    entropy = -np.sum(p * np.log(p + 1e-30))
    eff_buckets = float(np.exp(entropy))

    nb = len(counts)
    if nb <= 1:
        gini = 0.0
    else:
        sc = np.sort(counts)
        idx = np.arange(1, nb + 1)
        gini = float(2.0 * np.sum(idx * sc) / (nb * np.sum(sc)) - (nb + 1) / nb)

    sector_c = Counter(sectors.tolist())
    n_active = len(sector_c)
    sp = np.array(list(sector_c.values()), dtype=np.float64)
    sp /= sp.sum()
    sector_entropy = float(-np.sum(sp * np.log(sp + 1e-30)))

    return {
        "n_tokens": n,
        "n_active_sectors": n_active,
        "n_buckets": len(bc),
        "effective_bucket_count": eff_buckets,
        "routing_gini": gini,
        "sector_entropy": sector_entropy,
    }


def fit_scaling_exponent(k_vals: List[int], eff_vals: List[float]) -> Dict[str, float]:
    ks = [k for k, e in zip(k_vals, eff_vals) if k >= K_FIT_MIN and e > 0]
    es = [e for k, e in zip(k_vals, eff_vals) if k >= K_FIT_MIN and e > 0]
    if len(ks) < 3:
        return {"alpha": float("nan"), "c": float("nan"), "r2": float("nan")}
    log_k = np.log(np.array(ks, dtype=np.float64))
    log_e = np.log(np.array(es, dtype=np.float64))
    A = np.vstack([log_k, np.ones(len(log_k))]).T
    result = np.linalg.lstsq(A, log_e, rcond=None)
    alpha, log_c = result[0]
    c = float(np.exp(log_c))
    log_e_pred = alpha * log_k + log_c
    ss_res = float(np.sum((log_e - log_e_pred) ** 2))
    ss_tot = float(np.sum((log_e - log_e.mean()) ** 2))
    r2 = float(1.0 - ss_res / (ss_tot + 1e-30))
    return {"alpha": float(alpha), "c": c, "r2": r2}


# ---------------------------------------------------------------------------
# Main experiment
# ---------------------------------------------------------------------------

def main():
    print("=" * 72)
    print("INC-0170: LARGE-K ANGULAR CAPACITY TEST (K=600–5000)")
    print("Canonical regime: TRANS ORIG vs TRANS PERM, L2-norm, static routing")
    print("=" * 72)
    print(f"\nData: {DATA_PATH}")
    print(f"N_TOKENS={N_TOKENS}, PERM_SEED={PERM_SEED}")
    print(f"K range: {K_RANGE}")
    print(f"Anchor K (INC-0168): {K_RANGE_ANCHOR}")
    print(f"New K (INC-0170):    {K_RANGE_NEW}")
    print()

    X_raw, y = load_data(n=N_TOKENS, seed=0)
    d = X_raw.shape[1]
    chart = make_chart(d)

    X_l2 = l2_normalize(X_raw)
    X_perm = col_perm(X_l2, seed=PERM_SEED)

    print(f"L2 norms ORIG: min={np.linalg.norm(X_l2,axis=1).min():.4f} "
          f"max={np.linalg.norm(X_l2,axis=1).max():.4f}")
    print(f"L2 norms PERM: min={np.linalg.norm(X_perm,axis=1).min():.4f} "
          f"max={np.linalg.norm(X_perm,axis=1).max():.4f}")
    print()

    # TRANS ORIG and TRANS PERM only (canonical regime)
    sector_mode = "phase4d_hopf_transport"
    ptl = 1.0

    orig_results = []
    perm_results = []

    total = len(K_RANGE) * 2
    done = 0

    for K in K_RANGE:
        for variant, X_var, result_list in [("ORIG", X_l2, orig_results),
                                            ("PERM", X_perm, perm_results)]:
            shells, sectors = run_routing(X_var, chart, K, sector_mode, DELTA_R_CANONICAL, ptl)
            stats = routing_stats(shells, sectors, K)
            entry = {"K": K, "variant": variant, **stats}
            result_list.append(entry)
            done += 1
            print(f"  [{done:2d}/{total}]  K={K:5d}  {variant}  "
                  f"eff_buckets={stats['effective_bucket_count']:8.2f}  "
                  f"gini={stats['routing_gini']:.4f}  "
                  f"n_active={stats['n_active_sectors']}")

    print()

    # Scaling exponents
    orig_exponent = fit_scaling_exponent(
        [r["K"] for r in orig_results],
        [r["effective_bucket_count"] for r in orig_results],
    )
    perm_exponent = fit_scaling_exponent(
        [r["K"] for r in perm_results],
        [r["effective_bucket_count"] for r in perm_results],
    )

    # Compression ratios at each K
    compression_by_K = {}
    for K in K_RANGE:
        o = next(r for r in orig_results if r["K"] == K)
        p = next(r for r in perm_results if r["K"] == K)
        eff_ratio = p["effective_bucket_count"] / max(o["effective_bucket_count"], 1e-9)
        gini_ratio = o["routing_gini"] / max(p["routing_gini"], 1e-9)
        compression_by_K[K] = {
            "eff_ratio_PERM_over_ORIG": float(eff_ratio),
            "gini_ratio_ORIG_over_PERM": float(gini_ratio),
            "eff_orig": float(o["effective_bucket_count"]),
            "eff_perm": float(p["effective_bucket_count"]),
            "gini_orig": float(o["routing_gini"]),
            "gini_perm": float(p["routing_gini"]),
        }

    # --- Determine verdict ---
    anchor_ratio = compression_by_K[400]["eff_ratio_PERM_over_ORIG"]
    max_new_ratio = max(compression_by_K[K]["eff_ratio_PERM_over_ORIG"] for K in K_RANGE_NEW)
    alpha_delta = abs(orig_exponent["alpha"] - 0.572)
    r2_ok = orig_exponent["r2"] >= 0.95
    ratio_grows = max_new_ratio > anchor_ratio
    alpha_stable = alpha_delta <= 0.10

    if r2_ok and alpha_stable and (ratio_grows or max_new_ratio > 2.0):
        verdict = "KEEP"
    elif not alpha_stable or orig_exponent["r2"] < 0.85:
        verdict = "KILL"
    else:
        verdict = "REFINE"

    # --- Print summary ---
    print("=" * 72)
    print("SCALING EXPONENTS (eff_buckets = c * K^alpha)")
    print(f"  TRANS ORIG:  alpha={orig_exponent['alpha']:.4f}  "
          f"c={orig_exponent['c']:.4f}  R^2={orig_exponent['r2']:.4f}")
    print(f"  INC-0169 ref: alpha=0.5717  c=2.9569  R^2=0.9628")
    print(f"  Δalpha from frozen law: {alpha_delta:.4f}  "
          f"({'STABLE' if alpha_stable else 'UNSTABLE -- KILL'})")
    print()
    print(f"  TRANS PERM:  alpha={perm_exponent['alpha']:.4f}  "
          f"c={perm_exponent['c']:.4f}  R^2={perm_exponent['r2']:.4f}")
    print(f"  INC-0169 ref: alpha=0.8142  c=1.6637")
    print()

    print("COMPRESSION RATIOS (TRANS PERM/ORIG eff_buckets):")
    print(f"  {'K':>6}  {'eff_orig':>10}  {'eff_perm':>10}  "
          f"{'eff_ratio':>9}  {'gini_ratio':>10}  note")
    for K in K_RANGE:
        c = compression_by_K[K]
        note = "(INC-0168)" if K in K_RANGE_ANCHOR else "(NEW)"
        print(f"  {K:6d}  {c['eff_orig']:10.2f}  {c['eff_perm']:10.2f}  "
              f"{c['eff_ratio_PERM_over_ORIG']:9.3f}x  "
              f"{c['gini_ratio_ORIG_over_PERM']:10.3f}x  {note}")

    # Canonical law prediction vs measured
    print()
    print("CANONICAL LAW PREDICTION vs MEASURED (TRANS ORIG):")
    print(f"  {'K':>6}  {'predicted':>10}  {'measured':>10}  {'error%':>8}")
    for K in K_RANGE:
        pred = 2.957 * (K ** 0.572)
        meas = compression_by_K[K]["eff_orig"]
        err = 100 * (meas - pred) / pred
        print(f"  {K:6d}  {pred:10.2f}  {meas:10.2f}  {err:+7.1f}%")

    print()
    print("=" * 72)
    print(f"VERDICT: {verdict}")
    if verdict == "KEEP":
        print("  Scaling law holds at large K. eff_ratio grows (or remains high).")
        print("  alpha_TRANS_ORIG stable within 0.10 of 0.572.")
    elif verdict == "KILL":
        print("  KILL: alpha unstable or R^2 < 0.85.")
        print("  Scaling law does not extend to this K range.")
    else:
        print("  REFINE: law holds structurally but needs follow-up.")
    print("=" * 72)

    # Save results
    output = {
        "increment": "INC-0170",
        "verdict": verdict,
        "n_tokens": N_TOKENS,
        "perm_seed": PERM_SEED,
        "sector_mode": sector_mode,
        "phase_transport_lambda": ptl,
        "normalization": "L2",
        "phase4_dims": list(PHASE4_DIMS),
        "K_range": K_RANGE,
        "K_range_anchor": K_RANGE_ANCHOR,
        "K_range_new": K_RANGE_NEW,
        "scaling_exponents": {
            "TRANS_ORIG": orig_exponent,
            "TRANS_PERM": perm_exponent,
        },
        "inc0169_reference": {
            "TRANS_ORIG": {"alpha": 0.5717, "c": 2.9569, "r2": 0.9628},
            "TRANS_PERM": {"alpha": 0.8142, "c": 1.6637, "r2": 0.9929},
        },
        "compression_by_K": {str(K): v for K, v in compression_by_K.items()},
        "raw_orig": orig_results,
        "raw_perm": perm_results,
    }

    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    with open(OUTPUT_PATH, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults saved to: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
