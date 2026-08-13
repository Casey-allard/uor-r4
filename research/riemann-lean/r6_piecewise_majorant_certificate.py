#!/usr/bin/env python3
"""Build an R6 piecewise-majorant witness candidate from cached probe data.

This script converts an existing multi-mode probe output into a finite-window
certificate aligned to `R6DualBandPiecewisePowerMajorantWitnessTerm`.
It does not claim a proof. It prepares constants and checks that can be
formalized and then paired with an asymptotic theorem term.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Any, Dict, List, Tuple

import numpy as np


def _read_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _dominant_mode(mode_rows: List[Dict[str, Any]]) -> Tuple[int, Dict[str, Any]]:
    if not mode_rows:
        raise ValueError("mode_rows is empty")
    best_i = 0
    best_amp = float("-inf")
    for i, row in enumerate(mode_rows):
        amp = float(row.get("amplitude", 0.0))
        if amp > best_amp:
            best_amp = amp
            best_i = i
    return best_i, mode_rows[best_i]


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


def build_certificate(
    multimode_path: str,
    eta_override: float | None,
    x0_override: float | None,
    x1_override: float | None,
    safety_factor: float,
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
    dominant_index, dominant_row = _dominant_mode(mode_rows)
    mode_count = int(best.get("mode_count", len(mode_rows)))
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

    eta = float(eta_override) if eta_override is not None else float(best["remainder_majorant_eta"])
    if eta <= 0.0:
        raise ValueError(f"eta must be > 0 (got {eta})")

    x0 = float(x0_override) if x0_override is not None else float(best["remainder_majorant_tail_start_x"])
    x1 = float(x1_override) if x1_override is not None else float(best["remainder_majorant_tail_end_x"])
    if x1 < x0:
        raise ValueError(f"x1 must be >= x0 (x0={x0}, x1={x1})")

    mask = (x >= x0) & (x <= x1)
    if not np.any(mask):
        raise ValueError("no grid points fall in [x0, x1]")

    xw = x[mask]
    rw = r[mask]
    scaled = np.abs(rw) * np.power(xw, eta)
    c_window_raw = float(np.max(scaled))
    c_window = float(max(1.0e-300, safety_factor * c_window_raw))
    bound_w = c_window * np.power(xw, -eta)
    ratio_w = np.abs(rw) / np.maximum(bound_w, 1.0e-300)

    tail_ratio_sup_to_amp_total = float(best.get("tail_ratio_sup_to_amp_total", math.inf))

    checks = {
        "mode_count_ge_6": mode_count >= 6,
        "dominant_index_valid": 0 <= dominant_index < mode_count,
        "beta_gt_half": beta > 0.5,
        "eta_pos": eta > 0.0,
        "c_window_nonneg": c_window >= 0.0,
        "window_bound_verified": float(np.max(ratio_w)) <= 1.0 + 1.0e-12,
    }

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source_files": {
            "multimode_probe": multimode_path,
            "cache_npz": cache_path,
        },
        "candidate_term": "R6DualBandPiecewisePowerMajorantWitnessTerm",
        "candidate_pack": {
            "beta": beta,
            "tau_main": float(dominant_row["tau"]),
            "a_main": float(dominant_row["a"]),
            "b_main": float(dominant_row["b"]),
            "dominant_index": dominant_index,
            "mode_count": mode_count,
            "all_mode_rows": mode_rows,
            "decay_term_coeff": decay_coeff_val,
            "piecewise_majorant": {
                "C": c_window,
                "eta": eta,
                "x0": x0,
                "x1": x1,
                "window_grid_points": int(xw.size),
                "window_max_ratio_to_bound": float(np.max(ratio_w)),
                "window_mean_ratio_to_bound": float(np.mean(ratio_w)),
            },
            "fit_quality": {
                "tail_ratio_sup_to_amp_total": tail_ratio_sup_to_amp_total,
                "global_rmse": float(best.get("global_rmse", math.nan)),
                "tail_rmse": float(best.get("tail_rmse", math.nan)),
                "score": float(best.get("score", math.nan)),
            },
        },
        "checks": checks,
        "asymptotic_obligation": {
            "open": True,
            "required_shape": (
                "For all x >= x1, |R(x)/x^beta| <= C*x^(-eta) with the same C, eta as the finite window certificate."
            ),
            "x1": x1,
            "eta": eta,
            "C": c_window,
        },
        "interpretation": {
            "status": (
                "finite-window-certificate-ready"
                if all(checks.values())
                else "finite-window-certificate-failed"
            ),
            "note": (
                "This artifact certifies the finite-window piece of the R6 piecewise contract "
                "and leaves only the asymptotic tail theorem obligation open."
            ),
        },
    }
    return result


def write_markdown(result: Dict[str, Any], out_md: str) -> None:
    p = result["candidate_pack"]
    w = p["piecewise_majorant"]
    checks = result["checks"]
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# R6 Piecewise Majorant Certificate Candidate\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        f.write("## Candidate term\n")
        f.write(f"- `{result['candidate_term']}`\n\n")
        f.write("## Decomposition metadata\n")
        f.write(f"- beta: {p['beta']:.12g}\n")
        f.write(f"- tau_main: {p['tau_main']:.12g}\n")
        f.write(f"- a_main: {p['a_main']:.12g}\n")
        f.write(f"- b_main: {p['b_main']:.12g}\n")
        f.write(f"- dominant_index: {p['dominant_index']}\n")
        f.write(f"- mode_count: {p['mode_count']}\n")
        if p["decay_term_coeff"] is not None:
            f.write(f"- decay_term_coeff: {p['decay_term_coeff']:.12g}\n")
        f.write("\n")
        f.write("## Finite-window certificate\n")
        f.write(f"- C: {w['C']:.12g}\n")
        f.write(f"- eta: {w['eta']:.12g}\n")
        f.write(f"- x0: {w['x0']:.12g}\n")
        f.write(f"- x1: {w['x1']:.12g}\n")
        f.write(f"- window_grid_points: {w['window_grid_points']}\n")
        f.write(f"- window_max_ratio_to_bound: {w['window_max_ratio_to_bound']:.12g}\n")
        f.write(f"- window_mean_ratio_to_bound: {w['window_mean_ratio_to_bound']:.12g}\n\n")
        f.write("## Checks\n")
        for k, v in checks.items():
            f.write(f"- {k}: {v}\n")
        f.write("\n")
        f.write("## Remaining asymptotic obligation\n")
        f.write(f"- open: {result['asymptotic_obligation']['open']}\n")
        f.write(f"- required_shape: {result['asymptotic_obligation']['required_shape']}\n")
        f.write(f"- x1: {result['asymptotic_obligation']['x1']:.12g}\n")
        f.write(f"- eta: {result['asymptotic_obligation']['eta']:.12g}\n")
        f.write(f"- C: {result['asymptotic_obligation']['C']:.12g}\n\n")
        f.write("## Interpretation\n")
        f.write(f"- status: {result['interpretation']['status']}\n")
        f.write(f"- note: {result['interpretation']['note']}\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="Build R6 piecewise-majorant witness candidate")
    ap.add_argument(
        "--multimode",
        type=str,
        default="research/output/k1_multimode_phase_probe_2026-02-24_x1e18_m12_beta055.json",
    )
    ap.add_argument("--eta", type=float, default=None)
    ap.add_argument("--x0", type=float, default=None)
    ap.add_argument("--x1", type=float, default=None)
    ap.add_argument("--safety-factor", type=float, default=1.05)
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/r6_piecewise_majorant_certificate_2026-02-24_beta055_m12.json",
    )
    args = ap.parse_args()

    result = build_certificate(
        multimode_path=args.multimode,
        eta_override=args.eta,
        x0_override=args.x0,
        x1_override=args.x1,
        safety_factor=float(args.safety_factor),
    )

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    out_md = args.output.replace(".json", ".md")
    write_markdown(result, out_md)

    print(f"wrote: {args.output}")
    print(f"wrote: {out_md}")
    print("status:", result["interpretation"]["status"])
    print("window_max_ratio_to_bound:", result["candidate_pack"]["piecewise_majorant"]["window_max_ratio_to_bound"])


if __name__ == "__main__":
    main()

