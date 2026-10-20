#!/usr/bin/env python3
"""Lemma D probe: base-uniformity of triangle-transfer constants."""

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


def explicit_surrogate_s(xs: Sequence[int], zeros: Sequence[float]) -> List[float]:
    if not xs or not zeros:
        return [0.0 for _ in xs]
    s4 = __import__("spinning_top_r4_candidate_a")
    np = s4.np
    if np is not None:
        u = np.log(np.asarray(xs, dtype=np.float64))
        g = np.asarray(zeros, dtype=np.float64)
        den = 0.25 + g * g
        phase = np.outer(u, g)
        c = np.cos(phase)
        s = np.sin(phase)
        re_term = (0.5 * c + s * g) / den
        return list((-2.0 * re_term.sum(axis=1)).astype(float))
    out = []
    for x in xs:
        u = math.log(x)
        acc = 0.0
        for g in zeros:
            den = 0.25 + g * g
            acc += (0.5 * math.cos(g * u) + g * math.sin(g * u)) / den
        out.append(-2.0 * acc)
    return out


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
    ap = argparse.ArgumentParser(description="Lemma D base-uniformity probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,600000,1000000")
    ap.add_argument("--x-step", type=int, default=2000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/lemma_d_base_uniformity_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/lemma_d_base_uniformity_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zero data")
    if args.m_zero <= 0 or args.m_zero >= args.m_ref:
        raise ValueError("require 0 < m-zero < m-ref")
    zeros_m = zeros_all[: args.m_zero]
    zeros_ref = zeros_all[: args.m_ref]

    # Freeze (a_ref,b_ref) and C0 from pooled reference.
    pooled_h_ref = []
    pooled_e = []
    cache = []
    x0 = max(2, args.x_step)
    max_n = max(n_values)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    psi_max = hx.psi_exact_samples(xs_max)
    e_max = [(v - x) / math.sqrt(x) for x, v in zip(xs_max, psi_max)]
    x_map = {}
    e_map = {}
    count = {}
    for n in n_values:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n] = xs_max[:k]
        e_map[n] = e_max[:k]
        count[n] = k
    zero_terms_m = hx.prepare_zero_terms(zeros_m, args.zero_kernel, args.kernel_scale)
    zero_terms_r = hx.prepare_zero_terms(zeros_ref, args.zero_kernel, args.kernel_scale)
    zsig_m = zeros_sig(zeros_m)
    zsig_r = zeros_sig(zeros_ref)
    event_stride = max(1, args.event_stride)
    use_cache = not args.no_cache
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

    def build_base_rows(b: int):
        nonlocal cache_hits
        km = cache_key({"version": 1, "mode": "m", "base": b, "max_n": max_n, "x_step": args.x_step, "u_mode": args.u_mode, "zero_kernel": args.zero_kernel, "kernel_scale": args.kernel_scale, "chunk_n": args.chunk_n, "event_stride": event_stride, "event_scale": bool(args.event_scale), "weights": WEIGHTS, "zsig": zsig_m})
        kr = cache_key({"version": 1, "mode": "r", "base": b, "max_n": max_n, "x_step": args.x_step, "u_mode": args.u_mode, "zero_kernel": args.zero_kernel, "kernel_scale": args.kernel_scale, "chunk_n": args.chunk_n, "event_stride": event_stride, "event_scale": bool(args.event_scale), "weights": WEIGHTS, "zsig": zsig_r})
        pm = os.path.join(args.cache_dir, f"{km}.npz")
        pr = os.path.join(args.cache_dir, f"{kr}.npz")
        hm = load_cache(pm) if use_cache else None
        hr = load_cache(pr) if use_cache else None
        if hm is not None and len(hm) == len(xs_max):
            cache_hits += 1
        else:
            seq_n, g = hx.stream_weighted_events(max_n, b, zeros_m, args.u_mode, args.zero_kernel, args.kernel_scale, WEIGHTS, args.chunk_n, zero_terms=zero_terms_m, event_stride=event_stride, event_scale=args.event_scale)
            hm = h_from_prefix(seq_n, prefix(g), xs_max)
            if use_cache:
                save_cache(pm, hm)
        if hr is not None and len(hr) == len(xs_max):
            cache_hits += 1
        else:
            seq_n, g = hx.stream_weighted_events(max_n, b, zeros_ref, args.u_mode, args.zero_kernel, args.kernel_scale, WEIGHTS, args.chunk_n, zero_terms=zero_terms_r, event_stride=event_stride, event_scale=args.event_scale)
            hr = h_from_prefix(seq_n, prefix(g), xs_max)
            if use_cache:
                save_cache(pr, hr)
        out = []
        for n in n_values:
            k = count[n]
            xs = x_map[n]
            e_scaled = e_map[n]
            s_ref = explicit_surrogate_s(xs, zeros_ref)
            a_se, b_se, _ = hx.fit_linear(s_ref, e_scaled)
            out.append({"n_max": n, "base": b, "x": xs, "e": e_scaled, "h_m": hm[:k], "h_ref": hr[:k], "s_ref": s_ref, "a_se": a_se, "b_se": b_se})
        return out

    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(build_base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                cache.extend(fut.result())
    else:
        for b in bases:
            cache.extend(build_base_rows(b))
    cache.sort(key=lambda r: (r["n_max"], r["base"]))
    for r in cache:
        pooled_h_ref.extend(r["h_ref"])
        pooled_e.extend(r["e"])

    a_ref, b_ref, _ = hx.fit_linear(pooled_h_ref, pooled_e)
    c0 = 0.0
    for row in cache:
        for ev, hv in zip(row["e"], row["h_ref"]):
            c0 = max(c0, abs(ev - (a_ref * hv + b_ref)))

    # Measure base-uniform profiles.
    per_n = []
    all_rel_spreads = []
    max_uniform_gap = 0.0
    for n_max in n_values:
        rows = [r for r in cache if r["n_max"] == n_max]
        base_rows = []
        max_res_vals = []
        max_delta_vals = []
        ratio_vals = []
        corr_vals = []
        for r in rows:
            e = r["e"]
            h_m = r["h_m"]
            h_ref = r["h_ref"]
            resid = [abs(ev - (a_ref * hm + b_ref)) for ev, hm in zip(e, h_m)]
            delta = [abs(hm - hr) for hm, hr in zip(h_m, h_ref)]
            max_res = max(resid) if resid else 0.0
            max_d = max(delta) if delta else 0.0
            rhs = c0 + abs(a_ref) * max_d
            gap = max(0.0, max_res - rhs)
            max_uniform_gap = max(max_uniform_gap, gap)
            ratio = (max_res / max(1e-15, rhs))
            corr_he = abs(hx.corr(h_m, e))
            base_rows.append(
                {
                    "base": r["base"],
                    "max_residual_h_to_e": max_res,
                    "max_delta_m_ref": max_d,
                    "rhs_uniform_bound": rhs,
                    "ratio_res_over_rhs": ratio,
                    "abs_corr_h_vs_e": corr_he,
                }
            )
            max_res_vals.append(max_res)
            max_delta_vals.append(max_d)
            ratio_vals.append(ratio)
            corr_vals.append(corr_he)

        m_res, s_res = mean_std(max_res_vals)
        m_del, s_del = mean_std(max_delta_vals)
        m_rat, s_rat = mean_std(ratio_vals)
        m_cor, s_cor = mean_std(corr_vals)
        rel_spread_res = s_res / max(1e-15, m_res)
        rel_spread_delta = s_del / max(1e-15, m_del)
        rel_spread_ratio = s_rat / max(1e-15, m_rat)
        rel_spread_corr = s_cor / max(1e-15, m_cor)
        all_rel_spreads.extend([rel_spread_res, rel_spread_delta, rel_spread_ratio, rel_spread_corr])
        per_n.append(
            {
                "n_max": n_max,
                "per_base": sorted(base_rows, key=lambda z: z["base"]),
                "uniformity_summary": {
                    "rel_spread_max_residual": rel_spread_res,
                    "rel_spread_max_delta": rel_spread_delta,
                    "rel_spread_ratio_res_over_rhs": rel_spread_ratio,
                    "rel_spread_abs_corr_h_vs_e": rel_spread_corr,
                },
            }
        )

    # Aggregate uniformity score (lower is better).
    uniformity_score = sum(all_rel_spreads) / max(1, len(all_rel_spreads))
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "m_zero": args.m_zero,
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
            "cache_misses": max(0, 2 * len(bases) - cache_hits),
        },
        "reference_constants": {
            "a_ref": a_ref,
            "b_ref": b_ref,
            "C0_ref_residual": c0,
        },
        "uniformity": {
            "uniformity_score": uniformity_score,
            "max_uniform_gap_res_minus_rhs": max_uniform_gap,
            "holds_uniform_triangle_on_grid": max_uniform_gap <= 1e-12,
        },
        "per_n": per_n,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        rc = report["reference_constants"]
        un = report["uniformity"]
        f.write("# Lemma D Base Uniformity Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- a_ref: {rc['a_ref']:.9f}\n")
        f.write(f"- b_ref: {rc['b_ref']:.9f}\n")
        f.write(f"- C0_ref_residual: {rc['C0_ref_residual']:.9f}\n")
        f.write(f"- uniformity_score (lower better): {un['uniformity_score']:.6f}\n")
        f.write(f"- max uniform gap (res-rhs): {un['max_uniform_gap_res_minus_rhs']:.3e}\n")
        f.write(f"- holds uniform triangle on grid: {un['holds_uniform_triangle_on_grid']}\n\n")
        for row in report["per_n"]:
            f.write(f"## n_max={row['n_max']}\n\n")
            us = row["uniformity_summary"]
            f.write(
                f"- rel_spread_res={us['rel_spread_max_residual']:.6f} "
                f"rel_spread_delta={us['rel_spread_max_delta']:.6f} "
                f"rel_spread_ratio={us['rel_spread_ratio_res_over_rhs']:.6f} "
                f"rel_spread_corr={us['rel_spread_abs_corr_h_vs_e']:.6f}\n"
            )
            for b in row["per_base"]:
                f.write(
                    f"- base={b['base']} max_res={b['max_residual_h_to_e']:.6f} "
                    f"max_delta={b['max_delta_m_ref']:.6f} rhs={b['rhs_uniform_bound']:.6f} "
                    f"ratio={b['ratio_res_over_rhs']:.6f} |corr|={b['abs_corr_h_vs_e']:.6f}\n"
                )
            f.write("\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
