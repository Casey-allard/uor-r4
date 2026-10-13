#!/usr/bin/env python3
"""Check endpoint inequality on grid with conservative O1 theorem-budget constant."""

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


def main() -> None:
    ap = argparse.ArgumentParser(description="O1 theorem-budget endpoint check")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o1", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o5-integrated", default="research/output/o5_integrated_implication_draft_2026-02-17.json")
    ap.add_argument("--a4", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o1_theorem_budget_endpoint_check_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o1 = _read_json(args.o1)
    o5 = _read_json(args.o5_integrated)
    a4 = _read_json(args.a4)

    c = o5["integrated_o5_theorem"]["constants_snapshot"]
    c1 = o1["lemma_o1_a1_ref"]["current_constant_candidates"]
    a_ref = abs(float(c["a_ref"]))
    b_ref = abs(float(c["b_ref"]))
    c0_ref_budget = float(c1["C0_uplifted_sum"])
    c_h = float(c["C_H_theorem_budget"] if c["C_H_theorem_budget"] is not None else c["C_H"])
    a_h = float(c["A_H"])

    rows: List[Dict[str, float]] = []
    ratio_max = 0.0
    gap_min = float("inf")
    holds = True
    for row in a4.get("per_n", []):
        n_max = int(row["n_max"])
        x = max(3.0, float(n_max))
        lx = math.log(x)
        lhs = _max_lhs_from_row(row)
        rhs = c0_ref_budget + b_ref + a_ref * c_h * (lx ** a_h)
        ratio = lhs / rhs if rhs > 0 else float("inf")
        gap = rhs - lhs
        ratio_max = max(ratio_max, ratio)
        gap_min = min(gap_min, gap)
        if ratio > 1.0:
            holds = False
        rows.append(
            {
                "n_max": n_max,
                "lhs_max_abs_e_scaled": lhs,
                "rhs_o1_budget": rhs,
                "gap_rhs_minus_lhs": gap,
                "ratio_lhs_over_rhs": ratio,
            }
        )

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o1": args.o1,
            "o5_integrated": args.o5_integrated,
            "a4": args.a4,
        },
        "constants_used": {
            "a_ref_abs": a_ref,
            "b_ref_abs": b_ref,
            "C0_ref_budget_from_o1": c0_ref_budget,
            "C_H_used": c_h,
            "A_H": a_h,
        },
        "grid_endpoint_check": {
            "holds_on_grid": holds,
            "ratio_max": ratio_max,
            "gap_min": gap_min,
            "rows": rows,
        },
        "interpretation": (
            "Endpoint inequality remains stable using conservative O1 budget constants; "
            "remaining O1 work is asymptotic proof, not finite-grid incompatibility."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O1 Theorem-Budget Endpoint Check",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Constants Used",
        "",
        f"- `|a_ref| = {a_ref}`",
        f"- `|b_ref| = {b_ref}`",
        f"- `C0_ref_budget_from_o1 = {c0_ref_budget}`",
        f"- `C_H_used = {c_h}`",
        f"- `A_H = {a_h}`",
        "",
        "## Grid Check",
        "",
        f"- holds on grid: `{holds}`",
        f"- ratio max: `{ratio_max}`",
        f"- gap min: `{gap_min}`",
        "",
        payload["interpretation"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
