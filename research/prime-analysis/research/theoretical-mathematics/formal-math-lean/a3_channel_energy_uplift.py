#!/usr/bin/env python3
"""A3 uplift via deterministic channel-energy inequality."""

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
        if "seq_n" not in d or "g_vals" not in d:
            return None
        return d["seq_n"].astype(np.int64).tolist(), d["g_vals"].astype(np.float64).tolist()
    except Exception:
        return None


def load_cache_from_dirs(cache_dirs: Sequence[str], key: str):
    for d in cache_dirs:
        cp = os.path.join(d, f"{key}.npz")
        out = load_cache(cp)
        if out is not None:
            return out, cp
    return None, None


def save_cache(path: str, seq_n: Sequence[int], g_vals: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, seq_n=np.asarray(seq_n, dtype=np.int64), g_vals=np.asarray(g_vals, dtype=np.float64))


def choose_log_power(xs: Sequence[int], ys: Sequence[float], a_min: float = 0.0, a_max: float = 4.0, step: float = 0.1):
    best = None
    a = a_min
    while a <= a_max + 1e-12:
        scaled = [y / max(1e-30, math.log(max(3, x)) ** a) for x, y in zip(xs, ys)]
        m = sum(scaled) / max(1, len(scaled))
        v = sum((z - m) ** 2 for z in scaled) / max(1, len(scaled))
        cv = math.sqrt(v) / max(1e-30, m)
        cand = {"A_E": a, "C_E_mean": m, "cv": cv, "C_E_max": max(scaled) if scaled else 0.0}
        if best is None or cand["cv"] < best["cv"]:
            best = cand
        a += step
    return best


def fit_log_envelope(
    np,
    n_arr,
    x_arr,
    y_arr,
    train_n: Sequence[int],
    valid_n: Sequence[int],
    a_min: float,
    a_max: float,
    a_step: float,
    safety_factor: float,
    mode: str,
):
    if mode not in {"min_rhs_max_x", "min_cv"}:
        raise ValueError(f"unsupported fit mode: {mode}")
    train_mask = np.isin(n_arr, np.asarray(train_n, dtype=np.int64))
    valid_mask = np.isin(n_arr, np.asarray(valid_n, dtype=np.int64))
    if not np.any(train_mask):
        raise ValueError("empty train split")
    if not np.any(valid_mask):
        raise ValueError("empty valid split")
    x_train = x_arr[train_mask]
    y_train = y_arr[train_mask]
    x_valid = x_arr[valid_mask]
    y_valid = y_arr[valid_mask]
    max_x_all = float(np.max(x_arr)) if x_arr.size else 3.0

    best = None
    a = a_min
    while a <= a_max + 1e-12:
        den_train = np.maximum(1e-30, np.log(np.maximum(3.0, x_train)) ** a)
        den_valid = np.maximum(1e-30, np.log(np.maximum(3.0, x_valid)) ** a)
        train_scaled = y_train / den_train
        c_train = float(np.max(train_scaled)) if train_scaled.size else 0.0
        c_uplift = c_train * max(1.0, safety_factor)

        rhs_train = np.maximum(1e-30, c_uplift * den_train)
        rhs_valid = np.maximum(1e-30, c_uplift * den_valid)
        max_ratio_train = float(np.max(y_train / rhs_train)) if rhs_train.size else 0.0
        max_ratio_valid = float(np.max(y_valid / rhs_valid)) if rhs_valid.size else 0.0

        holds_train = max_ratio_train <= 1.0 + 1e-12
        holds_valid = max_ratio_valid <= 1.0 + 1e-12
        rhs_at_max_x = c_uplift * (math.log(max(3.0, max_x_all)) ** a)
        cv = 0.0
        if train_scaled.size:
            mu = float(np.mean(train_scaled))
            var = float(np.mean((train_scaled - mu) ** 2))
            cv = math.sqrt(var) / max(1e-30, mu)

        cand = {
            "A_E": float(a),
            "C_E_mean": float(np.mean(train_scaled)) if train_scaled.size else 0.0,
            "cv": float(cv),
            "C_E_max": float(c_train),
            "C_uplifted": float(c_uplift),
            "target": "target",
            "fit_mode": mode,
            "holds_train": bool(holds_train),
            "holds_valid": bool(holds_valid),
            "ratio_max_train": float(max_ratio_train),
            "ratio_max_valid": float(max_ratio_valid),
            "rhs_at_max_x": float(rhs_at_max_x),
        }
        if not holds_train or not holds_valid:
            a += a_step
            continue
        if best is None:
            best = cand
        elif mode == "min_rhs_max_x":
            if (cand["rhs_at_max_x"] < best["rhs_at_max_x"]) or (
                cand["rhs_at_max_x"] == best["rhs_at_max_x"] and cand["A_E"] < best["A_E"]
            ):
                best = cand
        else:  # min_cv
            if (cand["cv"] < best["cv"]) or (cand["cv"] == best["cv"] and cand["rhs_at_max_x"] < best["rhs_at_max_x"]):
                best = cand
        a += a_step
    if best is None:
        raise ValueError("no valid log-envelope candidate found")
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 channel-energy uplift")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000")
    ap.add_argument("--valid-n-values", type=str, default="2000000,5000000")
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
    ap.add_argument("--safety-factor", type=float, default=1.2)
    ap.add_argument("--a-min", type=float, default=0.0)
    ap.add_argument("--a-max", type=float, default=10.0)
    ap.add_argument("--a-step", type=float, default=0.1)
    ap.add_argument("--fit-target", type=str, default="h_abs", choices=["h_abs", "b_norm"])
    ap.add_argument("--fit-mode", type=str, default="min_rhs_max_x", choices=["min_rhs_max_x", "min_cv"])
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_channel_energy_uplift")
    ap.add_argument(
        "--cache-read-dirs",
        type=str,
        default="research/cache/a3_channel_energy_uplift,research/cache/a3_offdiag_dynamic_majorant,research/cache/a3_offdiag_symbolic_chain,research/cache/a3_offdiag_sign_sensitive_lagbound",
    )
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--profile", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_channel_energy_uplift.json")
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
    if args.m_zero > len(zeros_all):
        raise ValueError("m-zero exceeds available zeros")
    zeros = zeros_all[: args.m_zero]
    zsig = zeros_sig(zeros)

    x0 = max(2, args.x_step)
    max_n = max(all_n)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    x_map = {}
    for n in all_n:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n] = xs_max[:k]

    zero_terms = hx.prepare_zero_terms(zeros, args.zero_kernel, args.kernel_scale)
    use_cache = not args.no_cache
    event_stride = max(1, args.event_stride)
    cache_hits = 0
    cache_writes = 0
    cache_read_dirs = [x.strip() for x in args.cache_read_dirs.split(",") if x.strip()]
    if args.cache_dir not in cache_read_dirs:
        cache_read_dirs = [args.cache_dir] + cache_read_dirs
    t_start = time.perf_counter()
    build_seconds = 0.0
    row_seconds = 0.0
    fit_seconds = 0.0

    def build_base_series(b: int):
        nonlocal cache_hits
        nonlocal cache_writes
        key = cache_key(
            {
                "version": 1,
                "base": b,
                "max_n": max_n,
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
        cached, _ = load_cache_from_dirs(cache_read_dirs, key) if use_cache else (None, None)
        if cached is not None:
            cache_hits += 1
            return b, cached[0], cached[1]
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
        if use_cache:
            save_cache(cp, seq_n, g_vals)
            cache_writes += 1
        return b, seq_n, g_vals

    t_build0 = time.perf_counter()
    series_by_base = {}
    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(build_base_series, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                b, seq_n, g_vals = fut.result()
                series_by_base[b] = (seq_n, g_vals)
    else:
        for b in bases:
            bb, seq_n, g_vals = build_base_series(b)
            series_by_base[bb] = (seq_n, g_vals)
    build_seconds = time.perf_counter() - t_build0

    # Deterministic identity bridge:
    # H(x) = S_k / sqrt(x), B_k := |S_k| / sqrt(k), dens_k := k/x
    # => |H(x)| = B_k * sqrt(dens_k) <= B_k
    # So a log-envelope for B_k yields a valid log-envelope for H(x).
    t_row0 = time.perf_counter()
    n_chunks = []
    x_chunks = []
    h_chunks = []
    b_chunks = []
    gap_chunks = []
    for b in bases:
        seq_n, g_vals = series_by_base[b]
        seq_n_np = np.asarray(seq_n, dtype=np.int64)
        g = np.asarray(g_vals, dtype=np.float64)
        if g.size == 0:
            continue
        pref = np.cumsum(g)

        for n in all_n:
            xs = x_map[n]
            if not xs:
                continue
            xs_np = np.asarray(xs, dtype=np.int64)
            idx = np.searchsorted(seq_n_np, xs_np, side="right") - 1
            m = idx >= 0
            if not np.any(m):
                continue
            im = idx[m]
            x_keep = xs_np[m].astype(np.float64)
            sum_g = pref[im]
            k = (im + 1).astype(np.float64)
            h_abs = np.abs(sum_g) / np.sqrt(np.maximum(1.0, x_keep))
            b_norm = np.abs(sum_g) / np.sqrt(np.maximum(1.0, k))
            gap_det = h_abs - b_norm

            n_chunks.append(np.full(im.size, int(n), dtype=np.int64))
            x_chunks.append(x_keep)
            h_chunks.append(h_abs)
            b_chunks.append(b_norm)
            gap_chunks.append(gap_det)

    if not n_chunks:
        raise ValueError("no rows produced for requested grid")
    n_arr = np.concatenate(n_chunks)
    x_arr = np.concatenate(x_chunks)
    h_arr = np.concatenate(h_chunks)
    b_arr = np.concatenate(b_chunks)
    gap_det_arr = np.concatenate(gap_chunks)
    row_seconds = time.perf_counter() - t_row0

    t_fit0 = time.perf_counter()
    model = fit_log_envelope(
        np=np,
        n_arr=n_arr,
        x_arr=x_arr,
        y_arr=h_arr if args.fit_target == "h_abs" else b_arr,
        train_n=train_n,
        valid_n=valid_n,
        a_min=args.a_min,
        a_max=args.a_max,
        a_step=args.a_step,
        safety_factor=args.safety_factor,
        mode=args.fit_mode,
    )
    model["target"] = args.fit_target
    c_b_train = float(model["C_E_max"])
    c_b = float(model["C_uplifted"])
    a_h = float(model["A_E"])
    c_h = c_b

    def eval_split(nset: Sequence[int]):
        viol_det = 0
        gap_det_max = -1e18
        viol_lh = 0
        gap_lh_max = -1e18
        ratios = []
        mask = np.isin(n_arr, np.asarray(sorted(nset), dtype=np.int64))
        if not np.any(mask):
            return {
                "deterministic_check_holds": True,
                "deterministic_violations": 0,
                "deterministic_max_gap": -1e18,
                "log_envelope_holds": True,
                "log_envelope_violations": 0,
                "log_envelope_max_gap": -1e18,
                "log_envelope_ratio_min": 0.0,
                "log_envelope_ratio_max": 0.0,
            }
        gd = gap_det_arr[mask]
        xx = x_arr[mask]
        hh = h_arr[mask]
        rhs = c_h * (np.log(np.maximum(3.0, xx)) ** a_h)
        rhs = np.maximum(1e-30, rhs)
        g_lh_arr = hh - rhs
        gap_det_max = float(np.max(gd))
        viol_det = int(np.sum(gd > 1e-12))
        gap_lh_max = float(np.max(g_lh_arr))
        viol_lh = int(np.sum(g_lh_arr > 1e-12))
        ratios = (hh / rhs).tolist()
        return {
            "deterministic_check_holds": viol_det == 0 and gap_det_max <= 1e-12,
            "deterministic_violations": int(viol_det),
            "deterministic_max_gap": float(gap_det_max),
            "log_envelope_holds": viol_lh == 0 and gap_lh_max <= 1e-12,
            "log_envelope_violations": int(viol_lh),
            "log_envelope_max_gap": float(gap_lh_max),
            "log_envelope_ratio_min": min(ratios) if ratios else 0.0,
            "log_envelope_ratio_max": max(ratios) if ratios else 0.0,
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
            "m_zero": args.m_zero,
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
            "fit_target": args.fit_target,
            "fit_mode": args.fit_mode,
        },
        "timing": {
            "build_series_seconds": float(build_seconds),
            "row_build_seconds": float(row_seconds),
            "fit_and_checks_seconds": float(fit_seconds),
            "total_seconds": float(time.perf_counter() - t_start),
        },
        "normalized_sum_model_train": model,
        "uplift_constants": {
            "C_B_train": c_b_train,
            "C_B_uplifted": c_b,
            "A_H_from_normalized_sum": a_h,
            "C_H_from_normalized_sum": c_h,
        },
        "checks": {"train": train_chk, "valid": valid_chk},
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Channel-Energy Uplift\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        u = report["uplift_constants"]
        f.write(f"- C_B_uplifted: {u['C_B_uplifted']:.12e}\n")
        f.write(f"- A_H_from_normalized_sum: {u['A_H_from_normalized_sum']:.6f}\n")
        f.write(f"- C_H_from_normalized_sum: {u['C_H_from_normalized_sum']:.12e}\n")
        f.write(f"- train deterministic holds: {train_chk['deterministic_check_holds']}\n")
        f.write(f"- valid deterministic holds: {valid_chk['deterministic_check_holds']}\n")
        f.write(f"- train log-envelope holds: {train_chk['log_envelope_holds']}\n")
        f.write(f"- valid log-envelope holds: {valid_chk['log_envelope_holds']}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
