#!/usr/bin/env python3
"""Dual-phase gate probe for tau14/tau21 lower-envelope witnesses.

Target inequality on a gated subsequence:
  |Y_beta(x)| >= A1*|cos1| - A2*|cos2| - |R2(x)|

Gate:
  |cos1| >= a1,  |cos2| <= eps2.

Define alpha = A2/A1 and rr = |R2|/A1, then
  |Y_beta(x)| >= A1 * (a1 - alpha*eps2 - rr).
This script measures finite-window cofinal evidence for positive margin.
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

from fixed_error_psi_aligned_lattice_probe import _build_lattice_points, _cofinal_grid
from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(float(tok))
    if not vals:
        raise ValueError("empty float list")
    return vals


def _fit_two_mode(
    x: np.ndarray,
    y: np.ndarray,
    tau1: float,
    tau2: float,
    beta: float,
    include_decay_term: bool,
) -> Dict[str, float | np.ndarray]:
    t = np.log(x)
    c1 = np.cos(tau1 * t)
    s1 = np.sin(tau1 * t)
    c2 = np.cos(tau2 * t)
    s2 = np.sin(tau2 * t)
    cols = [c1, s1, c2, s2]
    if include_decay_term:
        cols.append(np.power(x, -beta))
    xmat = np.column_stack(cols)
    coef, *_ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    resid = y - yhat

    a1 = float(coef[0])
    b1 = float(coef[1])
    a2 = float(coef[2])
    b2 = float(coef[3])
    d = float(coef[4]) if include_decay_term else 0.0
    a1_amp = float(math.hypot(a1, b1))
    a2_amp = float(math.hypot(a2, b2))
    phi1 = float(math.atan2(-b1, a1))
    phi2 = float(math.atan2(-b2, a2))
    rmse = float(np.sqrt(np.mean(np.square(resid))))
    return {
        "coef": coef,
        "a1": a1,
        "b1": b1,
        "a2": a2,
        "b2": b2,
        "d": d,
        "A1": a1_amp,
        "A2": a2_amp,
        "phi1": phi1,
        "phi2": phi2,
        "rmse": rmse,
    }


def _upper_gap(x: float, c_beta: float, beta: float, c_endpoint: float) -> float:
    return c_beta * (x ** beta) - c_endpoint * (x ** 0.5) * (math.log(x) ** 2)


def _crossover_x(c_beta: float, beta: float, c_endpoint: float, x_lo: float) -> float | None:
    if c_beta <= 0.0 or beta <= 0.5 or c_endpoint <= 0.0:
        return None
    lo = max(2.0, x_lo)
    if _upper_gap(lo, c_beta, beta, c_endpoint) >= 0.0:
        return float(lo)
    hi = max(1.0e6, lo * 2.0)
    while hi < 1.0e300:
        if _upper_gap(hi, c_beta, beta, c_endpoint) >= 0.0:
            break
        hi *= 2.0
    else:
        return None
    for _ in range(160):
        mid = math.sqrt(lo * hi)
        if _upper_gap(mid, c_beta, beta, c_endpoint) >= 0.0:
            hi = mid
        else:
            lo = mid
    return float(hi)


@dataclass
class GateRow:
    eps2: float
    gated_points: int
    cos1_min: float
    cos2_max: float
    rr_cofinal_grid: float
    alpha_eps2: float
    q_eff_cofinal: float
    delta_eff: float
    c_beta_eff: float
    c_beta_obs_cofinal: float
    crossover_x_eff: float | None


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau14/Tau21 dual-phase gate probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--eps2-list", type=str, default="0.05,0.1,0.15,0.2,0.3")
    ap.add_argument("--local-window", type=int, default=6)
    ap.add_argument("--cofinal-grid-count", type=int, default=100)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--min-gated-points", type=int, default=8)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau14_tau21_phase_gate_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.a1 <= 0.0 or args.a1 > 1.0:
        raise ValueError("a1 must be in (0,1]")

    eps2_list = parse_float_list(args.eps2_list)
    eps2_list = sorted(set(float(v) for v in eps2_list if v >= 0.0))
    if not eps2_list:
        raise ValueError("eps2 list empty after filtering")

    xs = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x = xs.astype(np.float64)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))
    abs_e = np.abs(e_vals)

    c_endpoint = float(np.max(abs_e / (np.sqrt(x) * np.power(np.log(x), 2))))

    fit = _fit_two_mode(
        x=x,
        y=y,
        tau1=float(args.tau1),
        tau2=float(args.tau2),
        beta=float(args.beta),
        include_decay_term=bool(args.include_decay_term),
    )
    a1_amp = float(fit["A1"])
    a2_amp = float(fit["A2"])
    if a1_amp <= 1.0e-18:
        raise ValueError("A1 too small")
    alpha = float(a2_amp / a1_amp)
    phi1 = float(fit["phi1"])
    phi2 = float(fit["phi2"])
    coef = np.asarray(fit["coef"], dtype=np.float64)

    x_lat_i = _build_lattice_points(
        tau=float(args.tau1),
        phi=phi1,
        xmin=int(args.xmin),
        xmax=int(args.xmax),
        local_window=int(args.local_window),
    )
    if x_lat_i.size < 12:
        raise ValueError("not enough lattice points")
    x_lat = x_lat_i.astype(np.float64)
    psi_lat = psi_at_samples(x_lat_i, events)
    y_lat = (psi_lat - x_lat) / np.power(x_lat, float(args.beta))
    t_lat = np.log(x_lat)

    c1 = np.cos(float(args.tau1) * t_lat)
    s1 = np.sin(float(args.tau1) * t_lat)
    c2 = np.cos(float(args.tau2) * t_lat)
    s2 = np.sin(float(args.tau2) * t_lat)
    cols_eval = [c1, s1, c2, s2]
    if bool(args.include_decay_term):
        cols_eval.append(np.power(x_lat, -float(args.beta)))
    xmat_eval = np.column_stack(cols_eval)
    yhat_lat = xmat_eval @ coef
    r2_lat = y_lat - yhat_lat

    cos1_abs = np.abs(np.cos(float(args.tau1) * t_lat + phi1))
    cos2_abs = np.abs(np.cos(float(args.tau2) * t_lat + phi2))

    rows: List[GateRow] = []
    for eps2 in eps2_list:
        gate = (cos1_abs >= float(args.a1)) & (cos2_abs <= float(eps2))
        if np.count_nonzero(gate) < int(args.min_gated_points):
            continue

        xg = x_lat[gate]
        rr = np.abs(r2_lat[gate]) / a1_amp
        yabs = np.abs(y_lat[gate])
        cos1g = cos1_abs[gate]
        cos2g = cos2_abs[gate]

        x_end = float(args.cofinal_xmax_frac) * float(np.max(xg))
        if x_end <= xg[0]:
            x_end = float(np.max(xg))
        grid = _cofinal_grid(float(xg[0]), x_end, int(args.cofinal_grid_count))
        if grid.size == 0:
            continue

        min_rr_future: List[float] = []
        max_obs_future: List[float] = []
        for x0 in grid:
            m = xg >= x0
            if not np.any(m):
                continue
            min_rr_future.append(float(np.min(rr[m])))
            max_obs_future.append(float(np.max(yabs[m])))
        if not min_rr_future:
            continue

        rr_cofinal = float(max(min_rr_future))
        alpha_eps2 = float(alpha * float(eps2))
        q_eff = float(alpha_eps2 + rr_cofinal)
        delta_eff = float(float(args.a1) - q_eff)
        c_eff = float(max(0.0, delta_eff) * a1_amp)
        c_obs = float(min(max_obs_future))

        rows.append(
            GateRow(
                eps2=float(eps2),
                gated_points=int(np.count_nonzero(gate)),
                cos1_min=float(np.min(cos1g)),
                cos2_max=float(np.max(cos2g)),
                rr_cofinal_grid=rr_cofinal,
                alpha_eps2=alpha_eps2,
                q_eff_cofinal=q_eff,
                delta_eff=delta_eff,
                c_beta_eff=c_eff,
                c_beta_obs_cofinal=c_obs,
                crossover_x_eff=_crossover_x(c_eff, float(args.beta), c_endpoint, float(args.xmin)),
            )
        )

    rows.sort(key=lambda r: (-r.delta_eff, r.eps2))

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
            "eps2_list": eps2_list,
            "local_window": int(args.local_window),
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "min_gated_points": int(args.min_gated_points),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
            "endpoint_c_sup_window": c_endpoint,
        },
        "fit": {
            "A1": a1_amp,
            "A2": a2_amp,
            "alpha_A2_over_A1": alpha,
            "phi1": phi1,
            "phi2": phi2,
            "rmse_global_fit": float(fit["rmse"]),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window dual-phase gate evidence only. "
                "Positive delta_eff suggests viable lower-envelope witness on a cofinal gated subsequence."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14/Tau21 Dual-Phase Gate Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- `A1={a1_amp:.9e}`, `A2={a2_amp:.9e}`, `alpha=A2/A1={alpha:.6f}`")
    lines.append(f"- Gate base: `|cos1|>=a1={args.a1}`")
    lines.append("")
    lines.append("| eps2 | gated_n | rr_cofinal | alpha*eps2 | q_eff | delta_eff | c_eff | x_cross(c_eff) |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|")

    def fmt_x(v: float | None) -> str:
        return "NA" if v is None else f"{v:.6e}"

    for r in rows:
        lines.append(
            f"| {r.eps2:.4f} | {r.gated_points} | {r.rr_cofinal_grid:.6f} | {r.alpha_eps2:.6f} | "
            f"{r.q_eff_cofinal:.6f} | {r.delta_eff:.6f} | {r.c_beta_eff:.6e} | {fmt_x(r.crossover_x_eff)} |"
        )

    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
