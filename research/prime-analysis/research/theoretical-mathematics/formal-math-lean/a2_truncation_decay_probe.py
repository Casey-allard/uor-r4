#!/usr/bin/env python3
"""A2 truncation-decay probe: fit Delta_M <= C_D (log x)^beta tau(M)."""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file
import lemma_b_truncation_program as lb


def analytic_tail_sums_window(zeros_ref: Sequence[float], m: int, kernel: str, scale: float):
    t0 = 0.0
    t1 = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t0 += w / den
        t1 += w * (g / den)
    return t0, t1


def best_beta_and_c(xs: Sequence[int], deltas: Sequence[float], tails: Sequence[float], beta_grid: Sequence[float]):
    best = None
    eps = 1e-30
    for beta in beta_grid:
        vals = []
        for x, d, t in zip(xs, deltas, tails):
            denom = max(eps, (math.log(max(3, x)) ** beta) * t)
            vals.append(d / denom)
        c = max(vals) if vals else 0.0
        m = sum(vals) / max(1, len(vals))
        v = sum((z - m) ** 2 for z in vals) / max(1, len(vals))
        cv = math.sqrt(v) / max(eps, m)
        cand = {"beta": beta, "C_delta": c, "cv": cv, "mean_ratio": m}
        if best is None or cand["cv"] < best["cv"]:
            best = cand
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 truncation decay probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000,2000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--zero-limits", type=str, default="64,96,128,160,192,224,256,320,384")
    ap.add_argument("--ref-zero-limit", type=int, default=512)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/a2_truncation_decay_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a2_truncation_decay_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    limits = sorted(set(int(x.strip()) for x in args.zero_limits.split(",") if x.strip()))
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.ref_zero_limit > len(zeros_all):
        raise ValueError("ref-zero-limit exceeds available zeros")
    limits = [m for m in limits if 0 < m < args.ref_zero_limit]
    if not limits:
        raise ValueError("no valid zero-limits below ref")
    limits_ref = limits + [args.ref_zero_limit]
    zeros_ref = zeros_all[: args.ref_zero_limit]

    # Build data via lemma_b engine.
    by_n = []
    samples = []  # flattened for fitting beta/C
    for n_max in n_values:
        xs = list(range(max(2, args.x_step), n_max + 1, args.x_step))
        hs = lb.compute_series(
            bases=bases,
            n_max=n_max,
            xs=xs,
            zeros_all=zeros_all,
            limits=limits_ref,
            u_mode=args.u_mode,
            zero_kernel=args.zero_kernel,
            kernel_scale=args.kernel_scale,
            chunk_n=args.chunk_n,
            jobs=args.jobs,
            event_stride=max(1, args.event_stride),
            event_scale=args.event_scale,
            cache_dir=args.cache_dir,
            use_cache=(not args.no_cache),
        )
        per_base = []
        for b in bases:
            ref = hs[b][args.ref_zero_limit]
            rows = []
            for m in limits:
                cur = hs[b][m]
                mad = lb.max_abs_diff(cur, ref)
                t0, t1 = analytic_tail_sums_window(zeros_ref, m, args.zero_kernel, args.kernel_scale)
                rows.append(
                    {
                        "M": m,
                        "max_abs_diff_vs_ref": mad,
                        "tau_tail_F": t0,
                        "tau_tail_Fp": t1,
                    }
                )
                samples.append({"x": n_max, "base": b, "M": m, "delta": mad, "tau_F": t0, "tau_Fp": t1})
            per_base.append({"base": b, "rows": rows})
        by_n.append({"n_max": n_max, "per_base": per_base})

    xs = [s["x"] for s in samples]
    deltas = [s["delta"] for s in samples]
    tau_f = [max(1e-30, s["tau_F"]) for s in samples]
    tau_fp = [max(1e-30, s["tau_Fp"]) for s in samples]
    beta_grid = [0.0 + 0.1 * k for k in range(41)]

    fit_f = best_beta_and_c(xs, deltas, tau_f, beta_grid)
    fit_fp = best_beta_and_c(xs, deltas, tau_fp, beta_grid)

    chosen = "tau_tail_Fp" if fit_fp["cv"] <= fit_f["cv"] else "tau_tail_F"
    chosen_fit = fit_fp if chosen == "tau_tail_Fp" else fit_f

    # Validate chosen bound on all samples.
    violations = 0
    max_gap = -1e18
    ratios = []
    for s in samples:
        tau = s["tau_Fp"] if chosen == "tau_tail_Fp" else s["tau_F"]
        rhs = chosen_fit["C_delta"] * (math.log(max(3, s["x"])) ** chosen_fit["beta"]) * max(1e-30, tau)
        gap = s["delta"] - rhs
        max_gap = max(max_gap, gap)
        if gap > 1e-12:
            violations += 1
        ratios.append(s["delta"] / max(1e-30, rhs))

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "zero_limits": limits,
            "ref_zero_limit": args.ref_zero_limit,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": max(1, args.event_stride),
            "event_scale": bool(args.event_scale),
            "cache_dir": args.cache_dir,
            "cache_enabled": not args.no_cache,
        },
        "fits": {
            "tau_tail_F": fit_f,
            "tau_tail_Fp": fit_fp,
            "chosen_tau": chosen,
            "chosen_fit": chosen_fit,
        },
        "bound_check": {
            "holds_on_samples": violations == 0 and max_gap <= 1e-12,
            "total_violations": int(violations),
            "max_gap_delta_minus_rhs": float(max_gap),
            "ratio_min": min(ratios) if ratios else 0.0,
            "ratio_max": max(ratios) if ratios else 0.0,
        },
        "per_n": by_n,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A2 Truncation Decay Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- chosen tau: {chosen}\n")
        f.write(f"- beta: {chosen_fit['beta']:.3f}\n")
        f.write(f"- C_delta: {chosen_fit['C_delta']:.6e}\n")
        f.write(f"- cv: {chosen_fit['cv']:.6f}\n")
        b = report["bound_check"]
        f.write(f"- holds on samples: {b['holds_on_samples']}\n")
        f.write(f"- violations: {b['total_violations']}\n")
        f.write(f"- max gap (delta-rhs): {b['max_gap_delta_minus_rhs']:.3e}\n")
        f.write(f"- ratio range delta/rhs: [{b['ratio_min']:.6f}, {b['ratio_max']:.6f}]\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
