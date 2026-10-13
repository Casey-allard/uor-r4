#!/usr/bin/env python3
"""Lemma E endpoint probe from triangle-transfer constants."""

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


def choose_log_power(xs: Sequence[int], ys: Sequence[float], a_min: float = 0.0, a_max: float = 4.0, step: float = 0.1):
    best = None
    a = a_min
    while a <= a_max + 1e-12:
        scaled = [y / max(1e-15, (math.log(max(3, x)) ** a)) for x, y in zip(xs, ys)]
        m, s = mean_std(scaled)
        cv = s / max(1e-15, m)
        cand = {"A": a, "C_mean": m, "C_std": s, "cv": cv, "C_max": max(scaled) if scaled else 0.0}
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
    ap = argparse.ArgumentParser(description="Lemma E endpoint probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000,2000000,5000000")
    ap.add_argument("--x-step", type=int, default=5000)
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
    ap.add_argument("--cache-dir", type=str, default="research/cache/lemma_e_endpoint_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--a-ref", type=float, default=-0.0047075423063228546)
    ap.add_argument("--b-ref", type=float, default=-0.07903726672397497)
    ap.add_argument("--c0-ref", type=float, default=0.6126289168356691)
    ap.add_argument("--output", type=str, default="research/output/lemma_e_endpoint_probe_none_z128_ref512.json")
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

    per_n = []
    x_anchor = []
    obs_residual = []
    rhs_bound = []
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

    def base_rows(b: int):
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
            e_scaled = e_map[n]
            h_m = hm[:k]
            h_r = hr[:k]
            resid = [abs(ev - (args.a_ref * hm1 + args.b_ref)) for ev, hm1 in zip(e_scaled, h_m)]
            delta = [abs(hm1 - hr1) for hm1, hr1 in zip(h_m, h_r)]
            rhs = [args.c0_ref + abs(args.a_ref) * d for d in delta]
            mres = max(resid) if resid else 0.0
            mrhs = max(rhs) if rhs else 0.0
            out.append({"base": b, "max_residual_h_to_e": mres, "max_delta_m_ref": max(delta) if delta else 0.0, "max_rhs_triangle": mrhs, "ratio_res_over_rhs": mres / max(1e-15, mrhs), "abs_corr_h_vs_e": abs(hx.corr(h_m, e_scaled)), "n_max": n})
        return out

    by_n = {n: [] for n in n_values}
    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                for r in fut.result():
                    by_n[r["n_max"]].append({k: v for k, v in r.items() if k != "n_max"})
    else:
        for b in bases:
            for r in base_rows(b):
                by_n[r["n_max"]].append({k: v for k, v in r.items() if k != "n_max"})

    for n_max in n_values:
        base_rows = sorted(by_n[n_max], key=lambda z: z["base"])
        max_res = max((r["max_residual_h_to_e"] for r in base_rows), default=0.0)
        max_rhs = max((r["max_rhs_triangle"] for r in base_rows), default=0.0)
        per_n.append({"n_max": n_max, "per_base": sorted(base_rows, key=lambda z: z["base"]), "max_residual_over_bases": max_res, "max_rhs_over_bases": max_rhs})
        x_anchor.append(n_max)
        obs_residual.append(max_res)
        rhs_bound.append(max_rhs)

    model_res = choose_log_power(x_anchor, obs_residual, 0.0, 4.0, 0.1)
    model_rhs = choose_log_power(x_anchor, rhs_bound, 0.0, 4.0, 0.1)

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
            "a_ref": args.a_ref,
            "b_ref": args.b_ref,
            "c0_ref": args.c0_ref,
        },
        "per_n": per_n,
        "envelope_models": {
            "observed_residual": model_res,
            "triangle_rhs": model_rhs,
        },
        "rh_endpoint_indicator": {
            "candidate_form": "|E(x)|/sqrt(x) <= C*(log x)^A",
            "A_residual_best_cv": model_res["A"],
            "A_rhs_best_cv": model_rhs["A"],
            "residual_subpoly_support": model_res["A"] <= 4.0,
            "triangle_subpoly_support": model_rhs["A"] <= 4.0,
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lemma E Endpoint Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        mr = report["envelope_models"]["observed_residual"]
        mb = report["envelope_models"]["triangle_rhs"]
        f.write("## Log-Power Envelope Fits\n\n")
        f.write(f"- observed residual best A: {mr['A']:.2f}, C_max={mr['C_max']:.6f}, cv={mr['cv']:.6f}\n")
        f.write(f"- triangle RHS best A: {mb['A']:.2f}, C_max={mb['C_max']:.6f}, cv={mb['cv']:.6f}\n\n")
        f.write("## Per Scale (max over bases)\n\n")
        for r in per_n:
            f.write(
                f"- n_max={r['n_max']} max_residual={r['max_residual_over_bases']:.6f} "
                f"max_rhs={r['max_rhs_over_bases']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
