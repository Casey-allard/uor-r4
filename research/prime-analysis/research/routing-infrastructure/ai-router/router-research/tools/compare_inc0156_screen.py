#!/usr/bin/env python3
"""INC-0156 results analysis — spectral compression via equal-quality routing cost.

Reads spectral probe output and INC-0155 sweep data.
Computes spectral quality vs K curves and routing compression ratios.
"""
import json
import sys
from typing import Dict, List, Tuple

import numpy as np


QUALITY_METRICS = [
    "sector_lowfreq_energy",
    "label_indicator_lowfreq_max",
    "label_onehot_lowfreq_energy",
    "shell_lowfreq_energy",
    "true_margin_lowfreq_energy",
    "error_indicator_lowfreq_energy",
]

COST_METRICS_FILE = "results/analysis/inc0155_routing_compression_screen.json"
SPECTRAL_FILE = "results/analysis/inc0156_spectral_compression_screen.json"
RESIDUAL_FILE = "results/analysis/inc0156_residual_compression_screen.json"


def parse_route_id(rid: str):
    """Parse route_id like BASE_K25_ORIG -> (kind, K, transform)."""
    parts = rid.split("_")
    kind = parts[0]
    k_val = int(parts[1].replace("K", ""))
    transform = parts[2]
    return kind, k_val, transform


def load_spectral_data(path: str) -> Dict[str, Dict]:
    with open(path) as f:
        d = json.load(f)
    out = {}
    for entry in d["results"]:
        out[entry["route_id"]] = entry["metrics"]
    return out


def load_cost_data(path: str) -> Dict[str, Dict]:
    with open(path) as f:
        d = json.load(f)
    out = {}
    for rid, seeds in d["results"].items():
        out[rid] = seeds[0]["metrics"]
    return out


def build_series(spectral: Dict, cost: Dict):
    """Build per-series data: {series_key: [(K, spectral_metrics, cost_metrics)]}."""
    series = {}
    for rid, sm in spectral.items():
        kind, k_val, transform = parse_route_id(rid)
        key = f"{kind}_{transform}"
        cm = cost.get(rid, {})
        if key not in series:
            series[key] = []
        series[key].append((k_val, sm, cm))
    for key in series:
        series[key].sort(key=lambda x: x[0])
    return series


def interpolate_k_at_quality(ks, qualities, target_quality):
    """Log-linear interpolation: find K where quality = target.

    Quality generally DECREASES with K (more sectors = harder to be smooth).
    Returns None if target is outside range.
    """
    ks = np.array(ks, dtype=float)
    qs = np.array(qualities, dtype=float)

    if len(ks) < 2:
        return None

    # Quality may be non-monotonic; use linear interp on log(K)
    log_ks = np.log(ks)

    # Check if target is within range
    q_min, q_max = np.min(qs), np.max(qs)
    if target_quality < q_min or target_quality > q_max:
        return None

    # Find interpolation segment(s) — take the first valid crossing
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
    spectral = load_spectral_data(SPECTRAL_FILE)
    # Merge residual probe metrics into spectral dict
    try:
        residual = load_spectral_data(RESIDUAL_FILE)
        for rid, rm in residual.items():
            if rid in spectral:
                spectral[rid].update(rm)
            else:
                spectral[rid] = rm
    except FileNotFoundError:
        print(f"WARNING: {RESIDUAL_FILE} not found — skipping residual metrics\n")

    cost = load_cost_data(COST_METRICS_FILE)
    series = build_series(spectral, cost)

    all_ks = sorted(set(k for pts in series.values() for k, _, _ in pts))

    # --- Table 1: Spectral Quality vs K ---
    for metric in QUALITY_METRICS:
        print("=" * 80)
        print(f"TABLE: {metric} vs K")
        print("=" * 80)
        header = f"{'K':>5}"
        for key in ["BASE_ORIG", "BASE_PERM", "TRANS_ORIG", "TRANS_PERM"]:
            header += f"  {key:>12}"
        print(header)
        print("-" * 75)

        for k in all_ks:
            row = f"{k:>5}"
            for key in ["BASE_ORIG", "BASE_PERM", "TRANS_ORIG", "TRANS_PERM"]:
                pts = series.get(key, [])
                val = None
                for kk, sm, _ in pts:
                    if kk == k:
                        val = sm.get(metric)
                        break
                if val is not None:
                    row += f"  {val:>12.6f}"
                else:
                    row += f"  {'---':>12}"
            print(row)
        print()

    # --- Table 2: ORIG/PERM ratio at each K ---
    print("=" * 80)
    print("TABLE: Spectral Quality Ratio (ORIG / PERM) at each K")
    print("=" * 80)
    header = f"{'K':>5}"
    for metric in QUALITY_METRICS:
        short = metric.replace("_lowfreq_energy", "_lfe").replace("_lowfreq_max", "_lfm")
        header += f"  {'B_' + short:>16}  {'T_' + short:>16}"
    print(header)
    print("-" * (5 + len(QUALITY_METRICS) * 36))

    for k in all_ks:
        row = f"{k:>5}"
        for metric in QUALITY_METRICS:
            for kind in ["BASE", "TRANS"]:
                orig_val = None
                perm_val = None
                for kk, sm, _ in series.get(f"{kind}_ORIG", []):
                    if kk == k:
                        orig_val = sm.get(metric)
                        break
                for kk, sm, _ in series.get(f"{kind}_PERM", []):
                    if kk == k:
                        perm_val = sm.get(metric)
                        break
                if orig_val is not None and perm_val is not None and perm_val > 1e-12:
                    ratio = orig_val / perm_val
                    row += f"  {ratio:>16.3f}"
                else:
                    row += f"  {'---':>16}"
        print(row)

    # --- Routing Compression Analysis ---
    print()
    print("=" * 80)
    print("SPECTRAL ROUTING COMPRESSION ANALYSIS")
    print("=" * 80)

    for metric in QUALITY_METRICS:
        print(f"\n--- {metric} ---")
        for kind in ["BASE", "TRANS"]:
            orig_pts = series.get(f"{kind}_ORIG", [])
            perm_pts = series.get(f"{kind}_PERM", [])
            if len(orig_pts) < 2 or len(perm_pts) < 2:
                print(f"  {kind}: insufficient data")
                continue

            orig_ks = [k for k, _, _ in orig_pts]
            orig_qs = [sm.get(metric, 0) for _, sm, _ in orig_pts]
            perm_ks = [k for k, _, _ in perm_pts]
            perm_qs = [sm.get(metric, 0) for _, sm, _ in perm_pts]

            print(f"  {kind} ORIG: {[(k, f'{q:.4f}') for k, q in zip(orig_ks, orig_qs)]}")
            print(f"  {kind} PERM: {[(k, f'{q:.4f}') for k, q in zip(perm_ks, perm_qs)]}")

            ratios = []
            for k_orig, q_target in zip(orig_ks, orig_qs):
                k_perm = interpolate_k_at_quality(perm_ks, perm_qs, q_target)
                if k_perm is not None:
                    ratio = k_perm / k_orig
                    ratios.append(ratio)
                    tag = "COMPRESSION" if ratio > 1.5 else ("marginal" if ratio > 1.2 else "none")
                    print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM needs K≈{k_perm:.1f} → ratio={ratio:.2f} [{tag}]")
                else:
                    # Check if ORIG quality is better than all PERM values
                    if q_target > max(perm_qs):
                        print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): PERM never reaches this quality → ratio>max [COMPRESSION]")
                        ratios.append(float("inf"))
                    else:
                        print(f"    Q={q_target:.4f} (ORIG K={k_orig:>3}): below PERM range")

            if ratios:
                finite = [r for r in ratios if np.isfinite(r)]
                if finite:
                    print(f"  {kind} mean compression ratio: {np.mean(finite):.2f} (finite only)")
                inf_count = sum(1 for r in ratios if not np.isfinite(r))
                if inf_count:
                    print(f"  {kind}: {inf_count} quality targets unreachable by PERM")

    # --- Cost metrics from INC-0155 ---
    print()
    print("=" * 80)
    print("ROUTING COST METRICS (from INC-0155)")
    print("=" * 80)
    print(f"{'K':>5}  {'series':>12}  {'sec_ent':>8}  {'sec_pmax':>8}  {'slots':>5}  {'MSE':>10}")
    print("-" * 60)
    for key in ["BASE_ORIG", "BASE_PERM", "TRANS_ORIG", "TRANS_PERM"]:
        for k, sm, cm in series.get(key, []):
            ent = cm.get("sector_entropy", 0)
            pm = cm.get("sector_pmax", 0)
            sl = cm.get("slots_used", 0)
            mse = cm.get("test_mse_after", 0)
            print(f"{k:>5}  {key:>12}  {ent:>8.4f}  {pm:>8.4f}  {sl:>5}  {mse:>10.6f}")

    # --- Decision ---
    print()
    print("=" * 80)
    print("DECISION CRITERIA")
    print("=" * 80)
    print("SUCCESS: spectral compression ratio K_perm/K_orig > 1.5 at any matched quality")
    print("FALSIFIED: spectral quality equally insensitive to routing quality at all K")
    print("REFINE: intermediate — some signal but below 1.5 threshold")


if __name__ == "__main__":
    main()
