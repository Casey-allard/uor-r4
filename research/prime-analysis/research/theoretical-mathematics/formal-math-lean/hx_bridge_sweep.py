#!/usr/bin/env python3
"""Cross-base / cross-scale sweep for H(x) bridge functional."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


def build_weighted_g_series(points, idx_all: Sequence[int], weights: Sequence[float]):
    ch = s4.candidate_series_for_base(points, idx_all)
    subset = ["Y11s", "Y20", "su2_trace", "phase_vel"]
    m = min(len(ch[k]) for k in subset)
    seq_n_all = list(idx_all[:m])
    g_vals_all = [
        (
            weights[0] * ch["Y11s"][j]
            + weights[1] * ch["Y20"][j]
            + weights[2] * ch["su2_trace"][j]
            + weights[3] * ch["phase_vel"][j]
        )
        for j in range(m)
    ]
    return seq_n_all, g_vals_all


def _stream_base_worker(payload):
    base, n_max, zeros, u_mode, zero_kernel, kernel_scale, weights, chunk_n = payload
    seq_n, g_vals = hx.stream_weighted_events(
        n_max=n_max,
        base=base,
        zeros=zeros,
        u_mode=u_mode,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
        weights=weights,
        chunk_n=chunk_n,
    )
    return base, seq_n, g_vals


def eval_base(
    n_max: int,
    base: int,
    seq_n_all: Sequence[int],
    g_vals_all: Sequence[float],
    psi_tab: Sequence[float],
    x_step: int,
) -> Dict[str, float]:
    cut = bisect.bisect_right(seq_n_all, n_max)
    seq_n = seq_n_all[:cut]
    g_vals = g_vals_all[:cut]

    xs = list(range(max(2, x_step), n_max + 1, x_step))
    h_scaled = hx.h_scaled_from_events(seq_n, g_vals, xs)
    e_scaled = [(psi_tab[x] - x) / math.sqrt(x) for x in xs]
    c = hx.corr(h_scaled, e_scaled)
    a, b, rmse = hx.fit_linear(h_scaled, e_scaled)
    fit = [a * v + b for v in h_scaled]
    c_fit = hx.corr(fit, e_scaled)
    return {
        "base": base,
        "corr_hscaled_vs_escaled": c,
        "corr_fitted_vs_escaled": c_fit,
        "slope": a,
        "intercept": b,
        "rmse": rmse,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Sweep H(x) bridge across bases and n_max values")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="120000,200000,300000")
    ap.add_argument("--x-step", type=int, default=1000)
    ap.add_argument("--weights", type=str, default="-1,-1,0,-1")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--streaming", action="store_true", help="compute per-base events with chunked streaming")
    ap.add_argument("--chunk-n", type=int, default=20000, help="streaming chunk size in n")
    ap.add_argument("--jobs", type=int, default=1, help="parallel base workers in streaming mode")
    ap.add_argument("--output", type=str, default="research/output/hx_bridge_sweep.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    weights = [float(x.strip()) for x in args.weights.split(",") if x.strip()]
    if len(weights) != 4:
        raise ValueError("weights must contain exactly 4 values")

    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    rows = []
    n_values = sorted(n_values)
    n_max_global = n_values[-1]
    xs_by_n = {n: list(range(max(2, args.x_step), n + 1, args.x_step)) for n in n_values}
    all_xs = sorted({x for xs in xs_by_n.values() for x in xs})
    psi_at = {x: v for x, v in zip(all_xs, hx.psi_exact_samples(all_xs))}

    if args.streaming:
        payloads = [
            (b, n_max_global, zeros, args.u_mode, args.zero_kernel, args.kernel_scale, weights, args.chunk_n)
            for b in bases
        ]
        base_series = {}
        if args.jobs > 1 and len(payloads) > 1:
            try:
                with concurrent.futures.ProcessPoolExecutor(max_workers=args.jobs) as ex:
                    for b, seq_n, g_vals in ex.map(_stream_base_worker, payloads):
                        base_series[b] = (seq_n, g_vals)
            except (PermissionError, OSError):
                for p in payloads:
                    b, seq_n, g_vals = _stream_base_worker(p)
                    base_series[b] = (seq_n, g_vals)
        else:
            for p in payloads:
                b, seq_n, g_vals = _stream_base_worker(p)
                base_series[b] = (seq_n, g_vals)
    else:
        points = s4.manifold_points(n_max_global, zeros, args.u_mode, args.zero_kernel, args.kernel_scale)
        base_idx = {b: [n for n in range(2, n_max_global + 1) if s4.gcd(n, b) == 1] for b in bases}
        base_series = {b: build_weighted_g_series(points, base_idx[b], weights) for b in bases}

    for n_max in n_values:
        per_base = []
        for b in bases:
            seq_n_all, g_vals_all = base_series[b]
            per_base.append(eval_base(n_max, b, seq_n_all, g_vals_all, psi_at, args.x_step))
        per_base.sort(key=lambda r: r["base"])
        rows.append({"n_max": n_max, "per_base": per_base})

    # Stability summary on absolute correlation.
    summary = {}
    for b in bases:
        vals = [
            abs(next(x for x in row["per_base"] if x["base"] == b)["corr_hscaled_vs_escaled"])
            for row in rows
        ]
        m = sum(vals) / len(vals)
        var = sum((v - m) ** 2 for v in vals) / len(vals)
        summary[str(b)] = {"abs_corr_mean": m, "abs_corr_std": math.sqrt(var)}

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": sorted(n_values),
            "x_step": args.x_step,
            "weights": weights,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "max_zeta_zeros": len(zeros),
            "streaming": bool(args.streaming),
            "chunk_n": args.chunk_n if args.streaming else 0,
            "jobs": args.jobs if args.streaming else 1,
        },
        "rows": rows,
        "stability_summary": summary,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# H(x) Bridge Sweep\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Weights (Y11s,Y20,su2_trace,phase_vel): {weights}\n\n")
        for row in rows:
            f.write(f"## n_max={row['n_max']}\n\n")
            for r in row["per_base"]:
                f.write(
                    f"- base={r['base']} corr={r['corr_hscaled_vs_escaled']:.6f} "
                    f"corr_fit={r['corr_fitted_vs_escaled']:.6f} rmse={r['rmse']:.6f}\n"
                )
            f.write("\n")
        f.write("## Stability Summary (|corr| over n_max)\n\n")
        for b in bases:
            s = summary[str(b)]
            f.write(f"- base={b} abs_corr_mean={s['abs_corr_mean']:.6f} abs_corr_std={s['abs_corr_std']:.6f}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
