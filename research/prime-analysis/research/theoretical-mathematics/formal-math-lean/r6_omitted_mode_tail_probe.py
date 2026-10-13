#!/usr/bin/env python3
"""Probe omitted-mode tail majorant for R6 model packs.

Given a multimode fit output, this reconstructs the normalized residual
`R(x)/x^beta` on the cached grid and measures the tail majorant constant
`C(x1) = sup_{x>=x1} |R(x)/x^beta| * x^eta` for a fixed eta.

Numerical evidence only; not a theorem proof.
"""

from __future__ import annotations

import argparse
import json
import os
from datetime import datetime, timezone
from typing import Any, Dict, List

import numpy as np


def _read_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _parse_float_grid(text: str) -> List[float]:
    vals: List[float] = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        vals.append(float(tok))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("x1 grid is empty")
    return vals


def _reconstruct_model(
    x: np.ndarray,
    t: np.ndarray,
    beta: float,
    mode_rows: List[Dict[str, Any]],
    decay_term_coeff: float | None,
) -> np.ndarray:
    yhat = np.zeros_like(x)
    for row in mode_rows:
        tau = float(row["tau"])
        a = float(row["a"])
        b = float(row["b"])
        yhat += a * np.cos(tau * t) + b * np.sin(tau * t)
    if decay_term_coeff is not None:
        yhat += float(decay_term_coeff) * np.power(x, -beta)
    return yhat


def run_probe(
    multimode_path: str,
    eta: float,
    x1_grid: List[float],
    min_points_for_summary: int,
) -> Dict[str, Any]:
    m = _read_json(multimode_path)
    cfg = m["config"]
    best = m["best_by_score"]
    cache_path = str(cfg["cache_path"])
    loaded = np.load(cache_path)
    x = loaded["x"].astype(np.float64)
    t = loaded["t"].astype(np.float64)
    y = loaded["y"].astype(np.float64)

    beta = float(cfg["beta"])
    mode_rows = list(best["mode_rows"])
    decay_term_coeff = best.get("decay_term_coeff")
    decay_coeff_val = float(decay_term_coeff) if decay_term_coeff is not None else None

    yhat = _reconstruct_model(
        x=x,
        t=t,
        beta=beta,
        mode_rows=mode_rows,
        decay_term_coeff=decay_coeff_val,
    )
    r = y - yhat

    rows: List[Dict[str, Any]] = []
    for x1 in x1_grid:
        mask = x >= x1
        if not np.any(mask):
            rows.append(
                {
                    "x1": float(x1),
                    "grid_points": 0,
                    "C_tail": None,
                    "tail_max_abs": None,
                    "tail_mean_abs": None,
                    "tail_ratio_max_to_bound": None,
                }
            )
            continue
        xt = x[mask]
        rt = r[mask]
        scaled = np.abs(rt) * np.power(xt, eta)
        c_tail = float(np.max(scaled))
        c_tail = max(c_tail, 1.0e-300)
        bound = c_tail * np.power(xt, -eta)
        ratio = np.abs(rt) / np.maximum(bound, 1.0e-300)
        rows.append(
            {
                "x1": float(x1),
                "grid_points": int(xt.size),
                "C_tail": c_tail,
                "tail_max_abs": float(np.max(np.abs(rt))),
                "tail_mean_abs": float(np.mean(np.abs(rt))),
                "tail_ratio_max_to_bound": float(np.max(ratio)),
            }
        )

    valid_rows = [
        rw for rw in rows
        if rw["C_tail"] is not None and int(rw["grid_points"]) >= min_points_for_summary
    ]
    valid_c = [float(rw["C_tail"]) for rw in valid_rows]
    summary: Dict[str, Any] = {}
    if valid_c:
        summary = {
            "C_tail_min": float(min(valid_c)),
            "C_tail_max": float(max(valid_c)),
            "C_tail_ratio_max_over_min": float(max(valid_c) / min(valid_c)),
            "rows_count": len(rows),
            "rows_used_for_summary": len(valid_rows),
            "min_points_for_summary": int(min_points_for_summary),
        }

    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source_files": {
            "multimode_probe": multimode_path,
            "cache_npz": cache_path,
        },
        "config": {
            "beta": beta,
            "eta": float(eta),
            "x1_grid": [float(v) for v in x1_grid],
            "mode_count": int(best.get("mode_count", len(mode_rows))),
            "include_decay_term": decay_coeff_val is not None,
        },
        "rows": rows,
        "summary": summary,
        "interpretation": {
            "note": (
                "Numerical tail majorant scan for omitted-mode residual; "
                "use as evidence for L2/L3 lemma calibration, not as a proof."
            )
        },
    }


def write_markdown(result: Dict[str, Any], out_md: str) -> None:
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# R6 Omitted-Mode Tail Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        cfg = result["config"]
        f.write("## Config\n")
        f.write(f"- beta: {cfg['beta']}\n")
        f.write(f"- eta: {cfg['eta']}\n")
        f.write(f"- mode_count: {cfg['mode_count']}\n")
        f.write(f"- include_decay_term: {cfg['include_decay_term']}\n")
        f.write(f"- x1_grid: {cfg['x1_grid']}\n\n")
        f.write("## Rows\n")
        for row in result["rows"]:
            f.write(
                f"- x1={row['x1']:.12g} | grid_points={row['grid_points']} | "
                f"C_tail={row['C_tail']} | max_ratio={row['tail_ratio_max_to_bound']}\n"
            )
        f.write("\n## Summary\n")
        for k, v in result.get("summary", {}).items():
            f.write(f"- {k}: {v}\n")
        f.write("\n## Interpretation\n")
        f.write(f"- {result['interpretation']['note']}\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="Probe omitted-mode tail majorant constants")
    ap.add_argument(
        "--multimode",
        type=str,
        default="research/output/k1_multimode_phase_probe_2026-02-24_x1e22_m12_beta055_growth.json",
    )
    ap.add_argument("--eta", type=float, default=0.034301034952287375)
    ap.add_argument("--x1-grid", type=str, default="1e18,1e19,1e20,1e21,1e22")
    ap.add_argument("--min-points-for-summary", type=int, default=64)
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/r6_omitted_mode_tail_probe_2026-02-24_beta055_eta0343.json",
    )
    args = ap.parse_args()

    x1_grid = _parse_float_grid(args.x1_grid)
    result = run_probe(
        multimode_path=args.multimode,
        eta=float(args.eta),
        x1_grid=x1_grid,
        min_points_for_summary=int(args.min_points_for_summary),
    )
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)
    out_md = args.output.replace(".json", ".md")
    write_markdown(result, out_md)
    print(f"wrote: {args.output}")
    print(f"wrote: {out_md}")
    print("summary:", result.get("summary", {}))


if __name__ == "__main__":
    main()
