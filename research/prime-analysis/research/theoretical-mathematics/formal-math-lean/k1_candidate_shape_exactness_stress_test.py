#!/usr/bin/env python3
"""Stress-test exact finite-head identity for the candidate-shape gate.

This probes whether a fixed finite head (default N=256) can match a larger
explicit-formula surrogate exactly on tail windows. It does not prove anything
formally; it provides robust nonzero-residual diagnostics by recomputing with
two chunk sizes and comparing numerical discrepancy.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np

from prime_geometry_loop import load_zeta_zeros_file
from r6_truncation_residual_probe import compute_total_and_heads


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        vals.append(int(token))
    vals = sorted(set(v for v in vals if v > 0))
    if not vals:
        raise ValueError("expected at least one positive integer")
    return vals


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        vals.append(float(token))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("expected at least one float")
    return vals


def parse_windows(text: str) -> List[Tuple[float, float]]:
    out: List[Tuple[float, float]] = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        if ":" not in token:
            raise ValueError(f"invalid window token: {token}")
        lo_s, hi_s = token.split(":", 1)
        lo = float(lo_s)
        hi = float(hi_s)
        if not (lo > 1.0 and hi > lo):
            raise ValueError(f"invalid window bounds: {token}")
        out.append((lo, hi))
    if not out:
        raise ValueError("expected at least one x-window")
    return out


def compute_residual(
    zeros: np.ndarray,
    beta: float,
    x_min: float,
    x_max: float,
    grid_size: int,
    zero_chunk: int,
    head_count: int,
) -> Dict[str, np.ndarray]:
    packed = compute_total_and_heads(
        zeros=zeros,
        beta=beta,
        x_min=x_min,
        x_max=x_max,
        grid_size=grid_size,
        zero_chunk=zero_chunk,
        head_counts=[head_count],
    )
    y_total = packed["y_total"]
    y_head = packed["head_signal"][head_count]
    residual = y_total - y_head
    return {
        "x": packed["x"],
        "total": y_total,
        "residual": residual,
    }


def summarize_config(
    zeros: np.ndarray,
    beta: float,
    x_min: float,
    x_max: float,
    grid_size: int,
    head_count: int,
    chunk_a: int,
    chunk_b: int,
) -> Dict[str, float | int | str | bool]:
    run_a = compute_residual(
        zeros=zeros,
        beta=beta,
        x_min=x_min,
        x_max=x_max,
        grid_size=grid_size,
        zero_chunk=chunk_a,
        head_count=head_count,
    )
    run_b = compute_residual(
        zeros=zeros,
        beta=beta,
        x_min=x_min,
        x_max=x_max,
        grid_size=grid_size,
        zero_chunk=chunk_b,
        head_count=head_count,
    )

    r_a = run_a["residual"]
    r_b = run_b["residual"]
    y_a = run_a["total"]

    sup_res = float(np.max(np.abs(r_a)))
    sup_total = float(np.max(np.abs(y_a)))
    sup_discrepancy = float(np.max(np.abs(r_a - r_b)))
    robust_nonzero_margin = max(0.0, sup_res - sup_discrepancy)
    robust_margin_ratio_to_total = robust_nonzero_margin / max(sup_total, 1.0e-300)
    sup_ratio_to_total = sup_res / max(sup_total, 1.0e-300)

    # Scale-aware verdict: residual is robustly nonzero if margin dominates
    # floating discrepancy by at least 10x.
    robust_nonzero = bool(robust_nonzero_margin > 10.0 * max(sup_discrepancy, 1.0e-18))

    return {
        "beta": float(beta),
        "x_min": float(x_min),
        "x_max": float(x_max),
        "grid_size": int(grid_size),
        "head_count": int(head_count),
        "chunk_a": int(chunk_a),
        "chunk_b": int(chunk_b),
        "sup_abs_residual": sup_res,
        "sup_abs_total": sup_total,
        "sup_abs_chunk_discrepancy": sup_discrepancy,
        "sup_residual_ratio_to_total": sup_ratio_to_total,
        "robust_nonzero_margin": robust_nonzero_margin,
        "robust_margin_ratio_to_total": robust_margin_ratio_to_total,
        "robust_nonzero": robust_nonzero,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Candidate-shape exactness stress test")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko_100k.json",
    )
    ap.add_argument("--m-values", type=str, default="20000,50000")
    ap.add_argument("--betas", type=str, default="0.51,0.55")
    ap.add_argument("--x-windows", type=str, default="1e21:1e22,1e23:1e24")
    ap.add_argument("--grid-size", type=int, default=1024)
    ap.add_argument("--head-count", type=int, default=256)
    ap.add_argument("--chunk-a", type=int, default=512)
    ap.add_argument("--chunk-b", type=int, default=1024)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w65_candidate_shape_exactness_stress_test_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    m_values = parse_int_list(args.m_values)
    betas = parse_float_list(args.betas)
    windows = parse_windows(args.x_windows)

    zeros_raw = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    zeros_all = np.asarray(sorted(z for z in zeros_raw if z > 0.0), dtype=np.float64)
    if zeros_all.size == 0:
        raise ValueError("no positive zeta zeros loaded")

    rows: List[Dict[str, float | int | str | bool]] = []
    for m in m_values:
        if m > zeros_all.size:
            raise ValueError(f"m={m} exceeds loaded zeros count={zeros_all.size}")
        z = zeros_all[:m]
        for beta in betas:
            for x_min, x_max in windows:
                row = summarize_config(
                    zeros=z,
                    beta=beta,
                    x_min=x_min,
                    x_max=x_max,
                    grid_size=args.grid_size,
                    head_count=args.head_count,
                    chunk_a=args.chunk_a,
                    chunk_b=args.chunk_b,
                )
                row["max_zeta_zeros"] = int(m)
                rows.append(row)

    robust_rows = [r for r in rows if bool(r["robust_nonzero"])]
    all_robust = len(robust_rows) == len(rows)
    min_margin_ratio = min(float(r["robust_margin_ratio_to_total"]) for r in rows)
    min_sup_ratio = min(float(r["sup_residual_ratio_to_total"]) for r in rows)
    max_discrepancy_ratio = max(
        float(r["sup_abs_chunk_discrepancy"]) / max(float(r["sup_abs_total"]), 1.0e-300)
        for r in rows
    )

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "loaded_zero_count": int(zeros_all.size),
            "m_values": m_values,
            "betas": betas,
            "x_windows": [{"x_min": lo, "x_max": hi} for lo, hi in windows],
            "grid_size": int(args.grid_size),
            "head_count": int(args.head_count),
            "chunk_a": int(args.chunk_a),
            "chunk_b": int(args.chunk_b),
        },
        "rows": rows,
        "summary": {
            "all_rows_robust_nonzero": all_robust,
            "robust_rows": len(robust_rows),
            "total_rows": len(rows),
            "min_robust_margin_ratio_to_total": min_margin_ratio,
            "min_sup_residual_ratio_to_total": min_sup_ratio,
            "max_chunk_discrepancy_ratio_to_total": max_discrepancy_ratio,
        },
        "interpretation": {
            "note": (
                "If all rows are robustly nonzero, finite-head exact identity is not supported "
                "for this explicit-formula surrogate across tested tail windows; this favors a "
                "finite-head-plus-majorant theorem target over exact finite collapse."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Candidate-Shape Exactness Stress Test ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Summary")
    s = result["summary"]
    lines.append(f"- all_rows_robust_nonzero: {s['all_rows_robust_nonzero']}")
    lines.append(f"- robust_rows / total_rows: {s['robust_rows']} / {s['total_rows']}")
    lines.append(f"- min_robust_margin_ratio_to_total: {s['min_robust_margin_ratio_to_total']:.6e}")
    lines.append(f"- min_sup_residual_ratio_to_total: {s['min_sup_residual_ratio_to_total']:.6e}")
    lines.append(f"- max_chunk_discrepancy_ratio_to_total: {s['max_chunk_discrepancy_ratio_to_total']:.6e}")
    lines.append("")
    lines.append("| M | beta | x-window | sup_res/total | robust_margin/total | chunk_discrepancy/total | robust_nonzero |")
    lines.append("|---:|---:|---:|---:|---:|---:|:---:|")
    for r in rows:
        denom = max(float(r["sup_abs_total"]), 1.0e-300)
        disc_ratio = float(r["sup_abs_chunk_discrepancy"]) / denom
        lines.append(
            f"| {int(r['max_zeta_zeros'])} | {float(r['beta']):.4f} | "
            f"[{float(r['x_min']):.1e}, {float(r['x_max']):.1e}] | "
            f"{float(r['sup_residual_ratio_to_total']):.6e} | "
            f"{float(r['robust_margin_ratio_to_total']):.6e} | "
            f"{disc_ratio:.6e} | "
            f"{'yes' if bool(r['robust_nonzero']) else 'no'} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append(f"- {result['interpretation']['note']}")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

