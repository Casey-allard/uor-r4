#!/usr/bin/env python3
"""Buffered rotation-density probe for tau1-anchor phase sequence.

For theta_k = 2*pi*rho*k + beta0 with rho=tau2/tau1 and beta0=phi2-rho*phi1,
measure empirical fractions:
  frac_N(c0) = #{1<=k<=N : cos(theta_k) >= c0}/N
and compare against irrational-rotation density target
  target(c0) = arccos(c0)/pi,  c0 in [-1,1].

Finite-N evidence only; this does not prove equidistribution.
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


def parse_path_list(text: str) -> List[Path]:
    vals: List[Path] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(Path(tok))
    if not vals:
        raise ValueError("empty path list")
    return vals


def target_fraction(c0: float) -> float:
    if c0 <= -1.0:
        return 1.0
    if c0 >= 1.0:
        return 0.0
    return float(math.acos(c0) / math.pi)


@dataclass
class C0Row:
    c0: float
    empirical_fraction: float
    target_fraction: float
    abs_gap: float


@dataclass
class NRow:
    n: int
    c0_rows: List[C0Row]
    max_abs_gap: float


@dataclass
class FitFileResult:
    fit_file: str
    phi1: float
    phi2: float
    rho: float
    beta0: float
    n_rows: List[NRow]
    max_abs_gap_overall: float


def evaluate_fit(
    fit_file: Path,
    tau1: float,
    tau2: float,
    n_list: List[int],
    c0_list: List[float],
) -> FitFileResult:
    d = json.loads(fit_file.read_text(encoding="utf-8"))
    if "fit" not in d:
        raise ValueError(f"missing fit in {fit_file}")
    fit = d["fit"]
    phi1 = float(fit["phi1"])
    phi2 = float(fit["phi2"])

    rho = float(tau2 / tau1)
    beta0 = float(phi2 - rho * phi1)
    two_pi = 2.0 * math.pi

    n_rows: List[NRow] = []
    max_abs_gap_overall = 0.0

    for n in n_list:
        k = np.arange(1, n + 1, dtype=np.float64)
        theta = two_pi * rho * k + beta0
        c = np.cos(theta)

        c0_rows: List[C0Row] = []
        max_gap = 0.0
        for c0 in c0_list:
            emp = float(np.mean(c >= c0))
            tgt = target_fraction(c0)
            gap = abs(emp - tgt)
            max_gap = max(max_gap, gap)
            c0_rows.append(
                C0Row(
                    c0=float(c0),
                    empirical_fraction=emp,
                    target_fraction=tgt,
                    abs_gap=gap,
                )
            )
        n_rows.append(NRow(n=int(n), c0_rows=c0_rows, max_abs_gap=max_gap))
        max_abs_gap_overall = max(max_abs_gap_overall, max_gap)

    return FitFileResult(
        fit_file=str(fit_file),
        phi1=phi1,
        phi2=phi2,
        rho=rho,
        beta0=beta0,
        n_rows=n_rows,
        max_abs_gap_overall=max_abs_gap_overall,
    )


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau12 buffered rotation-density probe")
    ap.add_argument(
        "--fit-files",
        type=str,
        required=True,
        help="Comma-separated JSON files containing fit{phi1,phi2}",
    )
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--n-list", type=str, default="10000,50000,100000,300000,500000")
    ap.add_argument("--c0-list", type=str, default="0.0,0.2,0.4,0.6,0.8")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/tau12_buffered_rotation_density_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")

    fit_files = parse_path_list(args.fit_files)
    n_list = parse_int_list(args.n_list)
    c0_list = sorted(set(parse_float_list(args.c0_list)))

    rows = [
        evaluate_fit(
            fit_file=ff,
            tau1=float(args.tau1),
            tau2=float(args.tau2),
            n_list=n_list,
            c0_list=c0_list,
        )
        for ff in fit_files
    ]

    max_abs_gap_overall = max(r.max_abs_gap_overall for r in rows) if rows else 0.0

    payload: Dict[str, object] = {
        "meta": {
            "date": date.today().isoformat(),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "n_list": n_list,
            "c0_list": c0_list,
            "fit_files": [str(p) for p in fit_files],
        },
        "rows": [asdict(r) for r in rows],
        "summary": {
            "max_abs_gap_overall": max_abs_gap_overall,
        },
        "interpretation": {
            "note": (
                "Finite-N diagnostic for buffered density targets; supports (but does not prove) "
                "the irrational-rotation C2 existence route."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Buffered Rotation Density Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `tau1={args.tau1}`, `tau2={args.tau2}`")
    lines.append(f"- `c0_list={c0_list}`")
    lines.append(f"- `n_list={n_list}`")
    lines.append(f"- max abs gap overall: `{max_abs_gap_overall:.6e}`")
    lines.append("")
    for r in rows:
        lines.append(f"## Fit: `{r.fit_file}`")
        lines.append(f"- `rho={r.rho:.12f}`, `beta0={r.beta0:.12f}`")
        lines.append(f"- max abs gap over this fit: `{r.max_abs_gap_overall:.6e}`")
        lines.append("")
        lines.append("| N | c0 | empirical | target | abs_gap |")
        lines.append("|---:|---:|---:|---:|---:|")
        for nr in r.n_rows:
            for cr in nr.c0_rows:
                lines.append(
                    f"| {nr.n} | {cr.c0:.3f} | {cr.empirical_fraction:.6f} | "
                    f"{cr.target_fraction:.6f} | {cr.abs_gap:.6e} |"
                )
        lines.append("")
    lines.append("Finite-N report only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

