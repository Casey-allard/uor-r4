#!/usr/bin/env python3
"""Inverse feasibility map for K1/L2A Branch-A -> Branch-B closure.

This script solves the inverse question:
for a fixed target total Branch-B constant `C_target`, how large can the
Branch-A remainder constant `C_A` be while still satisfying

  C_total = C_band(beta_lower, theta) + C_high + C_A * M(x1, delta) <= C_target

where:
  delta = (theta + beta_lower - 1) - eta_target > 0
  M(x1, delta) = sup_{x >= x1} (log x)^2 x^{-delta}.

Outputs are numeric constraints on the remaining math obligation
(`C_A`, `beta_lower`, `theta`), with no Lean/formal pipeline changes.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List

import numpy as np


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        vals.append(float(tok))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("expected at least one numeric value")
    return vals


def parse_range(lo: float, hi: float, step: float) -> List[float]:
    vals: List[float] = []
    t = lo
    while t <= hi + 1.0e-12:
        vals.append(round(t, 10))
        t += step
    return vals


def n_upper_trudgian_like(t: np.ndarray) -> np.ndarray:
    """Explicit upper envelope for N(T), T >= e."""
    tt = np.maximum(t, math.e)
    main = (tt / (2.0 * math.pi)) * np.log(tt / (2.0 * math.pi * math.e))
    err = 0.112 * np.log(tt) + 0.278 * np.log(np.log(tt)) + 2.510 + 0.2 / tt
    return np.maximum(0.0, main + 0.875 + err)


def build_s1_lookup(gamma_n: float, t_max: float, n_grid: int) -> Dict[str, np.ndarray]:
    """Lookup for S1(b) <= U(b)/b + integral_{gamma_n}^b U(t)/t^2 dt."""
    u0 = math.log(gamma_n)
    u1 = math.log(t_max)
    u = np.linspace(u0, u1, n_grid, dtype=np.float64)
    t = np.exp(u)
    uval = n_upper_trudgian_like(t)
    integrand = uval / t
    du = u[1:] - u[:-1]
    trap = 0.5 * (integrand[1:] + integrand[:-1]) * du
    integ = np.zeros_like(u)
    integ[1:] = np.cumsum(trap)
    return {"u": u, "integ": integ}


def s1_upper(gamma_n: float, b: np.ndarray, lut: Dict[str, np.ndarray]) -> np.ndarray:
    out = np.zeros_like(b, dtype=np.float64)
    mask = b > gamma_n
    if not np.any(mask):
        return out
    bb = b[mask]
    ub = n_upper_trudgian_like(bb)
    int_part = np.interp(np.log(bb), lut["u"], lut["integ"])
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


def log_absorb_sup(x1: float, delta: float) -> float:
    """Sup of (log x)^2 x^{-delta} on [x1, inf), delta>0."""
    if delta <= 0.0:
        return float("inf")
    u1 = math.log(x1)
    u_star = 2.0 / delta
    if u1 >= u_star:
        return (u1 * u1) * math.exp(-delta * u1)
    return (4.0 / (delta * delta)) * math.exp(-2.0)


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 L2A inverse feasibility map")
    ap.add_argument("--gamma-n", type=float, default=478.942181535)
    ap.add_argument("--eta-target", type=float, default=0.01)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--xmax", type=float, default=1.0e120)
    ap.add_argument("--x-grid", type=int, default=6000)
    ap.add_argument("--beta-min", type=float, default=0.51)
    ap.add_argument("--beta-max", type=float, default=0.75)
    ap.add_argument("--beta-step", type=float, default=0.01)
    ap.add_argument("--theta-min", type=float, default=0.13)
    ap.add_argument("--theta-max", type=float, default=0.75)
    ap.add_argument("--theta-step", type=float, default=0.01)
    ap.add_argument("--t-grid", type=int, default=160000)
    ap.add_argument("--a0", type=float, default=(1.0 / 48.0718))
    ap.add_argument(
        "--c-targets",
        type=str,
        default="20,30,50,100",
        help="Comma-separated C_total targets for inverse C_A constraints.",
    )
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w35_l2a_inverse_feasibility_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    c_targets = parse_float_list(args.c_targets)
    betas = parse_range(args.beta_min, args.beta_max, args.beta_step)
    thetas = parse_range(args.theta_min, args.theta_max, args.theta_step)

    x = np.exp(np.linspace(math.log(args.x1), math.log(args.xmax), args.x_grid, dtype=np.float64))
    x_eta = np.power(x, args.eta_target)
    t_max = float(np.max(np.power(x, max(thetas))))
    lut = build_s1_lookup(gamma_n=args.gamma_n, t_max=t_max, n_grid=args.t_grid)

    # External high-constant selection at eta-target level.
    c_fks = float(np.max(fks_bound(x) * x_eta))
    c_bellotti = float(np.max(bellotti_2025_model(x, args.a0) * x_eta))
    c_high = min(c_fks, c_bellotti)

    # Precompute theta-dependent base term for C_band:
    # c_band(beta, theta) = max_x [2*S1(x^theta) * x^(eta + 0.5 - beta)].
    theta_base: Dict[float, np.ndarray] = {}
    for th in thetas:
        b = np.power(x, th)
        s1 = s1_upper(gamma_n=args.gamma_n, b=b, lut=lut)
        theta_base[th] = 2.0 * s1 * x_eta

    rows: List[Dict[str, float | bool]] = []
    for beta in betas:
        x_beta_factor = np.power(x, 0.5 - beta)
        for th in thetas:
            eta_raw = th + beta - 1.0
            feasible = bool(eta_raw > args.eta_target)
            if feasible:
                delta = eta_raw - args.eta_target
                m = log_absorb_sup(args.x1, delta)
            else:
                delta = eta_raw - args.eta_target
                m = float("inf")
            c_band = float(np.max(theta_base[th] * x_beta_factor))
            c_floor = c_band + c_high

            row: Dict[str, float | bool] = {
                "beta_lower": float(beta),
                "theta": float(th),
                "eta_raw": float(eta_raw),
                "feasible_eta": feasible,
                "delta": float(delta),
                "log_absorb_sup": float(m),
                "c_band": c_band,
                "c_high_selected": c_high,
                "c_floor_band_plus_high": c_floor,
            }
            for ct in c_targets:
                key = f"ca_max_for_c_target_{ct:g}"
                if feasible and math.isfinite(m) and m > 0.0 and c_floor < ct:
                    row[key] = float((ct - c_floor) / m)
                else:
                    row[key] = float("-inf")
            rows.append(row)

    best_by_beta_and_target: Dict[str, Dict[str, float | bool]] = {}
    for beta in betas:
        beta_rows = [r for r in rows if abs(float(r["beta_lower"]) - beta) < 1.0e-12]
        for ct in c_targets:
            key = f"ca_max_for_c_target_{ct:g}"
            best = max(beta_rows, key=lambda r: float(r[key]))
            best_by_beta_and_target[f"beta_{beta:.2f}_ct_{ct:g}"] = best

    # Track the easiest global point for each C_target (largest allowable C_A).
    best_global: Dict[str, Dict[str, float | bool]] = {}
    for ct in c_targets:
        key = f"ca_max_for_c_target_{ct:g}"
        best_global[f"ct_{ct:g}"] = max(rows, key=lambda r: float(r[key]))

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "gamma_n": args.gamma_n,
            "eta_target": args.eta_target,
            "x1": args.x1,
            "xmax": args.xmax,
            "x_grid": args.x_grid,
            "beta_min": args.beta_min,
            "beta_max": args.beta_max,
            "beta_step": args.beta_step,
            "theta_min": args.theta_min,
            "theta_max": args.theta_max,
            "theta_step": args.theta_step,
            "t_grid": args.t_grid,
            "a0": args.a0,
            "c_targets": c_targets,
        },
        "external_high_constants_eta": {
            "c_fks": c_fks,
            "c_bellotti_2025_model": c_bellotti,
            "c_selected": c_high,
        },
        "rows": rows,
        "best_by_beta_and_target": best_by_beta_and_target,
        "best_global_by_target": best_global,
        "interpretation": {
            "note": (
                "For each (beta_lower, theta), `ca_max_for_c_target_*` is the maximum Branch-A "
                "constant compatible with that target total constant under the explicit band/high models. "
                "Negative infinity means infeasible."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 L2A Inverse Feasibility Map ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## External High Constants (eta target)")
    lines.append(f"- c_fks: {c_fks:.12g}")
    lines.append(f"- c_bellotti_2025_model: {c_bellotti:.12g}")
    lines.append(f"- c_selected: {c_high:.12g}")
    lines.append("")
    lines.append("## Global Best (largest admissible C_A) by C_target")
    for ct in c_targets:
        key = f"ct_{ct:g}"
        row = best_global[key]
        ca_key = f"ca_max_for_c_target_{ct:g}"
        lines.append(
            f"- C_target={ct:g}: beta_lower={float(row['beta_lower']):.2f}, "
            f"theta={float(row['theta']):.2f}, eta_raw={float(row['eta_raw']):.4f}, "
            f"c_floor={float(row['c_floor_band_plus_high']):.6g}, "
            f"max_CA={float(row[ca_key]):.6g}"
        )
    lines.append("")
    lines.append("## Per-beta Best (largest admissible C_A)")
    lines.append("| beta_lower | " + " | ".join([f"C_target={ct:g}" for ct in c_targets]) + " |")
    lines.append("|---:|" + "|".join([":---:" for _ in c_targets]) + "|")
    for beta in betas:
        cells: List[str] = []
        for ct in c_targets:
            row = best_by_beta_and_target[f"beta_{beta:.2f}_ct_{ct:g}"]
            ca_key = f"ca_max_for_c_target_{ct:g}"
            ca_val = float(row[ca_key])
            if math.isfinite(ca_val):
                cell = (
                    f"theta={float(row['theta']):.2f}, "
                    f"eta={float(row['eta_raw']):.3f}, "
                    f"CA_max={ca_val:.4g}"
                )
            else:
                cell = "infeasible"
            cells.append(cell)
        lines.append(f"| {beta:.2f} | " + " | ".join(cells) + " |")
    lines.append("")
    lines.append("## Interpretation")
    lines.append(
        "- This map isolates the remaining theorem obligation numerically: supply a Branch-A constant `C_A` "
        "and a justified `beta_lower` regime that land above one of these feasibility thresholds."
    )
    lines.append(
        "- It does not prove RH; it quantifies the exact constant envelope still needed for the final math bridge."
    )
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
