#!/usr/bin/env python3
"""Tied-phase beta0 stability probe.

Goal: estimate beta0 across x-windows with a fit that enforces
  phi2 = rho*phi1 + beta0
instead of fitting phi1 and phi2 independently.

For fixed (phi1, beta0), solve amplitudes linearly:
  y(x) ~ A1*cos(tau1*log x + phi1) + A2*cos(tau2*log x + rho*phi1 + beta0) + d*x^{-beta}

Then search (phi1, beta0) by coarse-to-fine grid and compare drift against
the untied two-mode fit.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List, Sequence, Tuple

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from fixed_error_psi_tau14_tau21_phase_gate_probe import _fit_two_mode


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    vals = sorted(set(v for v in vals if v >= 100))
    if not vals:
        raise ValueError("empty int list")
    return vals


@dataclass
class WindowRow:
    x_max: int
    untied_beta0: float
    untied_rmse: float
    tied_beta0: float
    tied_phi1: float
    tied_phi2: float
    tied_a1_amp: float
    tied_a2_amp: float
    tied_decay: float
    tied_rmse: float
    tied_rmse_ratio_vs_untied: float
    abs_delta_tied_to_ref: float
    abs_delta_untied_to_ref: float


def _solve_linear_amplitudes(y: np.ndarray, b1: np.ndarray, b2: np.ndarray, b3: np.ndarray | None) -> Tuple[np.ndarray, float]:
    if b3 is None:
        xmat = np.column_stack([b1, b2])
    else:
        xmat = np.column_stack([b1, b2, b3])
    coef, *_ = np.linalg.lstsq(xmat, y, rcond=None)
    resid = y - xmat @ coef
    rmse = float(np.sqrt(np.mean(np.square(resid))))
    return coef, rmse


def _grid_values(center: float, half_width: float, n: int) -> np.ndarray:
    return np.linspace(center - half_width, center + half_width, n, dtype=np.float64)


def _fit_tied_phase(
    t: np.ndarray,
    x: np.ndarray,
    y: np.ndarray,
    tau1: float,
    tau2: float,
    rho: float,
    beta: float,
    include_decay_term: bool,
    coarse_n: int,
    refine_n: int,
    refine_rounds: int,
) -> Dict[str, float]:
    decay_col = np.power(x, -beta) if include_decay_term else None

    # Start with full-range coarse scan.
    phi1_center = 0.0
    beta0_center = 0.0
    phi1_half = math.pi
    beta0_half = math.pi

    best: Dict[str, float] = {
        "phi1": 0.0,
        "beta0": 0.0,
        "rmse": float("inf"),
        "A1": 0.0,
        "A2": 0.0,
        "d": 0.0,
    }

    for round_idx in range(refine_rounds + 1):
        n = coarse_n if round_idx == 0 else refine_n
        phi1_grid = _grid_values(phi1_center, phi1_half, n)
        beta0_grid = _grid_values(beta0_center, beta0_half, n)

        for phi1 in phi1_grid:
            b1 = np.cos(tau1 * t + float(phi1))
            for beta0 in beta0_grid:
                phi2 = rho * float(phi1) + float(beta0)
                b2 = np.cos(tau2 * t + phi2)
                coef, rmse = _solve_linear_amplitudes(y=y, b1=b1, b2=b2, b3=decay_col)
                if rmse < float(best["rmse"]):
                    best = {
                        "phi1": float(phi1),
                        "beta0": float(beta0),
                        "rmse": float(rmse),
                        "A1": float(coef[0]),
                        "A2": float(coef[1]),
                        "d": float(coef[2]) if include_decay_term else 0.0,
                    }

        # shrink around current best
        phi1_center = float(best["phi1"])
        beta0_center = float(best["beta0"])
        phi1_half *= 0.4
        beta0_half *= 0.4

    # Normalize beta0 to (-pi, pi]
    beta0 = float(best["beta0"])
    beta0 = float((beta0 + math.pi) % (2.0 * math.pi) - math.pi)
    phi1 = float(best["phi1"])
    phi2 = float(rho * phi1 + beta0)
    return {
        "phi1": phi1,
        "phi2": phi2,
        "beta0": beta0,
        "rmse": float(best["rmse"]),
        "A1": float(best["A1"]),
        "A2": float(best["A2"]),
        "d": float(best["d"]),
    }


def _estimate_eta(x: np.ndarray, e: np.ndarray) -> Dict[str, float | None]:
    mask = (x > 0.0) & (e > 1e-12)
    if int(np.sum(mask)) < 2:
        return {"eta_hat": None, "c_hat": None, "r2": None}
    lx = np.log(x[mask])
    ly = np.log(e[mask])
    a = np.column_stack([np.ones_like(lx), lx])
    coef, *_ = np.linalg.lstsq(a, ly, rcond=None)
    yhat = a @ coef
    ss_res = float(np.sum((ly - yhat) ** 2))
    ss_tot = float(np.sum((ly - np.mean(ly)) ** 2))
    r2 = 1.0 if ss_tot == 0.0 else float(1.0 - ss_res / ss_tot)
    return {
        "eta_hat": float(-coef[1]),
        "c_hat": float(math.exp(coef[0])),
        "r2": r2,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Tied-phase beta0 stability probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax-list", type=str, default="10000000,20000000,30000000,40000000,50000000")
    ap.add_argument("--fit-samples", type=int, default=7000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--coarse-n", type=int, default=61)
    ap.add_argument("--refine-n", type=int, default=25)
    ap.add_argument("--refine-rounds", type=int, default=2)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_phase_lock_tied_fit_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")
    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")

    xmax_list = parse_int_list(args.xmax_list)
    rho = float(args.tau2 / args.tau1)

    rows_tmp: List[Dict[str, float]] = []
    for xmax in xmax_list:
        xs = logspace_int(args.xmin, int(xmax), int(args.fit_samples))
        x = xs.astype(np.float64)
        t = np.log(x)
        events = psi_events(int(xmax), Path(args.cache_dir))
        psi_vals = psi_at_samples(xs, events)
        y = (psi_vals - x) / np.power(x, float(args.beta))

        untied = _fit_two_mode(
            x=x,
            y=y,
            tau1=float(args.tau1),
            tau2=float(args.tau2),
            beta=float(args.beta),
            include_decay_term=bool(args.include_decay_term),
        )
        untied_beta0 = float(untied["phi2"] - rho * float(untied["phi1"]))
        untied_beta0 = float((untied_beta0 + math.pi) % (2.0 * math.pi) - math.pi)

        tied = _fit_tied_phase(
            t=t,
            x=x,
            y=y,
            tau1=float(args.tau1),
            tau2=float(args.tau2),
            rho=rho,
            beta=float(args.beta),
            include_decay_term=bool(args.include_decay_term),
            coarse_n=int(args.coarse_n),
            refine_n=int(args.refine_n),
            refine_rounds=int(args.refine_rounds),
        )

        rows_tmp.append(
            {
                "x_max": float(xmax),
                "untied_beta0": untied_beta0,
                "untied_rmse": float(untied["rmse"]),
                "tied_beta0": float(tied["beta0"]),
                "tied_phi1": float(tied["phi1"]),
                "tied_phi2": float(tied["phi2"]),
                "tied_a1_amp": float(tied["A1"]),
                "tied_a2_amp": float(tied["A2"]),
                "tied_decay": float(tied["d"]),
                "tied_rmse": float(tied["rmse"]),
                "tied_rmse_ratio_vs_untied": float(tied["rmse"] / float(untied["rmse"])) if float(untied["rmse"]) > 0.0 else 1.0,
            }
        )

    x_ref = max(r["x_max"] for r in rows_tmp)
    ref_rows = [r for r in rows_tmp if r["x_max"] == x_ref]
    tied_ref = float(np.mean([r["tied_beta0"] for r in ref_rows]))
    untied_ref = float(np.mean([r["untied_beta0"] for r in ref_rows]))

    rows: List[WindowRow] = []
    for r in rows_tmp:
        rows.append(
            WindowRow(
                x_max=int(r["x_max"]),
                untied_beta0=float(r["untied_beta0"]),
                untied_rmse=float(r["untied_rmse"]),
                tied_beta0=float(r["tied_beta0"]),
                tied_phi1=float(r["tied_phi1"]),
                tied_phi2=float(r["tied_phi2"]),
                tied_a1_amp=float(r["tied_a1_amp"]),
                tied_a2_amp=float(r["tied_a2_amp"]),
                tied_decay=float(r["tied_decay"]),
                tied_rmse=float(r["tied_rmse"]),
                tied_rmse_ratio_vs_untied=float(r["tied_rmse_ratio_vs_untied"]),
                abs_delta_tied_to_ref=abs(float(r["tied_beta0"]) - tied_ref),
                abs_delta_untied_to_ref=abs(float(r["untied_beta0"]) - untied_ref),
            )
        )
    rows.sort(key=lambda z: z.x_max)

    x_arr = np.array([float(r.x_max) for r in rows], dtype=np.float64)
    tied_err = np.array([float(r.abs_delta_tied_to_ref) for r in rows], dtype=np.float64)
    untied_err = np.array([float(r.abs_delta_untied_to_ref) for r in rows], dtype=np.float64)
    tied_fit = _estimate_eta(x=x_arr, e=tied_err)
    untied_fit = _estimate_eta(x=x_arr, e=untied_err)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax_list": xmax_list,
            "fit_samples": int(args.fit_samples),
            "beta": float(args.beta),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "rho": rho,
            "coarse_n": int(args.coarse_n),
            "refine_n": int(args.refine_n),
            "refine_rounds": int(args.refine_rounds),
            "include_decay_term": bool(args.include_decay_term),
        },
        "reference": {
            "x_ref": float(x_ref),
            "tied_beta0_ref": tied_ref,
            "untied_beta0_ref": untied_ref,
        },
        "rows": [asdict(r) for r in rows],
        "drift_fit": {
            "tied": tied_fit,
            "untied": untied_fit,
        },
        "interpretation": {
            "note": (
                "Tied-phase estimator enforces phi2=rho*phi1+beta0 to probe phase-lock stability. "
                "Finite evidence only; not a theorem proof."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Phase-Lock Tied-Fit Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `rho={rho:.12f}`")
    lines.append(f"- reference `x_ref={int(x_ref)}`")
    lines.append(f"- tied `beta0_ref={tied_ref:.12f}`")
    lines.append(f"- untied `beta0_ref={untied_ref:.12f}`")
    lines.append("")
    lines.append("| x_max | tied_beta0 | untied_beta0 | |tied-ref| | |untied-ref| | tied_rmse/untied_rmse |")
    lines.append("|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.x_max} | {r.tied_beta0:.12f} | {r.untied_beta0:.12f} | "
            f"{r.abs_delta_tied_to_ref:.6e} | {r.abs_delta_untied_to_ref:.6e} | "
            f"{r.tied_rmse_ratio_vs_untied:.6f} |"
        )
    lines.append("")
    lines.append("## Drift Fit |beta0(x)-beta0(ref)| ~ C*x^{-eta}")
    lines.append(f"- tied: `eta_hat={tied_fit['eta_hat']}` `r2={tied_fit['r2']}`")
    lines.append(f"- untied: `eta_hat={untied_fit['eta_hat']}` `r2={untied_fit['r2']}`")
    lines.append("")
    lines.append("Finite evidence only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
