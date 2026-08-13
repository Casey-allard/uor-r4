#!/usr/bin/env python3
"""Probe fixed eta exponents for held-out validity margins."""

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
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def parse_ints(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


def parse_floats(raw: str) -> List[float]:
    return [float(x.strip()) for x in raw.split(",") if x.strip()]


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


def save_cache(path: str, seq_n: Sequence[int], g_vals: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, seq_n=np.asarray(seq_n, dtype=np.int64), g_vals=np.asarray(g_vals, dtype=np.float64))


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 eta exponent probe")
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
    ap.add_argument("--a-grid", type=str, default="4.0,4.5,5.0,5.5,6.0,6.5")
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_eta_exponent_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_eta_exponent_probe_2026-02-17.json")
    args = ap.parse_args()

    bases = sorted(set(parse_ints(args.bases)))
    train_n = sorted(set(parse_ints(args.train_n_values)))
    valid_n = sorted(set(parse_ints(args.valid_n_values)))
    all_n = sorted(set(train_n + valid_n))
    a_grid = sorted(set(parse_floats(args.a_grid)))
    if not all_n:
        raise ValueError("need non-empty n grid")
    if not a_grid:
        raise ValueError("need non-empty a-grid")

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

    def build_base_series(b: int):
        nonlocal cache_hits
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
        cached = load_cache(cp) if use_cache else None
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
        return b, seq_n, g_vals

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

    rows = []
    for b in bases:
        seq_n, g_vals = series_by_base[b]
        pref_s = []
        pref_e2 = []
        a1 = 0.0
        a2 = 0.0
        for g in g_vals:
            a1 += g
            a2 += g * g
            pref_s.append(a1)
            pref_e2.append(a2)
        for n in all_n:
            for x in x_map[n]:
                i = bisect.bisect_right(seq_n, x) - 1
                if i < 0:
                    continue
                s = pref_s[i]
                e2 = pref_e2[i]
                offdiag = s * s - e2
                eta_pos = max(0.0, offdiag / max(1e-30, e2))
                rows.append({"n_max": n, "x": x, "eta_pos": eta_pos})

    train_rows = [r for r in rows if int(r["n_max"]) in set(train_n)]
    valid_rows = [r for r in rows if int(r["n_max"]) in set(valid_n)]

    out_rows = []
    for a in a_grid:
        train_scaled = [float(r["eta_pos"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** a) for r in train_rows]
        valid_scaled = [float(r["eta_pos"]) / max(1e-30, math.log(max(3.0, float(r["x"]))) ** a) for r in valid_rows]
        c_train = max(train_scaled) if train_scaled else 0.0
        c_valid = max(valid_scaled) if valid_scaled else 0.0
        needed_safety = c_valid / max(1e-30, c_train)
        out_rows.append(
            {
                "A_eta": a,
                "C_train_max": c_train,
                "C_valid_max": c_valid,
                "safety_needed_for_valid": needed_safety,
                "holds_with_safety_2p0": needed_safety <= 2.0 + 1e-12,
                "holds_with_safety_3p0": needed_safety <= 3.0 + 1e-12,
            }
        )

    best_safety3 = [r for r in out_rows if r["holds_with_safety_3p0"]]
    winner = min(best_safety3, key=lambda r: (r["A_eta"], r["safety_needed_for_valid"])) if best_safety3 else None

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "m_zero": args.m_zero,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "event_stride": event_stride,
            "event_scale": bool(args.event_scale),
            "cache_hits": cache_hits,
        },
        "grid_results": out_rows,
        "recommended_under_safety_3p0": winner,
    }

    out = args.output
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    md = out.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Eta Exponent Probe\n\n")
        f.write(f"Generated: {payload['timestamp_utc']}\n\n")
        f.write("| A_eta | safety_needed_for_valid | holds@2.0 | holds@3.0 |\n")
        f.write("|---:|---:|---:|---:|\n")
        for r in out_rows:
            f.write(
                f"| {r['A_eta']:.2f} | {r['safety_needed_for_valid']:.6f} | "
                f"{r['holds_with_safety_2p0']} | {r['holds_with_safety_3p0']} |\n"
            )
        if winner is not None:
            f.write(
                "\nRecommended under safety 3.0: "
                f"A_eta={winner['A_eta']:.2f}, required safety={winner['safety_needed_for_valid']:.6f}\n"
            )
    print(json.dumps({"json": out, "md": md, "winner": winner}, indent=2))


if __name__ == "__main__":
    main()
