#!/usr/bin/env python3
"""Probe the eta-band necessity condition for K1/R6 tail majorants.

For the explicit finite-band component used in K1/L2A:
  y_band(x) = x^(1/2 - beta) * 2*S1(x^theta),
where S1(b) is an upper envelope for sum_{gamma in (gamma_n,b]} 1/gamma.

To enforce a global x^{-eta} tail majorant, one needs finite
  C_band = sup_{x>=x1} y_band(x) * x^eta.

This script measures C_band growth vs xmax and beta to diagnose whether C_band
stabilizes (finite) or grows (divergent in the tested range).
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List

import numpy as np

from k1_l2a_inverse_feasibility import build_s1_lookup, s1_upper


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        vals.append(float(tok))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("empty numeric list")
    return vals


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 eta-band necessity probe")
    ap.add_argument("--gamma-n", type=float, default=478.942181535)
    ap.add_argument("--eta", type=float, default=0.01)
    ap.add_argument("--theta", type=float, default=0.50)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--betas", type=str, default="0.5001,0.51,0.52,0.55")
    ap.add_argument("--xmax-list", type=str, default="1e30,1e60,1e120,1e240")
    ap.add_argument("--x-grid", type=int, default=6000)
    ap.add_argument("--t-grid", type=int, default=120000)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w36_eta_band_necessity_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    betas = parse_float_list(args.betas)
    xmax_list = parse_float_list(args.xmax_list)

    rows: List[Dict[str, float]] = []
    by_beta: Dict[str, List[Dict[str, float]]] = {}

    for beta in betas:
        key = f"{beta:.10g}"
        by_beta[key] = []
        prev = None
        for xmax in xmax_list:
            x = np.exp(np.linspace(math.log(args.x1), math.log(xmax), args.x_grid, dtype=np.float64))
            b = np.power(x, args.theta)
            t_max = float(np.max(b))
            lut = build_s1_lookup(args.gamma_n, t_max, args.t_grid)
            s1 = s1_upper(args.gamma_n, b, lut)
            y_band = np.power(x, 0.5 - beta) * (2.0 * s1)
            c_band = float(np.max(y_band * np.power(x, args.eta)))
            ratio = float(c_band / prev) if (prev is not None and prev > 0.0) else float("nan")
            rec = {
                "beta": float(beta),
                "xmax": float(xmax),
                "c_band": c_band,
                "ratio_vs_prev": ratio,
            }
            by_beta[key].append(rec)
            rows.append(rec)
            prev = c_band

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "gamma_n": args.gamma_n,
            "eta": args.eta,
            "theta": args.theta,
            "x1": args.x1,
            "betas": betas,
            "xmax_list": xmax_list,
            "x_grid": args.x_grid,
            "t_grid": args.t_grid,
        },
        "rows": rows,
        "by_beta": by_beta,
        "analytic_note": {
            "stability_condition": "heuristically requires beta > 1/2 + eta for fixed theta when S1 grows polylogarithmically",
            "for_eta_0p01": "beta > 0.51 (strict) is the practical threshold",
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Eta-Band Necessity Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("| beta | xmax | C_band | ratio_vs_prev |")
    lines.append("|---:|---:|---:|---:|")
    for beta in betas:
        for r in by_beta[f"{beta:.10g}"]:
            ratio = r["ratio_vs_prev"]
            rs = f"{ratio:.6g}" if math.isfinite(ratio) else "-"
            lines.append(
                f"| {r['beta']:.4f} | {r['xmax']:.0e} | {r['c_band']:.6g} | {rs} |"
            )
    lines.append("")
    lines.append("## Interpretation")
    lines.append(
        "- Growth in `C_band` as `xmax` increases indicates the x^{-eta} majorant constant does not stabilize in the tested range."
    )
    lines.append(
        "- Stabilization indicates finite majorant compatibility for that `(beta, eta, theta)` regime."
    )
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
