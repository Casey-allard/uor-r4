#!/usr/bin/env python3
"""Probe sign/density behavior of anchor rotation sequence.

Sequence:
  theta_k = 2*pi*rho*k + beta0
with
  rho = tau2/tau1
  beta0 = phi2 - rho*phi1
from tau1-anchor relation.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import List

import numpy as np


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    if not vals:
        raise ValueError("empty int list")
    return vals


@dataclass
class NRow:
    n: int
    nonnegative_fraction: float
    positive_fraction: float
    mean_cos: float
    min_cos: float
    max_cos: float
    abs_gap_nonnegative_from_half: float


def main() -> None:
    ap = argparse.ArgumentParser(description="Anchor rotation sign probe")
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--phi1", type=float, required=True)
    ap.add_argument("--phi2", type=float, required=True)
    ap.add_argument("--n-list", type=str, default="100,500,1000,5000,10000,50000,100000")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_rotation_sign_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0:
        raise ValueError("tau1 must be positive")
    rho = float(args.tau2 / args.tau1)
    beta0 = float(args.phi2 - rho * args.phi1)
    n_list = sorted(set(n for n in parse_int_list(args.n_list) if n >= 1))
    if not n_list:
        raise ValueError("n-list empty")

    rows: List[NRow] = []
    two_pi = 2.0 * math.pi
    for n in n_list:
        k = np.arange(1, n + 1, dtype=np.float64)
        theta = two_pi * rho * k + beta0
        c = np.cos(theta)
        nonneg = float(np.mean(c >= 0.0))
        pos = float(np.mean(c > 0.0))
        mean_c = float(np.mean(c))
        rows.append(
            NRow(
                n=int(n),
                nonnegative_fraction=nonneg,
                positive_fraction=pos,
                mean_cos=mean_c,
                min_cos=float(np.min(c)),
                max_cos=float(np.max(c)),
                abs_gap_nonnegative_from_half=float(abs(nonneg - 0.5)),
            )
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "phi1": float(args.phi1),
            "phi2": float(args.phi2),
            "rho": rho,
            "beta0": beta0,
            "n_list": n_list,
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-N rotation evidence only. "
                "Used to diagnose empirical sign density for constructive C2 planning."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Rotation Sign Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `rho = tau2/tau1 = {rho:.12f}`")
    lines.append(f"- `beta0 = phi2 - rho*phi1 = {beta0:.12f}`")
    lines.append("")
    lines.append("| N | frac(cos>=0) | frac(cos>0) | mean(cos) | min(cos) | max(cos) | |frac-0.5| |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.n} | {r.nonnegative_fraction:.6f} | {r.positive_fraction:.6f} | {r.mean_cos:.6f} | "
            f"{r.min_cos:.6f} | {r.max_cos:.6f} | {r.abs_gap_nonnegative_from_half:.6f} |"
        )
    lines.append("")
    lines.append("Finite-N report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
