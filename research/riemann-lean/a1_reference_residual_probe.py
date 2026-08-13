#!/usr/bin/env python3
"""A1 reference residual probe: calibrate C0 for H^(M_ref)."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

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


def choose_log_power(xs: Sequence[int], ys: Sequence[float], a_min: float = 0.0, a_max: float = 3.0, step: float = 0.1):
    best = None
    a = a_min
    while a <= a_max + 1e-12:
        scaled = [y / max(1e-15, (math.log(max(3, x)) ** a)) for x, y in zip(xs, ys)]
        m, s = mean_std(scaled)
        cv = s / max(1e-15, m)
        cand = {"alpha": a, "C_mean": m, "C_std": s, "cv": cv, "C_max": max(scaled) if scaled else 0.0}
        if best is None or cand["cv"] < best["cv"]:
            best = cand
        a += step
    return best


def cache_key(payload: Dict[str, object]) -> str:
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
    ap = argparse.ArgumentParser(description="A1 reference residual probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000,2000000,5000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/a1_reference_residual_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a1_reference_residual_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zero data")
    zeros_ref = zeros_all[: args.m_ref]

    # Freeze a_ref,b_ref from pooled reference series.
    pooled_h = []
    pooled_e = []
    cache = []
    x0 = max(2, args.x_step)
    max_n = max(n_values)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    psi_max = hx.psi_exact_samples(xs_max)
    e_max = [(v - x) / math.sqrt(x) for x, v in zip(xs_max, psi_max)]
    x_map = {}
    e_map = {}
    counts = {}
    for n_max in n_values:
        k = ((n_max - x0) // args.x_step) + 1 if n_max >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n_max] = xs_max[:k]
        e_map[n_max] = e_max[:k]
        counts[n_max] = k

    zero_terms = hx.prepare_zero_terms(zeros_ref, args.zero_kernel, args.kernel_scale)
    zsig = zeros_sig(zeros_ref)
    use_cache = not args.no_cache
    event_stride = max(1, args.event_stride)
    cache_hits = 0

    def prefix(vals: Sequence[float]):
        out = []
        a = 0.0
        for v in vals:
            a += v
            out.append(a)
        return out

    def h_from_prefix(seq_n: Sequence[int], pref: Sequence[float], xs: Sequence[int]):
        out = []
        for x in xs:
            i = bisect.bisect_right(seq_n, x) - 1
            a = pref[i] if i >= 0 else 0.0
            out.append(a / math.sqrt(x))
        return out

    def build_base_rows(b: int):
        nonlocal cache_hits
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
                zeros=zeros_ref,
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
        rows = []
        for n_max in n_values:
            k = counts[n_max]
            rows.append({"n_max": n_max, "base": b, "x": x_map[n_max], "h_ref": h_max[:k], "e_scaled": e_map[n_max]})
        return rows

    if args.jobs > 1 and len(bases) > 1:
        workers = min(args.jobs, len(bases))
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
            futs = [ex.submit(build_base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                cache.extend(fut.result())
    else:
        for b in bases:
            cache.extend(build_base_rows(b))
    cache.sort(key=lambda r: (r["n_max"], r["base"]))
    for r in cache:
        pooled_h.extend(r["h_ref"])
        pooled_e.extend(r["e_scaled"])

    a_ref, b_ref, _ = hx.fit_linear(pooled_h, pooled_e)

    per_n = []
    x_anchor = []
    c0_anchor = []
    for n_max in n_values:
        rows = [r for r in cache if r["n_max"] == n_max]
        base_rows = []
        c0_n = 0.0
        for r in rows:
            resid = [abs(e - (a_ref * h + b_ref)) for h, e in zip(r["h_ref"], r["e_scaled"])]
            mres = max(resid) if resid else 0.0
            c0_n = max(c0_n, mres)
            base_rows.append(
                {
                    "base": r["base"],
                    "max_residual_ref": mres,
                    "mean_residual_ref": sum(resid) / max(1, len(resid)),
                    "abs_corr_h_vs_e": abs(hx.corr(r["h_ref"], r["e_scaled"])),
                }
            )
        per_n.append({"n_max": n_max, "per_base": sorted(base_rows, key=lambda z: z["base"]), "C0_n": c0_n})
        x_anchor.append(n_max)
        c0_anchor.append(c0_n)

    model = choose_log_power(x_anchor, c0_anchor, 0.0, 3.0, 0.1)
    # Conservative bound from model
    c0_const = max(c0_anchor) if c0_anchor else 0.0
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "m_ref": args.m_ref,
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
        "reference_fit": {
            "a_ref": a_ref,
            "b_ref": b_ref,
        },
        "c0_profile": {
            "C0_max_over_tested": c0_const,
            "log_power_model": model,
        },
        "per_n": per_n,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A1 Reference Residual Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        rf = report["reference_fit"]
        cp = report["c0_profile"]
        lm = cp["log_power_model"]
        f.write(f"- a_ref: {rf['a_ref']:.9f}\n")
        f.write(f"- b_ref: {rf['b_ref']:.9f}\n")
        f.write(f"- C0_max_over_tested: {cp['C0_max_over_tested']:.9f}\n")
        f.write(f"- best alpha (log-power): {lm['alpha']:.2f}\n")
        f.write(f"- C_max for alpha: {lm['C_max']:.9f}\n")
        f.write(f"- model cv: {lm['cv']:.6f}\n\n")
        for row in per_n:
            f.write(f"## n_max={row['n_max']} C0_n={row['C0_n']:.9f}\n\n")
            for b in row["per_base"]:
                f.write(
                    f"- base={b['base']} max_res={b['max_residual_ref']:.6f} "
                    f"mean_res={b['mean_residual_ref']:.6f} |corr|={b['abs_corr_h_vs_e']:.6f}\n"
                )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
