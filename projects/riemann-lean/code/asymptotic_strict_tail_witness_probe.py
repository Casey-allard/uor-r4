#!/usr/bin/env python3
"""Probe finite-range witnesses for asymptotic strict-tail remainder control.

Numerical goal:
- Fix a beta/tau single-mode fit for E(x)/x^beta.
- Compute ratio(x) = |remainder(x)| / amplitude(main_mode).
- For each epsilon, find x_epsilon such that sup_{u>=x} ratio(u) <= epsilon
  on the sampled grid.

This is numerical evidence only, not a theorem.
"""

from __future__ import annotations

import argparse
import json
import os
from datetime import datetime, timezone
from typing import Dict, List

import numpy as np

from k1_source_shape_probe import compute_scaled_signal, key_hash, zeros_signature
from prime_geometry_loop import load_zeta_zeros_file


def parse_epsilons(text: str) -> List[float]:
    vals = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        vals.append(float(tok))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("epsilon grid is empty")
    return vals


def fit_fixed_tau(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau: float,
    include_decay_term: bool,
) -> Dict[str, np.ndarray]:
    c = np.cos(tau * t)
    s = np.sin(tau * t)
    cols = [c, s]
    if include_decay_term:
        cols.append(x ** (-beta))
    xmat = np.column_stack(cols)
    coef, _, _, _ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    rem = y - yhat
    amp = float(np.hypot(float(coef[0]), float(coef[1])))
    ratio = np.abs(rem) / max(amp, 1.0e-12)
    tail_sup = np.maximum.accumulate(ratio[::-1])[::-1]
    return {
        "coef": coef,
        "remainder": rem,
        "ratio": ratio,
        "tail_sup": tail_sup,
        "amplitude": np.array([amp], dtype=np.float64),
    }


def finite_range_eps_witnesses(
    x: np.ndarray,
    tail_sup: np.ndarray,
    epsilons: List[float],
) -> List[Dict[str, object]]:
    out: List[Dict[str, object]] = []
    for eps in epsilons:
        mask = tail_sup <= eps
        if np.any(mask):
            idx = int(np.argmax(mask))
            out.append(
                {
                    "epsilon": float(eps),
                    "witness_found": True,
                    "x_epsilon": float(x[idx]),
                    "index": idx,
                    "tail_sup_at_x_epsilon": float(tail_sup[idx]),
                }
            )
        else:
            out.append(
                {
                    "epsilon": float(eps),
                    "witness_found": False,
                    "x_epsilon": None,
                    "index": None,
                    "tail_sup_at_x_epsilon": float(np.min(tail_sup)),
                }
            )
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Finite-range asymptotic strict-tail witness probe")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=100000)
    ap.add_argument("--beta", type=float, default=0.74)
    ap.add_argument("--tau", type=float, default=14.134725142)
    ap.add_argument("--x-min", type=float, default=1.0e7)
    ap.add_argument("--x-max", type=float, default=1.0e20)
    ap.add_argument("--grid-size", type=int, default=6144)
    ap.add_argument("--zero-chunk", type=int, default=512)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--epsilons", type=str, default="0.95,0.98,0.99,0.995,1.0,1.005,1.01,1.02,1.05,1.10")
    ap.add_argument("--cache-dir", type=str, default="research/cache/k1_source_shape_probe")
    ap.add_argument("--output", type=str, default="research/output/asymptotic_strict_tail_witness_probe.json")
    args = ap.parse_args()

    zeros_raw = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    zeros = [z for z in zeros_raw if z > 0.0]
    zeros.sort()
    if args.max_zeta_zeros > 0:
        zeros = zeros[: args.max_zeta_zeros]
    if not zeros:
        raise ValueError("no positive zeta zeros loaded")
    epsilons = parse_epsilons(args.epsilons)

    zsig = zeros_signature(zeros)
    cfg = {
        "zeros_sig": zsig,
        "zeros_used": len(zeros),
        "beta": float(args.beta),
        "x_min": float(args.x_min),
        "x_max": float(args.x_max),
        "grid_size": int(args.grid_size),
        "zero_chunk": int(args.zero_chunk),
    }
    os.makedirs(args.cache_dir, exist_ok=True)
    ckey = key_hash(cfg)
    cpath = os.path.join(args.cache_dir, ckey + ".npz")

    if os.path.exists(cpath):
        loaded = np.load(cpath)
        x = loaded["x"]
        t = loaded["t"]
        y = loaded["y"]
        cache_status = "warm"
    else:
        computed = compute_scaled_signal(
            zeros=zeros,
            beta=args.beta,
            x_min=args.x_min,
            x_max=args.x_max,
            grid_size=args.grid_size,
            zero_chunk=args.zero_chunk,
        )
        x = computed["x"]
        t = computed["t"]
        y = computed["y"]
        np.savez_compressed(cpath, x=x, t=t, y=y)
        cache_status = "cold"

    fit = fit_fixed_tau(
        x=x,
        t=t,
        y=y,
        beta=float(args.beta),
        tau=float(args.tau),
        include_decay_term=bool(args.include_decay_term),
    )
    coef = fit["coef"]
    tail_sup = fit["tail_sup"]
    amp = float(fit["amplitude"][0])
    witnesses = finite_range_eps_witnesses(x=x, tail_sup=tail_sup, epsilons=epsilons)
    min_tail_sup = float(np.min(tail_sup))

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "zeros_used": len(zeros),
            "beta": float(args.beta),
            "tau": float(args.tau),
            "x_min": float(args.x_min),
            "x_max": float(args.x_max),
            "grid_size": int(args.grid_size),
            "zero_chunk": int(args.zero_chunk),
            "include_decay_term": bool(args.include_decay_term),
            "epsilons": epsilons,
            "cache_dir": args.cache_dir,
            "cache_path": cpath,
            "cache_status": cache_status,
        },
        "fit": {
            "a": float(coef[0]),
            "b": float(coef[1]),
            "amplitude": amp,
            "decay_term_coeff": float(coef[2]) if bool(args.include_decay_term) else None,
            "tail_ratio_sup_to_amp_min": min_tail_sup,
            "tail_ratio_sup_to_amp_at_x_min": float(tail_sup[0]),
        },
        "epsilon_witnesses": witnesses,
        "interpretation": {
            "note": (
                "Finite-range witness table only. "
                "A full asymptotic strict-tail theorem requires a non-circular proof "
                "that every epsilon>0 has an eventual bound."
            )
        },
    }

    out_path = args.output
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)

    out_md = out_path.replace(".json", ".md")
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# Asymptotic Strict-Tail Witness Probe\n\n")
        f.write(f"Generated: {payload['timestamp_utc']}\n\n")
        f.write(f"- beta: {args.beta}\n")
        f.write(f"- tau: {args.tau}\n")
        f.write(f"- zeros used: {len(zeros)}\n")
        f.write(f"- x-range: [{args.x_min:g}, {args.x_max:g}]\n")
        f.write(f"- grid size: {args.grid_size}\n")
        f.write(f"- cache status: {cache_status}\n")
        f.write(f"- min tail sup ratio: {min_tail_sup:.9f}\n\n")
        f.write("| epsilon | witness_found | x_epsilon | tail_sup_at_x_epsilon |\n")
        f.write("|---:|---|---:|---:|\n")
        for row in witnesses:
            xe = row["x_epsilon"]
            xe_text = f"{xe:.6g}" if isinstance(xe, float) else "NA"
            f.write(
                f"| {row['epsilon']:.6f} | {str(row['witness_found']).lower()} | "
                f"{xe_text} | {row['tail_sup_at_x_epsilon']:.9f} |\n"
            )
        f.write("\n")
        f.write(payload["interpretation"]["note"] + "\n")

    print(f"wrote: {out_path}")
    print(f"wrote: {out_md}")
    print("tail_ratio_sup_to_amp_min:", f"{min_tail_sup:.9f}")


if __name__ == "__main__":
    main()
