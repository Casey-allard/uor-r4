#!/usr/bin/env python3
"""A3 offdiag dynamic majorant with eta(x) envelope."""

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
    fixed_a: float | None = None,
):
    train_mask = np.isin(n_arr, np.asarray(train_n, dtype=np.int64))
    valid_mask = np.isin(n_arr, np.asarray(valid_n, dtype=np.int64))
    if not np.any(train_mask) or not np.any(valid_mask):
        raise ValueError("empty split for fit")

    x_train = x_arr[train_mask]
    y_train = y_arr[train_mask]
    x_valid = x_arr[valid_mask]
    y_valid = y_arr[valid_mask]
    max_x_all = float(np.max(x_arr)) if x_arr.size else 3.0
    best = None
    if fixed_a is not None:
        a_values = [float(fixed_a)]
    else:
        a_values = []
        a = a_min
        while a <= a_max + 1e-12:
            a_values.append(float(a))
            a += a_step

    for a in a_values:
        den_train = np.maximum(1e-30, np.log(np.maximum(3.0, x_train)) ** a)
        den_valid = np.maximum(1e-30, np.log(np.maximum(3.0, x_valid)) ** a)
        train_scaled = y_train / den_train
        c_train = float(np.max(train_scaled)) if train_scaled.size else 0.0
        c_u = c_train * max(1.0, safety_factor)
        rhs_train = np.maximum(1e-30, c_u * den_train)
        rhs_valid = np.maximum(1e-30, c_u * den_valid)
        r_train = float(np.max(y_train / rhs_train)) if rhs_train.size else 0.0
        r_valid = float(np.max(y_valid / rhs_valid)) if rhs_valid.size else 0.0
        holds = r_train <= 1.0 + 1e-12 and r_valid <= 1.0 + 1e-12
        rhs_at_max_x = c_u * (math.log(max(3.0, max_x_all)) ** a)
        cand = {
            "A": float(a),
            "C_train_max": float(c_train),
            "C_uplifted": float(c_u),
            "ratio_max_train": float(r_train),
            "ratio_max_valid": float(r_valid),
            "rhs_at_max_x": float(rhs_at_max_x),
            "holds": bool(holds),
        }
        if holds:
            if best is None:
                best = cand
            elif (cand["rhs_at_max_x"] < best["rhs_at_max_x"]) or (
                cand["rhs_at_max_x"] == best["rhs_at_max_x"] and cand["A"] < best["A"]
            ):
                best = cand
    if best is None:
        raise ValueError("no held-out-valid envelope candidate found")
    return best


def fit_piecewise_eta(
    np,
    n_arr,
    x_arr,
    eta_pos_arr,
    train_n: Sequence[int],
    valid_n: Sequence[int],
    bins: int,
    safety: float,
):
    train_mask = np.isin(n_arr, np.asarray(train_n, dtype=np.int64))
    valid_mask = np.isin(n_arr, np.asarray(valid_n, dtype=np.int64))
    if not np.any(train_mask) or not np.any(valid_mask):
        raise ValueError("empty split for piecewise eta")

    lx_train = np.log(np.maximum(3.0, x_arr[train_mask]))
    if bins < 1:
        bins = 1
    q = np.linspace(0.0, 1.0, bins + 1)
    edges = np.quantile(lx_train, q).astype(np.float64)
    for i in range(1, edges.size):
        if edges[i] <= edges[i - 1]:
            edges[i] = edges[i - 1] + 1e-12

    train_idx = np.searchsorted(edges, lx_train, side="right") - 1
    train_idx = np.clip(train_idx, 0, bins - 1)
    eta_train = eta_pos_arr[train_mask]
    train_max = np.zeros(bins, dtype=np.float64)
    for i in range(bins):
        m = train_idx == i
        if np.any(m):
            train_max[i] = float(np.max(eta_train[m]))
    eta_by_bin = train_max * max(1.0, safety)

    def ratio_max(mask):
        lx = np.log(np.maximum(3.0, x_arr[mask]))
        yy = eta_pos_arr[mask]
        idx = np.searchsorted(edges, lx, side="right") - 1
        idx = np.clip(idx, 0, bins - 1)
        rhs = np.maximum(1e-30, eta_by_bin[idx])
        return float(np.max(yy / rhs)) if yy.size else 0.0

    return {
        "mode": "piecewise_constant",
        "bins": int(bins),
        "edges_log_x": edges.tolist(),
        "eta_by_bin": eta_by_bin.tolist(),
        "ratio_max_train": ratio_max(train_mask),
        "ratio_max_valid": ratio_max(valid_mask),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 offdiag dynamic majorant")
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
    ap.add_argument("--eta-a-min", type=float, default=0.0)
    ap.add_argument("--eta-a-max", type=float, default=12.0)
    ap.add_argument("--eta-a-step", type=float, default=0.1)
    ap.add_argument("--eta-a-fixed", type=float, default=None)
    ap.add_argument("--eta-safety", type=float, default=2.0)
    ap.add_argument("--eta-model", type=str, default="piecewise", choices=["piecewise", "global_log"])
    ap.add_argument("--eta-bins", type=int, default=8)
    ap.add_argument("--h-a-min", type=float, default=0.0)
    ap.add_argument("--h-a-max", type=float, default=6.0)
    ap.add_argument("--h-a-step", type=float, default=0.1)
    ap.add_argument("--h-safety", type=float, default=1.1)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_offdiag_dynamic_majorant")
    ap.add_argument(
        "--cache-read-dirs",
        type=str,
        default="research/cache/a3_offdiag_dynamic_majorant,research/cache/a3_offdiag_symbolic_chain,research/cache/a3_offdiag_sign_sensitive_lagbound",
    )
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--profile", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_offdiag_dynamic_majorant.json")
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
        raise ValueError("m-zero exceeds available zero data")
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
    rows_seconds = 0.0
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

    t_rows0 = time.perf_counter()
    n_chunks = []
    x_chunks = []
    h_chunks = []
    e2_chunks = []
    eta_chunks = []
    for b in bases:
        seq_n, g_vals = series_by_base[b]
        seq_n_np = np.asarray(seq_n, dtype=np.int64)
        g = np.asarray(g_vals, dtype=np.float64)
        if g.size == 0:
            continue
        pref_s = np.cumsum(g)
        pref_e2 = np.cumsum(g * g)
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
            s = pref_s[im]
            e2 = pref_e2[im]
            h_abs = np.abs(s) / np.sqrt(np.maximum(1.0, x_keep))
            eta_pos = np.maximum(0.0, (s * s - e2) / np.maximum(1e-30, e2))

            n_chunks.append(np.full(im.size, int(n), dtype=np.int64))
            x_chunks.append(x_keep)
            h_chunks.append(h_abs)
            e2_chunks.append(e2)
            eta_chunks.append(eta_pos)

    if not n_chunks:
        raise ValueError("no rows produced for requested grid")
    n_arr = np.concatenate(n_chunks)
    x_arr = np.concatenate(x_chunks)
    h_abs_arr = np.concatenate(h_chunks)
    e2_arr = np.concatenate(e2_chunks)
    eta_pos_arr = np.concatenate(eta_chunks)
    rows_seconds = time.perf_counter() - t_rows0

    t_fit0 = time.perf_counter()
    if args.eta_model == "global_log":
        eta_model = fit_log_envelope(
            np=np,
            n_arr=n_arr,
            x_arr=x_arr,
            y_arr=eta_pos_arr,
            train_n=train_n,
            valid_n=valid_n,
            a_min=args.eta_a_min,
            a_max=args.eta_a_max,
            a_step=args.eta_a_step,
            safety_factor=args.eta_safety,
            fixed_a=args.eta_a_fixed,
        )
        a_eta = float(eta_model["A"])
        c_eta = float(eta_model["C_uplifted"])
        eta_x_arr = c_eta * (np.log(np.maximum(3.0, x_arr)) ** a_eta)
    else:
        eta_model = fit_piecewise_eta(
            np=np,
            n_arr=n_arr,
            x_arr=x_arr,
            eta_pos_arr=eta_pos_arr,
            train_n=train_n,
            valid_n=valid_n,
            bins=max(1, args.eta_bins),
            safety=max(1.0, args.eta_safety),
        )
        a_eta = 0.0
        c_eta = 0.0
        edges = np.asarray(eta_model["edges_log_x"], dtype=np.float64)
        vals = np.asarray(eta_model["eta_by_bin"], dtype=np.float64)
        lx = np.log(np.maximum(3.0, x_arr))
        idx = np.searchsorted(edges, lx, side="right") - 1
        idx = np.clip(idx, 0, max(0, vals.size - 1))
        eta_x_arr = vals[idx]

    h_upper_arr = np.sqrt(np.maximum(0.0, (1.0 + eta_x_arr) * e2_arr / np.maximum(1.0, x_arr)))
    det_gap_arr = h_abs_arr - h_upper_arr

    h_model = fit_log_envelope(
        np=np,
        n_arr=n_arr,
        x_arr=x_arr,
        y_arr=h_upper_arr,
        train_n=train_n,
        valid_n=valid_n,
        a_min=args.h_a_min,
        a_max=args.h_a_max,
        a_step=args.h_a_step,
        safety_factor=args.h_safety,
    )
    a_h = float(h_model["A"])
    c_h = float(h_model["C_uplifted"])
    fit_seconds = time.perf_counter() - t_fit0

    def check_split(ns: set):
        mask = np.isin(n_arr, np.asarray(sorted(ns), dtype=np.int64))
        if not np.any(mask):
            return {
                "holds": True,
                "violations": 0,
                "max_gap_h_minus_rhs": -1e18,
                "ratio_min": 0.0,
                "ratio_max": 0.0,
                "deterministic_dynamic_eta_holds": True,
                "deterministic_dynamic_eta_violations": 0,
                "deterministic_dynamic_eta_max_gap": -1e18,
            }
        x_s = x_arr[mask]
        h_s = h_abs_arr[mask]
        det_s = det_gap_arr[mask]
        rhs = c_h * (np.log(np.maximum(3.0, x_s)) ** a_h)
        rhs = np.maximum(1e-30, rhs)
        gaps = h_s - rhs
        ratios = h_s / rhs
        det_v = int(np.sum(det_s > 1e-12))
        det_gap = float(np.max(det_s))
        v = int(np.sum(gaps > 1e-12))
        gap = float(np.max(gaps))
        return {
            "holds": v == 0 and gap <= 1e-12,
            "violations": int(v),
            "max_gap_h_minus_rhs": float(gap),
            "ratio_min": float(np.min(ratios)),
            "ratio_max": float(np.max(ratios)),
            "deterministic_dynamic_eta_holds": det_v == 0 and det_gap <= 1e-12,
            "deterministic_dynamic_eta_violations": int(det_v),
            "deterministic_dynamic_eta_max_gap": float(det_gap),
        }

    train_chk = check_split(set(train_n))
    valid_chk = check_split(set(valid_n))

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
            "eta_a_min": args.eta_a_min,
            "eta_a_max": args.eta_a_max,
            "eta_a_step": args.eta_a_step,
            "eta_a_fixed": args.eta_a_fixed,
            "eta_safety": max(1.0, args.eta_safety),
            "eta_model": args.eta_model,
            "eta_bins": int(args.eta_bins),
            "h_a_min": args.h_a_min,
            "h_a_max": args.h_a_max,
            "h_a_step": args.h_a_step,
            "h_safety": max(1.0, args.h_safety),
            "cache_dir": args.cache_dir,
            "cache_read_dirs": cache_read_dirs,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_writes": cache_writes,
            "cache_misses": max(0, len(bases) - cache_hits),
        },
        "timing": {
            "build_series_seconds": float(build_seconds),
            "row_build_seconds": float(rows_seconds),
            "fit_and_checks_seconds": float(fit_seconds),
            "total_seconds": float(time.perf_counter() - t_start),
        },
        "eta_envelope": {
            "A_eta": a_eta,
            "C_eta_uplifted": c_eta,
            "ratio_max_train": float(eta_model["ratio_max_train"]),
            "ratio_max_valid": float(eta_model["ratio_max_valid"]),
            "target": "eta_pos upper model",
            "model": eta_model,
        },
        "h_upper_envelope": {
            "A_H": a_h,
            "C_H_uplifted": c_h,
            "ratio_max_train": float(h_model["ratio_max_train"]),
            "ratio_max_valid": float(h_model["ratio_max_valid"]),
            "target": "|H| <= sqrt((1+eta(x))*E2/x)",
        },
        "h_transfer_envelope": {
            "A_H": a_h,
            "C_H_from_density_transfer": c_h,
            "transfer_identity": "|H| <= sqrt((1+eta(x))*E2/x) <= C_H(log x)^A_H",
        },
        "checks": {"train": train_chk, "valid": valid_chk},
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Offdiag Dynamic Majorant\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        ee = report["eta_envelope"]
        he = report["h_upper_envelope"]
        f.write(f"- A_eta: {ee['A_eta']:.6f}\n")
        f.write(f"- C_eta_uplifted: {ee['C_eta_uplifted']:.12e}\n")
        f.write(f"- A_H: {he['A_H']:.6f}\n")
        f.write(f"- C_H_uplifted: {he['C_H_uplifted']:.12e}\n")
        f.write(f"- train holds: {report['checks']['train']['holds']}\n")
        f.write(f"- valid holds: {report['checks']['valid']['holds']}\n")
        f.write(f"- deterministic dynamic-eta holds(train): {report['checks']['train']['deterministic_dynamic_eta_holds']}\n")
        f.write(f"- deterministic dynamic-eta holds(valid): {report['checks']['valid']['deterministic_dynamic_eta_holds']}\n")
    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
