#!/usr/bin/env python3
"""INC-0157 analysis — multi-seed spectral compression + bucket coherence.

Aggregates seed 0 + seed 1 data for:
  157A: spectral compression (label_indicator_lowfreq_max, true_margin_lowfreq_energy)
  157B: bucket semantic coherence (purity, entropy, MI)
"""
import json
import sys
from collections import defaultdict
from typing import Dict, List, Tuple

import numpy as np

# --- File paths ---
SPECTRAL_SEED0 = "results/analysis/inc0156_spectral_compression_screen.json"
SPECTRAL_SEED1 = "results/analysis/inc0157_spectral_signal_seed1.json"
RESIDUAL_SEED0 = "results/analysis/inc0156_residual_compression_screen.json"
RESIDUAL_SEED1 = "results/analysis/inc0157_residual_seed1.json"
BUCKET_SEED0 = "results/analysis/inc0157_bucket_coherence_seed0.json"
BUCKET_SEED1 = "results/analysis/inc0157_bucket_coherence_seed1.json"


def load_metrics(path: str) -> Dict[str, Dict]:
    with open(path) as f:
        d = json.load(f)
    return {r["route_id"]: r["metrics"] for r in d["results"]}


def parse_route_id(rid: str):
    parts = rid.split("_")
    kind = parts[0]
    k_val = int(parts[1].replace("K", ""))
    transform = parts[2]
    return kind, k_val, transform


def aggregate_seeds(seed_data: List[Dict[str, Dict]], metric_keys: List[str]):
    """Aggregate metrics across seeds.
    Returns: {route_id: {metric: (mean, std, [per_seed_values])}}
    """
    all_rids = set()
    for sd in seed_data:
        all_rids.update(sd.keys())
    result = {}
    for rid in sorted(all_rids):
        result[rid] = {}
        for mk in metric_keys:
            vals = []
            for sd in seed_data:
                if rid in sd and mk in sd[rid]:
                    vals.append(sd[rid][mk])
            if vals:
                result[rid][mk] = (float(np.mean(vals)), float(np.std(vals)), vals)
    return result


def interpolate_k_at_quality(ks, qualities, target_quality):
    ks = np.array(ks, dtype=float)
    qs = np.array(qualities, dtype=float)
    if len(ks) < 2:
        return None
    log_ks = np.log(ks)
    for i in range(len(qs) - 1):
        q_lo, q_hi = min(qs[i], qs[i + 1]), max(qs[i], qs[i + 1])
        if q_lo <= target_quality <= q_hi:
            if abs(qs[i + 1] - qs[i]) < 1e-12:
                continue
            t = (target_quality - qs[i]) / (qs[i + 1] - qs[i])
            log_k_interp = log_ks[i] + t * (log_ks[i + 1] - log_ks[i])
            return float(np.exp(log_k_interp))
    return None


def main():
    # Load all data
    spec0 = load_metrics(SPECTRAL_SEED0)
    spec1 = load_metrics(SPECTRAL_SEED1)
    res0 = load_metrics(RESIDUAL_SEED0)
    res1 = load_metrics(RESIDUAL_SEED1)
    bkt0 = load_metrics(BUCKET_SEED0)
    bkt1 = load_metrics(BUCKET_SEED1)

    # Merge residual into spectral
    for rid in res0:
        if rid in spec0:
            spec0[rid].update(res0[rid])
    for rid in res1:
        if rid in spec1:
            spec1[rid].update(res1[rid])

    # ============================================================
    # 157A: MULTI-SEED SPECTRAL COMPRESSION
    # ============================================================
    spectral_metrics = ["label_indicator_lowfreq_max", "true_margin_lowfreq_energy"]
    agg = aggregate_seeds([spec0, spec1], spectral_metrics)

    print("=" * 90)
    print("INC-0157A: MULTI-SEED SPECTRAL QUALITY (mean ± std over seeds 0,1)")
    print("=" * 90)

    for metric in spectral_metrics:
        short = metric.replace("_lowfreq_energy", "_lfe").replace("_lowfreq_max", "_lfm")
        print(f"\n--- {metric} ---")
        header = f"{'K':>5}  {'BASE_ORIG':>18}  {'BASE_PERM':>18}  {'ratio':>8}  {'TRANS_ORIG':>18}  {'TRANS_PERM':>18}  {'ratio':>8}"
        print(header)
        print("-" * 105)

        all_ks = sorted(set(k for rid in agg for _, k, _ in [parse_route_id(rid)]))

        for k in all_ks:
            row = f"{k:>5}"
            for kind in ["BASE", "TRANS"]:
                orig_rid = f"{kind}_K{k}_ORIG"
                perm_rid = f"{kind}_K{k}_PERM"
                if orig_rid in agg and metric in agg[orig_rid]:
                    om, os = agg[orig_rid][metric][0], agg[orig_rid][metric][1]
                    row += f"  {om:>7.4f}±{os:>6.4f}"
                else:
                    row += f"  {'---':>18}"
                if perm_rid in agg and metric in agg[perm_rid]:
                    pm, ps = agg[perm_rid][metric][0], agg[perm_rid][metric][1]
                    row += f"  {pm:>7.4f}±{ps:>6.4f}"
                else:
                    row += f"  {'---':>18}"
                if orig_rid in agg and perm_rid in agg and metric in agg[orig_rid] and metric in agg[perm_rid]:
                    ratio = agg[orig_rid][metric][0] / agg[perm_rid][metric][0] if agg[perm_rid][metric][0] > 1e-12 else float('inf')
                    row += f"  {ratio:>8.3f}"
                else:
                    row += f"  {'---':>8}"
            print(row)

    # Compression analysis for true_margin_lowfreq_energy
    print("\n" + "=" * 90)
    print("INC-0157A: TRUE_MARGIN COMPRESSION RATIOS (per seed)")
    print("=" * 90)
    for seed_idx, sd in enumerate([spec0, spec1]):
        print(f"\n--- Seed {seed_idx} ---")
        for kind in ["BASE", "TRANS"]:
            rids_orig = [(k, sd.get(f"{kind}_K{k}_ORIG", {}).get("true_margin_lowfreq_energy"))
                         for k in [4, 9, 16, 25, 50, 75, 100]]
            rids_perm = [(k, sd.get(f"{kind}_K{k}_PERM", {}).get("true_margin_lowfreq_energy"))
                         for k in [4, 9, 16, 25, 50, 75, 100]]
            orig_pts = [(k, v) for k, v in rids_orig if v is not None]
            perm_pts = [(k, v) for k, v in rids_perm if v is not None]
            if len(orig_pts) < 2 or len(perm_pts) < 2:
                print(f"  {kind}: insufficient data")
                continue
            perm_ks = [k for k, _ in perm_pts]
            perm_qs = [v for _, v in perm_pts]
            ratios = []
            for k_orig, q_target in orig_pts:
                k_perm = interpolate_k_at_quality(perm_ks, perm_qs, q_target)
                if k_perm is not None:
                    ratio = k_perm / k_orig
                    ratios.append(ratio)
                    tag = "COMPRESSION" if ratio > 1.5 else ("marginal" if ratio > 1.2 else "none")
                    print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM K≈{k_perm:.1f} → ratio={ratio:.2f} [{tag}]")
                else:
                    if q_target > max(perm_qs):
                        print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM never reaches → ratio>max [COMPRESSION]")
                        ratios.append(float("inf"))
                    else:
                        print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): below PERM range")
            finite = [r for r in ratios if np.isfinite(r)]
            if finite:
                print(f"  {kind} mean finite ratio: {np.mean(finite):.2f}")

    # ============================================================
    # 157B: BUCKET SEMANTIC COHERENCE
    # ============================================================
    bucket_metrics = ["bucket_purity_mean", "bucket_entropy_mean", "bucket_label_mi"]
    bkt_agg = aggregate_seeds([bkt0, bkt1], bucket_metrics + ["n_buckets_active"])

    print("\n" + "=" * 90)
    print("INC-0157B: BUCKET SEMANTIC COHERENCE (mean ± std over seeds 0,1)")
    print("=" * 90)

    for metric in bucket_metrics:
        short = metric.replace("bucket_", "").replace("_mean", "")
        direction = "↑ better" if metric in ("bucket_purity_mean", "bucket_label_mi") else "↓ better"
        print(f"\n--- {metric} ({direction}) ---")
        header = f"{'K':>5}  {'BASE_ORIG':>18}  {'BASE_PERM':>18}  {'ΔORIG':>8}  {'TRANS_ORIG':>18}  {'TRANS_PERM':>18}  {'ΔORIG':>8}"
        print(header)
        print("-" * 105)

        all_ks = sorted(set(k for rid in bkt_agg for _, k, _ in [parse_route_id(rid)]))

        for k in all_ks:
            row = f"{k:>5}"
            for kind in ["BASE", "TRANS"]:
                orig_rid = f"{kind}_K{k}_ORIG"
                perm_rid = f"{kind}_K{k}_PERM"
                om, pm = None, None
                if orig_rid in bkt_agg and metric in bkt_agg[orig_rid]:
                    om = bkt_agg[orig_rid][metric][0]
                    os_val = bkt_agg[orig_rid][metric][1]
                    row += f"  {om:>7.4f}±{os_val:>6.4f}"
                else:
                    row += f"  {'---':>18}"
                if perm_rid in bkt_agg and metric in bkt_agg[perm_rid]:
                    pm = bkt_agg[perm_rid][metric][0]
                    ps_val = bkt_agg[perm_rid][metric][1]
                    row += f"  {pm:>7.4f}±{ps_val:>6.4f}"
                else:
                    row += f"  {'---':>18}"
                if om is not None and pm is not None:
                    delta = om - pm
                    row += f"  {delta:>+8.4f}"
                else:
                    row += f"  {'---':>8}"
            print(row)

    # Purity advantage summary
    print("\n" + "=" * 90)
    print("INC-0157B: PURITY ADVANTAGE SUMMARY")
    print("=" * 90)
    print(f"{'K':>5}  {'series':>12}  {'purity_ORIG':>12}  {'purity_PERM':>12}  {'ratio':>8}  {'stable':>8}")
    print("-" * 65)
    for kind in ["BASE", "TRANS"]:
        for k in sorted(set(k for rid in bkt_agg for _, k2, _ in [parse_route_id(rid)] if k2)):
            orig_rid = f"{kind}_K{k}_ORIG"
            perm_rid = f"{kind}_K{k}_PERM"
            if orig_rid not in bkt_agg or perm_rid not in bkt_agg:
                continue
            if "bucket_purity_mean" not in bkt_agg[orig_rid]:
                continue
            om, os_val, ovals = bkt_agg[orig_rid]["bucket_purity_mean"]
            pm, ps_val, pvals = bkt_agg[perm_rid]["bucket_purity_mean"]
            ratio = om / pm if pm > 1e-12 else float('inf')
            # stable = both seeds show ORIG > PERM
            stable = all(ov > pv for ov, pv in zip(ovals, pvals)) if len(ovals) == len(pvals) else False
            print(f"{k:>5}  {kind:>12}  {om:>12.4f}  {pm:>12.4f}  {ratio:>8.3f}  {'YES' if stable else 'no':>8}")

    # Confirm input_transform bug fix is active
    print("\n" + "=" * 90)
    print("INPUT_TRANSFORM BUG FIX VERIFICATION")
    print("=" * 90)
    # Check that PERM bucket coherence differs from ORIG
    for seed_idx, bd in enumerate([bkt0, bkt1]):
        perm_vals = [bd[rid]["bucket_purity_mean"] for rid in bd if "PERM" in rid]
        orig_vals = [bd[rid]["bucket_purity_mean"] for rid in bd if "ORIG" in rid]
        perm_mean = np.mean(perm_vals)
        orig_mean = np.mean(orig_vals)
        diff = orig_mean - perm_mean
        print(f"Seed {seed_idx}: mean_purity ORIG={orig_mean:.4f} PERM={perm_mean:.4f} Δ={diff:+.4f}")
        if abs(diff) < 0.001:
            print(f"  WARNING: near-zero difference — input_transform may not be applied!")
        else:
            print(f"  OK: ORIG/PERM differ — input_transform is active")

    # ============================================================
    # DECISION
    # ============================================================
    print("\n" + "=" * 90)
    print("DECISION CRITERIA")
    print("=" * 90)
    print("SUCCESS CONDITIONS:")
    print("  157A: label_indicator_lowfreq_max ORIG/PERM ≥ 1.3 at both seeds")
    print("  157A: true_margin compression ratio > 1.5 at K ≤ 25, stable across seeds")
    print("  157B: purity ORIG > PERM at all K, stable across seeds")
    print("  157B: entropy ORIG < PERM at all K, stable across seeds")
    print()
    print("FALSIFICATION:")
    print("  Spectral ratios vanish under multi-seed averaging")
    print("  Bucket coherence shows no ORIG advantage")


if __name__ == "__main__":
    main()
