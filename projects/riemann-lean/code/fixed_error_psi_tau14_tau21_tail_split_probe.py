#!/usr/bin/env python3
"""Tau14/Tau21 constructive-gate tail-split probe.

Decompose on selected constructive subsequence:
  Y = M2 + (MM - M2) + (Y - MM)
where
  M2: two-mode model (tau1,tau2) + optional decay term
  MM: first M-mode model + optional decay term

So with R2 := Y - M2:
  R2 = Tail_M + Rem_M
  Tail_M := MM - M2
  Rem_M  := Y - MM

This script estimates cofinal split bounds:
  q_tail = cofinal(|Tail_M|/A1)
  q_rem  = cofinal(|Rem_M|/A1)
  q_split = q_tail + q_rem
  delta_split = a1 - q_split
for theorem-facing diagnostics on the constructive nonnegative gate.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from fixed_error_psi_tau14_tau21_constructive_gate_probe import _anchor_center, _phase_anchors
from fixed_error_psi_tau14_tau21_phase_gate_probe import _crossover_x, _fit_two_mode
from prime_geometry_loop import load_zeta_zeros_file


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    if not vals:
        raise ValueError("empty int list")
    return vals


def _fit_multimode(
    x: np.ndarray,
    y: np.ndarray,
    taus: List[float],
    beta: float,
    include_decay_term: bool,
) -> Dict[str, np.ndarray | float]:
    t = np.log(x)
    cols: List[np.ndarray] = []
    for tau in taus:
        cols.append(np.cos(float(tau) * t))
        cols.append(np.sin(float(tau) * t))
    if include_decay_term:
        cols.append(np.power(x, -beta))
    xmat = np.column_stack(cols)
    coef, *_ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    resid = y - yhat
    rmse = float(np.sqrt(np.mean(np.square(resid))))
    return {"coef": coef, "rmse": rmse}


def _predict_multimode(
    x: np.ndarray,
    coef: np.ndarray,
    taus: List[float],
    beta: float,
    include_decay_term: bool,
) -> np.ndarray:
    t = np.log(x)
    cols: List[np.ndarray] = []
    for tau in taus:
        cols.append(np.cos(float(tau) * t))
        cols.append(np.sin(float(tau) * t))
    if include_decay_term:
        cols.append(np.power(x, -beta))
    xmat = np.column_stack(cols)
    return xmat @ coef


def _cofinal_stat(x: np.ndarray, vals: np.ndarray, grid_count: int, xmax_frac: float) -> float:
    if x.size == 0 or vals.size == 0:
        return float("nan")
    x_end = float(xmax_frac) * float(np.max(x))
    if x_end <= x[0]:
        x_end = float(np.max(x))
    if x_end <= x[0]:
        return float(np.min(vals))
    grid = np.exp(np.linspace(math.log(float(x[0])), math.log(float(x_end)), int(grid_count)))
    mins: List[float] = []
    for x0 in grid:
        m = x >= x0
        if not np.any(m):
            continue
        mins.append(float(np.min(vals[m])))
    if not mins:
        return float(np.min(vals))
    return float(max(mins))


@dataclass
class TailSplitRow:
    mode_count: int
    taus_last: float
    selected_count: int
    q_raw_cofinal: float
    q_raw_neg_cofinal: float
    q_tail_cofinal: float
    q_tail_neg_cofinal: float
    q_rem_cofinal: float
    q_rem_neg_cofinal: float
    q_split_cofinal: float
    q_split_neg_cofinal: float
    delta_raw: float
    delta_raw_neg: float
    delta_split: float
    delta_split_neg: float
    c_beta_split: float
    c_beta_split_neg: float
    c_beta_raw: float
    c_beta_raw_neg: float
    rmse_global_mmode: float
    triangle_violation_max: float
    crossover_x_split: float | None
    crossover_x_split_neg: float | None


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau14/tau21 constructive tail-split probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--search-window", type=int, default=100)
    ap.add_argument("--mode-count-list", type=str, default="2,4,8,12,16,24,32")
    ap.add_argument("--cofinal-grid-count", type=int, default=80)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--min-selected-points", type=int, default=8)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau14_tau21_tail_split_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.a1 <= 0.0 or args.a1 > 1.0:
        raise ValueError("a1 must be in (0,1]")

    mode_counts = sorted(set(m for m in parse_int_list(args.mode_count_list) if m >= 2))
    if not mode_counts:
        raise ValueError("mode-count list must include values >=2")

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file) if float(z) > 0.0]
    zeros.sort()
    if len(zeros) < max(mode_counts):
        raise ValueError("not enough zeros for requested mode counts")

    xs = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x = xs.astype(np.float64)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))
    abs_e = np.abs(e_vals)
    c_endpoint = float(np.max(abs_e / (np.sqrt(x) * np.power(np.log(x), 2))))

    fit2 = _fit_two_mode(
        x=x,
        y=y,
        tau1=float(args.tau1),
        tau2=float(args.tau2),
        beta=float(args.beta),
        include_decay_term=bool(args.include_decay_term),
    )
    a1_amp = float(fit2["A1"])
    if a1_amp <= 1.0e-18:
        raise ValueError("A1 too small")
    phi1 = float(fit2["phi1"])
    phi2 = float(fit2["phi2"])

    # Build constructive nonnegative gate selection (phase-only criterion).
    anchors = _phase_anchors(float(args.tau1), phi1, int(args.xmin), int(args.xmax))
    if not anchors:
        raise ValueError("no anchors in range")
    centers = [_anchor_center(k, float(args.tau1), phi1) for k in anchors]
    w = int(args.search_window)
    cand_set: set[int] = set()
    for c in centers:
        lo = max(int(args.xmin), c - w)
        hi = min(int(args.xmax), c + w)
        if hi < lo:
            continue
        cand_set.update(range(lo, hi + 1))
    if not cand_set:
        raise ValueError("no candidates in search windows")
    cand_sorted = np.array(sorted(cand_set), dtype=np.int64)
    psi_cand = psi_at_samples(cand_sorted, events)
    psi_lookup: Dict[int, float] = {int(n): float(v) for n, v in zip(cand_sorted.tolist(), psi_cand.tolist())}

    selected: List[int] = []
    for c in centers:
        lo = max(int(args.xmin), c - w)
        hi = min(int(args.xmax), c + w)
        if hi < lo:
            continue
        best: Tuple[float, float, int] | None = None  # (|cos2|, tie(rr_proxy), n)
        for n in range(lo, hi + 1):
            nf = float(max(2, n))
            t = math.log(nf)
            cos1 = abs(math.cos(float(args.tau1) * t + phi1))
            if cos1 < float(args.a1):
                continue
            cos2_signed = math.cos(float(args.tau2) * t + phi2)
            if cos2_signed < 0.0:
                continue
            cos2_abs = abs(cos2_signed)
            cand = (cos2_abs, 0.0, n)
            if best is None or cand[0] < best[0] or (cand[0] == best[0] and cand[2] < best[2]):
                best = cand
        if best is None:
            continue
        selected.append(int(best[2]))

    selected = sorted(set(selected))
    if len(selected) < int(args.min_selected_points):
        raise ValueError("insufficient selected points")

    x_sel = np.array(selected, dtype=np.float64)
    psi_sel = np.array([psi_lookup[int(n)] for n in selected], dtype=np.float64)
    y_sel = (psi_sel - x_sel) / np.power(x_sel, float(args.beta))

    # Two-mode baseline model on selected points.
    coef2 = np.asarray(fit2["coef"], dtype=np.float64)
    yhat2_sel = _predict_multimode(
        x=x_sel,
        coef=coef2,
        taus=[float(args.tau1), float(args.tau2)],
        beta=float(args.beta),
        include_decay_term=bool(args.include_decay_term),
    )
    r2_sel = y_sel - yhat2_sel
    q_raw = np.abs(r2_sel) / a1_amp
    q_raw_neg = np.maximum(-r2_sel, 0.0) / a1_amp

    rows: List[TailSplitRow] = []
    for m in mode_counts:
        taus_m = zeros[:m]
        fitm = _fit_multimode(
            x=x,
            y=y,
            taus=taus_m,
            beta=float(args.beta),
            include_decay_term=bool(args.include_decay_term),
        )
        coefm = np.asarray(fitm["coef"], dtype=np.float64)
        yhatm_sel = _predict_multimode(
            x=x_sel,
            coef=coefm,
            taus=taus_m,
            beta=float(args.beta),
            include_decay_term=bool(args.include_decay_term),
        )

        tail_m = yhatm_sel - yhat2_sel
        rem_m = y_sel - yhatm_sel

        q_tail = np.abs(tail_m) / a1_amp
        q_rem = np.abs(rem_m) / a1_amp
        q_sum = q_tail + q_rem
        q_tail_neg = np.maximum(-tail_m, 0.0) / a1_amp
        q_rem_neg = np.maximum(-rem_m, 0.0) / a1_amp
        q_sum_neg = q_tail_neg + q_rem_neg

        q_raw_cof = _cofinal_stat(
            x=x_sel, vals=q_raw, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_raw_neg_cof = _cofinal_stat(
            x=x_sel, vals=q_raw_neg, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_tail_cof = _cofinal_stat(
            x=x_sel, vals=q_tail, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_tail_neg_cof = _cofinal_stat(
            x=x_sel, vals=q_tail_neg, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_rem_cof = _cofinal_stat(
            x=x_sel, vals=q_rem, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_rem_neg_cof = _cofinal_stat(
            x=x_sel, vals=q_rem_neg, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_sum_cof = _cofinal_stat(
            x=x_sel, vals=q_sum, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_sum_neg_cof = _cofinal_stat(
            x=x_sel, vals=q_sum_neg, grid_count=int(args.cofinal_grid_count), xmax_frac=float(args.cofinal_xmax_frac)
        )
        q_split = float(q_tail_cof + q_rem_cof)
        q_split_neg = float(q_tail_neg_cof + q_rem_neg_cof)
        delta_raw = float(float(args.a1) - q_raw_cof)
        delta_raw_neg = float(float(args.a1) - q_raw_neg_cof)
        delta_split = float(float(args.a1) - q_split)
        delta_split_neg = float(float(args.a1) - q_split_neg)
        c_raw = float(max(0.0, delta_raw) * a1_amp)
        c_raw_neg = float(max(0.0, delta_raw_neg) * a1_amp)
        c_split = float(max(0.0, delta_split) * a1_amp)
        c_split_neg = float(max(0.0, delta_split_neg) * a1_amp)
        tri_viol = float(np.max(np.abs(r2_sel) - (np.abs(tail_m) + np.abs(rem_m))))

        rows.append(
            TailSplitRow(
                mode_count=int(m),
                taus_last=float(taus_m[-1]),
                selected_count=len(selected),
                q_raw_cofinal=float(q_raw_cof),
                q_raw_neg_cofinal=float(q_raw_neg_cof),
                q_tail_cofinal=float(q_tail_cof),
                q_tail_neg_cofinal=float(q_tail_neg_cof),
                q_rem_cofinal=float(q_rem_cof),
                q_rem_neg_cofinal=float(q_rem_neg_cof),
                q_split_cofinal=q_split,
                q_split_neg_cofinal=q_split_neg,
                delta_raw=delta_raw,
                delta_raw_neg=delta_raw_neg,
                delta_split=delta_split,
                delta_split_neg=delta_split_neg,
                c_beta_split=c_split,
                c_beta_split_neg=c_split_neg,
                c_beta_raw=c_raw,
                c_beta_raw_neg=c_raw_neg,
                rmse_global_mmode=float(fitm["rmse"]),
                triangle_violation_max=tri_viol,
                crossover_x_split=_crossover_x(c_split, float(args.beta), c_endpoint, float(args.xmin)),
                crossover_x_split_neg=_crossover_x(c_split_neg, float(args.beta), c_endpoint, float(args.xmin)),
            )
        )

    rows.sort(key=lambda r: (r.mode_count,))

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "fit_samples": int(len(xs)),
            "beta": float(args.beta),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "a1": float(args.a1),
            "search_window": int(args.search_window),
            "mode_count_list": mode_counts,
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "min_selected_points": int(args.min_selected_points),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
            "endpoint_c_sup_window": c_endpoint,
            "selected_count": len(selected),
        },
        "two_mode_fit": {
            "A1": a1_amp,
            "A2": float(fit2["A2"]),
            "phi1": phi1,
            "phi2": phi2,
            "rmse_global_fit": float(fit2["rmse"]),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window constructive tail-split diagnostics only. "
                "delta_split>0 suggests viable decomposition-level margin for theorem development."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14/Tau21 Tail-Split Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Constructive gate: `cos1>=a1={args.a1}`, `cos2>=0`, `search_W={args.search_window}`")
    lines.append(f"- Selected points: `{len(selected)}`")
    lines.append(f"- `A1={a1_amp:.9e}`")
    lines.append("")
    lines.append("| M | q_raw | q_raw_neg | q_split | q_split_neg | delta_raw | delta_raw_neg | delta_split | delta_split_neg |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.mode_count} | {r.q_raw_cofinal:.6f} | {r.q_raw_neg_cofinal:.6f} | "
            f"{r.q_split_cofinal:.6f} | {r.q_split_neg_cofinal:.6f} | {r.delta_raw:.6f} | "
            f"{r.delta_raw_neg:.6f} | {r.delta_split:.6f} | {r.delta_split_neg:.6f} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
