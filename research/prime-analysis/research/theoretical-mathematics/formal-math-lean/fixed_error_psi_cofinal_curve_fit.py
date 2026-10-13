#!/usr/bin/env python3
"""Fit cofinal aligned remainder-ratio curves for theorem-facing mode admissibility.

For fixed beta and tau candidates:
  q(X) := min_{aligned x >= X} |R(x)|/A
on a threshold grid.

Fit model:
  q(X) ~ q_inf + B * X^(-eta),  with q_inf>=0, B>=0, eta>=0.

This directly targets the subsequence theorem shape:
  existence of q_inf < a0.
Finite-window evidence only.
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
from prime_geometry_loop import load_zeta_zeros_file


@dataclass
class TauCurveRow:
    tau: float
    amplitude: float
    phase: float
    aligned_count_tail: int
    q_inf_est: float
    b_est: float
    eta_est: float
    rmse_fit: float
    q_min_grid: float
    q_max_grid: float
    q_at_xmax_frac: float
    q_inf_below_a0: bool


def _fit_mode(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau: float,
    include_decay_term: bool,
) -> Dict[str, float | np.ndarray]:
    c = np.cos(tau * t)
    s = np.sin(tau * t)
    cols = [c, s]
    if include_decay_term:
        cols.append(np.power(x, -beta))
    xmat = np.column_stack(cols)
    coef, *_ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    r = y - yhat
    a = float(coef[0])
    b = float(coef[1])
    amp = float(math.hypot(a, b))
    phi = float(math.atan2(-b, a))
    return {"amp": amp, "phi": phi, "r": r}


def _cofinal_curve(
    x: np.ndarray,
    t: np.ndarray,
    r: np.ndarray,
    amp: float,
    tau: float,
    phi: float,
    tail_start: int,
    a0: float,
    grid_count: int,
    xmax_frac: float,
) -> Tuple[np.ndarray, np.ndarray, int]:
    if amp <= 1.0e-18:
        return np.array([], dtype=np.float64), np.array([], dtype=np.float64), 0
    phase = tau * t + phi
    cos_abs = np.abs(np.cos(phase))
    aligned_mask = np.zeros_like(cos_abs, dtype=bool)
    aligned_mask[tail_start:] = cos_abs[tail_start:] >= a0
    idx = np.nonzero(aligned_mask)[0]
    if idx.size == 0:
        return np.array([], dtype=np.float64), np.array([], dtype=np.float64), 0

    x_al = x[idx]
    rr_al = np.abs(r[idx]) / amp

    x_start = float(max(x_al[0], x[tail_start]))
    x_end = float(xmax_frac) * float(np.max(x_al))
    if x_end <= x_start:
        x_end = float(np.max(x_al))
    if x_end <= x_start:
        return np.array([], dtype=np.float64), np.array([], dtype=np.float64), int(idx.size)

    xg = np.exp(np.linspace(math.log(x_start), math.log(x_end), int(max(1, grid_count))))
    qg: List[float] = []
    xg_keep: List[float] = []
    for x0 in xg:
        m = x_al >= x0
        if np.any(m):
            xg_keep.append(float(x0))
            qg.append(float(np.min(rr_al[m])))
    if not xg_keep:
        return np.array([], dtype=np.float64), np.array([], dtype=np.float64), int(idx.size)
    return np.array(xg_keep, dtype=np.float64), np.array(qg, dtype=np.float64), int(idx.size)


def _fit_qinf_power(x: np.ndarray, q: np.ndarray) -> Dict[str, float]:
    # q(X)=inf_{x>=X} rr(x) is nondecreasing in X and approaches liminf from below.
    # Fit:
    #   q(X) ~ q_inf - B * X^(-eta),  with q_inf >= max(q), B>=0, eta>=0.
    # Linearized with z = q_inf - q > 0:
    #   log z = log B - eta log X.
    q_min = float(np.min(q))
    q_max = float(np.max(q))
    span = max(1.0e-9, q_max - q_min)
    q_candidates = np.linspace(q_max + 1.0e-9, q_max + 2.0 * span, 128)

    best = {
        "q_inf": 0.0,
        "b": 0.0,
        "eta": 0.0,
        "rmse": float("inf"),
    }

    lx = np.log(np.maximum(x, 1.0e-300))
    for q_inf in q_candidates:
        y = q_inf - q
        if np.any(y <= 0.0):
            continue
        ly = np.log(y)
        mx = float(np.mean(lx))
        my = float(np.mean(ly))
        den = float(np.sum((lx - mx) ** 2))
        if den <= 1.0e-30:
            continue
        slope = float(np.sum((lx - mx) * (ly - my)) / den)
        eta = -slope
        if eta < 0.0:
            continue
        b = float(math.exp(my + eta * mx))
        q_hat = q_inf - b * np.power(x, -eta)
        rmse = float(np.sqrt(np.mean((q - q_hat) ** 2)))
        if rmse < best["rmse"]:
            best = {"q_inf": float(q_inf), "b": b, "eta": float(eta), "rmse": rmse}
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="Fit cofinal aligned remainder-ratio curves")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=30_000_000)
    ap.add_argument("--samples", type=int, default=2600)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--tau-count", type=int, default=12)
    ap.add_argument("--tail-frac", type=float, default=0.5)
    ap.add_argument("--abs-cos-min", type=float, default=0.98)
    ap.add_argument("--cofinal-grid-count", type=int, default=40)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_cofinal_curve_fit_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file) if float(z) > 0.0]
    zeros.sort()
    taus = zeros[: max(1, int(args.tau_count))]
    if not taus:
        raise ValueError("no tau candidates")

    xs = logspace_int(args.xmin, args.xmax, args.samples)
    x = xs.astype(np.float64)
    t = np.log(x)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))

    tail_start = int((1.0 - float(args.tail_frac)) * len(x))
    tail_start = max(1, min(tail_start, len(x) - 2))
    a0 = float(args.abs_cos_min)

    rows: List[TauCurveRow] = []
    diag: List[Dict[str, object]] = []

    for tau in taus:
        fit = _fit_mode(
            x=x,
            t=t,
            y=y,
            beta=float(args.beta),
            tau=float(tau),
            include_decay_term=bool(args.include_decay_term),
        )
        amp = float(fit["amp"])
        phi = float(fit["phi"])
        r = np.asarray(fit["r"], dtype=np.float64)
        xg, qg, aligned_n = _cofinal_curve(
            x=x,
            t=t,
            r=r,
            amp=amp,
            tau=float(tau),
            phi=phi,
            tail_start=tail_start,
            a0=a0,
            grid_count=int(args.cofinal_grid_count),
            xmax_frac=float(args.cofinal_xmax_frac),
        )
        if xg.size < 8:
            continue

        m = _fit_qinf_power(x=xg, q=qg)
        q_inf = float(m["q_inf"])
        b_est = float(m["b"])
        eta_est = float(m["eta"])
        rmse = float(m["rmse"])

        rows.append(
            TauCurveRow(
                tau=float(tau),
                amplitude=amp,
                phase=phi,
                aligned_count_tail=int(aligned_n),
                q_inf_est=q_inf,
                b_est=b_est,
                eta_est=eta_est,
                rmse_fit=rmse,
                q_min_grid=float(np.min(qg)),
                q_max_grid=float(np.max(qg)),
                q_at_xmax_frac=float(qg[-1]),
                q_inf_below_a0=bool(q_inf < a0),
            )
        )
        diag.append(
            {
                "tau": float(tau),
                "x_grid": [float(v) for v in xg],
                "q_grid": [float(v) for v in qg],
                "fit": m,
            }
        )

    rows.sort(key=lambda r: (not r.q_inf_below_a0, r.q_inf_est, r.rmse_fit))

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "samples": int(len(xs)),
            "beta": float(args.beta),
            "tau_count": int(args.tau_count),
            "tail_frac": float(args.tail_frac),
            "abs_cos_min": a0,
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
        },
        "rows": [asdict(r) for r in rows],
        "diagnostics": diag,
        "interpretation": {
            "note": (
                "Finite-window cofinal-curve fit only. "
                "q_inf_est is a candidate liminf remainder ratio parameter."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Cofinal Curve Fit ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Alignment gate: `a0={a0}`")
    lines.append("")
    lines.append("## Tau Fit Table")
    lines.append("")
    lines.append("| tau | aligned_n | q_inf_est | eta_est | B_est | rmse | q_min | q_max | q_inf<a0 |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.tau:.6f} | {r.aligned_count_tail} | {r.q_inf_est:.6f} | {r.eta_est:.6f} | "
            f"{r.b_est:.6e} | {r.rmse_fit:.6e} | {r.q_min_grid:.6f} | {r.q_max_grid:.6f} | "
            f"{str(r.q_inf_below_a0).lower()} |"
        )
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
