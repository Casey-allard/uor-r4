#!/usr/bin/env python3
"""Math-first no-prime/no-zeta universality probe for R4-style rotation structure.

This probe studies two ingredients of the current C2 -> rounding -> q<a1 path
using synthetic phase rotations that do not use primes or zeta zeros:

1) Buffered-density behavior for theta_k = 2*pi*rho*k + beta0
2) Eventual rounding-preservation at tau1-anchor points

Finite evidence only; this does not prove RH or the full tail bound step.
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

DEPENDENCY_INTENT = "independent_or_generic"


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


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    vals = sorted(set(v for v in vals if v >= 1))
    if not vals:
        raise ValueError("empty int list")
    return vals


def target_fraction(c0: float) -> float:
    if c0 <= -1.0:
        return 1.0
    if c0 >= 1.0:
        return 0.0
    return float(math.acos(c0) / math.pi)


@dataclass
class DensityRow:
    n: int
    c0: float
    empirical_fraction: float
    target_fraction: float
    abs_gap: float


@dataclass
class RoundingSummary:
    tau1: float
    tau2: float
    a1: float
    c0_buffer: float
    x0_threshold: float
    k_max: int
    buffered_count: int
    checked_count: int
    fail_sign_count: int
    fail_cos1_count: int
    fail_either_count: int
    min_cos2_anchor_buffered: float
    min_cos2_integer_checked: float
    min_abs_cos1_integer_checked: float


@dataclass
class RotationCase:
    rho: float
    beta0: float
    density_rows: List[DensityRow]
    density_max_abs_gap: float
    rounding_summary: RoundingSummary


def anchor_center(k: int, tau1: float, phi1: float) -> float:
    return math.exp((2.0 * math.pi * float(k) - phi1) / tau1)


def evaluate_density(rho: float, beta0: float, n_list: List[int], c0_list: List[float]) -> List[DensityRow]:
    rows: List[DensityRow] = []
    two_pi = 2.0 * math.pi
    for n in n_list:
        k = np.arange(1, n + 1, dtype=np.float64)
        theta = two_pi * float(rho) * k + float(beta0)
        c = np.cos(theta)
        for c0 in c0_list:
            emp = float(np.mean(c >= c0))
            tgt = target_fraction(float(c0))
            rows.append(
                DensityRow(
                    n=int(n),
                    c0=float(c0),
                    empirical_fraction=emp,
                    target_fraction=tgt,
                    abs_gap=abs(emp - tgt),
                )
            )
    return rows


def evaluate_rounding(
    rho: float,
    beta0: float,
    tau1: float,
    a1: float,
    c0_buffer: float,
    k_max: int,
) -> RoundingSummary:
    phi1 = 0.0
    phi2 = float(beta0)
    tau2 = float(rho) * float(tau1)

    d1_allow = float(math.acos(float(a1)))
    if d1_allow <= 0.0:
        raise ValueError("invalid a1")
    if c0_buffer <= 0.0:
        raise ValueError("c0_buffer must be > 0")

    x1 = 0.5 + 0.5 * float(tau1) / d1_allow
    x2 = 0.5 + 0.5 * float(tau2) / float(c0_buffer)
    x0 = float(max(x1, x2))

    buffered = 0
    checked = 0
    fail_sign = 0
    fail_cos1 = 0
    fail_either = 0

    min_cos2_anchor = 1.0
    min_cos2_int = 1.0
    min_abs_cos1_int = 1.0

    for k in range(1, int(k_max) + 1):
        x_star = anchor_center(k=k, tau1=float(tau1), phi1=phi1)
        theta_anchor = float(tau2) * math.log(x_star) + phi2
        cos2_anchor = math.cos(theta_anchor)
        if cos2_anchor < float(c0_buffer):
            continue
        buffered += 1
        min_cos2_anchor = min(min_cos2_anchor, float(cos2_anchor))
        if x_star < x0:
            continue

        n = max(2, int(round(x_star)))
        t = math.log(float(n))
        cos2_int = math.cos(float(tau2) * t + phi2)
        abs_cos1_int = abs(math.cos(float(tau1) * t + phi1))

        checked += 1
        min_cos2_int = min(min_cos2_int, float(cos2_int))
        min_abs_cos1_int = min(min_abs_cos1_int, float(abs_cos1_int))

        sign_ok = cos2_int >= 0.0
        cos1_ok = abs_cos1_int >= float(a1)
        if not sign_ok:
            fail_sign += 1
        if not cos1_ok:
            fail_cos1 += 1
        if (not sign_ok) or (not cos1_ok):
            fail_either += 1

    if buffered == 0:
        min_cos2_anchor = 0.0
    if checked == 0:
        min_cos2_int = 0.0
        min_abs_cos1_int = 0.0

    return RoundingSummary(
        tau1=float(tau1),
        tau2=float(tau2),
        a1=float(a1),
        c0_buffer=float(c0_buffer),
        x0_threshold=float(x0),
        k_max=int(k_max),
        buffered_count=int(buffered),
        checked_count=int(checked),
        fail_sign_count=int(fail_sign),
        fail_cos1_count=int(fail_cos1),
        fail_either_count=int(fail_either),
        min_cos2_anchor_buffered=float(min_cos2_anchor),
        min_cos2_integer_checked=float(min_cos2_int),
        min_abs_cos1_integer_checked=float(min_abs_cos1_int),
    )


def main() -> None:
    ap = argparse.ArgumentParser(description="R4 independent rotation universality probe")
    ap.add_argument(
        "--rho-list",
        type=str,
        default="1.487262003881136,1.4142135623730951,1.6180339887498948,1.7320508075688772,1.2718281828459045",
    )
    ap.add_argument("--beta-list", type=str, default="0.0,0.4,0.8,1.2")
    ap.add_argument("--n-list", type=str, default="10000,100000,500000")
    ap.add_argument("--c0-list", type=str, default="0.2,0.4,0.6,0.8")
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--c0-buffer", type=float, default=0.2)
    ap.add_argument("--k-max", type=int, default=260)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/r4_independent_rotation_universality_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    rho_list = parse_float_list(args.rho_list)
    beta_list = parse_float_list(args.beta_list)
    n_list = parse_int_list(args.n_list)
    c0_list = sorted(set(parse_float_list(args.c0_list)))

    cases: List[RotationCase] = []
    for rho in rho_list:
        for beta0 in beta_list:
            drows = evaluate_density(rho=float(rho), beta0=float(beta0), n_list=n_list, c0_list=c0_list)
            max_gap = max((r.abs_gap for r in drows), default=0.0)
            rsum = evaluate_rounding(
                rho=float(rho),
                beta0=float(beta0),
                tau1=float(args.tau1),
                a1=float(args.a1),
                c0_buffer=float(args.c0_buffer),
                k_max=int(args.k_max),
            )
            cases.append(
                RotationCase(
                    rho=float(rho),
                    beta0=float(beta0),
                    density_rows=drows,
                    density_max_abs_gap=float(max_gap),
                    rounding_summary=rsum,
                )
            )

    worst_density_gap = max((c.density_max_abs_gap for c in cases), default=0.0)
    total_rounding_checked = int(sum(c.rounding_summary.checked_count for c in cases))
    total_rounding_fail = int(sum(c.rounding_summary.fail_either_count for c in cases))

    payload: Dict[str, object] = {
        "meta": {
            "date": date.today().isoformat(),
            "rho_list": rho_list,
            "beta_list": beta_list,
            "n_list": n_list,
            "c0_list": c0_list,
            "tau1": float(args.tau1),
            "a1": float(args.a1),
            "c0_buffer": float(args.c0_buffer),
            "k_max": int(args.k_max),
        },
        "cases": [asdict(c) for c in cases],
        "summary": {
            "case_count": len(cases),
            "worst_density_max_abs_gap": float(worst_density_gap),
            "total_rounding_checked": total_rounding_checked,
            "total_rounding_fail_either": total_rounding_fail,
            "all_checked_rounding_success": bool(total_rounding_checked > 0 and total_rounding_fail == 0),
        },
        "interpretation": {
            "note": (
                "No-prime/no-zeta finite evidence that buffered-density and eventual rounding-preservation "
                "are generic irrational-rotation features. This supports mechanism-independence but does "
                "not by itself close RH tail-bound obligations."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# R4 Independent Rotation Universality Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append("Synthetic test: no primes and no zeta-zero lists.")
    lines.append("")
    lines.append(f"- `case_count={len(cases)}`")
    lines.append(f"- `worst_density_max_abs_gap={worst_density_gap:.6e}`")
    lines.append(f"- `total_rounding_checked={total_rounding_checked}`")
    lines.append(f"- `total_rounding_fail_either={total_rounding_fail}`")
    lines.append("")
    lines.append("| rho | beta0 | density_max_abs_gap | checked | fail_either | x0_threshold |")
    lines.append("|---:|---:|---:|---:|---:|---:|")
    for c in cases:
        r = c.rounding_summary
        lines.append(
            f"| {c.rho:.12f} | {c.beta0:.3f} | {c.density_max_abs_gap:.6e} | "
            f"{r.checked_count} | {r.fail_either_count} | {r.x0_threshold:.6f} |"
        )
    lines.append("")
    lines.append("Finite evidence only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
