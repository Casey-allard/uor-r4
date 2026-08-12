#!/usr/bin/env python3
"""INC-0158 analysis — 4-seed finalize of bucket coherence + spectral compression.

Aggregates seeds 0, 1, 2, 3 data for:
  A: spectral compression (label_indicator_lowfreq_max, true_margin_lowfreq_energy)
  B: bucket semantic coherence (purity, entropy)
"""
import json
import sys
from collections import defaultdict
from typing import Dict, List

import numpy as np

# --- File paths (4 seeds) ---
SPECTRAL_FILES = [
    "results/analysis/inc0156_spectral_compression_screen.json",   # seed 0
    "results/analysis/inc0157_spectral_signal_seed1.json",         # seed 1
    "results/analysis/inc0158_spectral_signal_seed2.json",         # seed 2
    "results/analysis/inc0158_spectral_signal_seed3.json",         # seed 3
]
RESIDUAL_FILES = [
    "results/analysis/inc0156_residual_compression_screen.json",   # seed 0
    "results/analysis/inc0157_residual_seed1.json",                # seed 1
    "results/analysis/inc0158_residual_seed2.json",                # seed 2
    "results/analysis/inc0158_residual_seed3.json",                # seed 3
]
BUCKET_FILES = [
    "results/analysis/inc0157_bucket_coherence_seed0.json",        # seed 0
    "results/analysis/inc0157_bucket_coherence_seed1.json",        # seed 1
    "results/analysis/inc0158_bucket_coherence_seed2.json",        # seed 2
    "results/analysis/inc0158_bucket_coherence_seed3.json",        # seed 3
]


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
                result[rid][mk] = (float(np.mean(vals)), float(np.std(vals)),
                                   float(np.std(vals) / np.sqrt(len(vals))), vals)
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
    # Load all 4 seeds
    spec_data = []
    for sf, rf in zip(SPECTRAL_FILES, RESIDUAL_FILES):
        s = load_metrics(sf)
        r = load_metrics(rf)
        for rid in r:
            if rid in s:
                s[rid].update(r[rid])
        spec_data.append(s)

    bkt_data = [load_metrics(bf) for bf in BUCKET_FILES]

    n_seeds = len(spec_data)
    sem_label = f"mean ± std [SEM] over {n_seeds} seeds"

    # ============================================================
    # A: 4-SEED SPECTRAL COMPRESSION
    # ============================================================
    spectral_metrics = ["label_indicator_lowfreq_max", "true_margin_lowfreq_energy"]
    agg = aggregate_seeds(spec_data, spectral_metrics)

    print("=" * 110)
    print(f"INC-0158A: 4-SEED SPECTRAL QUALITY ({sem_label})")
    print("=" * 110)

    for metric in spectral_metrics:
        print(f"\n--- {metric} ---")
        header = (f"{'K':>5}  {'BASE_ORIG':>22}  {'BASE_PERM':>22}  {'ratio':>8}"
                  f"  {'TRANS_ORIG':>22}  {'TRANS_PERM':>22}  {'ratio':>8}")
        print(header)
        print("-" * 125)

        all_ks = sorted(set(k for rid in agg for _, k, _ in [parse_route_id(rid)]))

        for k in all_ks:
            row = f"{k:>5}"
            for kind in ["BASE", "TRANS"]:
                orig_rid = f"{kind}_K{k}_ORIG"
                perm_rid = f"{kind}_K{k}_PERM"
                for rid in [orig_rid, perm_rid]:
                    if rid in agg and metric in agg[rid]:
                        m, s, sem, _ = agg[rid][metric]
                        row += f"  {m:>7.4f}±{s:.4f}[{sem:.4f}]"
                    else:
                        row += f"  {'---':>22}"
                if (orig_rid in agg and perm_rid in agg and
                        metric in agg[orig_rid] and metric in agg[perm_rid]):
                    ratio = (agg[orig_rid][metric][0] / agg[perm_rid][metric][0]
                             if agg[perm_rid][metric][0] > 1e-12 else float('inf'))
                    row += f"  {ratio:>8.3f}"
                else:
                    row += f"  {'---':>8}"
            print(row)

    # label_indicator_lowfreq_max per-seed detail
    print("\n" + "=" * 110)
    print("INC-0158A: LABEL_INDICATOR_LOWFREQ_MAX — PER-SEED RATIOS")
    print("=" * 110)
    metric = "label_indicator_lowfreq_max"
    for seed_idx, sd in enumerate(spec_data):
        orig_vals = [sd.get(f"{k}_{kk}_ORIG", {}).get(metric)
                     for k in ["BASE", "TRANS"]
                     for kk in [f"K{v}" for v in [4, 9, 16, 25, 50, 75, 100]]]
        perm_vals = [sd.get(f"{k}_{kk}_PERM", {}).get(metric)
                     for k in ["BASE", "TRANS"]
                     for kk in [f"K{v}" for v in [4, 9, 16, 25, 50, 75, 100]]]
        # Just print BASE K=100 and TRANS K=100 for compactness
        for kind in ["BASE", "TRANS"]:
            for kvv in [100]:
                orig_rid = f"{kind}_K{kvv}_ORIG"
                perm_rid = f"{kind}_K{kvv}_PERM"
                ov = sd.get(orig_rid, {}).get(metric)
                pv = sd.get(perm_rid, {}).get(metric)
                if ov is not None and pv is not None and pv > 1e-12:
                    print(f"  Seed {seed_idx}: {kind} K={kvv}: ORIG={ov:.4f} PERM={pv:.4f} ratio={ov/pv:.3f}")

    # Compression analysis for true_margin_lowfreq_energy per seed
    print("\n" + "=" * 110)
    print("INC-0158A: TRUE_MARGIN COMPRESSION RATIOS (per seed)")
    print("=" * 110)
    all_seed_finite_ratios = []
    for seed_idx, sd in enumerate(spec_data):
        print(f"\n--- Seed {seed_idx} ---")
        for kind in ["BASE"]:
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
            seed_ratios = []
            for k_orig, q_target in orig_pts:
                if k_orig > 25:
                    continue   # focus on low-K compression
                k_perm = interpolate_k_at_quality(perm_ks, perm_qs, q_target)
                if k_perm is not None:
                    ratio = k_perm / k_orig
                    seed_ratios.append(ratio)
                    tag = "COMPRESSION" if ratio > 1.5 else ("marginal" if ratio > 1.2 else "none")
                    print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM K≈{k_perm:.1f} → ratio={ratio:.2f} [{tag}]")
                else:
                    if q_target > max(perm_qs):
                        print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM never reaches → ratio>max [COMPRESSION]")
                        seed_ratios.append(float("inf"))
            finite = [r for r in seed_ratios if np.isfinite(r)]
            if finite:
                print(f"  {kind} mean finite ratio: {np.mean(finite):.2f}")
                all_seed_finite_ratios.append(np.mean(finite))

    if all_seed_finite_ratios:
        print(f"\nCross-seed mean finite ratio (BASE, K≤25): {np.mean(all_seed_finite_ratios):.2f} ± {np.std(all_seed_finite_ratios):.2f}")

    # ============================================================
    # B: 4-SEED BUCKET SEMANTIC COHERENCE
    # ============================================================
    bucket_metrics = ["bucket_purity_mean", "bucket_entropy_mean"]
    bkt_agg = aggregate_seeds(bkt_data, bucket_metrics + ["n_buckets_active"])

    print("\n" + "=" * 110)
    print(f"INC-0158B: 4-SEED BUCKET SEMANTIC COHERENCE ({sem_label})")
    print("=" * 110)

    for metric in bucket_metrics:
        direction = "↑ better" if "purity" in metric else "↓ better"
        print(f"\n--- {metric} ({direction}) ---")
        header = (f"{'K':>5}  {'BASE_ORIG':>22}  {'BASE_PERM':>22}  {'ΔORIG':>8}"
                  f"  {'TRANS_ORIG':>22}  {'TRANS_PERM':>22}  {'ΔORIG':>8}")
        print(header)
        print("-" * 125)

        all_ks = sorted(set(k for rid in bkt_agg for _, k, _ in [parse_route_id(rid)]))

        for k in all_ks:
            row = f"{k:>5}"
            for kind in ["BASE", "TRANS"]:
                orig_rid = f"{kind}_K{k}_ORIG"
                perm_rid = f"{kind}_K{k}_PERM"
                om, pm = None, None
                for rid, label in [(orig_rid, "orig"), (perm_rid, "perm")]:
                    if rid in bkt_agg and metric in bkt_agg[rid]:
                        m, s, sem, _ = bkt_agg[rid][metric]
                        row += f"  {m:>7.4f}±{s:.4f}[{sem:.4f}]"
                        if label == "orig":
                            om = m
                        else:
                            pm = m
                    else:
                        row += f"  {'---':>22}"
                if om is not None and pm is not None:
                    delta = om - pm
                    row += f"  {delta:>+8.4f}"
                else:
                    row += f"  {'---':>8}"
            print(row)

    # Purity advantage summary with per-seed stability
    print("\n" + "=" * 110)
    print("INC-0158B: PURITY ADVANTAGE SUMMARY (4-seed)")
    print("=" * 110)
    print(f"{'K':>5}  {'series':>8}  {'pur_ORIG':>10}  {'pur_PERM':>10}  {'ratio':>8}  {'SEM_O':>7}  {'SEM_P':>7}  {'all_seeds':>10}")
    print("-" * 80)

    n_pass = 0
    n_total = 0
    trans_k100_ratio = None

    for kind in ["BASE", "TRANS"]:
        ks_for_kind = sorted(set(k2 for rid in bkt_agg for _, k2, _ in [parse_route_id(rid)]
                                 if any(f"{kind}_K{k2}" in r for r in bkt_agg)))
        for k in ks_for_kind:
            orig_rid = f"{kind}_K{k}_ORIG"
            perm_rid = f"{kind}_K{k}_PERM"
            if orig_rid not in bkt_agg or perm_rid not in bkt_agg:
                continue
            if "bucket_purity_mean" not in bkt_agg[orig_rid]:
                continue
            om, os_val, o_sem, ovals = bkt_agg[orig_rid]["bucket_purity_mean"]
            pm, ps_val, p_sem, pvals = bkt_agg[perm_rid]["bucket_purity_mean"]
            ratio = om / pm if pm > 1e-12 else float('inf')
            stable = all(ov > pv for ov, pv in zip(ovals, pvals))
            n_total += 1
            if om > pm:
                n_pass += 1
            if kind == "TRANS" and k == 100:
                trans_k100_ratio = ratio
            print(f"{k:>5}  {kind:>8}  {om:>10.4f}  {pm:>10.4f}  {ratio:>8.3f}  {o_sem:>7.4f}  {p_sem:>7.4f}  {'YES' if stable else 'no':>10}")

    # Entropy advantage summary
    print(f"\n{'K':>5}  {'series':>8}  {'ent_ORIG':>10}  {'ent_PERM':>10}  {'Δ(O-P)':>10}  {'all_seeds':>10}")
    print("-" * 60)
    for kind in ["BASE", "TRANS"]:
        ks_for_kind = sorted(set(k2 for rid in bkt_agg for _, k2, _ in [parse_route_id(rid)]
                                 if any(f"{kind}_K{k2}" in r for r in bkt_agg)))
        for k in ks_for_kind:
            orig_rid = f"{kind}_K{k}_ORIG"
            perm_rid = f"{kind}_K{k}_PERM"
            if orig_rid not in bkt_agg or perm_rid not in bkt_agg:
                continue
            if "bucket_entropy_mean" not in bkt_agg[orig_rid]:
                continue
            om, _, _, ovals = bkt_agg[orig_rid]["bucket_entropy_mean"]
            pm, _, _, pvals = bkt_agg[perm_rid]["bucket_entropy_mean"]
            stable = all(ov < pv for ov, pv in zip(ovals, pvals))
            print(f"{k:>5}  {kind:>8}  {om:>10.4f}  {pm:>10.4f}  {om - pm:>+10.4f}  {'YES' if stable else 'no':>10}")

    # input_transform bug fix verification
    print("\n" + "=" * 110)
    print("INPUT_TRANSFORM BUG FIX VERIFICATION (4 seeds)")
    print("=" * 110)
    for seed_idx, bd in enumerate(bkt_data):
        perm_vals = [bd[rid]["bucket_purity_mean"] for rid in bd if "PERM" in rid]
        orig_vals = [bd[rid]["bucket_purity_mean"] for rid in bd if "ORIG" in rid]
        perm_mean = np.mean(perm_vals)
        orig_mean = np.mean(orig_vals)
        diff = orig_mean - perm_mean
        status = "OK" if abs(diff) > 0.001 else "WARNING"
        print(f"  Seed {seed_idx}: mean_purity ORIG={orig_mean:.4f} PERM={perm_mean:.4f} Δ={diff:+.4f} [{status}]")

    # ============================================================
    # DECISION EVALUATION
    # ============================================================
    print("\n" + "=" * 110)
    print("DECISION EVALUATION")
    print("=" * 110)

    # Check label ratio
    label_ratios = []
    for kind in ["BASE", "TRANS"]:
        for kv in [100]:
            orig_rid = f"{kind}_K{kv}_ORIG"
            perm_rid = f"{kind}_K{kv}_PERM"
            if (orig_rid in agg and perm_rid in agg and
                    "label_indicator_lowfreq_max" in agg[orig_rid] and
                    "label_indicator_lowfreq_max" in agg[perm_rid]):
                om = agg[orig_rid]["label_indicator_lowfreq_max"][0]
                pm = agg[perm_rid]["label_indicator_lowfreq_max"][0]
                if pm > 1e-12:
                    label_ratios.append(om / pm)

    mean_label_ratio = np.mean(label_ratios) if label_ratios else 0
    purity_pass_rate = n_pass / n_total if n_total > 0 else 0

    print(f"  label_indicator_lowfreq_max mean ratio (K=100): {mean_label_ratio:.3f}"
          f"  {'PASS (≥1.2)' if mean_label_ratio >= 1.2 else 'FAIL (<1.2)'}")
    print(f"  Purity pass rate (ORIG > PERM in 4-seed mean): {n_pass}/{n_total}"
          f" = {purity_pass_rate:.0%}  {'PASS (≥90%)' if purity_pass_rate >= 0.9 else 'FAIL (<90%)'}")
    print(f"  TRANS K=100 purity ratio: {trans_k100_ratio:.3f}"
          f"  {'PASS (≥1.5)' if trans_k100_ratio and trans_k100_ratio >= 1.5 else 'FAIL (<1.5)'}"
          if trans_k100_ratio else "  TRANS K=100: no data")

    all_pass = (mean_label_ratio >= 1.2 and purity_pass_rate >= 0.9
                and trans_k100_ratio is not None and trans_k100_ratio >= 1.5)
    print(f"\n  OVERALL: {'KEEP — all criteria met' if all_pass else 'See individual criteria above'}")


if __name__ == "__main__":
    main()
