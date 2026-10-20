#!/usr/bin/env python3
"""Endpoint transfer audit in the M=M_ref regime (Delta_M = 0)."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _max_lhs_from_row(row: Dict[str, Any]) -> float:
    vals = [float(b["max_abs_e_scaled"]) for b in row.get("per_base", [])]
    return max(vals) if vals else float("nan")


def _target_exponent_passes(a_h: float, target: float) -> bool:
    # In M_ref transfer, the A2 Delta_M term is identically zero,
    # so the asymptotic exponent class is driven by A_H (plus constants).
    return a_h <= target


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit RH endpoint class in M_ref regime")
    ap.add_argument(
        "--canonical-manifest",
        default="research/output/proof_canonical_manifest.json",
    )
    ap.add_argument(
        "--a3",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument(
        "--a4",
        default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json",
    )
    ap.add_argument("--target-log-exponent", type=float, default=2.0)
    ap.add_argument(
        "--output",
        default="research/output/rh_endpoint_mref_transfer_2026-02-17.json",
    )
    args = ap.parse_args()

    manifest = _read_json(args.canonical_manifest)
    canonical = manifest.get("canonical", {})

    a3_path = args.a3 or canonical.get("a3", "")
    a4_path = args.a4 or canonical.get("a4", "")
    a3 = _read_json(a3_path)
    a4 = _read_json(a4_path)

    a_h = float(a3["h_transfer_envelope"]["A_H"])
    c_h = float(a3["h_transfer_envelope"]["C_H_from_density_transfer"])

    uc = a4["uniform_constants"]
    a_ref = float(uc["a_ref"])
    c0_ref = float(uc["C0_ref"])

    # M_ref endpoint surrogate bound (Delta_M term dropped):
    # |E(x)|/sqrt(x) <= C0_ref + |a_ref| * C_H * (log x)^A_H + |b_ref|.
    # For exponent-class checks, additive constants do not affect asymptotic exponent.
    rows: List[Dict[str, Any]] = []
    for row in a4.get("per_n", []):
        n_max = int(row["n_max"])
        x = max(3.0, float(n_max))
        lx = math.log(x)
        lhs = _max_lhs_from_row(row)
        rhs_no_delta = c0_ref + abs(a_ref) * c_h * (lx ** a_h)
        gap = rhs_no_delta - lhs
        ratio = lhs / rhs_no_delta if rhs_no_delta > 0 else float("inf")
        rows.append(
            {
                "n_max": n_max,
                "lhs_max_abs_e_scaled": lhs,
                "rhs_mref_no_delta": rhs_no_delta,
                "gap_rhs_minus_lhs": gap,
                "ratio_lhs_over_rhs": ratio,
            }
        )

    valid_ratios = [float(r["ratio_lhs_over_rhs"]) for r in rows if math.isfinite(float(r["ratio_lhs_over_rhs"]))]
    valid_gaps = [float(r["gap_rhs_minus_lhs"]) for r in rows if math.isfinite(float(r["gap_rhs_minus_lhs"]))]
    ratio_max = max(valid_ratios) if valid_ratios else float("inf")
    gap_min = min(valid_gaps) if valid_gaps else float("-inf")

    target = float(args.target_log_exponent)
    class_pass = _target_exponent_passes(a_h, target)

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a3": a3_path,
            "a4": a4_path,
        },
        "mref_transfer": {
            "statement": "At M=M_ref, Delta_M term is identically zero in endpoint transfer inequality.",
            "delta_term_removed": True,
            "implied_endpoint_exponent": a_h,
            "target_endpoint_exponent": target,
            "reaches_target_class": class_pass,
            "exponent_gap": a_h - target,
        },
        "constants_used": {
            "a_ref": a_ref,
            "C0_ref": c0_ref,
            "A_H": a_h,
            "C_H": c_h,
        },
        "empirical_grid_check": {
            "rows": rows,
            "ratio_max": ratio_max,
            "gap_min": gap_min,
            "holds_on_grid": bool(ratio_max <= 1.0),
        },
        "math_interpretation": {
            "main_point": "The beta(A2) barrier is bypassed in the direct M_ref endpoint route because Delta_M vanishes.",
            "remaining_proof_work": [
                "Prove the endpoint transfer inequality in theorem form at M_ref (not only on sampled grids).",
                "Prove A3 asymptotic bound |H(x)| <= C_H (log x)^A_H with explicit hypotheses.",
                "Map resulting endpoint bound to a standard RH-equivalent formulation with complete assumptions.",
            ],
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# RH Endpoint Mref Transfer Audit",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Core Result",
        "",
        "- In the `M=M_ref` regime, `Delta_M=0` so the A2 `beta` exponent does not enter the endpoint growth class.",
        f"- Implied endpoint exponent class from A3: `A_H={a_h}`",
        f"- RH-style target exponent: `{target}`",
        f"- Reachable in this regime: `{class_pass}`",
        f"- Exponent gap `A_H-target`: `{a_h - target}`",
        "",
        "## Constants Used",
        "",
        f"- `a_ref={a_ref}`",
        f"- `C0_ref={c0_ref}`",
        f"- `C_H={c_h}`",
        f"- `A_H={a_h}`",
        "",
        "## Empirical Grid Check (No Delta Term)",
        "",
        f"- `holds_on_grid={payload['empirical_grid_check']['holds_on_grid']}`",
        f"- `ratio_max={ratio_max}`",
        f"- `gap_min={gap_min}`",
        "",
        "## Remaining Theorem Work (O5)",
        "",
    ]
    for item in payload["math_interpretation"]["remaining_proof_work"]:
        lines.append(f"- {item}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
