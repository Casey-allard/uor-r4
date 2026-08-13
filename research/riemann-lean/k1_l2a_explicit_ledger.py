#!/usr/bin/env python3
"""Explicit-only L2A constant ledger (no empirical phase fitting).

Builds a band-bound from an explicit zero-count upper function and combines
it with envelope/remainder models at a target eta.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict

import numpy as np


def n_upper_trudgian_like(t: np.ndarray) -> np.ndarray:
    """Explicit upper envelope for N(T), T >= e.

    Uses the classical main term plus explicit error constants from
    Trudgian-style bounds:
    |N(T) - (T/(2π) log(T/(2πe)) - 7/8)| <= 0.112 log T + 0.278 log log T + 2.510 + 0.2/T.
    So:
    N(T) <= main + 7/8 + error.
    """

    tt = np.maximum(t, math.e)
    main = (tt / (2.0 * math.pi)) * np.log(tt / (2.0 * math.pi * math.e))
    err = 0.112 * np.log(tt) + 0.278 * np.log(np.log(tt)) + 2.510 + 0.2 / tt
    return np.maximum(0.0, main + 0.875 + err)


def build_s1_lookup(gamma_n: float, t_max: float, n_grid: int) -> Dict[str, np.ndarray]:
    """Build lookup for S1(b) <= U(b)/b + ∫_{gamma_n}^b U(t)/t^2 dt.

    Integration in u = log t:
    ∫ U(t)/t^2 dt = ∫ U(e^u)/e^u du.
    """

    u0 = math.log(gamma_n)
    u1 = math.log(t_max)
    u = np.linspace(u0, u1, n_grid, dtype=np.float64)
    t = np.exp(u)
    uval = n_upper_trudgian_like(t)
    f = uval / t  # integrand in du

    du = u[1:] - u[:-1]
    trap = 0.5 * (f[1:] + f[:-1]) * du
    integ = np.zeros_like(u)
    integ[1:] = np.cumsum(trap)

    return {"u": u, "t": t, "uval": uval, "integ": integ}


def s1_upper_from_lookup(b: np.ndarray, gamma_n: float, lut: Dict[str, np.ndarray]) -> np.ndarray:
    out = np.zeros_like(b, dtype=np.float64)
    mask = b > gamma_n
    if not np.any(mask):
        return out

    bb = b[mask]
    u_b = np.log(bb)
    u_grid = lut["u"]
    integ_grid = lut["integ"]
    ub = n_upper_trudgian_like(bb)
    int_part = np.interp(u_b, u_grid, integ_grid)
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


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 L2A explicit-only constant ledger")
    ap.add_argument("--gamma-n", type=float, default=478.942181535)
    ap.add_argument("--n-head", type=int, default=256)
    ap.add_argument("--beta", type=float, default=0.55)
    ap.add_argument("--eta", type=float, default=0.01)
    ap.add_argument("--theta", type=float, default=0.13)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--xmax", type=float, default=1.0e120)
    ap.add_argument("--x-grid", type=int, default=12000)
    ap.add_argument("--t-grid", type=int, default=160000)
    ap.add_argument("--a0", type=float, default=(1.0 / 48.0718))
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w32_l2a_explicit_ledger_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    x = np.exp(np.linspace(math.log(args.x1), math.log(args.xmax), args.x_grid, dtype=np.float64))
    t_cut = np.power(x, args.theta)
    t_max = float(np.max(t_cut))

    lut = build_s1_lookup(gamma_n=args.gamma_n, t_max=t_max, n_grid=args.t_grid)
    s1 = s1_upper_from_lookup(b=t_cut, gamma_n=args.gamma_n, lut=lut)

    # Per-mode amplitude bound: sqrt(1+4g^2)/(1/4+g^2) <= 2/g for g>0.
    # Hence omitted finite-band term <= 2 * S1.
    y_band = np.power(x, 0.5 - args.beta) * (2.0 * s1)
    c_band = float(np.max(y_band * np.power(x, args.eta)))

    # External high-envelope candidates converted to eta target.
    fks = fks_bound(x)
    bel = bellotti_2025_model(x, args.a0)
    c_fks = float(np.max(fks * np.power(x, args.eta)))
    c_bel = float(np.max(bel * np.power(x, args.eta)))
    c_high = min(c_fks, c_bel)

    # Two remainder normalizations to make assumptions explicit.
    # A: R ~ (log x)^2 / x^theta
    rem_a = (np.log(x) ** 2) * np.power(x, -args.theta)
    c_rem_a = float(np.max(rem_a * np.power(x, args.eta)))
    # B: R ~ x^(1/2-beta) (log x)^2 / x^theta
    rem_b = np.power(x, 0.5 - args.beta) * (np.log(x) ** 2) * np.power(x, -args.theta)
    c_rem_b = float(np.max(rem_b * np.power(x, args.eta)))

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "gamma_n": args.gamma_n,
            "n_head": args.n_head,
            "beta": args.beta,
            "eta": args.eta,
            "theta": args.theta,
            "x1": args.x1,
            "xmax": args.xmax,
            "x_grid": args.x_grid,
            "t_grid": args.t_grid,
            "a0": args.a0,
        },
        "explicit_band_bound": {
            "definition": "y_band(x)=x^(1/2-beta) * 2 * S1(gamma_n, x^theta)",
            "S1_upper": "U(b)/b + integral_{gamma_n}^b U(t)/t^2 dt",
            "U_model": "Trudgian-like explicit N(T) upper function",
            "c_band_eta": c_band,
        },
        "external_high_envelopes_eta": {
            "c_fks": c_fks,
            "c_bellotti_2025_model": c_bel,
            "c_selected_min": c_high,
        },
        "remainder_candidates_eta": {
            "c_rem_model_A_log2_over_xtheta": c_rem_a,
            "c_rem_model_B_xhalfminusbeta_log2_over_xtheta": c_rem_b,
        },
        "combined_candidates": {
            "c_total_A": c_band + c_high + c_rem_a,
            "c_total_B": c_band + c_high + c_rem_b,
        },
        "interpretation": {
            "note": (
                "This is an explicit-only constant ledger: no empirical band/high decomposition. "
                "Remaining theorem work is to justify the chosen remainder normalization and "
                "replace model-level high envelope usage with the exact target theorem chain."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines = []
    lines.append(f"# K1 L2A Explicit Ledger ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Explicit Band Bound")
    lines.append(f"- c_band_eta: {c_band:.12g}")
    lines.append("")
    lines.append("## External High Envelopes")
    lines.append(f"- c_fks: {c_fks:.12g}")
    lines.append(f"- c_bellotti_2025_model: {c_bel:.12g}")
    lines.append(f"- c_selected_min: {c_high:.12g}")
    lines.append("")
    lines.append("## Remainder Candidates")
    lines.append(f"- c_rem_model_A_log2_over_xtheta: {c_rem_a:.12g}")
    lines.append(f"- c_rem_model_B_xhalfminusbeta_log2_over_xtheta: {c_rem_b:.12g}")
    lines.append("")
    lines.append("## Combined Candidates")
    lines.append(f"- c_total_A: {result['combined_candidates']['c_total_A']:.12g}")
    lines.append(f"- c_total_B: {result['combined_candidates']['c_total_B']:.12g}")
    lines.append("")
    lines.append("## Interpretation")
    lines.append(f"- {result['interpretation']['note']}")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

