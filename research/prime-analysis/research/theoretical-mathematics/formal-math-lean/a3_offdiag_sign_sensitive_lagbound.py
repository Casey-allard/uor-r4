#!/usr/bin/env python3
"""Sign-sensitive lag-band bound probe for A3 offdiag eta_+(x)."""

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


def build_lag_list(
    lag_cut: int,
    mode: str,
    dense_cut: int,
    tail_mult: float,
) -> List[int]:
    m = max(1, int(lag_cut))
    if mode == "dense":
        return list(range(1, m + 1))
    dense = max(1, min(m, int(dense_cut)))
    out = list(range(1, dense + 1))
    if dense >= m:
        return out
    cur = float(dense)
    mul = max(1.05, float(tail_mult))
    seen = set(out)
    while cur < m:
        cur = max(cur + 1.0, cur * mul)
        d = min(m, int(round(cur)))
        if d not in seen:
            out.append(d)
            seen.add(d)
        if d >= m:
            break
    if out[-1] != m:
        out.append(m)
    return sorted(set(out))


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
    rows: Sequence[Dict[str, float]],
    train_n: Sequence[int],
    valid_n: Sequence[int],
    target_key: str,
    a_eta: float,
    safety_factor: float,
):
    tset = set(int(v) for v in train_n)
    vset = set(int(v) for v in valid_n)
    train_rows = [r for r in rows if int(r["n_max"]) in tset]
    valid_rows = [r for r in rows if int(r["n_max"]) in vset]
    if not train_rows or not valid_rows:
        raise ValueError("empty split for fit")

    train_scaled = []
    valid_scaled = []
    for r in train_rows:
        den = max(1e-30, math.log(max(3.0, float(r["x"]))) ** a_eta)
        train_scaled.append(float(r[target_key]) / den)
    for r in valid_rows:
        den = max(1e-30, math.log(max(3.0, float(r["x"]))) ** a_eta)
        valid_scaled.append(float(r[target_key]) / den)

    c_train = max(train_scaled) if train_scaled else 0.0
    c_u = c_train * max(1.0, safety_factor)
    r_train = 0.0
    r_valid = 0.0
    for r in train_rows:
        rhs = c_u * (math.log(max(3.0, float(r["x"]))) ** a_eta)
        r_train = max(r_train, float(r[target_key]) / max(1e-30, rhs))
    for r in valid_rows:
        rhs = c_u * (math.log(max(3.0, float(r["x"]))) ** a_eta)
        r_valid = max(r_valid, float(r[target_key]) / max(1e-30, rhs))

    return {
        "A": float(a_eta),
        "C_train_max": float(c_train),
        "C_uplifted": float(c_u),
        "ratio_max_train": float(r_train),
        "ratio_max_valid": float(r_valid),
        "holds": bool(r_train <= 1.0 + 1e-12 and r_valid <= 1.0 + 1e-12),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 sign-sensitive lag-band bound probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000,2000000")
    ap.add_argument("--valid-n-values", type=str, default="5000000,10000000")
    ap.add_argument("--x-step", type=int, default=2500)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--a-eta", type=float, default=4.0)
    ap.add_argument("--lag-cut", type=int, default=32)
    ap.add_argument("--lag-mode", type=str, default="adaptive", choices=["dense", "adaptive"])
    ap.add_argument("--lag-dense-cut", type=int, default=12)
    ap.add_argument("--lag-tail-mult", type=float, default=1.4)
    ap.add_argument("--tail-calib-safety", type=float, default=1.05)
    ap.add_argument("--tail-mode", type=str, default="empirical", choices=["empirical", "deterministic"])
    ap.add_argument("--k-tail-fixed", type=float, default=1.0)
    ap.add_argument("--envelope-safety", type=float, default=2.0)
    ap.add_argument("--c-eta-budget", type=float, default=1.2170134478356474)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_offdiag_dynamic_majorant")
    ap.add_argument(
        "--cache-read-dirs",
        type=str,
        default="research/cache/a3_offdiag_dynamic_majorant,research/cache/a3_offdiag_symbolic_chain,research/cache/a3_offdiag_sign_sensitive_lagbound",
    )
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--profile", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17.json")
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
    lag_cut = max(1, int(args.lag_cut))
    lag_list = build_lag_list(lag_cut, args.lag_mode, args.lag_dense_cut, args.lag_tail_mult)
    cache_hits = 0
    cache_writes = 0
    cache_read_dirs = [x.strip() for x in args.cache_read_dirs.split(",") if x.strip()]
    if args.cache_dir not in cache_read_dirs:
        cache_read_dirs = [args.cache_dir] + cache_read_dirs
    t_start = time.perf_counter()
    build_seconds = 0.0
    lagprep_seconds = 0.0
    eval_seconds = 0.0

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

    rows = []
    tail_calib_requirements = []
    low_lag_profiles = {}

    for b in bases:
        t_lag0 = time.perf_counter()
        seq_n, g_list = series_by_base[b]
        g = np.asarray(g_list, dtype=np.float64)
        if g.size == 0:
            continue

        pref_s = np.cumsum(g)
        pref_e2 = np.cumsum(g * g)
        pref_a1 = np.cumsum(np.abs(g))
        seq_n_np = np.asarray(seq_n, dtype=np.int64)

        d_max = max(0, g.size - 1)
        use_lags = [d for d in lag_list if d <= d_max]
        if not use_lags:
            continue
        lag_signed_total: List[float] = []
        lag_pos_total: List[float] = []
        lag_neg_total: List[float] = []
        lag_abs_total: List[float] = []

        # Build per-base checkpoint rows once; lag accumulation is then vectorized across checkpoints.
        base_rows: List[Dict[str, object]] = []
        base_idx: List[int] = []
        base_split: List[str] = []
        for n in all_n:
            split = "train" if n in train_n else "valid"
            xs = x_map[n]
            if not xs:
                continue
            idxs = np.searchsorted(seq_n_np, np.asarray(xs, dtype=np.int64), side="right") - 1
            for x, i in zip(xs, idxs.tolist()):
                if i >= 0:
                    base_rows.append({"base": b, "n_max": n, "x": x, "split": split})
                    base_idx.append(i)
                    base_split.append(split)

        if not base_rows:
            continue

        idx_arr = np.asarray(base_idx, dtype=np.int64)
        t_arr = idx_arr + 1
        s_arr = pref_s[idx_arr]
        e2_arr = pref_e2[idx_arr]
        l1_arr = pref_a1[idx_arr]
        valid_e2 = e2_arr > 1e-30
        eta_pos_arr = np.zeros_like(e2_arr)
        eta_pos_arr[valid_e2] = np.maximum(0.0, (s_arr[valid_e2] * s_arr[valid_e2] - e2_arr[valid_e2]) / e2_arr[valid_e2])
        abs_total_pairs_arr = np.maximum(0.0, (l1_arr * l1_arr - e2_arr) / 2.0)
        low_signed_pairs_arr = np.zeros_like(e2_arr)
        low_abs_pairs_arr = np.zeros_like(e2_arr)

        for d in use_lags:
            prod = g[:-d] * g[d:]
            pos = np.maximum(prod, 0.0)
            neg = np.maximum(-prod, 0.0)
            aa = np.abs(prod)
            lag_signed_total.append(float(np.sum(prod)))
            lag_pos_total.append(float(np.sum(pos)))
            lag_neg_total.append(float(np.sum(neg)))
            lag_abs_total.append(float(np.sum(aa)))

            c_signed = np.cumsum(prod)
            c_abs = np.cumsum(aa)
            j = t_arr - d - 1
            mask = j >= 0
            if np.any(mask):
                jm = j[mask]
                low_signed_pairs_arr[mask] += c_signed[jm]
                low_abs_pairs_arr[mask] += c_abs[jm]

        low_lag_profiles[str(b)] = {
            "lag": use_lags,
            "signed_mass": lag_signed_total,
            "pos_mass": lag_pos_total,
            "neg_mass": lag_neg_total,
            "abs_mass": lag_abs_total,
            "signed_over_abs": [
                float(s / a) if a > 1e-30 else 0.0 for s, a in zip(lag_signed_total, lag_abs_total)
            ],
            "pos_over_abs": [
                float(p / a) if a > 1e-30 else 0.0 for p, a in zip(lag_pos_total, lag_abs_total)
            ],
        }
        lagprep_seconds += time.perf_counter() - t_lag0

        t_eval0 = time.perf_counter()
        high_abs_pairs_arr = np.maximum(0.0, abs_total_pairs_arr - low_abs_pairs_arr)
        req_arr = np.zeros_like(e2_arr)
        with np.errstate(divide="ignore", invalid="ignore"):
            req_arr = np.maximum(0.0, ((eta_pos_arr * e2_arr / 2.0) - low_signed_pairs_arr) / np.maximum(1e-30, high_abs_pairs_arr))
        no_high_mask = high_abs_pairs_arr <= 1e-30
        bad_no_high = no_high_mask & (((eta_pos_arr * e2_arr / 2.0) - low_signed_pairs_arr) > 1e-10)
        req_arr[bad_no_high] = np.inf
        for split, req in zip(base_split, req_arr.tolist()):
            if split == "train":
                tail_calib_requirements.append(float(req))

        for i, r in enumerate(base_rows):
            e2 = float(e2_arr[i])
            if e2 <= 1e-30:
                continue
            rows.append(
                {
                    "base": b,
                    "n_max": int(r["n_max"]),
                    "x": int(r["x"]),
                    "split": r["split"],
                    "eta_pos": float(eta_pos_arr[i]),
                    "e2": e2,
                    "low_signed_pairs": float(low_signed_pairs_arr[i]),
                    "low_abs_pairs": float(low_abs_pairs_arr[i]),
                    "high_abs_pairs": float(high_abs_pairs_arr[i]),
                }
            )
        eval_seconds += time.perf_counter() - t_eval0

    finite_reqs = [r for r in tail_calib_requirements if math.isfinite(r)]
    k_train_need = max(finite_reqs) if finite_reqs else 1.0
    if args.tail_mode == "deterministic":
        # Deterministic theorem-side choice: full absolute tail.
        k_tail = min(1.0, max(0.0, float(args.k_tail_fixed)))
    else:
        k_tail = min(1.0, max(0.0, k_train_need * max(1.0, args.tail_calib_safety)))

    violation_count = 0
    max_gap = -1e18
    for r in rows:
        eta_ss = max(0.0, (2.0 * (r["low_signed_pairs"] + k_tail * r["high_abs_pairs"])) / max(1e-30, r["e2"]))
        r["eta_ss"] = eta_ss
        gap = float(r["eta_pos"]) - float(eta_ss)
        max_gap = max(max_gap, gap)
        if gap > 1e-12:
            violation_count += 1

    env = fit_log_envelope(rows, train_n, valid_n, "eta_ss", a_eta=args.a_eta, safety_factor=args.envelope_safety)

    train_rows = [r for r in rows if r["split"] == "train"]
    valid_rows = [r for r in rows if r["split"] == "valid"]

    c_pos_train = max((float(r["eta_pos"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** args.a_eta) for r in train_rows), default=0.0)
    c_pos_valid = max((float(r["eta_pos"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** args.a_eta) for r in valid_rows), default=0.0)
    c_ss_train = max((float(r["eta_ss"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** args.a_eta) for r in train_rows), default=0.0)
    c_ss_valid = max((float(r["eta_ss"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** args.a_eta) for r in valid_rows), default=0.0)

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "a_eta": args.a_eta,
            "lag_cut": lag_cut,
            "lag_mode": args.lag_mode,
            "lag_dense_cut": int(args.lag_dense_cut),
            "lag_tail_mult": float(args.lag_tail_mult),
            "lag_count": len(lag_list),
            "tail_calib_safety": args.tail_calib_safety,
            "tail_mode": args.tail_mode,
            "k_tail_fixed": float(args.k_tail_fixed),
            "envelope_safety": args.envelope_safety,
            "c_eta_budget": args.c_eta_budget,
            "cache_hits": cache_hits,
            "cache_writes": cache_writes,
        },
        "timing": {
            "build_series_seconds": float(build_seconds),
            "lagprep_seconds": float(lagprep_seconds),
            "eval_seconds": float(eval_seconds),
            "total_seconds": float(time.perf_counter() - t_start),
        },
        "tail_calibration": {
            "mode": args.tail_mode,
            "k_train_need": float(k_train_need),
            "k_tail_used": float(k_tail),
            "all_requirements_finite": len(finite_reqs) == len(tail_calib_requirements),
            "n_train_points": len(tail_calib_requirements),
        },
        "chain_checks": {
            "eta_pos_le_eta_ss_holds": bool(violation_count == 0 and max_gap <= 1e-12),
            "violations": int(violation_count),
            "max_gap_eta_pos_minus_eta_ss": float(max_gap),
        },
        "normalized_constants": {
            "C_pos_train": float(c_pos_train),
            "C_pos_valid": float(c_pos_valid),
            "C_ss_train": float(c_ss_train),
            "C_ss_valid": float(c_ss_valid),
            "C_ss_uplifted": float(env["C_uplifted"]),
            "valid_ratio_ss_over_uplifted": float(env["ratio_max_valid"]),
            "ss_over_budget_ratio": float(env["C_uplifted"] / max(1e-30, float(args.c_eta_budget))),
        },
        "eta_ss_log_envelope": env,
        "low_lag_profiles": low_lag_profiles,
    }

    out = args.output
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)

    md = out.replace(".json", ".md")
    nc = payload["normalized_constants"]
    ch = payload["chain_checks"]
    tc = payload["tail_calibration"]
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Sign-Sensitive Lag-Band Bound\n\n")
        f.write(f"Generated: {payload['timestamp_utc']}\n\n")
        f.write("## Calibration\n\n")
        f.write(f"- `lag_cut`: {lag_cut}\n")
        f.write(f"- `lag_mode`: {args.lag_mode}\n")
        f.write(f"- `lag_count`: {len(lag_list)}\n")
        f.write(f"- `tail_mode`: {args.tail_mode}\n")
        f.write(f"- `k_train_need`: {tc['k_train_need']:.12g}\n")
        f.write(f"- `k_tail_used`: {tc['k_tail_used']:.12g}\n")
        f.write(f"- train points: {tc['n_train_points']}\n\n")
        f.write("## Chain Check\n\n")
        f.write(f"- holds (`eta_pos <= eta_ss`): {ch['eta_pos_le_eta_ss_holds']}\n")
        f.write(f"- violations: {ch['violations']}\n")
        f.write(f"- max gap (`eta_pos - eta_ss`): {ch['max_gap_eta_pos_minus_eta_ss']:.12g}\n\n")
        f.write("## Normalized Constants (`A_eta` fixed)\n\n")
        f.write(f"- `C_pos_train`: {nc['C_pos_train']:.12g}\n")
        f.write(f"- `C_pos_valid`: {nc['C_pos_valid']:.12g}\n")
        f.write(f"- `C_ss_train`: {nc['C_ss_train']:.12g}\n")
        f.write(f"- `C_ss_valid`: {nc['C_ss_valid']:.12g}\n")
        f.write(f"- `C_ss_uplifted`: {nc['C_ss_uplifted']:.12g}\n")
        f.write(f"- valid ratio over uplifted: {nc['valid_ratio_ss_over_uplifted']:.12g}\n")
        f.write(f"- uplifted/budget ratio: {nc['ss_over_budget_ratio']:.12g}\n")

    print(json.dumps({"json": out, "md": md}, indent=2))


if __name__ == "__main__":
    main()
