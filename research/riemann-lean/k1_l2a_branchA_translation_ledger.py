#!/usr/bin/env python3
"""Branch-A -> Branch-B translation ledger for L2A.

Branch A remainder input:
  |Rem_A(x,T)| <= C_A * x * (log x)^2 / T

Normalized by x^beta (beta >= beta_lower):
  |Rem_A(x,T)| / x^beta <= C_A * (log x)^2 * x^(1-beta_lower) / T

With schedule T(x)=x^theta:
  <= C_A * (log x)^2 * x^{-(theta + beta_lower - 1)}.

This script computes explicit eta-target constants and scans admissible theta.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List

import numpy as np


def n_upper_trudgian_like(t: np.ndarray) -> np.ndarray:
    tt = np.maximum(t, math.e)
    main = (tt / (2.0 * math.pi)) * np.log(tt / (2.0 * math.pi * math.e))
    err = 0.112 * np.log(tt) + 0.278 * np.log(np.log(tt)) + 2.510 + 0.2 / tt
    return np.maximum(0.0, main + 0.875 + err)


def build_s1_lookup(gamma_n: float, t_max: float, n_grid: int) -> Dict[str, np.ndarray]:
    u0 = math.log(gamma_n)
    u1 = math.log(t_max)
    u = np.linspace(u0, u1, n_grid, dtype=np.float64)
    t = np.exp(u)
    uval = n_upper_trudgian_like(t)
    f = uval / t
    du = u[1:] - u[:-1]
    trap = 0.5 * (f[1:] + f[:-1]) * du
    integ = np.zeros_like(u)
    integ[1:] = np.cumsum(trap)
    return {"u": u, "integ": integ}


def s1_upper(gamma_n: float, b: np.ndarray, lut: Dict[str, np.ndarray]) -> np.ndarray:
    out = np.zeros_like(b, dtype=np.float64)
    mask = b > gamma_n
    if not np.any(mask):
        return out
    bb = b[mask]
    u_b = np.log(bb)
    int_part = np.interp(u_b, lut["u"], lut["integ"])
    ub = n_upper_trudgian_like(bb)
    out[mask] = (ub / bb) + int_part
    return out


def omega_vk(x: np.ndarray, a0: float) -> np.ndarray:
    lx = np.log(np.maximum(x, 1.0 + 1.0e-300))
    llx = np.log(np.maximum(lx, 1.0 + 1.0e-300))
    c = ((5.0**6 * a0**3) / (2.0**2 * 3.0**4)) ** (1.0 / 5.0)
    return c * np.power(lx, 3.0 / 5.0) / np.power(llx, 1.0 / 5.0)


def fks_bound(x: np.ndarray) -> np.ndarray:
    lx = np.log(np.maximum(x, 1.0 + 1.0e-300))
    return 9.22022 * np.power(lx, 1.5) * np.exp(-0.8476836 * np.sqrt(lx))


def bellotti_2025_model(x: np.ndarray, a0: float) -> np.ndarray:
    return math.exp(55.0 * a0) * np.exp(-omega_vk(x, a0))


def log_absorb_sup(x1: float, delta: float) -> Dict[str, float]:
    """Sup of f(x)=(log x)^2 x^{-delta} on [x1, inf), delta>0."""
    if delta <= 0:
        return {
            "finite": False,
            "value": float("inf"),
            "x_star": float("nan"),
            "threshold": float("nan"),
        }
    u1 = math.log(x1)
    u_star = 2.0 / delta
    x_star = math.exp(u_star) if u_star < 700.0 else float("inf")
    if u1 >= u_star:
        # decreasing from x1 onward
        val = (u1 * u1) * math.exp(-delta * u1)
        return {"finite": True, "value": val, "x_star": x1, "threshold": x_star}
    val = (4.0 / (delta * delta)) * math.exp(-2.0)
    return {"finite": True, "value": val, "x_star": x_star, "threshold": x_star}


def eval_theta(
    theta: float,
    x: np.ndarray,
    gamma_n: float,
    beta_lower: float,
    eta_target: float,
    c_a: float,
    c_high: float,
    lut: Dict[str, np.ndarray],
) -> Dict[str, float | bool]:
    b = np.power(x, theta)
    s1 = s1_upper(gamma_n=gamma_n, b=b, lut=lut)
    y_band = np.power(x, 0.5 - beta_lower) * (2.0 * s1)
    c_band = float(np.max(y_band * np.power(x, eta_target)))

    eta_raw = theta + beta_lower - 1.0
    feasible = eta_raw > eta_target

    if feasible:
        delta = eta_raw - eta_target
        h = log_absorb_sup(x1=float(x[0]), delta=delta)
        c_rem_factor = float(h["value"])
        c_rem = c_a * c_rem_factor
        c_total = c_band + c_high + c_rem
        x_star = float(h["x_star"])
    else:
        delta = eta_raw - eta_target
        c_rem_factor = float("inf")
        c_rem = float("inf")
        c_total = float("inf")
        x_star = float("nan")

    return {
        "theta": theta,
        "eta_raw": eta_raw,
        "eta_target": eta_target,
        "feasible_eta_target": feasible,
        "delta_log_absorb": delta,
        "c_band": c_band,
        "c_high_selected": c_high,
        "c_rem_factor_per_CA": c_rem_factor,
        "c_rem": c_rem,
        "c_total": c_total,
        "x_star_log_absorb": x_star,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 L2A Branch-A translation ledger")
    ap.add_argument("--gamma-n", type=float, default=478.942181535)
    ap.add_argument("--beta-lower", type=float, default=0.55)
    ap.add_argument("--eta-target", type=float, default=0.01)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--xmax", type=float, default=1.0e120)
    ap.add_argument("--x-grid", type=int, default=8000)
    ap.add_argument("--theta-start", type=float, default=0.13)
    ap.add_argument("--theta-end", type=float, default=0.70)
    ap.add_argument("--theta-step", type=float, default=0.01)
    ap.add_argument("--t-grid", type=int, default=160000)
    ap.add_argument("--c-a", type=float, default=1.0, help="Constant C_A from Branch-A remainder")
    ap.add_argument("--a0", type=float, default=(1.0 / 48.0718))
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w34_branchA_to_B_translation_ledger_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    x = np.exp(np.linspace(math.log(args.x1), math.log(args.xmax), args.x_grid, dtype=np.float64))
    thetas: List[float] = []
    t = args.theta_start
    while t <= args.theta_end + 1.0e-12:
        thetas.append(round(t, 10))
        t += args.theta_step

    t_max = float(np.max(np.power(x, max(thetas))))
    lut = build_s1_lookup(gamma_n=args.gamma_n, t_max=t_max, n_grid=args.t_grid)

    c_fks = float(np.max(fks_bound(x) * np.power(x, args.eta_target)))
    c_bel = float(np.max(bellotti_2025_model(x, args.a0) * np.power(x, args.eta_target)))
    c_high = min(c_fks, c_bel)

    rows = [
        eval_theta(
            theta=th,
            x=x,
            gamma_n=args.gamma_n,
            beta_lower=args.beta_lower,
            eta_target=args.eta_target,
            c_a=args.c_a,
            c_high=c_high,
            lut=lut,
        )
        for th in thetas
    ]

    feasible_rows = [r for r in rows if bool(r["feasible_eta_target"])]
    best = min(feasible_rows, key=lambda r: float(r["c_total"])) if feasible_rows else None
    theta_min_required = 1.0 - args.beta_lower + args.eta_target

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "gamma_n": args.gamma_n,
            "beta_lower": args.beta_lower,
            "eta_target": args.eta_target,
            "x1": args.x1,
            "xmax": args.xmax,
            "x_grid": args.x_grid,
            "theta_start": args.theta_start,
            "theta_end": args.theta_end,
            "theta_step": args.theta_step,
            "t_grid": args.t_grid,
            "c_a": args.c_a,
            "a0": args.a0,
        },
        "derived_thresholds": {
            "theta_min_for_eta_target_under_branch_A": theta_min_required,
        },
        "external_high_constants_eta": {
            "c_fks": c_fks,
            "c_bellotti_2025_model": c_bel,
            "c_selected": c_high,
        },
        "rows": rows,
        "best_feasible_row": best,
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines = []
    lines.append(f"# K1 Branch-A -> Branch-B Translation Ledger ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Threshold")
    lines.append(
        "- theta_min_for_eta_target_under_branch_A: "
        f"{result['derived_thresholds']['theta_min_for_eta_target_under_branch_A']}"
    )
    lines.append("")
    lines.append("## External High Constants")
    e = result["external_high_constants_eta"]
    lines.append(f"- c_fks: {e['c_fks']:.12g}")
    lines.append(f"- c_bellotti_2025_model: {e['c_bellotti_2025_model']:.12g}")
    lines.append(f"- c_selected: {e['c_selected']:.12g}")
    lines.append("")
    lines.append("| theta | feasible | eta_raw | c_band | c_rem_factor/CA | c_total (CA applied) |")
    lines.append("|---:|:---:|---:|---:|---:|---:|")
    for r in rows:
        feas = "yes" if r["feasible_eta_target"] else "no"
        ctot = r["c_total"]
        if math.isfinite(float(ctot)):
            ctot_s = f"{float(ctot):.6g}"
        else:
            ctot_s = "inf"
        cremf = r["c_rem_factor_per_CA"]
        if math.isfinite(float(cremf)):
            cremf_s = f"{float(cremf):.6g}"
        else:
            cremf_s = "inf"
        lines.append(
            f"| {float(r['theta']):.2f} | {feas} | {float(r['eta_raw']):.4f} | "
            f"{float(r['c_band']):.6g} | {cremf_s} | {ctot_s} |"
        )
    lines.append("")
    if best is not None:
        lines.append("## Best Feasible")
        lines.append(f"- theta: {best['theta']}")
        lines.append(f"- eta_raw: {best['eta_raw']}")
        lines.append(f"- c_band: {best['c_band']}")
        lines.append(f"- c_rem_factor_per_CA: {best['c_rem_factor_per_CA']}")
        lines.append(f"- c_total: {best['c_total']}")
        lines.append("")
    lines.append("## Interpretation")
    lines.append(
        "- This ledger enforces the Branch-A normalization gate; rows below theta_min are marked infeasible."
    )
    lines.append(
        "- To change absolute totals, replace `c_a` with theorem-grade remainder constants from imported sources."
    )
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
