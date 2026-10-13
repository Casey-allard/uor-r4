#!/usr/bin/env python3
"""A2 theorem-constant pack: fixed beta, explicit tau(M), sup-ratio C_delta."""

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
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def parse_ints(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


def tau_tail_fp(zeros_ref: Sequence[float], m: int, kernel: str, scale: float) -> float:
    s = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        s += w * (g / den)
    return s


def prefix_sums(vals: Sequence[float]) -> List[float]:
    out = []
    acc = 0.0
    for v in vals:
        acc += v
        out.append(acc)
    return out


def h_from_prefix(seq_n: Sequence[int], pref: Sequence[float], xs: Sequence[int]) -> List[float]:
    out = []
    for x in xs:
        i = bisect.bisect_right(seq_n, x) - 1
        acc = pref[i] if i >= 0 else 0.0
        out.append(acc / math.sqrt(x))
    return out


def build_x_map(n_values: Sequence[int], x_step: int) -> Dict[int, List[int]]:
    n_values = sorted(set(n_values))
    if not n_values:
        return {}
    x0 = max(2, x_step)
    xs_max = list(range(x0, max(n_values) + 1, x_step))
    out = {}
    for n in n_values:
        k = ((n - x0) // x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        out[n] = xs_max[:k]
    return out


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


def save_cache(path: str, h_max: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, h_max=np.asarray(h_max, dtype=np.float64))


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 theorem constant pack (fixed-beta, sup ratio)")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000")
    ap.add_argument("--valid-n-values", type=str, default="2000000,5000000")
    ap.add_argument("--x-step", type=int, default=10000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--m-grid", type=str, default="64,128,192,256")
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--beta", type=float, default=2.6)
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--safety-factor", type=float, default=1.2)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a2_theorem_constant_pack")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/a2_theorem_constant_pack_gauss100_ref512.json",
    )
    args = ap.parse_args()

    bases = sorted(set(parse_ints(args.bases)))
    train_n = sorted(set(parse_ints(args.train_n_values)))
    valid_n = sorted(set(parse_ints(args.valid_n_values)))
    all_n = sorted(set(train_n + valid_n))
    if not all_n:
        raise ValueError("need non-empty n-values")

    m_grid = sorted(set(parse_ints(args.m_grid)))
    m_grid = [m for m in m_grid if 0 < m < args.m_ref]
    if not m_grid:
        raise ValueError("m-grid must contain values in (0,m-ref)")

    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zeros")
    zeros_ref = zeros_all[: args.m_ref]

    x_map = build_x_map(all_n, args.x_step)
    if sum(len(x_map[n]) for n in train_n) < 10:
        raise ValueError("insufficient train grid points; increase train n-values or lower x-step")
    xs_max = x_map[max(all_n)]
    x_counts = {n: len(x_map[n]) for n in all_n}

    tau_by_m = {
        m: tau_tail_fp(zeros_ref, m, args.zero_kernel, args.kernel_scale)
        for m in m_grid
    }

    max_n = max(all_n)
    zero_terms: Dict[int, Tuple[object, object, object]] = {}
    for m in m_grid + [args.m_ref]:
        zero_terms[m] = hx.prepare_zero_terms(
            zeros=zeros_all[:m],
            zero_kernel=args.zero_kernel,
            kernel_scale=args.kernel_scale,
        )

    # Build H series once per (base, M) at max_n, then slice to all n-values.
    zsig = zeros_sig(zeros_ref)
    use_cache = not args.no_cache
    event_stride = max(1, args.event_stride)

    def build_h_max_for_pair(b: int, m: int) -> Tuple[int, int, List[float], bool]:
        key = cache_key(
            {
                "version": 2,
                "base": b,
                "m": m,
                "max_n": max_n,
                "x_step": args.x_step,
                "u_mode": args.u_mode,
                "zero_kernel": args.zero_kernel,
                "kernel_scale": args.kernel_scale,
                "chunk_n": args.chunk_n,
                "event_stride": event_stride,
                "event_scale": bool(args.event_scale),
                "weights": WEIGHTS,
                "m_ref": args.m_ref,
                "zsig": zsig,
            }
        )
        cache_path = os.path.join(args.cache_dir, f"{key}.npz")
        if use_cache:
            cached = load_cache(cache_path)
            if cached is not None and len(cached) == len(xs_max):
                return b, m, cached, True

        seq_n, g_vals = hx.stream_weighted_events(
            n_max=max_n,
            base=b,
            zeros=zeros_all[:m],
            u_mode=args.u_mode,
            zero_kernel=args.zero_kernel,
            kernel_scale=args.kernel_scale,
            weights=WEIGHTS,
            chunk_n=args.chunk_n,
            zero_terms=zero_terms[m],
            event_stride=event_stride,
            event_scale=args.event_scale,
        )
        pref = prefix_sums(g_vals)
        h_max = h_from_prefix(seq_n, pref, xs_max)
        if use_cache:
            save_cache(cache_path, h_max)
        return b, m, h_max, False

    pairs = [(b, m) for b in bases for m in (m_grid + [args.m_ref])]
    h_full: Dict[int, Dict[int, List[float]]] = {b: {} for b in bases}
    cache_hits = 0
    if args.jobs > 1 and len(pairs) > 1:
        workers = min(args.jobs, len(pairs))
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
            futs = [ex.submit(build_h_max_for_pair, b, m) for b, m in pairs]
            for fut in concurrent.futures.as_completed(futs):
                b, m, h_max, hit = fut.result()
                h_full[b][m] = h_max
                cache_hits += 1 if hit else 0
    else:
        for b, m in pairs:
            b, m, h_max, hit = build_h_max_for_pair(b, m)
            h_full[b][m] = h_max
            cache_hits += 1 if hit else 0

    h_cache: Dict[int, Dict[int, Dict[int, List[float]]]] = {b: {} for b in bases}
    for b in bases:
        for m in (m_grid + [args.m_ref]):
            h_cache[b][m] = {}
            hm = h_full[b][m]
            for n in all_n:
                h_cache[b][m][n] = hm[: x_counts[n]]

    def max_abs_diff(a: Sequence[float], b: Sequence[float]) -> float:
        if len(a) != len(b):
            return 0.0
        if not a:
            return 0.0
        return max(abs(x - y) for x, y in zip(a, b))

    rows = []
    c_train = 0.0
    for n in all_n:
        lx = math.log(max(3, n)) ** args.beta
        for b in bases:
            h_ref = h_cache[b][args.m_ref][n]
            for m in m_grid:
                delta = max_abs_diff(h_cache[b][m][n], h_ref)
                tau = max(1e-30, tau_by_m[m])
                ratio = delta / max(1e-30, lx * tau)
                row = {"n_max": n, "base": b, "M": m, "delta": delta, "tau": tau, "ratio": ratio}
                rows.append(row)
                if n in train_n:
                    c_train = max(c_train, ratio)

    c_delta = max(1e-30, c_train) * max(1.0, args.safety_factor)

    train_viol = 0
    valid_viol = 0
    train_max_gap = -1e18
    valid_max_gap = -1e18
    ratio_train = []
    ratio_valid = []
    for r in rows:
        rhs = c_delta * (math.log(max(3, r["n_max"])) ** args.beta) * r["tau"]
        gap = r["delta"] - rhs
        rr = r["delta"] / max(1e-30, rhs)
        if r["n_max"] in train_n:
            if gap > 1e-12:
                train_viol += 1
            train_max_gap = max(train_max_gap, gap)
            ratio_train.append(rr)
        else:
            if gap > 1e-12:
                valid_viol += 1
            valid_max_gap = max(valid_max_gap, gap)
            ratio_valid.append(rr)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "m_grid": m_grid,
            "m_ref": args.m_ref,
            "beta_fixed": args.beta,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": max(1, args.event_stride),
            "event_scale": bool(args.event_scale),
            "safety_factor": max(1.0, args.safety_factor),
            "cache_dir": args.cache_dir,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_misses": len(pairs) - cache_hits,
        },
        "tau_by_M": tau_by_m,
        "C_delta_train_sup_ratio_raw": c_train,
        "C_delta_theorem": c_delta,
        "train_check": {
            "holds": train_viol == 0 and train_max_gap <= 1e-12,
            "violations": int(train_viol),
            "max_gap_delta_minus_rhs": train_max_gap,
            "ratio_min": min(ratio_train) if ratio_train else 0.0,
            "ratio_max": max(ratio_train) if ratio_train else 0.0,
        },
        "valid_check": {
            "holds": valid_viol == 0 and valid_max_gap <= 1e-12,
            "violations": int(valid_viol),
            "max_gap_delta_minus_rhs": valid_max_gap,
            "ratio_min": min(ratio_valid) if ratio_valid else 0.0,
            "ratio_max": max(ratio_valid) if ratio_valid else 0.0,
        },
        "rows": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A2 Theorem Constant Pack\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write("- fixed form: Delta_M(x;W) <= C_delta * (log x)^beta * tau(M)\n")
        f.write(f"- beta (fixed): {args.beta}\n")
        f.write(f"- C_delta (train sup-ratio * safety): {c_delta:.12e}\n")
        f.write(f"- train holds: {report['train_check']['holds']}  violations: {report['train_check']['violations']}\n")
        f.write(f"- valid holds: {report['valid_check']['holds']}  violations: {report['valid_check']['violations']}\n")
        f.write(f"- valid max_gap(delta-rhs): {report['valid_check']['max_gap_delta_minus_rhs']:.6e}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
