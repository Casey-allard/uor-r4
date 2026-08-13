#!/usr/bin/env python3
"""A1 uplift: residual decomposition via explicit-formula smoothing surrogate."""

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


def parse_ints(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


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


def explicit_surrogate_s(xs: Sequence[int], zeros: Sequence[float]) -> List[float]:
    if not xs or not zeros:
        return [0.0 for _ in xs]
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


def to_builtin(obj):
    if isinstance(obj, dict):
        return {k: to_builtin(v) for k, v in obj.items()}
    if isinstance(obj, (list, tuple)):
        return [to_builtin(v) for v in obj]
    # Handle numpy scalars without importing numpy directly.
    if hasattr(obj, "item") and callable(getattr(obj, "item")):
        try:
            return obj.item()
        except Exception:
            return obj
    return obj


def main() -> None:
    ap = argparse.ArgumentParser(description="A1 smoothing uplift pack")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000")
    ap.add_argument("--valid-n-values", type=str, default="2000000,5000000")
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
    ap.add_argument("--safety-factor", type=float, default=1.2)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a1_smoothing_uplift_pack")
    ap.add_argument(
        "--cache-read-dirs",
        type=str,
        default="research/cache/a1_smoothing_uplift_pack,research/cache/a4_uniform_assumption_check,research/cache/a3_offdiag_dynamic_majorant,research/cache/a3_offdiag_symbolic_chain,research/cache/a3_offdiag_sign_sensitive_lagbound",
    )
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--profile", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a1_smoothing_uplift_pack.json")
    args = ap.parse_args()
    np = s4.np
    if np is None:
        raise RuntimeError("numpy backend unavailable via spinning_top_r4_candidate_a.np")

    bases = sorted(set(parse_ints(args.bases)))
    train_n = sorted(set(parse_ints(args.train_n_values)))
    valid_n = sorted(set(parse_ints(args.valid_n_values)))
    all_n = sorted(set(train_n + valid_n))
    if not all_n:
        raise ValueError("need non-empty n grid")

    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zero data")
    zeros_ref = zeros_all[: args.m_ref]
    zsig = zeros_sig(zeros_ref)

    # Shared x-grid maps.
    x0 = max(2, args.x_step)
    max_n = max(all_n)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    xs_max_np = np.asarray(xs_max, dtype=np.int64)
    x_float_max = xs_max_np.astype(np.float64)
    psi_max = hx.psi_exact_samples(xs_max)
    psi_max_np = np.asarray(psi_max, dtype=np.float64)
    e_max_np = (psi_max_np - x_float_max) / np.sqrt(np.maximum(1.0, x_float_max))
    x_map = {}
    e_map = {}
    c_map = {}
    s_map = {}
    s_max = explicit_surrogate_s(xs_max, zeros_ref)
    s_max_np = np.asarray(s_max, dtype=np.float64)
    for n in all_n:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n] = xs_max_np[:k]
        e_map[n] = e_max_np[:k]
        c_map[n] = k
        s_map[n] = s_max_np[:k]

    # Build H_ref per base once with cache.
    zero_terms = hx.prepare_zero_terms(zeros_ref, args.zero_kernel, args.kernel_scale)
    event_stride = max(1, args.event_stride)
    use_cache = not args.no_cache
    cache_hits = 0
    cache_writes = 0
    cache_read_dirs = [x.strip() for x in args.cache_read_dirs.split(",") if x.strip()]
    if args.cache_dir not in cache_read_dirs:
        cache_read_dirs = [args.cache_dir] + cache_read_dirs
    t_start = time.perf_counter()
    build_seconds = 0.0
    fit_seconds = 0.0

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

    h_by_base = {}

    def build_base_h(b: int):
        nonlocal cache_hits
        nonlocal cache_writes
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
        h_max, _ = load_cache_from_dirs(cache_read_dirs, key) if use_cache else (None, None)
        if h_max is not None and len(h_max) == len(xs_max_np):
            cache_hits += 1
            return b, np.asarray(h_max, dtype=np.float64)
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
        h_max = h_from_events(seq_n, g_vals, xs_max_np)
        if use_cache:
            save_cache(cp, h_max.tolist())
            cache_writes += 1
        return b, h_max

    t_build0 = time.perf_counter()
    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(build_base_h, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                b, h = fut.result()
                h_by_base[b] = h
    else:
        for b in bases:
            bb, h = build_base_h(b)
            h_by_base[bb] = h
    build_seconds = time.perf_counter() - t_build0

    # Fit affine maps on train split only.
    t_fit0 = time.perf_counter()
    pooled_h = []
    pooled_e = []
    pooled_s = []
    for n in train_n:
        e = np.asarray(e_map[n], dtype=np.float64)
        s = np.asarray(s_map[n], dtype=np.float64)
        for b in bases:
            h = np.asarray(h_by_base[b][: c_map[n]], dtype=np.float64)
            pooled_h.extend(h.tolist())
            pooled_e.extend(e.tolist())
            pooled_s.extend(s.tolist())

    if len(pooled_h) < 2:
        raise ValueError("insufficient train points")

    a_ref, b_ref, _ = hx.fit_linear(pooled_h, pooled_e)
    a_se, b_se, _ = hx.fit_linear(pooled_s, pooled_e)

    # Compute decomposition constants on train.
    c_smooth = 0.0
    c_link = 0.0
    for n in train_n:
        e = np.asarray(e_map[n], dtype=np.float64)
        s = np.asarray(s_map[n], dtype=np.float64)
        for b in bases:
            h = np.asarray(h_by_base[b][: c_map[n]], dtype=np.float64)
            smooth_res = np.abs(e - (a_se * s + b_se))
            link_res = np.abs((a_ref * h + b_ref) - (a_se * s + b_se))
            c_smooth = max(c_smooth, float(np.max(smooth_res)) if smooth_res.size else 0.0)
            c_link = max(c_link, float(np.max(link_res)) if link_res.size else 0.0)
    c0_train = c_smooth + c_link
    c0_uplift = c0_train * max(1.0, args.safety_factor)

    # Validate on train/valid.
    def eval_split(nset: Sequence[int]):
        viol = 0
        max_gap = -1e18
        ratios = []
        for n in nset:
            e = np.asarray(e_map[n], dtype=np.float64)
            for b in bases:
                h = np.asarray(h_by_base[b][: c_map[n]], dtype=np.float64)
                lhs = np.abs(e - (a_ref * h + b_ref))
                rhs = max(1e-30, c0_uplift)
                gaps = lhs - rhs
                if gaps.size:
                    viol += int(np.sum(gaps > 1e-12))
                    max_gap = max(max_gap, float(np.max(gaps)))
                    ratios.extend((lhs / rhs).tolist())
        return {
            "holds": viol == 0 and max_gap <= 1e-12,
            "violations": int(viol),
            "max_gap_lhs_minus_rhs": float(max_gap),
            "ratio_min": min(ratios) if ratios else 0.0,
            "ratio_max": max(ratios) if ratios else 0.0,
        }
    fit_seconds = time.perf_counter() - t_fit0

    train_chk = eval_split(train_n)
    valid_chk = eval_split(valid_n)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
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
            "cache_read_dirs": cache_read_dirs,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_writes": cache_writes,
            "cache_misses": max(0, len(bases) - cache_hits),
            "safety_factor": max(1.0, args.safety_factor),
        },
        "timing": {
            "build_series_seconds": float(build_seconds),
            "fit_and_checks_seconds": float(fit_seconds),
            "total_seconds": float(time.perf_counter() - t_start),
        },
        "affine_maps_train": {
            "a_ref": a_ref,
            "b_ref": b_ref,
            "a_s_to_e": a_se,
            "b_s_to_e": b_se,
        },
        "decomposition_constants": {
            "C_smooth_train": c_smooth,
            "C_link_train": c_link,
            "C0_train_sum": c0_train,
            "C0_uplifted": c0_uplift,
        },
        "checks": {"train": train_chk, "valid": valid_chk},
    }

    report = to_builtin(report)
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A1 Smoothing Uplift Pack\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        d = report["decomposition_constants"]
        f.write(f"- C_smooth_train: {d['C_smooth_train']:.12f}\n")
        f.write(f"- C_link_train: {d['C_link_train']:.12f}\n")
        f.write(f"- C0_uplifted: {d['C0_uplifted']:.12f}\n")
        f.write(f"- train holds: {train_chk['holds']} (viol={train_chk['violations']})\n")
        f.write(f"- valid holds: {valid_chk['holds']} (viol={valid_chk['violations']})\n")
        f.write(f"- valid max gap: {valid_chk['max_gap_lhs_minus_rhs']:.6e}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
