#!/usr/bin/env python3
"""Tau14-focused multimode lattice residual decomposition.

For a fixed tau_focus and phase-locked lattice points:
  - fit baseline single-mode amplitude A_focus
  - progressively add nearby zero-frequencies as extra cosine/sine modes
  - measure residual ratio |R_K|/A_focus on aligned lattice subsequence

This is a finite-window diagnostic for tail-splitting strategy design.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List

import numpy as np

from fixed_error_psi_aligned_lattice_probe import _build_lattice_points, _cofinal_grid, _fit_mode
from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from prime_geometry_loop import load_zeta_zeros_file


@dataclass
class DecompRow:
    extra_mode_count: int
    tau_list: List[float]
    rr_max_used: float
    rr_cofinal_grid: float
    delta_rr_cofinal_grid: float
    delta_tri_cofinal_grid: float
    c_beta_tri_cofinal_grid: float
    c_beta_obs_cofinal_grid: float
    rmse_lattice: float


def _fit_multimode(
    x: np.ndarray,
    y: np.ndarray,
    taus: List[float],
    beta: float,
    include_decay_term: bool,
) -> Dict[str, np.ndarray]:
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
    return {"coef": coef, "yhat": yhat, "resid": resid, "rmse": np.array([rmse])}


def _nearest_other_taus(zeros: List[float], tau_focus: float, count: int) -> List[float]:
    vals = [float(z) for z in zeros if float(z) > 0.0 and abs(float(z) - tau_focus) > 1.0e-12]
    vals.sort(key=lambda z: (abs(z - tau_focus), z))
    return vals[: max(0, int(count))]


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau14 multimode lattice decomposition")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau-focus", type=float, default=14.134725142)
    ap.add_argument("--extra-mode-max", type=int, default=8)
    ap.add_argument("--abs-cos-min", type=float, default=0.98)
    ap.add_argument("--local-window", type=int, default=6)
    ap.add_argument("--cofinal-grid-count", type=int, default=100)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau14_multimode_lattice_decompose_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")

    xs_fit = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x_fit = xs_fit.astype(np.float64)
    t_fit = np.log(x_fit)
    events = psi_events(args.xmax, Path(args.cache_dir))

    psi_fit = psi_at_samples(xs_fit, events)
    y_fit = (psi_fit - x_fit) / np.power(x_fit, float(args.beta))

    base = _fit_mode(
        x=x_fit,
        t=t_fit,
        y=y_fit,
        beta=float(args.beta),
        tau=float(args.tau_focus),
        include_decay_term=bool(args.include_decay_term),
    )
    a_focus = float(base["amp"])
    phi_focus = float(base["phi"])
    if a_focus <= 1.0e-18:
        raise ValueError("tau_focus amplitude too small")

    x_lattice_i = _build_lattice_points(
        tau=float(args.tau_focus),
        phi=phi_focus,
        xmin=int(args.xmin),
        xmax=int(args.xmax),
        local_window=int(args.local_window),
    )
    if x_lattice_i.size < 12:
        raise ValueError("not enough lattice points")
    x_lattice = x_lattice_i.astype(np.float64)
    psi_lat = psi_at_samples(x_lattice_i, events)
    y_lattice = (psi_lat - x_lattice) / np.power(x_lattice, float(args.beta))
    phase_focus = float(args.tau_focus) * np.log(x_lattice) + phi_focus
    cos_abs = np.abs(np.cos(phase_focus))
    use = cos_abs >= float(args.abs_cos_min)
    if np.count_nonzero(use) < 10:
        raise ValueError("not enough aligned points after abs-cos gate")

    x_use = x_lattice[use]
    y_use = y_lattice[use]
    cos_use = cos_abs[use]

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file) if float(z) > 0.0]
    zeros.sort()
    extra_taus = _nearest_other_taus(zeros, float(args.tau_focus), int(args.extra_mode_max))
    a0 = float(args.abs_cos_min)

    rows: List[DecompRow] = []
    for k in range(0, int(args.extra_mode_max) + 1):
        tau_list = [float(args.tau_focus)] + extra_taus[:k]
        # Fit on dense global samples; evaluate on aligned lattice holdout.
        fit = _fit_multimode(
            x=x_fit,
            y=y_fit,
            taus=tau_list,
            beta=float(args.beta),
            include_decay_term=bool(args.include_decay_term),
        )
        # Use global-fit coefficients on aligned points.
        t_use = np.log(x_use)
        cols_eval: List[np.ndarray] = []
        for tau_eval in tau_list:
            cols_eval.append(np.cos(float(tau_eval) * t_use))
            cols_eval.append(np.sin(float(tau_eval) * t_use))
        if bool(args.include_decay_term):
            cols_eval.append(np.power(x_use, -float(args.beta)))
        xmat_eval = np.column_stack(cols_eval)
        coef_global = np.asarray(fit["coef"], dtype=np.float64)
        yhat_use = xmat_eval @ coef_global
        resid_use = y_use - yhat_use
        rr = np.abs(resid_use) / a_focus
        tri = np.maximum(cos_use - rr, 0.0)
        yabs = np.abs(y_use)

        x_end = float(args.cofinal_xmax_frac) * float(np.max(x_use))
        if x_end <= x_use[0]:
            x_end = float(np.max(x_use))
        xg = _cofinal_grid(float(x_use[0]), x_end, int(args.cofinal_grid_count))
        if xg.size == 0:
            continue

        min_rr_future: List[float] = []
        max_tri_future: List[float] = []
        max_obs_future: List[float] = []
        for x0 in xg:
            m = x_use >= x0
            if not np.any(m):
                continue
            min_rr_future.append(float(np.min(rr[m])))
            max_tri_future.append(float(np.max(tri[m])))
            max_obs_future.append(float(np.max(yabs[m])))
        if not min_rr_future:
            continue

        rr_cofinal = float(max(min_rr_future))
        delta_rr = float(a0 - rr_cofinal)
        delta_tri = float(min(max_tri_future))
        c_tri = float(max(0.0, delta_tri) * a_focus)
        c_obs = float(min(max_obs_future))
        rmse = float(fit["rmse"][0])

        rows.append(
            DecompRow(
                extra_mode_count=int(k),
                tau_list=tau_list,
                rr_max_used=float(np.max(rr)),
                rr_cofinal_grid=rr_cofinal,
                delta_rr_cofinal_grid=delta_rr,
                delta_tri_cofinal_grid=delta_tri,
                c_beta_tri_cofinal_grid=c_tri,
                c_beta_obs_cofinal_grid=c_obs,
                rmse_lattice=rmse,
            )
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "fit_samples": int(len(xs_fit)),
            "beta": float(args.beta),
            "tau_focus": float(args.tau_focus),
            "a_focus": a_focus,
            "phi_focus": phi_focus,
            "abs_cos_min": a0,
            "local_window": int(args.local_window),
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "extra_mode_max": int(args.extra_mode_max),
            "extra_taus_by_distance": extra_taus,
            "aligned_points_used": int(np.count_nonzero(use)),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window multimode decomposition only. "
                "Use rr_cofinal trend vs extra_mode_count to design analytic tail split."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14 Multimode Lattice Decomposition ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Tau focus: `{args.tau_focus}`")
    lines.append(f"- Focus amplitude A: `{a_focus:.9e}`")
    lines.append(f"- Aligned points used: `{int(np.count_nonzero(use))}`")
    lines.append("")
    lines.append("| K extra | rr_cofinal | delta_rr | delta_tri | rmse |")
    lines.append("|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.extra_mode_count} | {r.rr_cofinal_grid:.6f} | {r.delta_rr_cofinal_grid:.6f} | "
            f"{r.delta_tri_cofinal_grid:.6f} | {r.rmse_lattice:.6e} |"
        )
    lines.append("")
    lines.append("Finite-window decomposition only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
