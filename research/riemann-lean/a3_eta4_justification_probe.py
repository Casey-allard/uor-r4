#!/usr/bin/env python3
"""Empirical-structural justification probe for eta_+(x) <= C_eta (log x)^4."""

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


def bin_index(x: float, edges: Sequence[float]) -> int:
    for i in range(len(edges) - 1):
        if edges[i] <= x < edges[i + 1]:
            return i
    return len(edges) - 2


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 eta^4 justification probe")
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
    ap.add_argument("--tail-frac", type=float, default=0.25)
    ap.add_argument("--safety", type=float, default=1.1)
    ap.add_argument("--num-bins", type=int, default=12)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_eta4_justification_probe")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_eta4_justification_probe_2026-02-17.json")
    args = ap.parse_args()

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
                eta_pos = max(0.0, (s * s - e2) / max(1e-30, e2))
                lx = math.log(max(3.0, float(x)))
                z = eta_pos / max(1e-30, lx ** args.a_eta)
                rows.append({"base": b, "n_max": n, "x": x, "split": "train" if n in train_n else "valid", "z": z})

    train_z = [float(r["z"]) for r in rows if r["split"] == "train"]
    valid_z = [float(r["z"]) for r in rows if r["split"] == "valid"]
    c_train = max(train_z) if train_z else 0.0
    c_valid = max(valid_z) if valid_z else 0.0
    c_eta = c_train * max(1.0, args.safety)
    valid_ratio = c_valid / max(1e-30, c_eta)

    per_base = {}
    for b in bases:
        bz = [float(r["z"]) for r in rows if int(r["base"]) == b]
        per_base[str(b)] = {"z_max": max(bz) if bz else 0.0, "z_mean": (sum(bz) / len(bz)) if bz else 0.0}

    xmin = min(float(r["x"]) for r in rows) if rows else 1.0
    xmax = max(float(r["x"]) for r in rows) if rows else 2.0
    bins = max(2, int(args.num_bins))
    edges = [xmin + (xmax - xmin) * i / bins for i in range(bins + 1)]
    by_bin = [[] for _ in range(bins)]
    for r in rows:
        idx = bin_index(float(r["x"]), edges)
        by_bin[idx].append(float(r["z"]))

    bin_means = [sum(v) / len(v) if v else 0.0 for v in by_bin]
    noninc_steps = 0
    dec_steps = 0
    for i in range(1, len(bin_means)):
        if bin_means[i] <= bin_means[i - 1] + 1e-12:
            noninc_steps += 1
        if bin_means[i] < bin_means[i - 1] - 1e-12:
            dec_steps += 1

    # Tail behavior: compare upper quantiles of last tail_frac vs full.
    rows_sorted = sorted(rows, key=lambda r: float(r["x"]))
    cut = int((1.0 - max(0.0, min(0.9, args.tail_frac))) * len(rows_sorted))
    tail = rows_sorted[cut:] if cut < len(rows_sorted) else rows_sorted
    tail_z = [float(r["z"]) for r in tail]
    tail_max = max(tail_z) if tail_z else 0.0
    tail_mean = (sum(tail_z) / len(tail_z)) if tail_z else 0.0

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "a_eta": args.a_eta,
            "safety": args.safety,
            "tail_frac": args.tail_frac,
            "cache_hits": cache_hits,
        },
        "fit_summary": {
            "C_train_max": c_train,
            "C_valid_max": c_valid,
            "C_eta_with_safety": c_eta,
            "valid_ratio_over_C_eta": valid_ratio,
            "valid_holds": valid_ratio <= 1.0 + 1e-12,
        },
        "trend_summary": {
            "bin_means_z": bin_means,
            "nonincreasing_step_fraction": noninc_steps / max(1, len(bin_means) - 1),
            "strict_decrease_step_fraction": dec_steps / max(1, len(bin_means) - 1),
            "tail_max_z": tail_max,
            "tail_mean_z": tail_mean,
        },
        "per_base_summary": per_base,
    }

    out = args.output
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    md = out.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        s = payload["fit_summary"]
        t = payload["trend_summary"]
        f.write("# A3 Eta^4 Justification Probe\n\n")
        f.write(f"Generated: {payload['timestamp_utc']}\n\n")
        f.write(f"- C_train_max: {s['C_train_max']:.12g}\n")
        f.write(f"- C_valid_max: {s['C_valid_max']:.12g}\n")
        f.write(f"- C_eta_with_safety: {s['C_eta_with_safety']:.12g}\n")
        f.write(f"- valid_ratio_over_C_eta: {s['valid_ratio_over_C_eta']:.12g}\n")
        f.write(f"- valid_holds: {s['valid_holds']}\n")
        f.write(f"- nonincreasing_step_fraction: {t['nonincreasing_step_fraction']:.6f}\n")
        f.write(f"- strict_decrease_step_fraction: {t['strict_decrease_step_fraction']:.6f}\n")
        f.write(f"- tail_mean_z: {t['tail_mean_z']:.12g}\n")
        f.write(f"- tail_max_z: {t['tail_max_z']:.12g}\n")
    print(json.dumps({"json": out, "md": md, "valid_holds": payload["fit_summary"]["valid_holds"]}, indent=2))


if __name__ == "__main__":
    main()
