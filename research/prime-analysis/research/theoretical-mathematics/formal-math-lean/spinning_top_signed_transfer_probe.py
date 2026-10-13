#!/usr/bin/env python3
"""Finite-range probe for the spinning-top signed-transfer target T.

Pure-Python implementation (no numpy dependency) so it runs in constrained
environments while preserving cache reuse and deterministic outputs.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List

from prime_geometry_loop import load_zeta_zeros_file


def key_hash(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_signature(zeros: List[float]) -> str:
    if not zeros:
        return "empty"
    text = ",".join(f"{z:.10f}" for z in zeros[: min(2048, len(zeros))])
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


def cumulative_tail_sup(arr: List[float]) -> List[float]:
    out = [0.0] * len(arr)
    best = float("-inf")
    for i in range(len(arr) - 1, -1, -1):
        if arr[i] > best:
            best = arr[i]
        out[i] = best
    return out


def tail_sup_constant(signal: List[float], xs: List[float], beta: float, tail_guard: int) -> Dict[str, float]:
    ratio_pos = [signal[i] / (xs[i] ** beta) for i in range(len(signal))]
    ratio_neg = [(-signal[i]) / (xs[i] ** beta) for i in range(len(signal))]
    sup_pos = cumulative_tail_sup(ratio_pos)
    sup_neg = cumulative_tail_sup(ratio_neg)
    stop = max(1, len(signal) - max(1, tail_guard))
    c_pos = min(sup_pos[:stop])
    c_neg = min(sup_neg[:stop])
    c_best = max(c_pos, c_neg)
    dominant_sign = "positive" if c_pos >= c_neg else "negative"
    return {
        "beta": beta,
        "c_pos": c_pos,
        "c_neg": c_neg,
        "c_best": c_best,
        "dominant_sign": dominant_sign,
    }


def effective_exponent_abs_envelope(abs_signal: List[float], xs: List[float], tail_guard: int, sample_count: int = 128) -> float:
    env = cumulative_tail_sup(abs_signal)
    stop = max(2, len(abs_signal) - max(1, tail_guard))
    take = min(sample_count, stop)
    idx = [int(round(i * (stop - 1) / max(1, take - 1))) for i in range(take)]
    lx = [math.log(max(xs[i], 1e-300)) for i in idx]
    ly = [math.log(max(env[i], 1e-300)) for i in idx]
    mx = sum(lx) / len(lx)
    my = sum(ly) / len(ly)
    num = sum((lx[i] - mx) * (ly[i] - my) for i in range(len(lx)))
    den = sum((x - mx) ** 2 for x in lx)
    return num / den if den > 0 else 0.0


def linspace(a: float, b: float, n: int) -> List[float]:
    if n <= 1:
        return [a]
    step = (b - a) / float(n - 1)
    return [a + i * step for i in range(n)]


def compute_signal(zeros: List[float], x_min: float, x_max: float, grid_size: int) -> Dict[str, List[float]]:
    log_xs = linspace(math.log(x_min), math.log(x_max), grid_size)
    xs = [math.exp(t) for t in log_xs]
    sqrt_x = [math.sqrt(x) for x in xs]
    weights = [1.0 / math.sqrt(0.25 + z * z) for z in zeros]
    signal: List[float] = []
    for i, t in enumerate(log_xs):
        acc = 0.0
        for z, w in zip(zeros, weights):
            acc += w * math.cos(z * t)
        signal.append(sqrt_x[i] * acc)
    return {"x": xs, "signal": signal}


def main() -> None:
    ap = argparse.ArgumentParser(description="Probe spinning-top signed-transfer target T on cached zeta zeros")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=256)
    ap.add_argument("--x-min", type=float, default=1.0e4)
    ap.add_argument("--x-max", type=float, default=1.0e8)
    ap.add_argument("--grid-size", type=int, default=16000)
    ap.add_argument("--beta-start", type=float, default=0.51)
    ap.add_argument("--beta-stop", type=float, default=0.75)
    ap.add_argument("--beta-count", type=int, default=25)
    ap.add_argument("--tail-guard", type=int, default=512)
    ap.add_argument("--c-practical-threshold", type=float, default=1.0e-6)
    ap.add_argument("--cache-dir", type=str, default="research/cache/spinning_top_signed_transfer_probe")
    ap.add_argument("--output", type=str, default="research/output/spinning_top_signed_transfer_probe.json")
    args = ap.parse_args()

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    if args.max_zeta_zeros > 0:
        zeros = zeros[: args.max_zeta_zeros]
    if not zeros:
        raise ValueError("no zeta zeros available")

    cfg = {
        "zeros_sig": zeros_signature(zeros),
        "max_zeta_zeros": len(zeros),
        "x_min": float(args.x_min),
        "x_max": float(args.x_max),
        "grid_size": int(args.grid_size),
    }
    key = key_hash(cfg)
    os.makedirs(args.cache_dir, exist_ok=True)
    cache_path = os.path.join(args.cache_dir, key + ".json")

    if os.path.exists(cache_path):
        with open(cache_path, "r", encoding="utf-8") as f:
            cached = json.load(f)
        xs = [float(x) for x in cached["x"]]
        signal = [float(v) for v in cached["signal"]]
    else:
        computed = compute_signal(zeros=zeros, x_min=args.x_min, x_max=args.x_max, grid_size=args.grid_size)
        xs = computed["x"]
        signal = computed["signal"]
        with open(cache_path, "w", encoding="utf-8") as f:
            json.dump({"x": xs, "signal": signal}, f)

    beta_grid = linspace(args.beta_start, args.beta_stop, args.beta_count)
    rows: List[Dict[str, float]] = []
    for beta in beta_grid:
        rows.append(tail_sup_constant(signal=signal, xs=xs, beta=beta, tail_guard=args.tail_guard))

    best = max(rows, key=lambda r: r["c_best"])
    practical = [r for r in rows if r["c_best"] >= args.c_practical_threshold]
    practical_beta_max = max((r["beta"] for r in practical), default=None)

    abs_signal = [abs(v) for v in signal]
    beta_eff = effective_exponent_abs_envelope(abs_signal=abs_signal, xs=xs, tail_guard=args.tail_guard)
    abs_max = max(abs_signal)
    abs_sorted = sorted(abs_signal)
    abs_median = abs_sorted[len(abs_sorted) // 2]

    superhalf_margin = beta_eff - 0.5
    if superhalf_margin <= 0.01:
        interpretation = "no_superhalf_growth_detected"
    elif practical_beta_max is None:
        interpretation = "no_practical_superhalf_support_on_grid"
    elif practical_beta_max < 0.55:
        interpretation = "superhalf_support_very_weak"
    else:
        interpretation = "superhalf_support_detected_finite_range"

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "zeros_used": len(zeros),
            "x_min": args.x_min,
            "x_max": args.x_max,
            "grid_size": args.grid_size,
            "beta_start": args.beta_start,
            "beta_stop": args.beta_stop,
            "beta_count": args.beta_count,
            "tail_guard": args.tail_guard,
            "c_practical_threshold": args.c_practical_threshold,
            "cache_path": cache_path,
        },
        "metrics": {
            "signal_abs_max": abs_max,
            "signal_abs_median": abs_median,
            "effective_exponent_abs_envelope": beta_eff,
            "effective_superhalf_margin": superhalf_margin,
            "best_beta_on_grid": best["beta"],
            "best_c_on_grid": best["c_best"],
            "best_sign_on_grid": best["dominant_sign"],
            "practical_beta_max": practical_beta_max,
        },
        "beta_scan": rows,
        "interpretation": interpretation,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    md_path = args.output.replace(".json", ".md")
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# Spinning-Top Signed Transfer Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        f.write(f"- zeros used: {result['config']['zeros_used']}\n")
        f.write(f"- x-range: [{args.x_min:g}, {args.x_max:g}]\n")
        f.write(f"- effective abs-envelope exponent: {beta_eff:.6f}\n")
        f.write(f"- best beta on grid: {best['beta']:.6f}\n")
        f.write(f"- best c on grid: {best['c_best']:.6e} ({best['dominant_sign']})\n")
        f.write(f"- practical beta max (threshold {args.c_practical_threshold:.1e}): {practical_beta_max}\n")
        f.write(f"- interpretation: {interpretation}\n")
        f.write(f"- cache path: {cache_path}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md_path}")
    print("effective_exponent_abs_envelope:", round(beta_eff, 6))
    print("best_beta_on_grid:", round(float(best["beta"]), 6))
    print("best_c_on_grid:", f"{best['c_best']:.6e}")
    print("practical_beta_max:", practical_beta_max)
    print("interpretation:", interpretation)


if __name__ == "__main__":
    main()
