#!/usr/bin/env python3
"""Symbolic offdiag inequality chain probe for eta_+(x)."""

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


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 offdiag symbolic chain probe")
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
    ap.add_argument("--safety", type=float, default=3.0)
    ap.add_argument("--c-eta-budget", type=float, default=1.2170134478356474)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_offdiag_symbolic_chain")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_offdiag_symbolic_chain_2026-02-17.json")
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
        pref_a1 = []
        a1 = 0.0
        a2 = 0.0
        aa = 0.0
        for g in g_vals:
            a1 += g
            a2 += g * g
            aa += abs(g)
            pref_s.append(a1)
            pref_e2.append(a2)
            pref_a1.append(aa)
        for n in all_n:
            for x in x_map[n]:
                i = bisect.bisect_right(seq_n, x) - 1
                if i < 0:
                    continue
                s = pref_s[i]
                e2 = pref_e2[i]
                l1 = pref_a1[i]
                eta_pos = max(0.0, (s * s - e2) / max(1e-30, e2))
                # Bilinear absolute majorant:
                # 2*sum_{i<j}|g_i g_j| = (sum|g_i|)^2 - sum g_i^2
                eta_symbolic = max(0.0, (l1 * l1 - e2) / max(1e-30, e2))
                lx = math.log(max(3.0, float(x)))
                z_pos = eta_pos / max(1e-30, lx ** args.a_eta)
                z_sym = eta_symbolic / max(1e-30, lx ** args.a_eta)
                rows.append(
                    {
                        "base": b,
                        "n_max": n,
                        "x": x,
                        "split": "train" if n in train_n else "valid",
                        "eta_pos": eta_pos,
                        "eta_symbolic": eta_symbolic,
                        "z_pos": z_pos,
                        "z_sym": z_sym,
                    }
                )

    train_rows = [r for r in rows if r["split"] == "train"]
    valid_rows = [r for r in rows if r["split"] == "valid"]
    c_train_pos = max((float(r["z_pos"]) for r in train_rows), default=0.0)
    c_valid_pos = max((float(r["z_pos"]) for r in valid_rows), default=0.0)
    c_train_sym = max((float(r["z_sym"]) for r in train_rows), default=0.0)
    c_valid_sym = max((float(r["z_sym"]) for r in valid_rows), default=0.0)
    c_sym_safe = c_train_sym * max(1.0, args.safety)
    valid_ratio_sym = c_valid_sym / max(1e-30, c_sym_safe)
    budget_ratio = c_sym_safe / max(1e-30, float(args.c_eta_budget))

    # Deterministic chain checks.
    viol_chain = 0
    gap_chain = -1e18
    for r in rows:
        g = float(r["eta_pos"]) - float(r["eta_symbolic"])
        gap_chain = max(gap_chain, g)
        if g > 1e-12:
            viol_chain += 1

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "a_eta": args.a_eta,
            "safety": args.safety,
            "c_eta_budget": args.c_eta_budget,
            "cache_hits": cache_hits,
        },
        "chain_checks": {
            "eta_pos_le_eta_symbolic_holds": viol_chain == 0 and gap_chain <= 1e-12,
            "violations": int(viol_chain),
            "max_gap_eta_pos_minus_eta_symbolic": float(gap_chain),
        },
        "normalized_constants": {
            "C_pos_train": c_train_pos,
            "C_pos_valid": c_valid_pos,
            "C_sym_train": c_train_sym,
            "C_sym_valid": c_valid_sym,
            "C_sym_safe": c_sym_safe,
            "valid_ratio_sym_over_safe": valid_ratio_sym,
            "sym_over_budget_ratio": budget_ratio,
        },
    }

    out = args.output
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    md = out.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        ch = payload["chain_checks"]
        nc = payload["normalized_constants"]
        f.write("# A3 Offdiag Symbolic Chain\n\n")
        f.write(f"Generated: {payload['timestamp_utc']}\n\n")
        f.write("- Deterministic chain:\n")
        f.write("  - `eta_pos <= eta_symbolic := ((sum|g|)^2 - sum g^2)/sum g^2`\n")
        f.write(f"- chain holds: {ch['eta_pos_le_eta_symbolic_holds']}\n")
        f.write(f"- max gap eta_pos-eta_symbolic: {ch['max_gap_eta_pos_minus_eta_symbolic']:.12g}\n")
        f.write(f"- C_pos_train: {nc['C_pos_train']:.12g}\n")
        f.write(f"- C_pos_valid: {nc['C_pos_valid']:.12g}\n")
        f.write(f"- C_sym_train: {nc['C_sym_train']:.12g}\n")
        f.write(f"- C_sym_valid: {nc['C_sym_valid']:.12g}\n")
        f.write(f"- C_sym_safe: {nc['C_sym_safe']:.12g}\n")
        f.write(f"- valid_ratio_sym_over_safe: {nc['valid_ratio_sym_over_safe']:.12g}\n")
        f.write(f"- sym_over_budget_ratio (vs selected C_eta budget): {nc['sym_over_budget_ratio']:.12g}\n")
    print(json.dumps({"json": out, "md": md}, indent=2))


if __name__ == "__main__":
    main()
