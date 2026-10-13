#!/usr/bin/env python3
"""Rational-approximation probe for rho=tau2/tau1.

Searches best |rho - p/q| over 1 <= q <= qmax via nearest-integer numerators.
Finite-precision diagnostic only; this does not prove irrationality.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date
from pathlib import Path
from typing import Dict, List


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau ratio rational approximation probe")
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--qmax-list", type=str, default="1000,10000,100000,1000000")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_tau_ratio_rational_approx_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0:
        raise ValueError("tau1 must be positive")

    qmax_list = sorted(
        set(int(tok.strip()) for tok in args.qmax_list.split(",") if tok.strip())
    )
    if not qmax_list or qmax_list[0] < 1:
        raise ValueError("qmax-list must contain positive integers")

    rho = float(args.tau2 / args.tau1)
    rows: List[Dict[str, float | int]] = []

    best_err = float("inf")
    best_p = 0
    best_q = 1
    qmax_iter = iter(qmax_list)
    current_qmax = next(qmax_iter, None)

    for q in range(1, qmax_list[-1] + 1):
        p = int(round(rho * q))
        err = abs(rho - (p / q))
        if err < best_err:
            best_err = err
            best_p = p
            best_q = q
        while current_qmax is not None and q == current_qmax:
            rows.append(
                {
                    "qmax": int(current_qmax),
                    "best_p": int(best_p),
                    "best_q": int(best_q),
                    "best_approx": float(best_p / best_q),
                    "abs_error": float(best_err),
                    "scaled_error_q2": float(best_err * (best_q**2)),
                }
            )
            current_qmax = next(qmax_iter, None)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "rho": rho,
            "qmax_list": qmax_list,
        },
        "rows": rows,
        "interpretation": {
            "note": (
                "Finite denominator-search diagnostic from floating rho. "
                "Use as plausibility evidence only; not a proof of irrationality."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau Ratio Rational Approximation Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `rho=tau2/tau1={rho:.15f}`")
    lines.append("")
    lines.append("| qmax | best p/q | approx | abs_error | abs_error * q^2 |")
    lines.append("|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {int(r['qmax'])} | {int(r['best_p'])}/{int(r['best_q'])} | "
            f"{float(r['best_approx']):.15f} | {float(r['abs_error']):.6e} | "
            f"{float(r['scaled_error_q2']):.6f} |"
        )
    lines.append("")
    lines.append("Finite search report only; not an irrationality proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

