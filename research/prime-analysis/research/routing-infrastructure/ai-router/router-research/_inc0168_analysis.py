#!/usr/bin/env python3
"""INC-0168: Norm-Geometry Diagnostic — Angular vs Radial Routing.

Tests three routing geometry configurations to determine whether the
previously-observed sqrt(K) routing scaling is:
  (A) Purely angular (a consequence of hyperspherical unit-norm routing)
  (B) Radial + angular (would change with radial degrees of freedom)
  (C) Norm-dependent (changes as unit-surface shape changes under Lp norms)

Experiment A (L2-baseline):
  Standard data: x -> x / ||x||_2. All tokens on unit S^{99} sphere.
  r_eff = 1.0 for all tokens. Routing purely angular.

Experiment B (radial-aware):
  x -> x / ||x||_1. Tokens NOT on unit L2 sphere.
  L2 norms vary (r_eff in [0.22..0.46]). Tests whether radial shell
  structure activates and whether it changes routing metrics.
  Also tests with delta_r adjusted to force shell activation.

Experiment C (alternative norm geometry):
  x -> x / ||x||_3  and  x -> x / ||x||_4.
  Changes unit-surface shape while preserving some radial structure.
  L2 norms: L3 in [1.03..1.29], L4 in [1.03..1.41].

CRITICAL: No MSE anywhere. Only structural metrics.
Valid metrics: effective_bucket_count, routing_gini, bucket_purity,
               sector_entropy, sector_gini, shell_activation.
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
    safe_norm,
    LOG_PHI,
    PHI,
)

# Canonical routing dimensions from prior INC (INC-0162 through INC-0167)
PHASE4_DIMS = (3, 65, 2, 21)
FIELD4_DIMS = (4, 66, 5, 22)
COMPLEX_DIMS = (1, 3)
DATA_PATH = os.path.join(ROOT, "data/wikitext2_proxy/ppmi_proxy.npz")
N_TOKENS = 5000
DELTA_R_CANONICAL = 3.6
K_RANGE = [10, 25, 50, 75, 100, 150, 200, 400]
K_FIT_MIN = 25  # K values used for scaling-exponent fit


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_data(n: int = N_TOKENS, seed: int = 0) -> Tuple[np.ndarray, np.ndarray]:
    """Load and subset PPMI-SVD proxy data. Returns (X_train, y_train)."""
    d = np.load(DATA_PATH)
    X = d["x_train"].astype(np.float64)
    y = d["y_train"].astype(np.float64)
    rng = np.random.RandomState(seed)
    idx = rng.permutation(X.shape[0])[:n]
    return X[idx], y[idx]


# ---------------------------------------------------------------------------
# Normalization variants
# ---------------------------------------------------------------------------

def lp_normalize(X: np.ndarray, p: int) -> np.ndarray:
    """Normalize rows of X to unit Lp norm."""
    norms = np.linalg.norm(X, ord=p, axis=1, keepdims=True)
    return X / np.maximum(norms, 1e-12)


def col_perm(X: np.ndarray, seed: int = 42) -> np.ndarray:
    """Column-permute X to produce a structurally destroyed control."""
    rng = np.random.RandomState(seed)
    X_perm = X.copy()
    for j in range(X.shape[1]):
        X_perm[:, j] = rng.permutation(X_perm[:, j])
    return X_perm


# ---------------------------------------------------------------------------
# Routing
# ---------------------------------------------------------------------------

def make_chart(d: int) -> Chart:
    """Identity chart — no learned rotation or scaling."""
    return Chart(R=np.eye(d, dtype=np.float64), s_global=None,
                 S_radial=None, scale_mode="global")


def run_routing(
    X: np.ndarray,
    chart: Chart,
    K: int,
    sector_mode: str,
    delta_r: float,
    phase_transport_lambda: float = 1.0,
) -> Tuple[np.ndarray, np.ndarray]:
    """Static routing: return (shells, sectors) for X."""
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

def routing_stats(
    shells: np.ndarray,
    sectors: np.ndarray,
    y: np.ndarray,
    K: int,
) -> Dict[str, Any]:
    """
    Compute structural routing metrics.
    No MSE. No cosine similarity as metric.
    Returns: eff_buckets, gini, bucket_purity, sector_entropy, sector_gini,
             n_shells, shell_distribution, n_active_sectors, sectors_above_1pct.
    """
    n = len(shells)
    # --- bucket counts ---
    keys = list(zip(shells.tolist(), sectors.tolist()))
    bc = Counter(keys)
    counts = np.array(sorted(bc.values(), reverse=True), dtype=np.float64)

    # Effective bucket count (perplexity of bucket distribution)
    p = counts / counts.sum()
    entropy = -np.sum(p * np.log(p + 1e-30))
    eff_buckets = float(np.exp(entropy))

    # Routing Gini
    nb = len(counts)
    if nb <= 1:
        gini = 0.0
    else:
        sc = np.sort(counts)
        idx = np.arange(1, nb + 1)
        gini = float(2.0 * np.sum(idx * sc) / (nb * np.sum(sc)) - (nb + 1) / nb)

    # Top-half concentration
    half = max(1, nb // 2)
    top_half = float(np.sum(counts[:half]) / np.sum(counts))

    # Bucket purity (using y labels; y is continuous proxy — discretise to bins)
    # Use 10 bins for the label to define "class"
    if y is not None and len(y) > 0:
        y1d = y[:, 0] if y.ndim > 1 else y
        n_bins = min(10, len(np.unique(y1d)))
        percentiles = np.linspace(0, 100, n_bins + 1)
        bin_edges = np.percentile(y1d, percentiles)
        bin_edges[-1] += 1e-9
        y_class = np.digitize(y1d, bin_edges[1:-1])  # 0-based bucket index
        # purity: fraction of tokens in dominant class per bucket
        per_bucket_purity = []
        for bk in bc.keys():
            mask = np.array([keys[i] == bk for i in range(n)])
            if not any(mask):
                continue
            labels_in_bucket = y_class[mask]
            max_count = Counter(labels_in_bucket.tolist()).most_common(1)[0][1]
            per_bucket_purity.append(max_count / len(labels_in_bucket))
        bucket_purity = float(np.mean(per_bucket_purity)) if per_bucket_purity else 0.0
    else:
        bucket_purity = float("nan")

    # Shell distribution
    shell_c = Counter(shells.tolist())
    shell_dist = {int(k): v for k, v in sorted(shell_c.items())}

    # Sector stats
    sector_c = Counter(sectors.tolist())
    n_active_sectors = len(sector_c)
    threshold_1pct = 0.01 * n
    sectors_above_1pct = sum(1 for c in sector_c.values() if c > threshold_1pct)
    sp = np.array(list(sector_c.values()), dtype=np.float64)
    sp /= sp.sum()
    sector_entropy = float(-np.sum(sp * np.log(sp + 1e-30)))
    ns = len(sp)
    sc2 = np.sort(np.array(list(sector_c.values()), dtype=np.float64))
    if ns <= 1:
        sector_gini = 0.0
    else:
        idx2 = np.arange(1, ns + 1)
        sector_gini = float(2.0 * np.sum(idx2 * sc2) / (ns * np.sum(sc2)) - (ns + 1) / ns)

    return {
        "n_tokens": n,
        "n_shells": len(shell_dist),
        "shell_distribution": shell_dist,
        "n_active_sectors": n_active_sectors,
        "sectors_above_1pct": sectors_above_1pct,
        "n_buckets": len(bc),
        "effective_bucket_count": eff_buckets,
        "routing_gini": gini,
        "top_half_concentration": top_half,
        "bucket_purity": bucket_purity,
        "sector_entropy": sector_entropy,
        "sector_gini": sector_gini,
    }


def r_eff_stats(X: np.ndarray, chart: Chart) -> Dict[str, float]:
    """Report the L2-norm distribution of tokens entering the router."""
    from hyperbolic_router_so8 import apply_chart
    z = apply_chart(X, chart)
    r = safe_norm(z, axis=1, keepdims=False)
    return {
        "min": float(np.min(r)),
        "max": float(np.max(r)),
        "mean": float(np.mean(r)),
        "std": float(np.std(r)),
        "p05": float(np.percentile(r, 5)),
        "p25": float(np.percentile(r, 25)),
        "p50": float(np.percentile(r, 50)),
        "p75": float(np.percentile(r, 75)),
        "p95": float(np.percentile(r, 95)),
        "shell1_threshold": float(DELTA_R_CANONICAL * (PHI - 1)),
        "pct_above_shell1": float(np.mean(r >= DELTA_R_CANONICAL * (PHI - 1)) * 100),
    }


def fit_scaling_exponent(K_vals: List[int], metric_vals: List[float]) -> Dict[str, float]:
    """Fit y = c * K^alpha in log-log space (OLS). Returns alpha, c, r2."""
    k = np.array([float(x) for x in K_vals])
    m = np.array([float(x) for x in metric_vals])
    if len(k) < 3 or np.any(m <= 0):
        return {"alpha": float("nan"), "c": float("nan"), "r2": float("nan")}
    log_k = np.log(k)
    log_m = np.log(np.maximum(m, 1e-9))
    A = np.vstack([log_k, np.ones(len(log_k))]).T
    alpha, log_c = np.linalg.lstsq(A, log_m, rcond=None)[0]
    c = float(np.exp(log_c))
    # R^2
    log_m_pred = alpha * log_k + log_c
    ss_res = np.sum((log_m - log_m_pred) ** 2)
    ss_tot = np.sum((log_m - log_m.mean()) ** 2)
    r2 = float(1.0 - ss_res / (ss_tot + 1e-30))
    return {"alpha": float(alpha), "c": c, "r2": r2}


# ---------------------------------------------------------------------------
# Per-experiment sweep
# ---------------------------------------------------------------------------

def run_experiment(
    label: str,
    X_orig: np.ndarray,
    y: np.ndarray,
    chart: Chart,
    K_range: List[int],
    delta_r: float,
    modes: List[Tuple[str, str, float]],  # (name, sector_mode, ptl)
    perm_seed: int = 42,
) -> Dict[str, Any]:
    """
    Run routing across K_range for one normalization variant.
    Returns nested results keyed by K and mode.
    """
    X_perm = col_perm(X_orig, seed=perm_seed)
    r_stats = r_eff_stats(X_orig, chart)

    results = []
    for K in K_range:
        for mode_name, sector_mode, ptl in modes:
            for variant_name, X_var, y_var in [("ORIG", X_orig, y),
                                               ("PERM", X_perm, y)]:
                shells, sectors = run_routing(X_var, chart, K, sector_mode, delta_r, ptl)
                stats = routing_stats(shells, sectors, y_var, K)
                entry = {
                    "K": K,
                    "mode": mode_name,
                    "variant": variant_name,
                    "delta_r": delta_r,
                    **stats,
                }
                results.append(entry)

    # Fit scaling exponents per mode×variant
    scaling = {}
    for mode_name, _, _ in modes:
        for variant in ["ORIG", "PERM"]:
            key = f"{mode_name}_{variant}"
            subset = [e for e in results
                      if e["mode"] == mode_name and e["variant"] == variant
                      and e["K"] >= K_FIT_MIN]
            k_vals = [e["K"] for e in subset]
            eff_vals = [e["effective_bucket_count"] for e in subset]
            scaling[key] = fit_scaling_exponent(k_vals, eff_vals)

    # Compression ratios at each K (PERM/ORIG for eff_buckets, ORIG/PERM for Gini)
    compression = {}
    for mode_name, _, _ in modes:
        compression[mode_name] = {}
        for K in K_range:
            orig_e = next((e for e in results if e["K"] == K and e["mode"] == mode_name
                          and e["variant"] == "ORIG"), None)
            perm_e = next((e for e in results if e["K"] == K and e["mode"] == mode_name
                          and e["variant"] == "PERM"), None)
            if orig_e and perm_e:
                eff_ratio = (perm_e["effective_bucket_count"]
                             / max(orig_e["effective_bucket_count"], 1e-9))
                gini_ratio = (orig_e["routing_gini"]
                              / max(perm_e["routing_gini"], 1e-9))
                compression[mode_name][K] = {
                    "eff_bucket_ratio_PERM_over_ORIG": float(eff_ratio),
                    "gini_ratio_ORIG_over_PERM": float(gini_ratio),
                    "eff_orig": float(orig_e["effective_bucket_count"]),
                    "eff_perm": float(perm_e["effective_bucket_count"]),
                    "gini_orig": float(orig_e["routing_gini"]),
                    "gini_perm": float(perm_e["routing_gini"]),
                    "purity_orig": float(orig_e["bucket_purity"]),
                    "purity_perm": float(perm_e["bucket_purity"]),
                }

    return {
        "label": label,
        "delta_r": delta_r,
        "r_eff_stats": r_stats,
        "routing_results": results,
        "scaling_exponents": scaling,
        "compression_by_K": compression,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 72)
    print("INC-0168: NORM-GEOMETRY DIAGNOSTIC — ANGULAR vs RADIAL ROUTING")
    print("=" * 72)

    # Load data
    print(f"\nLoading data: {DATA_PATH}")
    X_raw, y = load_data(n=N_TOKENS, seed=0)
    d = X_raw.shape[1]
    print(f"  X shape: {X_raw.shape}, y shape: {y.shape}")
    print(f"  Raw L2-norm range: {np.linalg.norm(X_raw,axis=1).min():.4f}"
          f"..{np.linalg.norm(X_raw,axis=1).max():.4f}")

    # Identity chart (no learned rotation: consistent across experiments)
    chart = make_chart(d)

    # Routing modes to test in each experiment
    modes = [
        ("BASE", "phase4d_hopf_base",      0.0),
        ("TRANS", "phase4d_hopf_transport", 1.0),
    ]

    # Shell 1 threshold for reference
    shell1_thresh = DELTA_R_CANONICAL * (PHI - 1)
    print(f"\n  Shell-1 activation threshold at delta_r={DELTA_R_CANONICAL}: "
          f"r_eff >= {shell1_thresh:.4f}")
    print(f"  LOG_PHI = {LOG_PHI:.6f}, PHI = {PHI:.6f}")

    all_results = {}

    # ================================================================
    # EXPERIMENT A — L2 baseline
    # ================================================================
    print("\n" + "=" * 72)
    print("EXPERIMENT A — L2-Normalized Baseline")
    print("  x -> x / ||x||_2   (all r_eff = 1.0, purely angular)")
    print("=" * 72)

    X_l2 = lp_normalize(X_raw, p=2)
    r_l2 = np.linalg.norm(X_l2, axis=1)
    print(f"  L2 norms: min={r_l2.min():.4f} max={r_l2.max():.4f} std={r_l2.std():.6f}")

    exp_a = run_experiment(
        label="ExpA_L2_baseline",
        X_orig=X_l2,
        y=y,
        chart=chart,
        K_range=K_RANGE,
        delta_r=DELTA_R_CANONICAL,
        modes=modes,
    )
    all_results["ExpA_L2"] = exp_a

    print(f"\n  r_eff: min={exp_a['r_eff_stats']['min']:.4f} "
          f"max={exp_a['r_eff_stats']['max']:.4f}  "
          f"pct_above_shell1={exp_a['r_eff_stats']['pct_above_shell1']:.1f}%")
    _print_summary_table(exp_a, modes)

    # ================================================================
    # EXPERIMENT B — Radial-aware (L1-normalized: variable L2 magnitude)
    # ================================================================
    print("\n" + "=" * 72)
    print("EXPERIMENT B — Radial-Aware (L1-Normalized)")
    print("  x -> x / ||x||_1   (r_eff varies in [~0.22..~0.46])")
    print("  Tests: does radial variation change routing structure?")
    print("=" * 72)

    X_l1 = lp_normalize(X_raw, p=1)
    r_l1 = np.linalg.norm(X_l1, axis=1)
    print(f"  L2 norms: min={r_l1.min():.4f} max={r_l1.max():.4f} "
          f"mean={r_l1.mean():.4f} std={r_l1.std():.4f}")
    print(f"  Shell-1 activates at r>={shell1_thresh:.4f}  (max r_eff={r_l1.max():.4f})")

    # B1: standard delta_r (shells don't activate — tests angular-only with different norms)
    exp_b1 = run_experiment(
        label="ExpB1_L1_deltaR_canonical",
        X_orig=X_l1,
        y=y,
        chart=chart,
        K_range=K_RANGE,
        delta_r=DELTA_R_CANONICAL,
        modes=modes,
    )
    all_results["ExpB1_L1_canonicalDeltaR"] = exp_b1

    print(f"\n  B1 (delta_r={DELTA_R_CANONICAL}): "
          f"pct_above_shell1={exp_b1['r_eff_stats']['pct_above_shell1']:.1f}%")
    _print_summary_table(exp_b1, modes)

    # B2: adjusted delta_r to activate shells for L1-normalized data
    # Shell-1 activates when r >= delta_r * (PHI - 1)
    # Set delta_r such that ~50th percentile of r activates shell-1
    r_p50_l1 = float(np.percentile(r_l1, 50))
    r_p05_l1 = float(np.percentile(r_l1, 5))
    delta_r_b2 = float(r_p50_l1 / (PHI - 1) * 0.95)
    print(f"\n  B2: adjusted delta_r={delta_r_b2:.4f} to activate shell-1 "
          f"(r_p50={r_p50_l1:.4f})")

    exp_b2 = run_experiment(
        label="ExpB2_L1_deltaR_adjusted",
        X_orig=X_l1,
        y=y,
        chart=chart,
        K_range=K_RANGE,
        delta_r=delta_r_b2,
        modes=modes,
    )
    all_results["ExpB2_L1_adjustedDeltaR"] = exp_b2

    print(f"  B2 (delta_r={delta_r_b2:.4f}): "
          f"pct_above_shell1={exp_b2['r_eff_stats']['pct_above_shell1']:.1f}%")
    _print_summary_table(exp_b2, modes)

    # ================================================================
    # EXPERIMENT C — Alternative norm geometry
    # ================================================================
    print("\n" + "=" * 72)
    print("EXPERIMENT C — Alternative Norm Geometry (L3 and L4)")
    print("  x -> x / ||x||_3  and  x -> x / ||x||_4")
    print("  Changes unit-surface shape; L2 norms vary slightly")
    print("=" * 72)

    # C1: L3-norm
    X_l3 = lp_normalize(X_raw, p=3)
    r_l3 = np.linalg.norm(X_l3, axis=1)
    print(f"\n  L3 norms: min={r_l3.min():.4f} max={r_l3.max():.4f} "
          f"mean={r_l3.mean():.4f} std={r_l3.std():.4f}")

    exp_c1 = run_experiment(
        label="ExpC1_L3_norm",
        X_orig=X_l3,
        y=y,
        chart=chart,
        K_range=K_RANGE,
        delta_r=DELTA_R_CANONICAL,
        modes=modes,
    )
    all_results["ExpC1_L3"] = exp_c1

    print(f"  C1 pct_above_shell1={exp_c1['r_eff_stats']['pct_above_shell1']:.1f}%")
    _print_summary_table(exp_c1, modes)

    # C2: L4-norm
    X_l4 = lp_normalize(X_raw, p=4)
    r_l4 = np.linalg.norm(X_l4, axis=1)
    print(f"\n  L4 norms: min={r_l4.min():.4f} max={r_l4.max():.4f} "
          f"mean={r_l4.mean():.4f} std={r_l4.std():.4f}")

    exp_c2 = run_experiment(
        label="ExpC2_L4_norm",
        X_orig=X_l4,
        y=y,
        chart=chart,
        K_range=K_RANGE,
        delta_r=DELTA_R_CANONICAL,
        modes=modes,
    )
    all_results["ExpC2_L4"] = exp_c2

    print(f"  C2 pct_above_shell1={exp_c2['r_eff_stats']['pct_above_shell1']:.1f}%")
    _print_summary_table(exp_c2, modes)

    # ================================================================
    # SCALING EXPONENT COMPARISON TABLE
    # ================================================================
    print("\n" + "=" * 72)
    print("SCALING EXPONENT COMPARISON: eff_buckets = c * K^alpha")
    print("=" * 72)
    print(f"\n{'Experiment':30s}  {'Mode':6s}  {'Variant':6s}  alpha     c       r2")
    print("-" * 72)
    for exp_name, exp_data in all_results.items():
        for mode_name, _, _ in modes:
            for variant in ["ORIG", "PERM"]:
                key = f"{mode_name}_{variant}"
                se = exp_data["scaling_exponents"].get(key, {})
                alpha = se.get("alpha", float("nan"))
                c = se.get("c", float("nan"))
                r2 = se.get("r2", float("nan"))
                print(f"  {exp_name:28s}  {mode_name:6s}  {variant:6s}  "
                      f"{alpha:+.4f}  {c:7.3f}  {r2:.4f}")

    # ================================================================
    # COMPRESSION RATIO TABLE (at K=100 and K=400)
    # ================================================================
    print("\n" + "=" * 72)
    print("COMPRESSION RATIOS (PERM/ORIG eff_buckets, ORIG/PERM Gini) at K=100, K=400")
    print("=" * 72)
    for K_check in [100, 400]:
        print(f"\n  K={K_check}:")
        print(f"  {'Experiment':28s}  {'Mode':6s}  eff_ratio  gini_ratio  purity_orig  purity_perm")
        for exp_name, exp_data in all_results.items():
            for mode_name, _, _ in modes:
                comp = exp_data["compression_by_K"].get(mode_name, {}).get(K_check, {})
                if comp:
                    print(f"    {exp_name:26s}  {mode_name:6s}  "
                          f"{comp['eff_bucket_ratio_PERM_over_ORIG']:8.3f}  "
                          f"{comp['gini_ratio_ORIG_over_PERM']:9.3f}  "
                          f"{comp['purity_orig']:10.4f}  "
                          f"{comp['purity_perm']:10.4f}")

    # ================================================================
    # SHELL ACTIVATION SUMMARY
    # ================================================================
    print("\n" + "=" * 72)
    print("SHELL ACTIVATION SUMMARY")
    print("=" * 72)
    for exp_name, exp_data in all_results.items():
        r_s = exp_data["r_eff_stats"]
        print(f"  {exp_name:30s}  r_eff=[{r_s['min']:.4f}..{r_s['max']:.4f}]  "
              f"std={r_s['std']:.4f}  pct_above_shell1={r_s['pct_above_shell1']:.1f}%")
        # Check actual shell activation in K=100 ORIG BASE
        k100_base_orig = next(
            (e for e in exp_data["routing_results"]
             if e["K"] == 100 and e["mode"] == "BASE" and e["variant"] == "ORIG"),
            None
        )
        if k100_base_orig:
            print(f"    K=100 BASE ORIG: n_shells={k100_base_orig['n_shells']}  "
                  f"shell_dist={k100_base_orig['shell_distribution']}")

    # ================================================================
    # Save results
    # ================================================================
    out_path = os.path.join(ROOT, "results/analysis/inc0168_norm_geometry.json")
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(all_results, f, indent=2, default=str)
    print(f"\nResults saved to {out_path}")
    print("\n" + "=" * 72)
    print("INC-0168 DIAGNOSTIC COMPLETE")
    print("=" * 72)


def _print_summary_table(exp_data: Dict, modes: List[Tuple]) -> None:
    """Print compact per-K summary for an experiment."""
    print(f"\n  {'K':>4s}  {'mode':>5s}  {'var':>4s}  "
          f"{'n_sh':>4s}  {'eff_b':>7s}  {'gini':>6s}  "
          f"{'purity':>7s}  {'sect_ent':>8s}")
    for entry in exp_data["routing_results"]:
        if entry["K"] in [25, 100, 400]:
            print(f"  {entry['K']:4d}  {entry['mode']:>5s}  {entry['variant']:>4s}  "
                  f"{entry['n_shells']:4d}  {entry['effective_bucket_count']:7.1f}  "
                  f"{entry['routing_gini']:6.4f}  "
                  f"{entry['bucket_purity']:7.4f}  "
                  f"{entry['sector_entropy']:8.4f}")


if __name__ == "__main__":
    main()
