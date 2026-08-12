#!/usr/bin/env python3
"""Pair-correlation and spacing diagnostics for zeta zeros.

Uses downloaded zero lists and computes:
- normalized nearest-neighbor spacing histogram
- Wigner-surmise fit error
- pair-correlation function estimate R2(s)
- sine-kernel fit error
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple


def load_zeros(path: str, limit: int) -> List[float]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if isinstance(data, dict):
        if "zeros" in data:
            zeros = [float(x) for x in data["zeros"]]
        elif "positive_zeros" in data:
            zeros = [float(x) for x in data["positive_zeros"]]
        else:
            zeros = []
    elif isinstance(data, list):
        zeros = [float(x) for x in data]
    else:
        zeros = []
    zeros = sorted(z for z in zeros if z > 0)
    if limit > 0:
        zeros = zeros[:limit]
    return zeros


def normalized_spacings(zeros: Sequence[float]) -> List[float]:
    if len(zeros) < 2:
        return []
    gaps = [zeros[i + 1] - zeros[i] for i in range(len(zeros) - 1)]
    m = sum(gaps) / len(gaps)
    if m <= 0:
        return []
    return [g / m for g in gaps]


def histogram(values: Sequence[float], bins: int, lo: float, hi: float) -> Tuple[List[float], List[float]]:
    counts = [0] * bins
    width = (hi - lo) / bins
    for v in values:
        if v < lo or v >= hi:
            continue
        idx = int((v - lo) / width)
        if 0 <= idx < bins:
            counts[idx] += 1
    total = sum(counts)
    density = []
    centers = []
    for i, c in enumerate(counts):
        centers.append(lo + (i + 0.5) * width)
        density.append((c / total) / width if total else 0.0)
    return centers, density


def wigner_surmise_density(s: float) -> float:
    # GOE-like nearest-neighbor spacing benchmark.
    return (math.pi / 2.0) * s * math.exp(-math.pi * s * s / 4.0)


def mse(a: Sequence[float], b: Sequence[float]) -> float:
    if not a:
        return 0.0
    return sum((x - y) ** 2 for x, y in zip(a, b)) / len(a)


def pair_corr_estimate(spacings: Sequence[float], s_max: float, ds: float) -> Tuple[List[float], List[float]]:
    # Build positions from normalized spacings: x_n = sum_{k < n} s_k
    x = [0.0]
    for s in spacings:
        x.append(x[-1] + s)
    n = len(x)
    if n < 3:
        return [], []

    grid = []
    vals = []
    m_bins = max(1, int(s_max / ds))
    counts = [0] * m_bins

    # O(n^2) but fine for a few thousand points.
    for i in range(n):
        xi = x[i]
        for j in range(i + 1, n):
            d = x[j] - xi
            if d >= s_max:
                break
            idx = int(d / ds)
            if 0 <= idx < m_bins:
                counts[idx] += 2  # symmetric pairs

    # Normalize by n and bin width to approximate density.
    for k in range(m_bins):
        s = (k + 0.5) * ds
        grid.append(s)
        vals.append(counts[k] / (n * ds))
    return grid, vals


def sine_kernel_pair_corr(s: float) -> float:
    if abs(s) < 1e-12:
        return 0.0
    t = math.sin(math.pi * s) / (math.pi * s)
    return 1.0 - t * t


def write_pair_svg(s_grid: Sequence[float], r2: Sequence[float], out_path: str) -> None:
    if not s_grid:
        return
    w, h = 920, 460
    pad = 50
    x0, x1 = min(s_grid), max(s_grid)
    y_obs_max = max(r2)
    y_ref_max = max(sine_kernel_pair_corr(s) for s in s_grid)
    y1 = max(1.0, y_obs_max, y_ref_max)

    def px(s: float) -> float:
        return pad + (s - x0) / max(1e-12, (x1 - x0)) * (w - 2 * pad)

    def py(v: float) -> float:
        return h - pad - v / y1 * (h - 2 * pad)

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n')
        f.write(f'<rect x="0" y="0" width="{w}" height="{h}" fill="#fafaf8"/>\n')
        f.write('<text x="20" y="28" font-size="18" fill="#111827">Pair Correlation R2(s)</text>\n')
        f.write(f'<line x1="{pad}" y1="{h-pad}" x2="{w-pad}" y2="{h-pad}" stroke="#9ca3af"/>\n')

        obs_pts = " ".join(f"{px(s):.2f},{py(v):.2f}" for s, v in zip(s_grid, r2))
        ref_pts = " ".join(f"{px(s):.2f},{py(sine_kernel_pair_corr(s)):.2f}" for s in s_grid)
        f.write(f'<polyline fill="none" stroke="#1f77b4" stroke-width="2" points="{obs_pts}"/>\n')
        f.write(f'<polyline fill="none" stroke="#d97706" stroke-width="2" points="{ref_pts}"/>\n')
        f.write('<text x="20" y="50" font-size="12" fill="#1f77b4">observed</text>\n')
        f.write('<text x="20" y="66" font-size="12" fill="#d97706">sine-kernel reference</text>\n')
        f.write("</svg>\n")


def quantile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    idx = int(max(0, min(len(v) - 1, round(q * (len(v) - 1)))))
    return v[idx]


def bootstrap_ci(
    spacings: Sequence[float],
    spacing_bins: int,
    spacing_max: float,
    pair_s_max: float,
    pair_ds: float,
    trials: int,
    seed: int,
    sample_size: int,
) -> Dict[str, object]:
    if trials <= 0 or not spacings:
        return {"trials": 0}

    rng = random.Random(seed)
    n = len(spacings)
    k = sample_size if sample_size > 0 else n
    k = min(k, n)

    wigner_mses = []
    pair_mses = []
    for _ in range(trials):
        boot = [spacings[rng.randrange(0, n)] for _ in range(k)]
        c, d = histogram(boot, bins=max(10, spacing_bins), lo=0.0, hi=spacing_max)
        wigner = [wigner_surmise_density(x) for x in c]
        wigner_mses.append(mse(d, wigner))

        s_grid, r2 = pair_corr_estimate(boot, s_max=pair_s_max, ds=pair_ds)
        ref = [sine_kernel_pair_corr(x) for x in s_grid]
        pair_mses.append(mse(r2, ref))

    return {
        "trials": trials,
        "seed": seed,
        "sample_size": k,
        "wigner_mse_ci90": [quantile(wigner_mses, 0.05), quantile(wigner_mses, 0.95)],
        "pair_mse_ci90": [quantile(pair_mses, 0.05), quantile(pair_mses, 0.95)],
        "wigner_mse_mean": sum(wigner_mses) / len(wigner_mses),
        "pair_mse_mean": sum(pair_mses) / len(pair_mses),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Pair-correlation probe for zeta zeros")
    parser.add_argument("--zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    parser.add_argument("--limit", type=int, default=4000)
    parser.add_argument("--spacing-bins", type=int, default=50)
    parser.add_argument("--spacing-max", type=float, default=4.0)
    parser.add_argument("--pair-s-max", type=float, default=6.0)
    parser.add_argument("--pair-ds", type=float, default=0.08)
    parser.add_argument("--bootstrap-trials", type=int, default=40)
    parser.add_argument("--bootstrap-seed", type=int, default=20260216)
    parser.add_argument("--bootstrap-sample-size", type=int, default=1500)
    parser.add_argument("--output", type=str, default="research/output/pair_correlation_probe.json")
    args = parser.parse_args()

    zeros = load_zeros(args.zeros_file, args.limit)
    s = normalized_spacings(zeros)

    c, d = histogram(s, bins=max(10, args.spacing_bins), lo=0.0, hi=args.spacing_max)
    wigner = [wigner_surmise_density(x) for x in c]
    spacing_mse = mse(d, wigner)

    s_grid, r2 = pair_corr_estimate(s, s_max=args.pair_s_max, ds=args.pair_ds)
    ref = [sine_kernel_pair_corr(x) for x in s_grid]
    pair_mse = mse(r2, ref)
    boot = bootstrap_ci(
        s,
        spacing_bins=args.spacing_bins,
        spacing_max=args.spacing_max,
        pair_s_max=args.pair_s_max,
        pair_ds=args.pair_ds,
        trials=max(0, args.bootstrap_trials),
        seed=args.bootstrap_seed,
        sample_size=max(0, args.bootstrap_sample_size),
    )

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeros_file": args.zeros_file,
            "zero_count": len(zeros),
            "spacing_bins": args.spacing_bins,
            "spacing_max": args.spacing_max,
            "pair_s_max": args.pair_s_max,
            "pair_ds": args.pair_ds,
        },
        "spacing_fit": {
            "wigner_mse": spacing_mse,
            "hist_centers": c,
            "hist_density": d,
            "wigner_density": wigner,
            "mean_spacing": sum(s) / max(1, len(s)) if s else 0.0,
        },
        "pair_correlation_fit": {
            "s_grid": s_grid,
            "observed_r2": r2,
            "sine_kernel_r2": ref,
            "mse": pair_mse,
        },
        "bootstrap": boot,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    svg = args.output.replace(".json", ".svg")
    write_pair_svg(s_grid, r2, svg)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Pair Correlation Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Zero count used: {len(zeros)}\n\n")
        f.write(f"- Wigner spacing MSE: {spacing_mse:.6f}\n")
        f.write(f"- Sine-kernel pair-correlation MSE: {pair_mse:.6f}\n")
        if boot.get("trials", 0) > 0:
            wci = boot["wigner_mse_ci90"]
            pci = boot["pair_mse_ci90"]
            f.write(f"- Wigner MSE CI90: [{wci[0]:.6f}, {wci[1]:.6f}]\n")
            f.write(f"- Pair-correlation MSE CI90: [{pci[0]:.6f}, {pci[1]:.6f}]\n")
        f.write("\nLower MSE indicates closer match to the reference law.\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print(f"wrote: {svg}")
    print(f"wigner_mse: {spacing_mse:.6f}")
    print(f"pair_mse: {pair_mse:.6f}")


if __name__ == "__main__":
    main()
