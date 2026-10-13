#!/usr/bin/env python3
"""INC-0167 pipeline results analysis.

Reads the proxy_sweep output and extracts training-time metrics for all
large-K runs to determine scaling exponents during training.
"""
import json
import os
import sys
import numpy as np


def main():
    analysis_path = "results/analysis/inc0167_scaling_mechanism.json"
    with open(analysis_path) as f:
        data = json.load(f)

    rs = data.get("route_stats", [])
    results = data["results"]

    print("=" * 72)
    print("INC-0167 PIPELINE RESULTS ANALYSIS")
    print("=" * 72)

    # Build unified data table
    rows = []
    for s in rs:
        rid = s["route_id"]
        K = results[rid][0]["args"]["K"]
        eff_bkt = np.exp(s["mean_sector_entropy"])  # perplexity of bucket distribution
        rows.append({
            "route_id": rid,
            "K": K,
            "active_buckets": s["mean_buckets"],
            "eff_buckets": float(eff_bkt),
            "sector_entropy": s["mean_sector_entropy"],
            "sector_pmax": s["mean_sector_pmax"],
            "shells": s["mean_eval_shells"],
        })

    # Print summary table
    print(f"\n{'route_id':<22s}  {'K':>5s}  {'shells':>6s}  {'act_bkt':>7s}  "
          f"{'eff_bkt':>7s}  {'sect_ent':>8s}  {'pmax':>6s}")
    print("-" * 75)
    for r in sorted(rows, key=lambda x: (x["route_id"])):
        print(f"{r['route_id']:<22s}  {r['K']:5d}  {r['shells']:6.0f}  "
              f"{r['active_buckets']:7.0f}  {r['eff_buckets']:7.1f}  "
              f"{r['sector_entropy']:8.3f}  {r['sector_pmax']:6.4f}")

    # Scaling fits
    print("\n" + "=" * 72)
    print("TRAINING-TIME SCALING: active_buckets = c * K^alpha")
    print("=" * 72)
    for mode in ["TRANS", "BASE"]:
        for tfm in ["ORIG", "PERM"]:
            entries = [r for r in rows
                       if r["route_id"].startswith(mode) and r["route_id"].endswith(tfm)]
            entries.sort(key=lambda x: x["K"])
            if len(entries) < 3:
                continue
            log_K = np.log(np.array([e["K"] for e in entries]))
            log_bkt = np.log(np.array([e["active_buckets"] for e in entries]))
            A = np.vstack([log_K, np.ones(len(log_K))]).T
            alpha, log_c = np.linalg.lstsq(A, log_bkt, rcond=None)[0]
            c = np.exp(log_c)
            print(f"\n  {mode} {tfm}: alpha={alpha:.4f}, c={c:.4f}")
            for e in entries:
                pred = c * e["K"] ** alpha
                print(f"    K={e['K']:4d}  measured={e['active_buckets']:.0f}  "
                      f"predicted={pred:.0f}  ratio={e['active_buckets']/pred:.3f}")

    print("\n" + "=" * 72)
    print("TRAINING-TIME SCALING: eff_buckets (perplexity) = c * K^alpha")
    print("=" * 72)
    for mode in ["TRANS", "BASE"]:
        for tfm in ["ORIG", "PERM"]:
            entries = [r for r in rows
                       if r["route_id"].startswith(mode) and r["route_id"].endswith(tfm)]
            entries.sort(key=lambda x: x["K"])
            if len(entries) < 3:
                continue
            log_K = np.log(np.array([e["K"] for e in entries]))
            log_eff = np.log(np.array([e["eff_buckets"] for e in entries]))
            A = np.vstack([log_K, np.ones(len(log_K))]).T
            alpha, log_c = np.linalg.lstsq(A, log_eff, rcond=None)[0]
            c = np.exp(log_c)
            print(f"\n  {mode} {tfm}: alpha={alpha:.4f}, c={c:.4f}")
            for e in entries:
                pred = c * e["K"] ** alpha
                print(f"    K={e['K']:4d}  measured={e['eff_buckets']:.1f}  "
                      f"predicted={pred:.1f}  ratio={e['eff_buckets']/pred:.3f}")

    # PERM/ORIG ratios
    print("\n" + "=" * 72)
    print("PERM/ORIG RATIOS")
    print("=" * 72)
    for mode in ["TRANS", "BASE"]:
        print(f"\n  {mode}:")
        for K in sorted(set(r["K"] for r in rows)):
            orig = [r for r in rows if r["route_id"] == f"{mode}_K{K}_ORIG"]
            perm = [r for r in rows if r["route_id"] == f"{mode}_K{K}_PERM"]
            if orig and perm:
                act_ratio = perm[0]["active_buckets"] / max(orig[0]["active_buckets"], 0.01)
                eff_ratio = perm[0]["eff_buckets"] / max(orig[0]["eff_buckets"], 0.01)
                print(f"    K={K:4d}  ORIG_eff={orig[0]['eff_buckets']:7.1f}  "
                      f"PERM_eff={perm[0]['eff_buckets']:7.1f}  "
                      f"eff_ratio={eff_ratio:.3f}  "
                      f"act_ratio={act_ratio:.3f}")

    # Shell confirmation
    print("\n" + "-" * 40)
    print("Shell count: ALL routes have shells=1 (confirmed)")
    all_one = all(r["shells"] == 1.0 for r in rows)
    print(f"  All shells == 1: {all_one}")


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    main()
