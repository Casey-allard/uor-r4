#!/usr/bin/env python3
"""Truncation stability probe for frozen H(x) bridge definition."""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)  # Frozen v1
CHANNELS = ("Y11s", "Y20", "su2_trace", "phase_vel")


def h_scaled_series(
    n_max: int,
    base: int,
    zeros: Sequence[float],
    x_step: int,
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
) -> Dict[str, Sequence[float]]:
    points = s4.manifold_points(n_max, zeros, u_mode, zero_kernel, kernel_scale)
    idx = [n for n in range(2, n_max + 1) if s4.gcd(n, base) == 1]
    ch = s4.candidate_series_for_base(points, idx)
    m = min(len(ch[k]) for k in CHANNELS)
    idx = idx[:m]
    g_by_n = [0.0] * (n_max + 1)
    for j, n in enumerate(idx):
        g_by_n[n] = (
            WEIGHTS[0] * ch["Y11s"][j]
            + WEIGHTS[1] * ch["Y20"][j]
            + WEIGHTS[2] * ch["su2_trace"][j]
            + WEIGHTS[3] * ch["phase_vel"][j]
        )
    h = [0.0] * (n_max + 1)
    for n in range(2, n_max + 1):
        h[n] = h[n - 1] + g_by_n[n]
    xs = list(range(max(2, x_step), n_max + 1, x_step))
    hs = [h[x] / math.sqrt(x) for x in xs]
    return {"x": xs, "h_scaled": hs}


def max_abs_diff(a: Sequence[float], b: Sequence[float]) -> float:
    return max(abs(x - y) for x, y in zip(a, b)) if a and len(a) == len(b) else 0.0


def l2_diff(a: Sequence[float], b: Sequence[float]) -> float:
    if not a or len(a) != len(b):
        return 0.0
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)) / len(a))


def main() -> None:
    ap = argparse.ArgumentParser(description="H(x) truncation stability vs zero cutoff")
    ap.add_argument("--n-max", type=int, default=300000)
    ap.add_argument("--base", type=int, default=210)
    ap.add_argument("--x-step", type=int, default=1000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--zero-limits", type=str, default="32,64,96,128,192,256")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--output", type=str, default="research/output/hx_truncation_probe.json")
    args = ap.parse_args()

    all_zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    limits = [int(x.strip()) for x in args.zero_limits.split(",") if x.strip()]
    limits = sorted(set(l for l in limits if l > 0 and l <= len(all_zeros)))
    if len(limits) < 2:
        raise ValueError("need at least two valid zero limits")

    psi_tab = hx.psi_exact_table(args.n_max)
    rows = []
    hs_map = {}
    for m in limits:
        zeros = all_zeros[:m]
        ser = h_scaled_series(
            args.n_max,
            args.base,
            zeros,
            args.x_step,
            args.u_mode,
            args.zero_kernel,
            args.kernel_scale,
        )
        xs = ser["x"]
        hs = ser["h_scaled"]
        es = [(psi_tab[x] - x) / math.sqrt(x) for x in xs]
        c = hx.corr(hs, es)
        a, b, rmse = hx.fit_linear(hs, es)
        rows.append(
            {
                "zero_limit": m,
                "corr_hscaled_vs_escaled": c,
                "slope": a,
                "intercept": b,
                "rmse": rmse,
            }
        )
        hs_map[m] = hs

    pairwise = []
    for i in range(len(limits) - 1):
        a = limits[i]
        b = limits[i + 1]
        da = max_abs_diff(hs_map[a], hs_map[b])
        dl2 = l2_diff(hs_map[a], hs_map[b])
        pairwise.append({"from_m": a, "to_m": b, "max_abs_diff": da, "l2_diff": dl2})

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "base": args.base,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "weights": WEIGHTS,
            "zero_limits": limits,
        },
        "rows": rows,
        "pairwise_diffs": pairwise,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# H(x) Truncation Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- base: {args.base}\n")
        f.write(f"- n_max: {args.n_max}\n")
        f.write(f"- u_mode: {args.u_mode}\n")
        f.write(f"- zero_kernel: {args.zero_kernel}\n")
        f.write(f"- kernel_scale: {args.kernel_scale}\n")
        f.write(f"- zero limits: {limits}\n\n")
        f.write("## Bridge Metrics by Zero Limit\n\n")
        for r in rows:
            f.write(
                f"- M={r['zero_limit']} corr={r['corr_hscaled_vs_escaled']:.6f} "
                f"slope={r['slope']:.6f} rmse={r['rmse']:.6f}\n"
            )
        f.write("\n## Consecutive Truncation Differences\n\n")
        for p in pairwise:
            f.write(
                f"- {p['from_m']} -> {p['to_m']}: max_abs_diff={p['max_abs_diff']:.6f} "
                f"l2_diff={p['l2_diff']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
