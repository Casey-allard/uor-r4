#!/usr/bin/env python3
"""Phase-lock stability probe for beta0 from constructive-gate fits.

This targets the math gap between finite-window fitted phases and a theorem-grade
asymptotic phase object by measuring beta0 drift across growing x-windows.
"""

from __future__ import annotations

import argparse
import glob
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List

import numpy as np


@dataclass
class PhaseRow:
    fit_file: str
    x_max: float
    phi1: float
    phi2: float
    beta0: float
    cos_beta0: float
    abs_delta_to_ref: float


def parse_fit_rows(paths: List[str], rho: float) -> List[Dict[str, float | str]]:
    out: List[Dict[str, float | str]] = []
    for p in paths:
        try:
            d = json.loads(Path(p).read_text(encoding="utf-8"))
        except Exception:
            continue
        fit = d.get("fit", {})
        meta = d.get("meta", {})
        if "phi1" not in fit or "phi2" not in fit:
            continue
        x_max = float(meta.get("xmax", 0.0))
        if x_max <= 0.0:
            continue
        phi1 = float(fit["phi1"])
        phi2 = float(fit["phi2"])
        beta0 = float(phi2 - rho * phi1)
        out.append(
            {
                "fit_file": str(p),
                "x_max": x_max,
                "phi1": phi1,
                "phi2": phi2,
                "beta0": beta0,
                "cos_beta0": float(math.cos(beta0)),
            }
        )
    out.sort(key=lambda r: (float(r["x_max"]), str(r["fit_file"])))
    return out


def estimate_eta_from_errors(x: np.ndarray, err: np.ndarray) -> Dict[str, float | None]:
    mask = (x > 0.0) & (err > 1e-10)
    if int(np.sum(mask)) < 2:
        return {"eta_hat": None, "c_hat": None, "r2": None}
    lx = np.log(x[mask])
    ly = np.log(err[mask])
    A = np.column_stack([np.ones_like(lx), lx])
    coef, *_ = np.linalg.lstsq(A, ly, rcond=None)
    a, b = float(coef[0]), float(coef[1])
    # err ~ exp(a) * x^b = C * x^{-eta}
    eta = float(-b)
    c_hat = float(math.exp(a))
    yhat = A @ coef
    ss_res = float(np.sum((ly - yhat) ** 2))
    ss_tot = float(np.sum((ly - np.mean(ly)) ** 2))
    r2 = float(1.0 - ss_res / ss_tot) if ss_tot > 0.0 else 1.0
    return {"eta_hat": eta, "c_hat": c_hat, "r2": r2}


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau12 phase-lock stability probe")
    ap.add_argument(
        "--fit-glob",
        type=str,
        default="research/output/*tau14_tau21_constructive_gate*2026-02-24*.json",
    )
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument(
        "--reference-mode",
        type=str,
        default="largest_window_mean",
        choices=["largest_window_mean", "largest_window_median"],
    )
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_phase_lock_stability_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")

    rho = float(args.tau2 / args.tau1)
    files = sorted(glob.glob(args.fit_glob))
    rows_raw = parse_fit_rows(paths=files, rho=rho)
    if not rows_raw:
        raise ValueError("no valid fit rows found")

    x_vals = sorted(set(float(r["x_max"]) for r in rows_raw))
    x_ref = max(x_vals)
    beta_ref_pool = [float(r["beta0"]) for r in rows_raw if float(r["x_max"]) == x_ref]
    if args.reference_mode == "largest_window_median":
        beta_ref = float(np.median(np.array(beta_ref_pool, dtype=np.float64)))
    else:
        beta_ref = float(np.mean(np.array(beta_ref_pool, dtype=np.float64)))

    rows: List[PhaseRow] = []
    for r in rows_raw:
        beta0 = float(r["beta0"])
        rows.append(
            PhaseRow(
                fit_file=str(r["fit_file"]),
                x_max=float(r["x_max"]),
                phi1=float(r["phi1"]),
                phi2=float(r["phi2"]),
                beta0=beta0,
                cos_beta0=float(r["cos_beta0"]),
                abs_delta_to_ref=abs(beta0 - beta_ref),
            )
        )

    by_x: Dict[float, List[float]] = {}
    for r in rows:
        by_x.setdefault(r.x_max, []).append(r.abs_delta_to_ref)

    x_arr = np.array(sorted(by_x.keys()), dtype=np.float64)
    err_arr = np.array([max(by_x[x]) for x in x_arr], dtype=np.float64)
    err_arr[np.abs(err_arr) < 1e-12] = 0.0
    fit = estimate_eta_from_errors(x=x_arr, err=err_arr)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "fit_glob": args.fit_glob,
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "rho": rho,
            "reference_mode": args.reference_mode,
            "x_reference": x_ref,
            "beta_reference": beta_ref,
        },
        "rows": [asdict(r) for r in rows],
        "window_max_error": [
            {"x_max": float(x), "max_abs_delta_to_ref": float(max(by_x[float(x)]))}
            for x in x_arr
        ],
        "error_fit": fit,
        "interpretation": {
            "note": (
                "Finite-window drift diagnostic for phase-lock behavior. "
                "Supports candidate asymptotic phase object if drift decays."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Phase-Lock Stability Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `fit_glob={args.fit_glob}`")
    lines.append(f"- `rho={rho:.12f}`")
    lines.append(f"- `x_reference={x_ref:.0f}`")
    lines.append(f"- `beta_reference={beta_ref:.12f}`")
    lines.append("")
    lines.append("## Window Max Error to Reference")
    lines.append("| x_max | max |beta0-beta_ref| |")
    lines.append("|---:|---:|")
    for x in x_arr:
        lines.append(f"| {x:.0f} | {max(by_x[float(x)]):.6e} |")
    lines.append("")
    lines.append("## Error Fit (max error by window)")
    lines.append(
        f"- `eta_hat={fit['eta_hat']}` `c_hat={fit['c_hat']}` `r2={fit['r2']}`"
    )
    lines.append("")
    lines.append("Finite evidence only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
