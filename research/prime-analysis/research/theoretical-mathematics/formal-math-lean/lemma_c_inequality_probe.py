#!/usr/bin/env python3
"""Lemma C inequality probe: calibrate residual <= C1*Delta_M + C2*R_smooth."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

import hx_bridge_probe as hx
from prime_geometry_loop import load_zeta_zeros_file
import spinning_top_r4_candidate_a as s4


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


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


def fit_nonneg_cover(d: Sequence[float], s: Sequence[float], r: Sequence[float], c1_max: float = 200.0, steps: int = 4000):
    # Find C1,C2 >=0 minimizing C1+C2 s.t. C1*d_i + C2*s_i >= r_i for all i.
    best = None
    eps = 1e-15
    for k in range(steps + 1):
        c1 = c1_max * (k / steps)
        c2_req = 0.0
        feasible = True
        for di, si, ri in zip(d, s, r):
            rem = ri - c1 * di
            if rem <= 0:
                continue
            if si <= eps:
                feasible = False
                break
            c2_req = max(c2_req, rem / si)
        if not feasible:
            continue
        obj = c1 + c2_req
        if best is None or obj < best["objective"]:
            best = {"C1": c1, "C2": c2_req, "objective": obj}
    if best is None:
        # fallback: pure smooth residual bound
        c2 = max((ri / max(si, eps)) for si, ri in zip(s, r))
        best = {"C1": 0.0, "C2": c2, "objective": c2}
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
    ap = argparse.ArgumentParser(description="Lemma C inequality probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,600000,1000000")
    ap.add_argument("--x-step", type=int, default=2000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=64)
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/lemma_c_inequality_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/lemma_c_inequality_probe.json")
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

    # Precompute all series and pooled fit parameters.
    rows = []
    pooled_h = []
    pooled_e = []
    pooled_s = []
    pooled_d = []
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
            s_m = explicit_surrogate_s(xs, zeros_m)
            a_se, b_se, _ = hx.fit_linear(s_m, e_scaled)
            smooth_res = [abs(e - (a_se * sv + b_se)) for e, sv in zip(e_scaled, s_m)]
            h_m = hm[:k]
            h_r = hr[:k]
            delta_m = [abs(a - b) for a, b in zip(h_m, h_r)]
            out.append({"n_max": n, "base": b, "x": xs, "h_m": h_m, "h_ref": h_r, "e_scaled": e_scaled, "s_m": s_m, "delta_m": delta_m, "smooth_residual": smooth_res, "a_s_to_e": a_se, "b_s_to_e": b_se})
        return out

    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(build_base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                rows.extend(fut.result())
    else:
        for b in bases:
            rows.extend(build_base_rows(b))
    rows.sort(key=lambda r: (r["n_max"], r["base"]))
    for row in rows:
        pooled_h.extend(row["h_m"])
        pooled_e.extend(row["e_scaled"])
        pooled_s.extend(row["smooth_residual"])
        pooled_d.extend(row["delta_m"])

    a_he, b_he, _ = hx.fit_linear(pooled_h, pooled_e)

    # Build pooled residual for inequality fit.
    pooled_r = [abs(e - (a_he * h + b_he)) for h, e in zip(pooled_h, pooled_e)]
    cover = fit_nonneg_cover(pooled_d, pooled_s, pooled_r, c1_max=500.0, steps=8000)
    c1 = cover["C1"]
    c2 = cover["C2"]

    # Evaluate inequality.
    per_row = []
    max_gap = -1e18
    max_residual = 0.0
    max_rhs = 0.0
    violations = 0
    for row in rows:
        h_m = row["h_m"]
        e_scaled = row["e_scaled"]
        d_m = row["delta_m"]
        s_res = row["smooth_residual"]
        resid = [abs(e - (a_he * h + b_he)) for h, e in zip(h_m, e_scaled)]
        rhs = [c1 * d + c2 * s for d, s in zip(d_m, s_res)]
        gaps = [rr - bb for rr, bb in zip(resid, rhs)]
        vcount = sum(1 for g in gaps if g > 1e-12)
        violations += vcount
        local_max_gap = max(gaps) if gaps else 0.0
        max_gap = max(max_gap, local_max_gap)
        max_residual = max(max_residual, max(resid) if resid else 0.0)
        max_rhs = max(max_rhs, max(rhs) if rhs else 0.0)
        per_row.append(
            {
                "n_max": row["n_max"],
                "base": row["base"],
                "corr_h_vs_e": hx.corr(h_m, e_scaled),
                "corr_s_vs_e": hx.corr(row["s_m"], e_scaled),
                "max_residual_h_to_e": max(resid) if resid else 0.0,
                "max_rhs": max(rhs) if rhs else 0.0,
                "max_gap_res_minus_rhs": local_max_gap,
                "violation_count": vcount,
            }
        )

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
        "fit": {
            "a_h_to_e": a_he,
            "b_h_to_e": b_he,
            "C1_delta": c1,
            "C2_smooth": c2,
            "objective_C1_plus_C2": cover["objective"],
        },
        "inequality_check": {
            "global_max_residual": float(max_residual),
            "global_max_rhs": float(max_rhs),
            "global_max_gap_res_minus_rhs": float(max_gap),
            "total_violations": int(violations),
            "holds_on_test_grid": bool((violations == 0) and (max_gap <= 1e-12)),
        },
        "rows": per_row,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lemma C Inequality Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        fit = report["fit"]
        chk = report["inequality_check"]
        f.write(f"- a(H->E): {fit['a_h_to_e']:.9f}\n")
        f.write(f"- b(H->E): {fit['b_h_to_e']:.9f}\n")
        f.write(f"- C1 (Delta_M): {fit['C1_delta']:.9f}\n")
        f.write(f"- C2 (R_smooth): {fit['C2_smooth']:.9f}\n")
        f.write(f"- holds on test grid: {chk['holds_on_test_grid']}\n")
        f.write(f"- global max gap (res-rhs): {chk['global_max_gap_res_minus_rhs']:.3e}\n")
        f.write(f"- total violations: {chk['total_violations']}\n\n")
        for r in sorted(report["rows"], key=lambda z: (z["n_max"], z["base"])):
            f.write(
                f"- n_max={r['n_max']} base={r['base']} "
                f"max_res={r['max_residual_h_to_e']:.6f} max_rhs={r['max_rhs']:.6f} "
                f"max_gap={r['max_gap_res_minus_rhs']:.3e} violations={r['violation_count']}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
