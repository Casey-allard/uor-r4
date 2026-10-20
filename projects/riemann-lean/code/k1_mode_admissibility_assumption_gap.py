#!/usr/bin/env python3
"""Admissibility assumption-gap ledger for robust tau candidates.

Compares target aligned remainder-ratio threshold (rr < a0) against
published-style global envelopes after normalization by A*x^beta:

  Y(x) = |E(x)| / x^beta
  rr_target <= a0  (for remainder ratio channel)

Envelope models used (from in-repo theorem constants notes):
  FKS model:
    Delta(x) <= 9.22022 (log x)^(3/2) exp(-0.8476836 sqrt(log x))
    => Y_fks(x) = x^(1-beta) * Delta(x)

  Bellotti-2025-style model (asymptotic shape in note):
    Delta(x) <= exp(55*A0) * exp(-omega(x)),
    A0 = 1/48.0718,
    omega(x) = (log x)^(3/5) / (log log x)^(1/5)
    => Y_bel(x) = x^(1-beta) * Delta(x)

For each robust tau, report gap factors:
  gap_model = Y_model(x)/(a0*A_min)
If gap_model > 1, the model bound alone cannot certify rr < a0 at that x.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List


@dataclass
class TauGapRow:
    tau: float
    a_min: float
    rr_max_cross_window: float
    delta_cons: float
    c_cons: float
    gap_fks_x1e7: float
    gap_fks_x1e21: float
    gap_bel_x1e7: float
    gap_bel_x1e21: float


def _read_json(path: str) -> Dict[str, object]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _fks_delta(x: float) -> float:
    lx = math.log(x)
    return 9.22022 * (lx ** 1.5) * math.exp(-0.8476836 * math.sqrt(lx))


def _bellotti_like_delta(x: float) -> float:
    a0 = 1.0 / 48.0718
    lx = math.log(x)
    llx = math.log(max(lx, math.e))
    omega = (lx ** (3.0 / 5.0)) / (llx ** (1.0 / 5.0))
    return math.exp(55.0 * a0) * math.exp(-omega)


def _y_from_delta(delta: float, x: float, beta: float) -> float:
    return (x ** (1.0 - beta)) * delta


def main() -> None:
    ap = argparse.ArgumentParser(description="Assumption-gap ledger for admissible tau route")
    ap.add_argument(
        "--scan-x1e7",
        type=str,
        default="research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x1e7_beta062.json",
    )
    ap.add_argument(
        "--scan-x3e7",
        type=str,
        default="research/output/k1_w44_fixed_error_psi_subsequence_tau_scan_2026-02-24_x3e7_beta062.json",
    )
    ap.add_argument("--a0", type=float, default=0.98)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_mode_admissibility_assumption_gap_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    j1 = _read_json(args.scan_x1e7)
    j3 = _read_json(args.scan_x3e7)
    by1 = {round(float(r["tau"]), 9): r for r in j1["rows"]}  # type: ignore[index]
    by3 = {round(float(r["tau"]), 9): r for r in j3["rows"]}  # type: ignore[index]

    x_ref1 = 1.0e7
    x_ref2 = 1.0e21

    y_fks_1 = _y_from_delta(_fks_delta(x_ref1), x_ref1, float(args.beta))
    y_fks_2 = _y_from_delta(_fks_delta(x_ref2), x_ref2, float(args.beta))
    y_bel_1 = _y_from_delta(_bellotti_like_delta(x_ref1), x_ref1, float(args.beta))
    y_bel_2 = _y_from_delta(_bellotti_like_delta(x_ref2), x_ref2, float(args.beta))

    rows: List[TauGapRow] = []
    for tau in sorted(set(by1) & set(by3)):
        r1 = by1[tau]
        r3 = by3[tau]
        rr_max = max(float(r1["rratio_cofinal_grid"]), float(r3["rratio_cofinal_grid"]))
        if rr_max >= float(args.a0):
            continue
        a_min = min(float(r1["amplitude"]), float(r3["amplitude"]))
        delta = float(args.a0) - rr_max
        c_cons = a_min * delta
        denom = max(1.0e-300, float(args.a0) * a_min)
        rows.append(
            TauGapRow(
                tau=float(tau),
                a_min=a_min,
                rr_max_cross_window=rr_max,
                delta_cons=delta,
                c_cons=c_cons,
                gap_fks_x1e7=y_fks_1 / denom,
                gap_fks_x1e21=y_fks_2 / denom,
                gap_bel_x1e7=y_bel_1 / denom,
                gap_bel_x1e21=y_bel_2 / denom,
            )
        )

    rows.sort(key=lambda r: r.c_cons, reverse=True)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "a0": float(args.a0),
            "beta": float(args.beta),
            "scan_x1e7": args.scan_x1e7,
            "scan_x3e7": args.scan_x3e7,
            "x_ref_1": x_ref1,
            "x_ref_2": x_ref2,
            "y_fks_ref": {"x1e7": y_fks_1, "x1e21": y_fks_2},
            "y_bel_ref": {"x1e7": y_bel_1, "x1e21": y_bel_2},
        },
        "rows": [asdict(r) for r in rows],
        "summary": {
            "robust_admissible_count": len(rows),
            "min_gap_fks_x1e7": min((r.gap_fks_x1e7 for r in rows), default=None),
            "min_gap_fks_x1e21": min((r.gap_fks_x1e21 for r in rows), default=None),
            "min_gap_bel_x1e7": min((r.gap_bel_x1e7 for r in rows), default=None),
            "min_gap_bel_x1e21": min((r.gap_bel_x1e21 for r in rows), default=None),
        },
        "interpretation": {
            "note": (
                "Gap > 1 means the corresponding global envelope is too weak (after normalization by A*x^beta) "
                "to certify rr<a0 directly at that scale."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Mode-Admissibility Assumption Gap ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- beta: `{args.beta}`")
    lines.append(f"- a0: `{args.a0}`")
    lines.append(f"- robust admissible taus analyzed: `{len(rows)}`")
    lines.append("")
    lines.append("## Gap Table")
    lines.append("")
    lines.append("| tau | A_min | c_cons | gap_FKS(1e7) | gap_FKS(1e21) | gap_Bel(1e7) | gap_Bel(1e21) |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.tau:.6f} | {r.a_min:.6e} | {r.c_cons:.6e} | {r.gap_fks_x1e7:.6e} | "
            f"{r.gap_fks_x1e21:.6e} | {r.gap_bel_x1e7:.6e} | {r.gap_bel_x1e21:.6e} |"
        )
    lines.append("")
    s = payload["summary"]
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- min gap FKS at 1e7: `{s['min_gap_fks_x1e7']}`")
    lines.append(f"- min gap FKS at 1e21: `{s['min_gap_fks_x1e21']}`")
    lines.append(f"- min gap Bellotti-like at 1e7: `{s['min_gap_bel_x1e7']}`")
    lines.append(f"- min gap Bellotti-like at 1e21: `{s['min_gap_bel_x1e21']}`")
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
