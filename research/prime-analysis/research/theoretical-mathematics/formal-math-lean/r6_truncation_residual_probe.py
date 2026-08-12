#!/usr/bin/env python3
"""Probe explicit-formula truncation residuals for finite mode heads.

For a truncated explicit-formula waveform built from the first `M` zeta zeros,
this script measures how much of the signal remains after keeping only the first
`N` frequencies (`N < M`), and fits a simple power-envelope majorant for that
residual on a tail window.

Numerical evidence only; not a proof term.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List

import numpy as np

from k1_source_shape_probe import majorant_profile
from prime_geometry_loop import load_zeta_zeros_file


def parse_counts(text: str) -> List[int]:
    vals: List[int] = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        vals.append(int(token))
    vals = sorted(set(v for v in vals if v > 0))
    if not vals:
        raise ValueError("head-counts is empty")
    return vals


def compute_total_and_heads(
    zeros: np.ndarray,
    beta: float,
    x_min: float,
    x_max: float,
    grid_size: int,
    zero_chunk: int,
    head_counts: List[int],
) -> Dict[str, object]:
    if x_min <= 1.0:
        raise ValueError("x_min must be > 1")
    if x_max <= x_min:
        raise ValueError("x_max must be > x_min")
    if grid_size < 64:
        raise ValueError("grid_size must be >= 64")
    if zero_chunk < 1:
        raise ValueError("zero_chunk must be >= 1")

    t = np.linspace(math.log(x_min), math.log(x_max), grid_size, dtype=np.float64)
    x = np.exp(t)
    scale = np.power(x, 0.5 - beta)

    g = zeros.astype(np.float64)
    den = 0.25 + g * g
    c_coef = 1.0 / den
    s_coef = (2.0 * g) / den

    y_total = np.zeros_like(x)
    y_prefix = np.zeros_like(x)
    head_signal: Dict[int, np.ndarray] = {}
    head_sorted = sorted(head_counts)
    hi = 0

    for start in range(0, g.size, zero_chunk):
        end = min(g.size, start + zero_chunk)
        gj = g[start:end]
        cj = c_coef[start:end]
        sj = s_coef[start:end]

        phase = np.outer(t, gj)
        mixed = np.cos(phase) * cj + np.sin(phase) * sj
        chunk_sum = -scale * np.sum(mixed, axis=1)
        y_total += chunk_sum

        needs_prefix = hi < len(head_sorted) and start < head_sorted[hi] <= end
        col_prefix = np.cumsum(mixed, axis=1) if needs_prefix else None

        while hi < len(head_sorted) and head_sorted[hi] <= end:
            n = head_sorted[hi]
            if n <= start:
                head_signal[n] = y_prefix.copy()
            else:
                take = n - start
                assert col_prefix is not None
                head_signal[n] = y_prefix + (-scale * col_prefix[:, take - 1])
            hi += 1

        y_prefix += chunk_sum

    while hi < len(head_sorted):
        head_signal[head_sorted[hi]] = y_prefix.copy()
        hi += 1

    return {"x": x, "t": t, "y_total": y_total, "head_signal": head_signal}


def run_probe(
    zeta_zeros_file: str,
    max_zeta_zeros: int,
    beta: float,
    x_min: float,
    x_max: float,
    grid_size: int,
    zero_chunk: int,
    head_counts: List[int],
    x1: float,
) -> Dict[str, object]:
    zeros_raw = [float(z) for z in load_zeta_zeros_file(zeta_zeros_file)]
    zeros = [z for z in zeros_raw if z > 0.0]
    zeros.sort()
    if max_zeta_zeros > 0:
        zeros = zeros[:max_zeta_zeros]
    if not zeros:
        raise ValueError("no positive zeta zeros loaded")

    m = len(zeros)
    heads = [n for n in head_counts if n < m]
    if not heads:
        raise ValueError("all head counts are >= max_zeta_zeros; need at least one N < M")

    packed = compute_total_and_heads(
        zeros=np.asarray(zeros, dtype=np.float64),
        beta=beta,
        x_min=x_min,
        x_max=x_max,
        grid_size=grid_size,
        zero_chunk=zero_chunk,
        head_counts=heads,
    )
    x = packed["x"]
    y_total = packed["y_total"]
    head_signal = packed["head_signal"]

    mask_tail = x >= x1
    if not np.any(mask_tail):
        raise ValueError(f"no points satisfy x >= x1={x1}")
    k_tail = int(np.sum(mask_tail))
    tail_frac = max(32.0 / float(x.size), float(k_tail) / float(x.size))

    total_tail_sup = float(np.max(np.abs(y_total[mask_tail])))
    total_all_sup = float(np.max(np.abs(y_total)))

    rows: List[Dict[str, object]] = []
    for n in heads:
        y_head = head_signal[n]
        residual = y_total - y_head
        maj = majorant_profile(x=x, r=residual, tail_frac=tail_frac)
        tail_sup = float(np.max(np.abs(residual[mask_tail])))
        all_sup = float(np.max(np.abs(residual)))
        tail_rmse = float(np.sqrt(np.mean(np.square(residual[mask_tail]))))
        all_rmse = float(np.sqrt(np.mean(np.square(residual))))
        rows.append(
            {
                "head_count": int(n),
                "omitted_count": int(m - n),
                "tail_points": k_tail,
                "tail_sup_abs_residual": tail_sup,
                "tail_rmse_residual": tail_rmse,
                "all_sup_abs_residual": all_sup,
                "all_rmse_residual": all_rmse,
                "tail_sup_ratio_to_total_tail_sup": float(
                    tail_sup / max(total_tail_sup, 1.0e-300)
                ),
                "all_sup_ratio_to_total_all_sup": float(
                    all_sup / max(total_all_sup, 1.0e-300)
                ),
                "majorant_eta": float(maj["eta"]),
                "majorant_C_tail": float(maj["C_tail"]),
                "majorant_C_all": float(maj["C_all"]),
                "majorant_tail_max_ratio_to_bound": float(maj["tail_max_ratio_to_bound"]),
                "majorant_all_max_ratio_to_bound": float(maj["all_max_ratio_to_bound"]),
                "implied_tendsto_zero_shape": bool(maj["implied_tendsto_zero_shape"]),
            }
        )

    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": zeta_zeros_file,
            "max_zeta_zeros": int(m),
            "beta": float(beta),
            "x_min": float(x_min),
            "x_max": float(x_max),
            "grid_size": int(grid_size),
            "zero_chunk": int(zero_chunk),
            "x1": float(x1),
            "tail_points": int(k_tail),
            "head_counts": [int(n) for n in heads],
        },
        "reference_totals": {
            "total_tail_sup_abs": total_tail_sup,
            "total_all_sup_abs": total_all_sup,
        },
        "rows": rows,
        "interpretation": {
            "note": (
                "This measures finite truncation residuals of a selected explicit-formula "
                "surrogate; it is evidence for tail-majorant behavior, not a formal theorem."
            )
        },
    }


def write_markdown(result: Dict[str, object], out_md: str) -> None:
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# R6 Truncation Residual Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        cfg = result["config"]
        f.write("## Config\n")
        for k in [
            "zeta_zeros_file",
            "max_zeta_zeros",
            "beta",
            "x_min",
            "x_max",
            "grid_size",
            "zero_chunk",
            "x1",
            "tail_points",
            "head_counts",
        ]:
            f.write(f"- {k}: {cfg[k]}\n")
        f.write("\n")

        ref = result["reference_totals"]
        f.write("## Total Signal Reference\n")
        f.write(f"- total_tail_sup_abs: {ref['total_tail_sup_abs']}\n")
        f.write(f"- total_all_sup_abs: {ref['total_all_sup_abs']}\n\n")

        f.write("## Residual Rows\n")
        for row in result["rows"]:
            f.write(
                "- N={head_count}, omitted={omitted_count}, "
                "tail_sup_ratio={tail_sup_ratio_to_total_tail_sup:.6f}, "
                "tail_rmse={tail_rmse_residual:.6e}, "
                "eta={majorant_eta:.6f}, C_tail={majorant_C_tail:.6e}, "
                "bound_ratio_tail={majorant_tail_max_ratio_to_bound:.6f}\n".format(**row)
            )
        f.write("\n")
        f.write("## Interpretation\n")
        f.write(f"- {result['interpretation']['note']}\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="R6 truncation residual probe")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko_100k.json",
    )
    ap.add_argument("--max-zeta-zeros", type=int, default=20000)
    ap.add_argument("--beta", type=float, default=0.55)
    ap.add_argument("--x-min", type=float, default=1.0e7)
    ap.add_argument("--x-max", type=float, default=1.0e22)
    ap.add_argument("--grid-size", type=int, default=8192)
    ap.add_argument("--zero-chunk", type=int, default=512)
    ap.add_argument("--head-counts", type=str, default="64,128,256,512,1024")
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/r6_truncation_residual_probe_2026-02-24.json",
    )
    args = ap.parse_args()

    result = run_probe(
        zeta_zeros_file=str(args.zeta_zeros_file),
        max_zeta_zeros=int(args.max_zeta_zeros),
        beta=float(args.beta),
        x_min=float(args.x_min),
        x_max=float(args.x_max),
        grid_size=int(args.grid_size),
        zero_chunk=int(args.zero_chunk),
        head_counts=parse_counts(str(args.head_counts)),
        x1=float(args.x1),
    )

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    out_md = args.output.replace(".json", ".md")
    write_markdown(result, out_md)
    print(f"wrote: {args.output}")
    print(f"wrote: {out_md}")
    for row in result["rows"]:
        print(
            "N={head_count} tail_sup_ratio={tail_sup_ratio_to_total_tail_sup:.6f} "
            "eta={majorant_eta:.6f}".format(**row)
        )


if __name__ == "__main__":
    main()
