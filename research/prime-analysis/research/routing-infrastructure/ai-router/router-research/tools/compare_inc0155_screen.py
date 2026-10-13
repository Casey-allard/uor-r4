#!/usr/bin/env python3
"""INC-0155 results analysis — routing compression (MSE vs K curves).

Reads the sweep results and computes:
1. MSE-vs-K curves for BASE_ORIG, BASE_PERM, TRANS_ORIG, TRANS_PERM
2. Routing compression ratio: at matched-MSE target, K_perm / K_orig
3. Decision: SUCCESS if ratio > 1.5, FALSIFIED if < 1.2
"""
import json
import sys
import numpy as np


def main():
    path = "results/analysis/inc0155_routing_compression_screen.json"
    with open(path) as f:
        d = json.load(f)
    routes = d["results"]

    # Parse route data into series
    series = {
        "BASE_ORIG": [], "BASE_PERM": [],
        "TRANS_ORIG": [], "TRANS_PERM": [],
    }

    for rid, seeds in routes.items():
        m = seeds[0]["metrics"]
        mse = m["test_mse_after"]
        # Parse route_id: e.g. BASE_K25_ORIG -> series=BASE_ORIG, K=25
        parts = rid.split("_")
        kind = parts[0]         # BASE or TRANS
        k_val = int(parts[1].replace("K", ""))
        transform = parts[2]    # ORIG or PERM
        key = f"{kind}_{transform}"
        if key in series:
            series[key].append((k_val, mse))

    # Sort each series by K
    for key in series:
        series[key].sort(key=lambda x: x[0])

    # --- Table 1: MSE vs K ---
    print("=" * 70)
    print("TABLE 1: MSE vs K (all routes)")
    print("=" * 70)
    all_ks = sorted(set(k for pts in series.values() for k, _ in pts))
    header = f"{'K':>5}"
    for key in ["BASE_ORIG", "BASE_PERM", "TRANS_ORIG", "TRANS_PERM"]:
        header += f"  {key:>12}"
    print(header)
    print("-" * 70)

    for k in all_ks:
        row = f"{k:>5}"
        for key in ["BASE_ORIG", "BASE_PERM", "TRANS_ORIG", "TRANS_PERM"]:
            val = dict(series[key]).get(k)
            if val is not None:
                row += f"  {val:>12.6f}"
            else:
                row += f"  {'---':>12}"
        print(row)

    # --- Table 2: ORIG-vs-PERM MSE delta at each K ---
    print()
    print("=" * 70)
    print("TABLE 2: MSE delta (PERM - ORIG) at each K")
    print("=" * 70)
    print(f"{'K':>5}  {'BASE_delta':>12}  {'BASE_%':>8}  {'TRANS_delta':>12}  {'TRANS_%':>8}")
    print("-" * 55)

    base_orig_d = dict(series["BASE_ORIG"])
    base_perm_d = dict(series["BASE_PERM"])
    trans_orig_d = dict(series["TRANS_ORIG"])
    trans_perm_d = dict(series["TRANS_PERM"])

    for k in all_ks:
        bo = base_orig_d.get(k)
        bp = base_perm_d.get(k)
        to = trans_orig_d.get(k)
        tp = trans_perm_d.get(k)

        row = f"{k:>5}"
        if bo is not None and bp is not None:
            bd = bp - bo
            bpct = bd / bo * 100 if bo > 0 else 0
            row += f"  {bd:>+12.6f}  {bpct:>+7.2f}%"
        else:
            row += f"  {'---':>12}  {'---':>8}"

        if to is not None and tp is not None:
            td = tp - to
            tpct = td / to * 100 if to > 0 else 0
            row += f"  {td:>+12.6f}  {tpct:>+7.2f}%"
        else:
            row += f"  {'---':>12}  {'---':>8}"
        print(row)

    # --- Routing compression computation ---
    print()
    print("=" * 70)
    print("ROUTING COMPRESSION ANALYSIS")
    print("=" * 70)

    for kind in ["BASE", "TRANS"]:
        orig_pts = series[f"{kind}_ORIG"]
        perm_pts = series[f"{kind}_PERM"]
        if len(orig_pts) < 2 or len(perm_pts) < 2:
            print(f"\n{kind}: insufficient data points for compression analysis")
            continue

        orig_ks = [k for k, _ in orig_pts]
        orig_mses = [m for _, m in orig_pts]
        perm_ks = [k for k, _ in perm_pts]
        perm_mses = [m for _, m in perm_pts]

        # Use log-linear interpolation for K vs MSE
        # MSE typically decreases as K increases — find matched-MSE points
        print(f"\n--- {kind} ---")
        print(f"ORIG: K={orig_ks}, MSE={[f'{m:.6f}' for m in orig_mses]}")
        print(f"PERM: K={perm_ks}, MSE={[f'{m:.6f}' for m in perm_mses]}")

        # Use the MSE of ORIG at each K as a target, find what K PERM needs
        # by linear interpolation on log(K) vs MSE
        perm_log_ks = np.log(np.array(perm_ks, dtype=float))
        perm_mse_arr = np.array(perm_mses)

        print(f"\nCompression ratios (K_perm / K_orig at matched MSE):")
        ratios = []
        for k_orig, mse_target in zip(orig_ks, orig_mses):
            # Check if mse_target is within PERM range
            if mse_target > max(perm_mses):
                # PERM never gets this bad — target is above range
                print(f"  MSE={mse_target:.6f} (ORIG K={k_orig}): PERM always better → ratio=1.0")
                continue
            if mse_target < min(perm_mses):
                # PERM never gets this good — target is below range
                print(f"  MSE={mse_target:.6f} (ORIG K={k_orig}): PERM never reaches this → ratio>max")
                continue

            # Interpolate: find log(K_perm) where PERM MSE = mse_target
            # MSE is generally decreasing with K, so invert for interp
            # Sort by MSE descending for interpolation
            sorted_idx = np.argsort(-perm_mse_arr)
            sorted_mses = perm_mse_arr[sorted_idx]
            sorted_log_ks = perm_log_ks[sorted_idx]

            k_perm_log = np.interp(mse_target, sorted_mses[::-1], sorted_log_ks[::-1])
            k_perm = np.exp(k_perm_log)
            ratio = k_perm / k_orig
            ratios.append(ratio)
            print(f"  MSE={mse_target:.6f} (ORIG K={k_orig:>3}): PERM needs K≈{k_perm:.1f} → ratio={ratio:.2f}")

        if ratios:
            mean_ratio = np.mean(ratios)
            print(f"\n  Mean compression ratio: {mean_ratio:.2f}")
        else:
            mean_ratio = None
            print(f"\n  No valid compression ratios computed")

    # --- Decision criteria ---
    print()
    print("=" * 70)
    print("DECISION CRITERIA")
    print("=" * 70)

    # Compute at K=25 and K=50 (mid-range where compression should be visible)
    for kind in ["BASE", "TRANS"]:
        orig_pts = dict(series[f"{kind}_ORIG"])
        perm_pts = dict(series[f"{kind}_PERM"])
        for k_check in [25, 50]:
            o = orig_pts.get(k_check)
            p = perm_pts.get(k_check)
            if o is not None and p is not None:
                delta_pct = (p - o) / o * 100 if o > 0 else 0
                print(f"  {kind} K={k_check}: ORIG={o:.6f}  PERM={p:.6f}  delta={delta_pct:+.2f}%")

    print()
    print("SUCCESS if K_perm/K_orig > 1.5 at any matched MSE target")
    print("FALSIFIED if K_perm/K_orig < 1.2 at ALL matched MSE targets")
    print("REFINE otherwise")


if __name__ == "__main__":
    main()
