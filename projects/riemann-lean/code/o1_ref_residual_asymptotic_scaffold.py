#!/usr/bin/env python3
"""Build O1 scaffold for asymptotic reference residual control (A1_ref)."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Create O1 A1_ref asymptotic scaffold")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--a1", default="research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json")
    ap.add_argument("--a4", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    a1 = _read_json(args.a1)
    a4 = _read_json(args.a4)

    d = a1["decomposition_constants"]
    c0_u = float(d["C0_uplifted"])
    smooth_u = float(d.get("C_smooth_uplifted", d.get("C_smooth_train", 0.0)))
    link_u = float(d.get("C_link_uplifted", d.get("C_link_train", 0.0)))
    checks = a1["checks"]["valid"]
    uc = a4["uniform_constants"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a1": args.a1,
            "a4": args.a4,
        },
        "lemma_o1_a1_ref": {
            "name": "Reference Residual Asymptotic Control",
            "quantified_statement": (
                "There exist x0>=3 and C0_ref>=0 such that for all x>=x0 and "
                "W in {30,210,2310,30030}, "
                "|E(x)/sqrt(x) - (a_ref H_W^(M_ref)(x) + b_ref)| <= C0_ref."
            ),
            "decomposition_plan": [
                "Residual <= smoothing_term + link_term.",
                "Bound each term uniformly in W and x>=x0.",
                "Set C0_ref = C_smooth + C_link.",
            ],
            "current_constant_candidates": {
                "C_smooth_uplifted": smooth_u,
                "C_link_uplifted": link_u,
                "C0_uplifted_sum": c0_u,
                "a_ref_from_a4": float(uc["a_ref"]),
                "b_ref_from_a4": float(uc["b_ref"]),
                "m_ref_from_a4": int(a4["config"]["m_ref"]),
            },
            "grid_validation_status": {
                "valid_holds": bool(checks["holds"]),
                "valid_ratio_max": float(checks["ratio_max"]),
                "valid_max_gap_lhs_minus_rhs": float(checks["max_gap_lhs_minus_rhs"]),
            },
        },
        "remaining_o1_blockers": [
            "Replace finite-window sup calibration by asymptotic smoothing bound valid for all x>=x0.",
            "Prove link-term bound without sampled-grid dependence.",
            "Prove wheel-family uniformity of C0_ref constants for all W in {30,210,2310,30030}.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O1 A1_ref Asymptotic Scaffold",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Quantified Statement",
        "",
        payload["lemma_o1_a1_ref"]["quantified_statement"],
        "",
        "## Decomposition Plan",
        "",
    ]
    for p in payload["lemma_o1_a1_ref"]["decomposition_plan"]:
        lines.append(f"- {p}")
    lines += [
        "",
        "## Current Constant Candidates",
        "",
        f"- `C_smooth_uplifted = {smooth_u}`",
        f"- `C_link_uplifted = {link_u}`",
        f"- `C0_uplifted_sum = {c0_u}`",
        f"- `a_ref = {uc['a_ref']}`",
        f"- `b_ref = {uc['b_ref']}`",
        f"- `m_ref = {a4['config']['m_ref']}`",
        "",
        "## Grid Validation Snapshot",
        "",
        f"- valid holds: `{checks['holds']}`",
        f"- valid ratio max: `{checks['ratio_max']}`",
        f"- valid max gap (lhs-rhs): `{checks['max_gap_lhs_minus_rhs']}`",
        "",
        "## Remaining O1 Blockers",
        "",
    ]
    for p in payload["remaining_o1_blockers"]:
        lines.append(f"- {p}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
