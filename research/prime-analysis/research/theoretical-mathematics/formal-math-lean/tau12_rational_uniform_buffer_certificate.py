#!/usr/bin/env python3
"""Rational-branch uniform buffer certificate for C2.

Math fact used:
  If rho = p/q (reduced), then {2*pi*rho*k + beta0 mod 2*pi} is a q-point orbit.
  The orbit points are equally spaced by 2*pi/q, so one point lies within pi/q of 0 mod 2*pi.
  Therefore:
    max_{0<=j<q} cos(2*pi*p*j/q + beta0) >= cos(pi/q),
  and this value repeats infinitely often along k.

Hence, once a denominator lower bound q >= q_min is certified, rational-branch C2 holds
with uniform buffer c0 = cos(pi/q_min), independent of fitted beta0.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date
from pathlib import Path
from typing import Dict, List, Tuple


def rho_interval_from_tau(
    tau1: float, tau2: float, tau1_abs_err: float, tau2_abs_err: float
) -> Tuple[float, float]:
    if tau1 <= tau1_abs_err:
        raise ValueError("tau1 must be strictly larger than tau1_abs_err")
    lo = (tau2 - tau2_abs_err) / (tau1 + tau1_abs_err)
    hi = (tau2 + tau2_abs_err) / (tau1 - tau1_abs_err)
    if lo > hi:
        lo, hi = hi, lo
    return float(lo), float(hi)


def p_candidates_for_q(rho_lo: float, rho_hi: float, q: int) -> List[int]:
    p_lo = int(math.ceil(rho_lo * q))
    p_hi = int(math.floor(rho_hi * q))
    if p_lo > p_hi:
        return []
    return list(range(p_lo, p_hi + 1))


def interval_distance_to_value(lo: float, hi: float, v: float) -> float:
    if lo <= v <= hi:
        return 0.0
    if hi < v:
        return float(v - hi)
    return float(lo - v)


def build_md(payload: Dict[str, object]) -> str:
    cfg = payload["config"]
    scan = payload["denominator_scan"]
    cert = payload["certificate"]

    lines: List[str] = []
    lines.append(f"# Tau12 Rational Uniform Buffer Certificate ({cfg['date']})")
    lines.append("")
    lines.append("## Ratio Interval")
    lines.append(f"- `rho_center = {cfg['rho_center']:.15f}`")
    lines.append(
        f"- `rho_interval = [{cfg['rho_interval_lo']:.15f}, {cfg['rho_interval_hi']:.15f}]`"
    )
    lines.append(f"- `q_scan_max = {cfg['q_scan_max']}`")
    lines.append("")
    lines.append("## Small-Denominator Scan")
    lines.append(
        f"- first denominator with any `p/q` in interval: `{scan['first_candidate_q']}`"
    )
    lines.append(
        f"- certified denominator lower bound from scan: `q >= {cert['q_lower_bound_from_scan']}`"
    )
    lines.append("")
    lines.append("## Key C2 Certificate")
    lines.append(f"- `q=2` excluded by interval: `{cert['q2_excluded']}`")
    lines.append(
        f"- distance from interval to `3/2`: `{cert['distance_interval_to_3_over_2']:.12f}`"
    )
    lines.append(
        f"- uniform rational-branch buffer from denominator bound:"
        f" `c0_uniform = cos(pi/{cert['q_lower_bound_from_scan']}) = {cert['c0_uniform_from_q_lower_bound']:.12f}`"
    )
    lines.append("")
    lines.append("## Mathematical Implication")
    lines.append(
        "If `rho` is rational and lies in this interval, then its reduced denominator satisfies"
        " `q >= q_lower_bound_from_scan`, and for every phase shift `beta0` there are infinitely many"
        " `k` with `cos(2*pi*rho*k + beta0) >= c0_uniform`."
    )
    lines.append(
        "This removes dependence on fitted `cos(beta0)` for the rational C2 branch."
    )
    lines.append("")
    lines.append("Finite-precision interval certificate; theorem use is conditional on the interval assumptions.")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Rational branch uniform C2 buffer certificate")
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--tau1-abs-err", type=float, default=1.0e-6)
    ap.add_argument("--tau2-abs-err", type=float, default=1.0e-6)
    ap.add_argument("--rho-lo", type=float, default=None)
    ap.add_argument("--rho-hi", type=float, default=None)
    ap.add_argument("--q-scan-max", type=int, default=1000)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_rational_uniform_buffer_certificate_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.q_scan_max < 2:
        raise ValueError("q-scan-max must be >= 2")
    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")
    if args.tau1_abs_err < 0.0 or args.tau2_abs_err < 0.0:
        raise ValueError("tau absolute errors must be nonnegative")

    rho_center = float(args.tau2 / args.tau1)
    if args.rho_lo is not None or args.rho_hi is not None:
        if args.rho_lo is None or args.rho_hi is None:
            raise ValueError("provide both --rho-lo and --rho-hi, or neither")
        rho_lo = float(min(args.rho_lo, args.rho_hi))
        rho_hi = float(max(args.rho_lo, args.rho_hi))
    else:
        rho_lo, rho_hi = rho_interval_from_tau(
            tau1=float(args.tau1),
            tau2=float(args.tau2),
            tau1_abs_err=float(args.tau1_abs_err),
            tau2_abs_err=float(args.tau2_abs_err),
        )

    rows: List[Dict[str, object]] = []
    first_candidate_q: int | None = None
    for q in range(1, int(args.q_scan_max) + 1):
        p_list = p_candidates_for_q(rho_lo=rho_lo, rho_hi=rho_hi, q=q)
        if p_list and first_candidate_q is None:
            first_candidate_q = int(q)
        rows.append(
            {
                "q": int(q),
                "candidate_count": int(len(p_list)),
                "p_candidates": p_list[:10],
                "interval_hits_any": bool(p_list),
            }
        )

    q_lower_bound = int(first_candidate_q) if first_candidate_q is not None else int(args.q_scan_max) + 1
    if q_lower_bound < 2:
        c0_uniform = 0.0
    else:
        c0_uniform = float(math.cos(math.pi / float(q_lower_bound)))

    # In the relevant range rho in (1,2), q=2 means rho = 3/2.
    dist_to_three_halves = interval_distance_to_value(rho_lo, rho_hi, 1.5)
    q2_candidates = p_candidates_for_q(rho_lo, rho_hi, 2)
    q2_excluded = len(q2_candidates) == 0

    payload: Dict[str, object] = {
        "config": {
            "date": date.today().isoformat(),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "tau1_abs_err": float(args.tau1_abs_err),
            "tau2_abs_err": float(args.tau2_abs_err),
            "rho_center": rho_center,
            "rho_interval_lo": rho_lo,
            "rho_interval_hi": rho_hi,
            "q_scan_max": int(args.q_scan_max),
        },
        "denominator_scan": {
            "first_candidate_q": first_candidate_q,
            "rows": rows,
        },
        "certificate": {
            "q2_candidates": q2_candidates,
            "q2_excluded": bool(q2_excluded),
            "distance_interval_to_3_over_2": float(dist_to_three_halves),
            "q_lower_bound_from_scan": int(q_lower_bound),
            "c0_uniform_from_q_lower_bound": float(c0_uniform),
        },
        "math_statement": {
            "orbit_fact": (
                "For rho=p/q reduced, max_j cos(2*pi*p*j/q + beta0) >= cos(pi/q) for any beta0."
            ),
            "infinite_repetition": (
                "Any residue-class witness j repeats infinitely often along k = j + m*q."
            ),
            "conditional_c2_closure": (
                "If rational rho in interval implies denominator q >= q_lower_bound_from_scan, "
                "then C2 holds on rational branch with c0 = cos(pi/q_lower_bound_from_scan)."
            ),
        },
        "interpretation": {
            "note": (
                "This is an interval-conditional arithmetic certificate; it avoids fitted-beta0 dependence "
                "for rational-branch buffered anchors."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    out_md.write_text(build_md(payload), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
