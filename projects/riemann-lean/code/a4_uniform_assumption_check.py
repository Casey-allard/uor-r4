#!/usr/bin/env python3
"""A4 check: unified base-uniform constants for A1/A2/A3 on one grid."""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import math
import os
import time
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def mean_std(vals: Sequence[float]):
    if not vals:
        return 0.0, 0.0
    m = sum(vals) / len(vals)
    v = sum((x - m) ** 2 for x in vals) / len(vals)
    return m, math.sqrt(v)


def tau_tail_fp(zeros_ref: Sequence[float], m: int, kernel: str, scale: float) -> float:
    t = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t += w * (g / den)
    return t


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


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


def load_cache_from_dirs(cache_dirs: Sequence[str], key: str):
    for d in cache_dirs:
        cp = os.path.join(d, f"{key}.npz")
        out = load_cache(cp)
        if out is not None:
            return out, cp
    return None, None


def save_cache(path: str, h_max: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, h_max=np.asarray(h_max, dtype=np.float64))


def main() -> None:
    ap = argparse.ArgumentParser(description="A4 unified assumption consistency check")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000,2000000,5000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--beta", type=float, default=2.6, help="A2 log exponent")
    ap.add_argument("--a-h", type=float, default=1.2, help="A3 log exponent")
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/a4_uniform_assumption_check")
    ap.add_argument(
        "--cache-read-dirs",
        type=str,
        default="research/cache/a4_uniform_assumption_check,research/cache/a3_offdiag_dynamic_majorant,research/cache/a3_offdiag_symbolic_chain,research/cache/a3_offdiag_sign_sensitive_lagbound",
    )
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--profile", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a4_uniform_assumption_check.json")
    args = ap.parse_args()
    np = s4.np
    if np is None:
        raise RuntimeError("numpy backend unavailable via spinning_top_r4_candidate_a.np")

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zero data")
    if args.m_zero <= 0 or args.m_zero >= args.m_ref:
        raise ValueError("require 0 < m-zero < m-ref")
    zeros_m = zeros_all[: args.m_zero]
    zeros_ref = zeros_all[: args.m_ref]
    tau_m = tau_tail_fp(zeros_ref, args.m_zero, args.zero_kernel, args.kernel_scale)
    event_stride = max(1, args.event_stride)
    use_cache = not args.no_cache
    cache_read_dirs = [x.strip() for x in args.cache_read_dirs.split(",") if x.strip()]
    if args.cache_dir not in cache_read_dirs:
        cache_read_dirs = [args.cache_dir] + cache_read_dirs
    t_start = time.perf_counter()
    build_seconds = 0.0
    poolfit_seconds = 0.0
    theorem_seconds = 0.0
    cache_writes = 0

    # Build pooled reference fit (a_ref, b_ref) with per-base max-n reuse.
    pooled_h_ref = []
    pooled_e = []
    cache = []
    max_n = max(n_values)
    zero_terms_m = hx.prepare_zero_terms(
        zeros=zeros_m,
        zero_kernel=args.zero_kernel,
        kernel_scale=args.kernel_scale,
    )
    zero_terms_ref = hx.prepare_zero_terms(
        zeros=zeros_ref,
        zero_kernel=args.zero_kernel,
        kernel_scale=args.kernel_scale,
    )

    x0 = max(2, args.x_step)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    xs_max_np = np.asarray(xs_max, dtype=np.int64)
    x_float_max = xs_max_np.astype(np.float64)
    psi_max = hx.psi_exact_samples(xs_max)
    psi_max_np = np.asarray(psi_max, dtype=np.float64)
    e_max_np = (psi_max_np - x_float_max) / np.sqrt(np.maximum(1.0, x_float_max))
    l_max_np = np.log(np.maximum(3.0, x_float_max)) ** args.beta
    x_map = {}
    e_map = {}
    l_map = {}
    x_counts = {}
    for n_max in n_values:
        k = ((n_max - x0) // args.x_step) + 1 if n_max >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n_max] = xs_max_np[:k]
        e_map[n_max] = e_max_np[:k]
        l_map[n_max] = l_max_np[:k]
        x_counts[n_max] = k

    def h_from_events(seq_n: Sequence[int], seq_g: Sequence[float], xs_np):
        seq_n_np = np.asarray(seq_n, dtype=np.int64)
        g_np = np.asarray(seq_g, dtype=np.float64)
        if seq_n_np.size == 0 or g_np.size == 0 or xs_np.size == 0:
            return np.zeros(xs_np.size, dtype=np.float64)
        pref = np.cumsum(g_np)
        idx = np.searchsorted(seq_n_np, xs_np, side="right") - 1
        out = np.zeros(xs_np.size, dtype=np.float64)
        m = idx >= 0
        if np.any(m):
            out[m] = pref[idx[m]] / np.sqrt(np.maximum(1.0, xs_np[m].astype(np.float64)))
        return out

    zsig_m = zeros_sig(zeros_m)
    zsig_r = zeros_sig(zeros_ref)
    cache_hits = 0

    def build_base_rows(b: int):
        nonlocal cache_hits
        nonlocal cache_writes
        out = []
        km = cache_key(
            {
                "version": 1,
                "mode": "m",
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
                "zsig": zsig_m,
            }
        )
        kr = cache_key(
            {
                "version": 1,
                "mode": "ref",
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
                "zsig": zsig_r,
            }
        )
        pm = os.path.join(args.cache_dir, f"{km}.npz")
        pr = os.path.join(args.cache_dir, f"{kr}.npz")
        h_m_max, _ = load_cache_from_dirs(cache_read_dirs, km) if use_cache else (None, None)
        h_r_max, _ = load_cache_from_dirs(cache_read_dirs, kr) if use_cache else (None, None)
        if h_m_max is not None and len(h_m_max) == len(xs_max_np):
            cache_hits += 1
            h_m_max_np = np.asarray(h_m_max, dtype=np.float64)
        else:
            seq_m_n, seq_m_g = hx.stream_weighted_events(
                n_max=max_n,
                base=b,
                zeros=zeros_m,
                u_mode=args.u_mode,
                zero_kernel=args.zero_kernel,
                kernel_scale=args.kernel_scale,
                weights=WEIGHTS,
                chunk_n=args.chunk_n,
                zero_terms=zero_terms_m,
                event_stride=event_stride,
                event_scale=args.event_scale,
            )
            h_m_max_np = h_from_events(seq_m_n, seq_m_g, xs_max_np)
            if use_cache:
                save_cache(pm, h_m_max_np.tolist())
                cache_writes += 1
        if h_r_max is not None and len(h_r_max) == len(xs_max_np):
            cache_hits += 1
            h_r_max_np = np.asarray(h_r_max, dtype=np.float64)
        else:
            seq_r_n, seq_r_g = hx.stream_weighted_events(
                n_max=max_n,
                base=b,
                zeros=zeros_ref,
                u_mode=args.u_mode,
                zero_kernel=args.zero_kernel,
                kernel_scale=args.kernel_scale,
                weights=WEIGHTS,
                chunk_n=args.chunk_n,
                zero_terms=zero_terms_ref,
                event_stride=event_stride,
                event_scale=args.event_scale,
            )
            h_r_max_np = h_from_events(seq_r_n, seq_r_g, xs_max_np)
            if use_cache:
                save_cache(pr, h_r_max_np.tolist())
                cache_writes += 1
        for n_max in n_values:
            k = x_counts[n_max]
            h_m = h_m_max_np[:k]
            h_ref = h_r_max_np[:k]
            out.append(
                {
                    "n_max": n_max,
                    "base": b,
                    "x": x_map[n_max].tolist(),
                    "e_scaled": e_map[n_max].tolist(),
                    "h_m": h_m.tolist(),
                    "h_ref": h_ref.tolist(),
                    "log_beta": l_map[n_max].tolist(),
                }
            )
        return out

    t_build0 = time.perf_counter()
    if args.jobs > 1 and len(bases) > 1:
        workers = min(args.jobs, len(bases))
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
            futs = [ex.submit(build_base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                rows = fut.result()
                for r in rows:
                    cache.append(r)
    else:
        for b in bases:
            rows = build_base_rows(b)
            for r in rows:
                cache.append(r)
    build_seconds = time.perf_counter() - t_build0

    cache.sort(key=lambda r: (r["n_max"], r["base"]))
    t_pool0 = time.perf_counter()
    for r in cache:
        pooled_h_ref.extend(r["h_ref"])
        pooled_e.extend(r["e_scaled"])

    a_ref, b_ref, _ = hx.fit_linear(pooled_h_ref, pooled_e)
    poolfit_seconds = time.perf_counter() - t_pool0

    # Extract uniform constants for A1/A2/A3 on this same grid.
    t_th0 = time.perf_counter()
    c0 = 0.0
    c_delta = 0.0
    c_h = 0.0
    per_row = []
    theorem_gap_max = -1e18
    theorem_viol = 0
    for row in cache:
        e = np.asarray(row["e_scaled"], dtype=np.float64)
        h_m = np.asarray(row["h_m"], dtype=np.float64)
        h_ref = np.asarray(row["h_ref"], dtype=np.float64)
        lb = np.asarray(row["log_beta"], dtype=np.float64)
        xx = np.asarray(row["x"], dtype=np.float64)
        if e.size == 0:
            continue
        resid_ref = np.abs(e - (a_ref * h_ref + b_ref))
        delta = np.abs(h_m - h_ref)
        abs_h = np.abs(h_m)
        c0 = max(c0, float(np.max(resid_ref)))
        c_delta = max(c_delta, float(np.max(delta / np.maximum(1e-30, lb * tau_m))))
        c_h = max(c_h, float(np.max(abs_h / np.maximum(1e-30, np.log(np.maximum(3.0, xx)) ** args.a_h))))

    # Validate theorem-shaped RHS using extracted constants.
    # |E/sqrt(x)| <= |a_ref| C_H (log x)^A_H + |b_ref| + C0 + |a_ref| C_delta (log x)^beta tau(M)
    ratios = []
    for row in cache:
        e = np.asarray(row["e_scaled"], dtype=np.float64)
        h_m = np.asarray(row["h_m"], dtype=np.float64)
        xx = np.asarray(row["x"], dtype=np.float64)
        rhs_static = abs(b_ref) + c0
        lhs = np.abs(e)
        lx = np.log(np.maximum(3.0, xx))
        rhs = (
            abs(a_ref) * c_h * (lx ** args.a_h)
            + rhs_static
            + abs(a_ref) * c_delta * (lx ** args.beta) * tau_m
        )
        gaps = lhs - rhs
        theorem_gap_max = max(theorem_gap_max, float(np.max(gaps)) if gaps.size else -1e18)
        theorem_viol += int(np.sum(gaps > 1e-12))
        if lhs.size:
            ratios.extend((lhs / np.maximum(1e-15, rhs)).tolist())

        # per-row diagnostics
        max_lhs = float(np.max(lhs)) if lhs.size else 0.0
        # Use max x within row for representative rhs max.
        x_m = float(xx[-1])
        rhs_m = (
            abs(a_ref) * c_h * (math.log(max(3, x_m)) ** args.a_h)
            + abs(b_ref)
            + c0
            + abs(a_ref) * c_delta * (math.log(max(3, x_m)) ** args.beta) * tau_m
        )
        per_row.append(
            {
                "n_max": row["n_max"],
                "base": row["base"],
                "max_abs_e_scaled": max_lhs,
                "rhs_at_row_max_x": rhs_m,
                "ratio_lhs_over_rhs": max_lhs / max(1e-15, rhs_m),
                    "abs_corr_h_vs_e": abs(hx.corr(h_m.tolist(), e.tolist())),
                }
            )
    theorem_seconds = time.perf_counter() - t_th0

    # base-uniform spread at each n
    per_n = []
    for n_max in n_values:
        rows = [r for r in per_row if r["n_max"] == n_max]
        lhs_vals = [r["max_abs_e_scaled"] for r in rows]
        rat_vals = [r["ratio_lhs_over_rhs"] for r in rows]
        corr_vals = [r["abs_corr_h_vs_e"] for r in rows]
        ml, sl = mean_std(lhs_vals)
        mr, sr = mean_std(rat_vals)
        mc, sc = mean_std(corr_vals)
        per_n.append(
            {
                "n_max": n_max,
                "per_base": sorted(rows, key=lambda z: z["base"]),
                "uniformity_summary": {
                    "rel_spread_max_abs_e_scaled": sl / max(1e-15, ml),
                    "rel_spread_ratio_lhs_over_rhs": sr / max(1e-15, mr),
                    "rel_spread_abs_corr_h_vs_e": sc / max(1e-15, mc),
                },
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
            "beta": args.beta,
            "a_h": args.a_h,
            "chunk_n": args.chunk_n,
            "event_stride": event_stride,
            "event_scale": bool(args.event_scale),
            "jobs": args.jobs,
            "cache_dir": args.cache_dir,
            "cache_read_dirs": cache_read_dirs,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_writes": cache_writes,
            "cache_misses": max(0, 2 * len(bases) - cache_hits),
        },
        "timing": {
            "build_series_seconds": float(build_seconds),
            "pool_fit_seconds": float(poolfit_seconds),
            "theorem_eval_seconds": float(theorem_seconds),
            "total_seconds": float(time.perf_counter() - t_start),
        },
        "uniform_constants": {
            "a_ref": a_ref,
            "b_ref": b_ref,
            "C0_ref": c0,
            "C_delta": c_delta,
            "tau_m_tail_fp": tau_m,
            "C_H": c_h,
        },
        "theorem_rhs_check": {
            "holds_on_grid": theorem_viol == 0 and theorem_gap_max <= 1e-12,
            "total_violations": int(theorem_viol),
            "max_gap_lhs_minus_rhs": float(theorem_gap_max),
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
        uc = report["uniform_constants"]
        th = report["theorem_rhs_check"]
        f.write("# A4 Uniform Assumption Check\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- a_ref: {uc['a_ref']:.12f}\n")
        f.write(f"- b_ref: {uc['b_ref']:.12f}\n")
        f.write(f"- C0_ref: {uc['C0_ref']:.9f}\n")
        f.write(f"- C_delta: {uc['C_delta']:.9e}\n")
        f.write(f"- tau_m_tail_fp: {uc['tau_m_tail_fp']:.9e}\n")
        f.write(f"- C_H: {uc['C_H']:.9e}\n\n")
        f.write(f"- theorem RHS holds on grid: {th['holds_on_grid']}\n")
        f.write(f"- theorem RHS violations: {th['total_violations']}\n")
        f.write(f"- theorem max gap (lhs-rhs): {th['max_gap_lhs_minus_rhs']:.3e}\n")
        f.write(f"- ratio range lhs/rhs: [{th['ratio_min']:.6f}, {th['ratio_max']:.6f}]\n\n")
        for row in report["per_n"]:
            us = row["uniformity_summary"]
            f.write(f"## n_max={row['n_max']}\n\n")
            f.write(
                f"- rel_spread_lhs={us['rel_spread_max_abs_e_scaled']:.6f} "
                f"rel_spread_ratio={us['rel_spread_ratio_lhs_over_rhs']:.6f} "
                f"rel_spread_corr={us['rel_spread_abs_corr_h_vs_e']:.6f}\n"
            )
            for b in row["per_base"]:
                f.write(
                    f"- base={b['base']} max|E|/sqrt={b['max_abs_e_scaled']:.6f} "
                    f"rhs={b['rhs_at_row_max_x']:.6f} ratio={b['ratio_lhs_over_rhs']:.6f} "
                    f"|corr|={b['abs_corr_h_vs_e']:.6f}\n"
                )
            f.write("\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
