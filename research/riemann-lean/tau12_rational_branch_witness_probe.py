#!/usr/bin/env python3
"""Rational-branch witness audit for C2 using fitted beta0 values.

For theta_k = 2*pi*rho*k + beta0 and rational rho = p/q,
k = m*q yields theta_{m*q} = 2*pi*p*m + beta0, hence cos(theta_{m*q}) = cos(beta0).

Therefore, if cos(beta0) >= c0 > 0, then the buffered set {k : cos(theta_k) >= c0}
is infinite on the rational branch.

This script estimates robustness of that witness mechanism across fitted windows.
Finite evidence only.
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


@dataclass
class FitWitnessRow:
    fit_file: str
    phi1: float
    phi2: float
    beta0: float
    cos_beta0: float


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau12 rational-branch witness probe")
    ap.add_argument(
        "--fit-glob",
        type=str,
        default="research/output/*tau14_tau21_constructive_gate*2026-02-24*.json",
    )
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--c0-candidates", type=str, default="0.1,0.2,0.3,0.4,0.5,0.6")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_rational_branch_witness_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")

    rho = float(args.tau2 / args.tau1)
    c0_candidates = sorted(set(parse_float_list(args.c0_candidates)))

    files = sorted(glob.glob(args.fit_glob))
    rows: List[FitWitnessRow] = []
    for p in files:
        try:
            d = json.loads(Path(p).read_text(encoding="utf-8"))
        except Exception:
            continue
        fit = d.get("fit", {})
        if "phi1" not in fit or "phi2" not in fit:
            continue
        phi1 = float(fit["phi1"])
        phi2 = float(fit["phi2"])
        beta0 = float(phi2 - rho * phi1)
        rows.append(
            FitWitnessRow(
                fit_file=str(p),
                phi1=phi1,
                phi2=phi2,
                beta0=beta0,
                cos_beta0=float(math.cos(beta0)),
            )
        )

    if not rows:
        raise ValueError("no valid fit files found from fit-glob")

    min_cos_beta0 = min(r.cos_beta0 for r in rows)
    max_cos_beta0 = max(r.cos_beta0 for r in rows)
    max_uniform_c0 = min_cos_beta0

    c0_pass: Dict[str, bool] = {}
    for c0 in c0_candidates:
        c0_pass[f"{c0:.6f}"] = all(r.cos_beta0 >= c0 for r in rows)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "fit_glob": args.fit_glob,
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "rho": rho,
            "row_count": len(rows),
            "c0_candidates": c0_candidates,
        },
        "rows": [asdict(r) for r in rows],
        "summary": {
            "min_cos_beta0": float(min_cos_beta0),
            "max_cos_beta0": float(max_cos_beta0),
            "max_uniform_c0_from_rows": float(max_uniform_c0),
            "c0_candidate_all_rows_pass": c0_pass,
        },
        "interpretation": {
            "note": (
                "If rho is rational and beta0 is fixed with cos(beta0)>=c0>0, then the buffered set is "
                "infinite via k=m*q. This file reports finite-window fit evidence for positive cos(beta0)."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Rational-Branch Witness Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `fit_glob={args.fit_glob}`")
    lines.append(f"- `rho=tau2/tau1={rho:.12f}`")
    lines.append(f"- `row_count={len(rows)}`")
    lines.append(f"- `min_cos_beta0={min_cos_beta0:.12f}`")
    lines.append(f"- `max_uniform_c0_from_rows={max_uniform_c0:.12f}`")
    lines.append("")
    lines.append("## c0 Candidates")
    for c0 in c0_candidates:
        lines.append(f"- `c0={c0:.3f}` all rows pass: `{c0_pass[f'{c0:.6f}']}`")
    lines.append("")
    lines.append("| fit_file | beta0 | cos(beta0) |")
    lines.append("|---|---:|---:|")
    for r in rows:
        lines.append(f"| `{Path(r.fit_file).name}` | {r.beta0:.12f} | {r.cos_beta0:.12f} |")
    lines.append("")
    lines.append("Finite evidence only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
