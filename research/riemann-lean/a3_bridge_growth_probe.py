#!/usr/bin/env python3
"""A3 bridge-growth probe: fit |H_W^(M)(x)| <= C_H (log x)^A_H."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import List, Sequence

import hx_bridge_probe as hx
from prime_geometry_loop import load_zeta_zeros_file
import spinning_top_r4_candidate_a as s4


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def mean_std(vals: Sequence[float]):
    if not vals:
        return 0.0, 0.0
    m = sum(vals) / len(vals)
    v = sum((x - m) ** 2 for x in vals) / len(vals)
    return m, math.sqrt(v)


def choose_log_power(xs: Sequence[int], ys: Sequence[float], a_min: float = 0.0, a_max: float = 10.0, step: float = 0.1):
    best = None
    a = a_min
    while a <= a_max + 1e-12:
        scaled = [y / max(1e-15, (math.log(max(3, x)) ** a)) for x, y in zip(xs, ys)]
        m, s = mean_std(scaled)
        cv = s / max(1e-15, m)
        cand = {"A_H": a, "C_mean": m, "C_std": s, "cv": cv, "C_max": max(scaled) if scaled else 0.0}
        if best is None or cand["cv"] < best["cv"]:
            best = cand
        a += step
    return best


def cache_key(payload):
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def load_cache(path: str):
    np = s4.np
    if np is None or not os.path.exists(path):
        return None
    try:
        d = np.load(path)
        if "h_max" not in d:
            return None
        return d["h_max"].astype(np.float64).tolist()
    except Exception:
        return None


def save_cache(path: str, h_max: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, h_max=np.asarray(h_max, dtype=np.float64))


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 bridge growth probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000,2000000,5000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_bridge_growth_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--a-min", type=float, default=0.0)
    ap.add_argument("--a-max", type=float, default=10.0)
    ap.add_argument("--a-step", type=float, default=0.1)
    ap.add_argument("--output", type=str, default="research/output/a3_bridge_growth_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_zero > len(zeros_all):
        raise ValueError("m-zero exceeds available zero data")
    zeros = zeros_all[: args.m_zero]

    per_n = []
    x_anchor = []
    h_anchor = []
    x0 = max(2, args.x_step)
    max_n = max(n_values)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    counts = {}
    for n in n_values:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        counts[n] = max(0, min(k, len(xs_max)))

    zero_terms = hx.prepare_zero_terms(zeros, args.zero_kernel, args.kernel_scale)
    zsig = zeros_sig(zeros)
    use_cache = not args.no_cache
    event_stride = max(1, args.event_stride)
    cache_hits = 0

    def prefix(vals):
        out = []
        a = 0.0
        for v in vals:
            a += v
            out.append(a)
        return out

    def h_from_prefix(seq_n, pref, xs):
        out = []
        for x in xs:
            i = bisect.bisect_right(seq_n, x) - 1
            a = pref[i] if i >= 0 else 0.0
            out.append(a / math.sqrt(x))
        return out

    h_by_base = {}
    for b in bases:
        key = cache_key(
            {
                "version": 1,
                "base": b,
                "max_n": max_n,
                "x_step": args.x_step,
                "u_mode": args.u_mode,
                "zero_kernel": args.zero_kernel,
                "kernel_scale": args.kernel_scale,
                "chunk_n": args.chunk_n,
                "event_stride": event_stride,
                "event_scale": bool(args.event_scale),
                "weights": WEIGHTS,
                "zsig": zsig,
            }
        )
        cp = os.path.join(args.cache_dir, f"{key}.npz")
        h_max = load_cache(cp) if use_cache else None
        if h_max is not None and len(h_max) == len(xs_max):
            cache_hits += 1
        else:
            seq_n, g_vals = hx.stream_weighted_events(
                n_max=max_n,
                base=b,
                zeros=zeros,
                u_mode=args.u_mode,
                zero_kernel=args.zero_kernel,
                kernel_scale=args.kernel_scale,
                weights=WEIGHTS,
                chunk_n=args.chunk_n,
                zero_terms=zero_terms,
                event_stride=event_stride,
                event_scale=args.event_scale,
            )
            h_max = h_from_prefix(seq_n, prefix(g_vals), xs_max)
            if use_cache:
                save_cache(cp, h_max)
        h_by_base[b] = h_max

    for n_max in n_values:
        per_base = []
        max_over_bases = 0.0
        k = counts[n_max]
        for b in bases:
            h = h_by_base[b][:k]
            ah = [abs(v) for v in h]
            max_h = max(ah) if ah else 0.0
            mean_h = sum(ah) / max(1, len(ah))
            max_over_bases = max(max_over_bases, max_h)
            per_base.append({"base": b, "max_abs_h_scaled": max_h, "mean_abs_h_scaled": mean_h})
        per_n.append({"n_max": n_max, "per_base": sorted(per_base, key=lambda z: z["base"]), "max_abs_h_scaled_over_bases": max_over_bases})
        x_anchor.append(n_max)
        h_anchor.append(max_over_bases)

    model = choose_log_power(x_anchor, h_anchor, args.a_min, args.a_max, args.a_step)
    c_h = model["C_max"]
    a_h = model["A_H"]

    # Validate the envelope on sampled anchors.
    violations = 0
    max_gap = -1e18
    ratios = []
    for x, h in zip(x_anchor, h_anchor):
        rhs = c_h * (math.log(max(3, x)) ** a_h)
        gap = h - rhs
        if gap > 1e-12:
            violations += 1
        max_gap = max(max_gap, gap)
        ratios.append(h / max(1e-15, rhs))

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "m_zero": args.m_zero,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": event_stride,
            "event_scale": bool(args.event_scale),
            "cache_dir": args.cache_dir,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_misses": max(0, len(bases) - cache_hits),
        },
        "growth_model": model,
        "envelope_check": {
            "holds_on_anchor_scales": violations == 0 and max_gap <= 1e-12,
            "total_violations": int(violations),
            "max_gap_h_minus_rhs": float(max_gap),
            "ratio_min": min(ratios) if ratios else 0.0,
            "ratio_max": max(ratios) if ratios else 0.0,
        },
        "per_n": per_n,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        gm = report["growth_model"]
        ec = report["envelope_check"]
        f.write("# A3 Bridge Growth Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- A_H (best): {gm['A_H']:.2f}\n")
        f.write(f"- C_H (C_max): {gm['C_max']:.9f}\n")
        f.write(f"- model cv: {gm['cv']:.6f}\n")
        f.write(f"- holds on anchor scales: {ec['holds_on_anchor_scales']}\n")
        f.write(f"- max gap (h-rhs): {ec['max_gap_h_minus_rhs']:.3e}\n")
        f.write(f"- ratio range h/rhs: [{ec['ratio_min']:.6f}, {ec['ratio_max']:.6f}]\n\n")
        for r in per_n:
            f.write(f"## n_max={r['n_max']} max_over_bases={r['max_abs_h_scaled_over_bases']:.6f}\n\n")
            for b in r["per_base"]:
                f.write(
                    f"- base={b['base']} max_abs_h={b['max_abs_h_scaled']:.6f} "
                    f"mean_abs_h={b['mean_abs_h_scaled']:.6f}\n"
                )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
