#!/usr/bin/env python3
"""Derive theorem-grade one-sided tail constants for the K1 q<a1 gate.

This script intentionally uses only explicit, source-backed ingredients:

1) Finite-band omitted-mode envelope:
     |Band(x)| <= x^(1/2-beta) * 2*S1(gamma_N, x^theta)
   with S1 upper-bounded from an explicit N(T) majorant.

2) Truncated explicit-formula remainder (Perron-style):
     |Rem_A(x,T)| <= C_A * x*(log x)^2 / T
   and T(x)=x^theta, so after normalization by x^beta:
     |Rem_A(x,T(x))| / x^beta <= C_A * (log x)^2 * x^(-(theta+beta-1)).

For target eta>0, if delta := theta + beta - 1 - eta > 0, then
  (log x)^2 * x^(-(theta+beta-1))
  = ((log x)^2 * x^(-delta)) * x^(-eta)
  <= M(x1,delta) * x^(-eta), x>=x1.

Hence:
  |R2(x)/x^beta| <= C_total(theta) * x^(-eta),  x>=x1
with
  C_total(theta) = C_band(theta) + C_A*M(x1,delta).

Notes:
- This script does not use the model-level high-tail envelope from prior ledgers.
- It reports explicit constants and resulting q-thresholds for q<a1.
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
        raise ValueError("expected at least one float")
    return vals


def n_upper_trudgian_like(t: np.ndarray) -> np.ndarray:
    """Explicit upper envelope for N(T), T >= e."""
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
    integrand = uval / t
    du = u[1:] - u[:-1]
    trap = 0.5 * (integrand[1:] + integrand[:-1]) * du
    integ = np.zeros_like(u)
    integ[1:] = np.cumsum(trap)
    return {"u": u, "integ": integ}


def s1_upper_from_lookup(b: np.ndarray, gamma_n: float, lut: Dict[str, np.ndarray]) -> np.ndarray:
    out = np.zeros_like(b, dtype=np.float64)
    mask = b > gamma_n
    if not np.any(mask):
        return out
    bb = b[mask]
    ub = n_upper_trudgian_like(bb)
    int_part = np.interp(np.log(bb), lut["u"], lut["integ"])
    out[mask] = (ub / bb) + int_part
    return out


def log_absorb_sup(x1: float, delta: float) -> Dict[str, float]:
    """Sup of (log x)^2 x^{-delta} on [x1, inf), delta>0."""
    if delta <= 0.0:
        return {"finite": False, "value": float("inf"), "x_star": float("nan")}
    u1 = math.log(x1)
    u_star = 2.0 / delta
    x_star = math.exp(u_star) if u_star < 700.0 else float("inf")
    if u1 >= u_star:
        val = (u1 * u1) * math.exp(-delta * u1)
        return {"finite": True, "value": val, "x_star": x1}
    val = (4.0 / (delta * delta)) * math.exp(-2.0)
    return {"finite": True, "value": val, "x_star": x_star}


def threshold_x(c_total: float, eta: float, q_target: float, x1: float) -> float:
    if c_total <= 0.0:
        return x1
    if eta <= 0.0 or q_target <= 0.0:
        return float("inf")
    if c_total <= q_target:
        return x1
    log_x = math.log(c_total / q_target) / eta
    if log_x >= 700.0:
        return float("inf")
    return max(x1, float(math.exp(log_x)))


def perron_window_checks(
    x1: float, theta: float, source: str
) -> Dict[str, float | bool | str]:
    """Check explicit-formula truncation side conditions at x1."""
    t1 = x1**theta
    out: Dict[str, float | bool | str] = {
        "source": source,
        "t_at_x1": float(t1),
        "theta_lt_half": bool(theta < 0.5),
    }
    if source == "dudek_prop":
        lower = 3.0 * math.log(x1)
        upper = math.sqrt(x1) / 3.0
        out.update(
            {
                "x1_gt_exp50": bool(x1 > math.exp(50.0)),
                "lower_bound_at_x1": float(lower),
                "upper_bound_at_x1": float(upper),
                "cond_t_gt_lower_at_x1": bool(t1 > lower),
                "cond_t_lt_upper_at_x1": bool(t1 < upper),
            }
        )
        return out
    if source == "cully_hugill_johnston":
        # Using the explicit admissible tuple cited in FKS 2022/2023:
        # log(x_M)=40, alpha=1/2, omega=0, M=2.091.
        lower = max(51.0, math.log(x1))
        upper = (math.sqrt(x1) - 2.0) / 5.0
        out.update(
            {
                "x1_ge_exp40": bool(x1 >= math.exp(40.0)),
                "lower_bound_at_x1": float(lower),
                "upper_bound_at_x1": float(upper),
                "cond_t_gt_lower_at_x1": bool(t1 > lower),
                "cond_t_lt_upper_at_x1": bool(t1 < upper),
            }
        )
        return out
    raise ValueError(f"unsupported perron source: {source}")


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 theorem-grade one-sided tail constant scanner")
    ap.add_argument("--gamma-n", type=float, default=478.942181535)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--eta", type=float, default=0.01)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--xmax", type=float, default=1.0e120)
    ap.add_argument("--x-grid", type=int, default=12000)
    ap.add_argument("--t-grid", type=int, default=160000)
    ap.add_argument("--theta-start", type=float, default=0.39)
    ap.add_argument("--theta-end", type=float, default=0.499)
    ap.add_argument("--theta-step", type=float, default=0.001)
    ap.add_argument(
        "--c-a",
        type=float,
        default=None,
        help="Explicit Perron remainder constant C_A in |Rem_A|<=C_A x(log x)^2/T",
    )
    ap.add_argument(
        "--perron-source",
        type=str,
        choices=["dudek_prop", "cully_hugill_johnston"],
        default="cully_hugill_johnston",
        help="Source for explicit truncation remainder constant/conditions",
    )
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--q-fractions", type=str, default="0.9,0.7,0.5")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_onesided_tail_theorem_grade_constants_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.eta <= 0.0:
        raise ValueError("eta must be > 0")
    if args.theta_step <= 0.0:
        raise ValueError("theta-step must be > 0")

    if args.c_a is None:
        c_a = 2.0 if args.perron_source == "dudek_prop" else 2.091
    else:
        c_a = float(args.c_a)
    if c_a <= 0.0:
        raise ValueError("c-a must be > 0")

    q_fracs = parse_float_list(args.q_fractions)
    if any(q <= 0.0 for q in q_fracs):
        raise ValueError("q-fractions must be positive")

    x = np.exp(np.linspace(math.log(args.x1), math.log(args.xmax), args.x_grid, dtype=np.float64))
    x_eta = np.power(x, args.eta)

    thetas: List[float] = []
    t = args.theta_start
    while t <= args.theta_end + 1.0e-12:
        thetas.append(round(t, 10))
        t += args.theta_step
    if not thetas:
        raise ValueError("empty theta grid")

    t_max = float(np.max(np.power(x, max(thetas))))
    lut = build_s1_lookup(gamma_n=args.gamma_n, t_max=t_max, n_grid=args.t_grid)

    rows: List[Dict[str, object]] = []
    for theta in thetas:
        delta = theta + args.beta - 1.0 - args.eta
        perron = perron_window_checks(x1=args.x1, theta=theta, source=args.perron_source)
        perron_ok = bool(
            perron.get("x1_gt_exp50", True)
            and perron.get("x1_ge_exp40", True)
            and perron["cond_t_gt_lower_at_x1"]
            and perron["cond_t_lt_upper_at_x1"]
            and perron["theta_lt_half"]
        )
        eta_ok = delta > 0.0
        feasible = bool(eta_ok and perron_ok)

        b = np.power(x, theta)
        s1 = s1_upper_from_lookup(b=b, gamma_n=args.gamma_n, lut=lut)
        y_band = np.power(x, 0.5 - args.beta) * (2.0 * s1)
        c_band = float(np.max(y_band * x_eta))

        m_info = log_absorb_sup(x1=args.x1, delta=delta)
        c_rem_factor = float(m_info["value"]) if bool(m_info["finite"]) else float("inf")
        c_rem = c_a * c_rem_factor if feasible else float("inf")
        c_total = c_band + c_rem if feasible else float("inf")

        th_rows: List[Dict[str, float]] = []
        for qf in q_fracs:
            q_target = qf * args.a1
            th_rows.append(
                {
                    "q_fraction_of_a1": float(qf),
                    "q_target": float(q_target),
                    "x_threshold": float(threshold_x(c_total, args.eta, q_target, args.x1))
                    if math.isfinite(c_total)
                    else float("inf"),
                }
            )

        rows.append(
            {
                "theta": float(theta),
                "delta_log_absorb": float(delta),
                "feasible": feasible,
                "perron_checks": perron,
                "c_band": float(c_band),
                "c_rem_factor_per_CA": float(c_rem_factor),
                "c_rem": float(c_rem),
                "c_total": float(c_total),
                "x_star_log_absorb": float(m_info["x_star"]),
                "thresholds": th_rows,
            }
        )

    feasible_rows = [r for r in rows if bool(r["feasible"])]
    best = min(feasible_rows, key=lambda r: float(r["c_total"])) if feasible_rows else None

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "gamma_n": float(args.gamma_n),
            "beta": float(args.beta),
            "eta": float(args.eta),
            "x1": float(args.x1),
            "xmax": float(args.xmax),
            "x_grid": int(args.x_grid),
            "t_grid": int(args.t_grid),
            "theta_start": float(args.theta_start),
            "theta_end": float(args.theta_end),
            "theta_step": float(args.theta_step),
            "c_a": float(c_a),
            "perron_source": str(args.perron_source),
            "a1": float(args.a1),
            "q_fractions": q_fracs,
        },
        "source_locked_ingredients": {
            "band_bound": "2*S1(gamma_N, x^theta) with explicit N(T) upper envelope",
            "perron_remainder": "|Rem_A(x,T)| <= C_A*x*(log x)^2/T with C_A fixed by selected source",
            "eta_target_form": "|R(x)/x^beta| <= C_total*x^{-eta}",
        },
        "rows": rows,
        "best_feasible_row": best,
        "interpretation": {
            "note": (
                "These constants are derived from explicit formulas only (no empirical fit constants, "
                "no model-level Bellotti-min shortcut). Remaining work is to align this envelope "
                "with the exact final decomposition contract."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 One-Sided Tail Theorem-Grade Constants ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in payload["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Formula")
    lines.append("- `|R(x)/x^beta| <= C_band(theta)*x^{-eta} + C_A*M(x1,delta)*x^{-eta}`")
    lines.append("- `delta = theta + beta - 1 - eta`")
    lines.append("- `C_total(theta) = C_band(theta) + C_A*M(x1,delta)`")
    lines.append("")
    lines.append("| theta | feasible | delta | C_band | C_rem_factor/CA | C_total |")
    lines.append("|---:|:---:|---:|---:|---:|---:|")
    for r in rows:
        feas = "yes" if bool(r["feasible"]) else "no"
        c_total = float(r["c_total"])
        c_total_s = f"{c_total:.6g}" if math.isfinite(c_total) else "inf"
        lines.append(
            f"| {float(r['theta']):.3f} | {feas} | {float(r['delta_log_absorb']):.6f} | "
            f"{float(r['c_band']):.6g} | {float(r['c_rem_factor_per_CA']):.6g} | {c_total_s} |"
        )
    lines.append("")
    if best is not None:
        lines.append("## Best Feasible Theta")
        lines.append(f"- theta: `{best['theta']}`")
        lines.append(f"- delta: `{best['delta_log_absorb']}`")
        lines.append(f"- C_band: `{best['c_band']}`")
        lines.append(f"- C_rem_factor_per_CA: `{best['c_rem_factor_per_CA']}`")
        lines.append(f"- C_total: `{best['c_total']}`")
        lines.append("")
        lines.append("### q<a1 Thresholds")
        lines.append("| q_target | x_threshold |")
        lines.append("|---:|---:|")
        for th in best["thresholds"]:
            xth = float(th["x_threshold"])
            xth_s = f"{xth:.6e}" if math.isfinite(xth) else "inf"
            lines.append(f"| {float(th['q_target']):.6f} | {xth_s} |")
        lines.append("")
    lines.append("Explicit-constant derivation report only.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
