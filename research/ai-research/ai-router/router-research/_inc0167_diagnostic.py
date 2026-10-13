#!/usr/bin/env python3
"""INC-0167 routing mechanism diagnostic.

Step 1: Report router mechanism structure (shell thresholds, sector grid).
Step 2: Shell activation sweep for large K values.
Step 3: Determine delta_r values for forced shell counts 1, 2, 3.
Step 4: Sector scaling analysis across K range.

Uses the same code paths as the production router.
"""
import sys
import os
import json
import numpy as np

sys.path.insert(0, os.path.dirname(__file__))

from hyperbolic_router_so8 import (
    route_addresses,
    shell_metric_components,
    assign_sectors_phase4d_hopf_base,
    assign_sectors_phase4d_hopf_transport,
    allocate_triplet_bins_budget,
    Chart,
    safe_norm,
    LOG_PHI,
    PHI,
)


def load_data(path="data/wikitext2_proxy/ppmi_proxy.npz"):
    d = np.load(path)
    X_tr = d["x_train"]
    y_tr = d["y_train"]
    X_te = d["x_test"]
    y_te = d["y_test"]
    return X_tr, y_tr, X_te, y_te


def apply_chart_transform(X, chart):
    """Replicate the chart transform used in route_addresses."""
    from hyperbolic_router_so8 import apply_chart, route_coordinate
    z = apply_chart(X, chart)
    route_z = route_coordinate(X, chart, sector_mode="phase4d_hopf_base", route_scale_lambda=1.0)
    return z, route_z


def compute_r_eff_distribution(route_z, delta_r=3.6):
    """Compute r_eff and report distribution statistics."""
    r = safe_norm(route_z, axis=1, keepdims=False)
    # With time_pressure_lambda=0, adaptive_shell_growth=0, shell_multiplier=1.0
    # r_eff = r (no modification)
    r_eff = r
    return r_eff


def shell_thresholds(delta_r, n_shells=6):
    """Compute the r_eff threshold for each shell boundary."""
    thresholds = []
    for s in range(n_shells):
        # shell = s when log1p(r_eff/delta_r) / LOG_PHI >= s
        # => r_eff/delta_r >= phi^s - 1
        # => r_eff >= delta_r * (phi^s - 1)
        r_thresh = delta_r * (PHI ** s - 1)
        thresholds.append((s, r_thresh))
    return thresholds


def sector_grid_for_k(K, mode="base"):
    """Report the sector grid structure for a given K and mode."""
    if mode == "base":
        kchi = max(1, int(np.floor(np.sqrt(max(int(K), 1)))))
        kdelta = max(1, int(np.ceil(float(max(int(K), 1)) / float(kchi))))
        return {"mode": "base", "K": K, "kchi": kchi, "kdelta": kdelta,
                "total_cells": kchi * kdelta, "used_max": K}
    elif mode == "transport":
        kchi, kdelta, kalpha = allocate_triplet_bins_budget(
            K, min_first=max(2, 2), min_second=2, min_third=2)
        return {"mode": "transport", "K": K, "kchi": kchi, "kdelta": kdelta,
                "kalpha": kalpha, "total_cells": kchi * kdelta * kalpha, "used_max": K}


def routing_stats(shells, sectors, K):
    """Compute routing statistics from shell and sector assignments."""
    unique_shells = np.unique(shells)
    unique_sectors = np.unique(sectors)
    n_tokens = len(shells)

    # Bucket keys: (shell, sector)
    bucket_keys = list(zip(shells.tolist(), sectors.tolist()))
    from collections import Counter
    bucket_counts = Counter(bucket_keys)
    counts = np.array(sorted(bucket_counts.values(), reverse=True), dtype=np.float64)

    # Effective bucket count (perplexity of distribution)
    p = counts / counts.sum()
    entropy = -np.sum(p * np.log(p + 1e-30))
    eff_buckets = np.exp(entropy)

    # Gini
    n = len(counts)
    if n <= 1:
        gini = 0.0
    else:
        sorted_c = np.sort(counts)
        index = np.arange(1, n + 1)
        gini = float(2.0 * np.sum(index * sorted_c) / (n * np.sum(sorted_c)) - (n + 1) / n)

    # Top-half concentration
    half = max(1, len(counts) // 2)
    top_half = float(np.sum(counts[:half]) / np.sum(counts))

    # Shell distribution
    shell_counts = Counter(shells.tolist())
    shell_dist = {int(k): v for k, v in sorted(shell_counts.items())}

    # Sector activity
    n_active_sectors = len(unique_sectors)
    sector_counts = Counter(sectors.tolist())
    # Sectors receiving >1% traffic
    threshold = 0.01 * n_tokens
    sectors_above_1pct = sum(1 for c in sector_counts.values() if c > threshold)

    # Sector entropy
    sector_p = np.array(list(sector_counts.values()), dtype=np.float64)
    sector_p = sector_p / sector_p.sum()
    sector_entropy = -np.sum(sector_p * np.log(sector_p + 1e-30))

    # Sector Gini
    sc = np.sort(np.array(list(sector_counts.values()), dtype=np.float64))
    ns = len(sc)
    if ns <= 1:
        sector_gini = 0.0
    else:
        idx = np.arange(1, ns + 1)
        sector_gini = float(2.0 * np.sum(idx * sc) / (ns * np.sum(sc)) - (ns + 1) / ns)

    return {
        "n_tokens": n_tokens,
        "n_shells": len(unique_shells),
        "shell_list": sorted(unique_shells.tolist()),
        "shell_distribution": shell_dist,
        "n_active_sectors": n_active_sectors,
        "sectors_above_1pct": sectors_above_1pct,
        "n_buckets": len(bucket_counts),
        "effective_bucket_count": float(eff_buckets),
        "routing_gini": float(gini),
        "top_half_concentration": float(top_half),
        "sector_entropy": float(sector_entropy),
        "sector_gini": float(sector_gini),
    }


def run_routing(X, chart, K, sector_mode, delta_r=3.6, phase_transport_lambda=1.0):
    """Run static routing and return shells, sectors."""
    shells_arr, sectors_arr, _, _ = route_addresses(
        X, delta_r=delta_r, C=None, chart=chart,
        sector_mode=sector_mode,
        phase_dim_i=0, phase_dim_j=1,
        phase4_dim_i=3, phase4_dim_j=65, phase4_dim_k=2, phase4_dim_l=21,
        field4_dim_i=4, field4_dim_j=66, field4_dim_k=5, field4_dim_l=22,
        complex_dim_i=1, complex_dim_j=3,
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
    return shells_arr, sectors_arr


def main():
    print("=" * 72)
    print("INC-0167 ROUTING MECHANISM DIAGNOSTIC")
    print("=" * 72)

    # Load data
    X_tr, y_tr, X_te, y_te = load_data()
    print(f"\nData: X_tr shape={X_tr.shape}, X_te shape={X_te.shape}")

    # Create chart (seed=0) - identity rotation, no scaling
    np.random.seed(0)
    d = X_tr.shape[1]
    R = np.eye(d, dtype=np.float64)
    # Random rotation to simulate chart_iters would change results;
    # use identity for reproducible mechanism audit
    chart = Chart(R=R, s_global=None, S_radial=None, scale_mode="global")

    # Apply chart transform to get route_z
    _, route_z = apply_chart_transform(X_tr[:5000], chart)
    r_eff = compute_r_eff_distribution(route_z, delta_r=3.6)

    # ---- STEP 1: Router Mechanism Audit ----
    print("\n" + "=" * 72)
    print("STEP 1: ROUTER MECHANISM AUDIT")
    print("=" * 72)

    print(f"\nShell mode: phi_log")
    print(f"Shell formula: shell = floor(log1p(r_eff / delta_r) / LOG_PHI)")
    print(f"  LOG_PHI = ln(φ) = {LOG_PHI:.6f}")
    print(f"  PHI = {PHI:.6f}")
    print(f"  delta_r = 3.6 (config)")

    print(f"\nShell thresholds (delta_r=3.6):")
    for s, r_thresh in shell_thresholds(3.6, n_shells=6):
        print(f"  Shell ≥ {s}: r_eff ≥ {r_thresh:.4f}")

    print(f"\nr_eff distribution (train, N={len(r_eff)}):")
    print(f"  min={np.min(r_eff):.4f}  max={np.max(r_eff):.4f}  mean={np.mean(r_eff):.4f}")
    for p in [5, 25, 50, 75, 95, 99]:
        print(f"  p{p:02d}={np.percentile(r_eff, p):.4f}", end="")
    print()

    # Current shells at delta_r=3.6
    shells_current, _, _ = shell_metric_components(r_eff, delta_r=3.6, shell_mode="phi_log")
    from collections import Counter
    shell_dist = Counter(shells_current.tolist())
    print(f"\nShell distribution at delta_r=3.6: {dict(sorted(shell_dist.items()))}")

    # Sector grid for various K
    print(f"\nSector grid structure:")
    for K in [25, 50, 75, 100, 125, 150, 200, 250, 300, 400, 600, 1000]:
        bg = sector_grid_for_k(K, mode="base")
        tg = sector_grid_for_k(K, mode="transport")
        print(f"  K={K:4d}  BASE: kchi={bg['kchi']:3d} × kdelta={bg['kdelta']:3d} = {bg['total_cells']:5d}"
              f"  |  TRANS: kchi={tg['kchi']:2d} × kdelta={tg['kdelta']:2d} × kalpha={tg['kalpha']:2d} = {tg['total_cells']:5d}")

    # ---- STEP 2: Shell Activation Sweep (large K) ----
    print("\n" + "=" * 72)
    print("STEP 2: SHELL ACTIVATION SWEEP (large K)")
    print("=" * 72)

    X_sub = X_tr[:5000]
    large_K_values = [250, 300, 400, 600, 1000]

    print(f"\nUsing N={len(X_sub)} training tokens, delta_r=3.6, seed=0")
    for K in large_K_values:
        for mode, sector_mode, ptl in [("BASE", "phase4d_hopf_base", 0.0),
                                        ("TRANS", "phase4d_hopf_transport", 1.0)]:
            shells, sectors = run_routing(X_sub, chart, K, sector_mode,
                                         delta_r=3.6, phase_transport_lambda=ptl)
            stats = routing_stats(shells, sectors, K)
            print(f"\n  K={K:4d} {mode:5s}: shells={stats['n_shells']}  "
                  f"active_sectors={stats['n_active_sectors']:4d}  "
                  f"buckets={stats['n_buckets']:4d}  "
                  f"eff_buckets={stats['effective_bucket_count']:.1f}  "
                  f"gini={stats['routing_gini']:.4f}  "
                  f"top_half={stats['top_half_concentration']:.3f}  "
                  f"sect_ent={stats['sector_entropy']:.3f}  "
                  f"sect>1%={stats['sectors_above_1pct']}")
            print(f"          shell_dist={stats['shell_distribution']}")

    # ---- STEP 3: Forced Shell Sensitivity ----
    print("\n" + "=" * 72)
    print("STEP 3: FORCED SHELL SENSITIVITY TEST")
    print("=" * 72)

    # First, find delta_r values that produce 1, 2, 3 shells
    r_max = float(np.max(r_eff))
    r_median = float(np.median(r_eff))
    r_p95 = float(np.percentile(r_eff, 95))
    print(f"\nr_eff: max={r_max:.4f}, median={r_median:.4f}, p95={r_p95:.4f}")

    # Compute delta_r for target shell counts
    # Shell=0 for all: need r_eff < delta_r * (PHI - 1), i.e. delta_r > r_max / (PHI - 1)
    delta_r_1shell = r_max / (PHI - 1) * 1.1  # 10% margin
    # Shell=1 for half+: need ~median r_eff to cross shell 1 boundary
    # Shell ≥ 1 when r_eff ≥ delta_r * (PHI - 1), Shell ≥ 2 when r_eff >= delta_r * (PHI^2 - 1)
    # Want ~50% in shell 1: delta_r such that r_median = delta_r * (PHI - 1)
    delta_r_2shell = r_median / (PHI - 1) * 0.9
    # Shell=2+: delta_r such that some tokens reach shell 2
    # Shell ≥ 2 when r_eff ≥ delta_r * (PHI^2 - 1) = delta_r * 1.618
    delta_r_3shell = r_p95 / (PHI ** 2 - 1) * 0.9

    print(f"\nComputed delta_r for target shell counts:")
    print(f"  1 shell (all shell 0): delta_r={delta_r_1shell:.2f}")
    print(f"  2 shells: delta_r={delta_r_2shell:.2f}")
    print(f"  3+ shells: delta_r={delta_r_3shell:.2f}")

    # Also test some round values
    delta_r_values = sorted(set([
        round(delta_r_3shell, 1),
        round(delta_r_2shell, 1),
        1.0,
        2.0,
        3.6,
        round(delta_r_1shell, 1),
        20.0,
    ]))

    K_fixed = 100
    print(f"\nK={K_fixed}, varying delta_r:")
    forced_shell_results = []

    for dr in delta_r_values:
        for mode, sector_mode, ptl in [("BASE", "phase4d_hopf_base", 0.0),
                                        ("TRANS", "phase4d_hopf_transport", 1.0)]:
            shells, sectors = run_routing(X_sub, chart, K_fixed, sector_mode,
                                         delta_r=dr, phase_transport_lambda=ptl)
            stats = routing_stats(shells, sectors, K_fixed)
            entry = {"delta_r": dr, "mode": mode, **stats}
            forced_shell_results.append(entry)
            print(f"  dr={dr:5.1f} {mode:5s}: shells={stats['n_shells']}  "
                  f"eff_buckets={stats['effective_bucket_count']:.1f}  "
                  f"gini={stats['routing_gini']:.4f}  "
                  f"top_half={stats['top_half_concentration']:.3f}  "
                  f"shell_dist={stats['shell_distribution']}")

    # ---- STEP 4: Sector Scaling Analysis ----
    print("\n" + "=" * 72)
    print("STEP 4: SECTOR SCALING ANALYSIS")
    print("=" * 72)

    K_range = [10, 15, 25, 35, 50, 75, 100, 125, 150, 200, 250, 300, 400, 600, 1000]
    print(f"\nSector usage vs K (delta_r=3.6, seed=0, N={len(X_sub)}):")
    print(f"\n{'K':>6s}  {'mode':>5s}  {'kchi':>4s}  {'kdel':>4s}  {'act_sec':>7s}  {'sec>1%':>6s}  "
          f"{'sec_ent':>7s}  {'sec_gini':>8s}  {'sqrt_K':>6s}  {'ratio':>6s}")

    sector_scaling_data = []
    for K in K_range:
        for mode, sector_mode, ptl in [("BASE", "phase4d_hopf_base", 0.0),
                                        ("TRANS", "phase4d_hopf_transport", 1.0)]:
            shells, sectors = run_routing(X_sub, chart, K, sector_mode,
                                         delta_r=3.6, phase_transport_lambda=ptl)
            stats = routing_stats(shells, sectors, K)
            sg = sector_grid_for_k(K, mode="base" if mode == "BASE" else "transport")
            sqrt_k = np.sqrt(K)
            ratio = stats['n_active_sectors'] / sqrt_k
            kchi = sg['kchi']
            kdel = sg.get('kdelta', sg.get('kdelta', '-'))

            entry = {"K": K, "mode": mode, "sqrt_K": float(sqrt_k),
                     "active_sectors": stats['n_active_sectors'],
                     "sectors_above_1pct": stats['sectors_above_1pct'],
                     "sector_entropy": stats['sector_entropy'],
                     "sector_gini": stats['sector_gini'],
                     "active_over_sqrt_K": float(ratio),
                     "effective_bucket_count": stats['effective_bucket_count'],
                     "routing_gini": stats['routing_gini'],
                     "kchi": kchi, "kdelta": kdel}
            sector_scaling_data.append(entry)

            print(f"{K:6d}  {mode:>5s}  {kchi:4d}  {kdel:4d}  {stats['n_active_sectors']:7d}  "
                  f"{stats['sectors_above_1pct']:6d}  {stats['sector_entropy']:7.3f}  "
                  f"{stats['sector_gini']:8.4f}  {sqrt_k:6.2f}  {ratio:6.2f}")

    # Fit scaling exponents for sector activity
    print("\n" + "-" * 40)
    print("Scaling fit: active_sectors = c * K^alpha")
    for mode in ["BASE", "TRANS"]:
        entries = [e for e in sector_scaling_data if e["mode"] == mode and e["K"] >= 25]
        if len(entries) < 3:
            continue
        log_K = np.log(np.array([e["K"] for e in entries]))
        log_act = np.log(np.array([e["active_sectors"] for e in entries]))
        # Linear regression in log space
        A = np.vstack([log_K, np.ones(len(log_K))]).T
        alpha, log_c = np.linalg.lstsq(A, log_act, rcond=None)[0]
        c = np.exp(log_c)
        print(f"  {mode}: alpha={alpha:.4f}, c={c:.4f}")

    print("\nFit: sectors_above_1pct = c * K^alpha")
    for mode in ["BASE", "TRANS"]:
        entries = [e for e in sector_scaling_data if e["mode"] == mode and e["K"] >= 25]
        if len(entries) < 3:
            continue
        log_K = np.log(np.array([e["K"] for e in entries]))
        vals = np.array([e["sectors_above_1pct"] for e in entries], dtype=np.float64)
        vals = np.maximum(vals, 1)  # avoid log(0)
        log_act = np.log(vals)
        A = np.vstack([log_K, np.ones(len(log_K))]).T
        alpha, log_c = np.linalg.lstsq(A, log_act, rcond=None)[0]
        c = np.exp(log_c)
        print(f"  {mode}: alpha={alpha:.4f}, c={c:.4f}")

    print("\nFit: effective_bucket_count = c * K^alpha")
    for mode in ["BASE", "TRANS"]:
        entries = [e for e in sector_scaling_data if e["mode"] == mode and e["K"] >= 25]
        if len(entries) < 3:
            continue
        log_K = np.log(np.array([e["K"] for e in entries]))
        log_eff = np.log(np.array([e["effective_bucket_count"] for e in entries]))
        A = np.vstack([log_K, np.ones(len(log_K))]).T
        alpha, log_c = np.linalg.lstsq(A, log_eff, rcond=None)[0]
        c = np.exp(log_c)
        print(f"  {mode}: alpha={alpha:.4f}, c={c:.4f}")

    # Save results
    results = {
        "step1_r_eff": {
            "min": float(np.min(r_eff)),
            "max": float(np.max(r_eff)),
            "mean": float(np.mean(r_eff)),
            "median": float(np.median(r_eff)),
            "p05": float(np.percentile(r_eff, 5)),
            "p25": float(np.percentile(r_eff, 25)),
            "p50": float(np.percentile(r_eff, 50)),
            "p75": float(np.percentile(r_eff, 75)),
            "p95": float(np.percentile(r_eff, 95)),
            "p99": float(np.percentile(r_eff, 99)),
        },
        "step2_shell_sweep": [e for e in sector_scaling_data if e["K"] in large_K_values],
        "step3_forced_shells": forced_shell_results,
        "step4_sector_scaling": sector_scaling_data,
    }

    out_path = "results/analysis/inc0167_diagnostic.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(results, f, indent=2, default=str)
    print(f"\nResults saved to {out_path}")

    print("\n" + "=" * 72)
    print("DIAGNOSTIC COMPLETE")
    print("=" * 72)


if __name__ == "__main__":
    main()
