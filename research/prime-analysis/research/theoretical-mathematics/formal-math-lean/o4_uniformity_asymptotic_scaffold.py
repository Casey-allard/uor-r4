#!/usr/bin/env python3
"""Build O4 scaffold for asymptotic wheel-uniform constants."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Create O4 asymptotic uniformity scaffold")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--a4", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--o1", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o3", default="research/output/o3_ref_bridge_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o4_uniformity_asymptotic_scaffold_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    a4 = _read_json(args.a4)
    o1 = _read_json(args.o1)
    o3 = _read_json(args.o3)

    uc = a4["uniform_constants"]
    tr = a4["theorem_rhs_check"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a4": args.a4,
            "o1": args.o1,
            "o3": args.o3,
        },
        "lemma_o4_uniformity": {
            "name": "Wheel-Uniform Asymptotic Constants",
            "quantified_statement": (
                "There exist x0>=3 and wheel-uniform constants such that for all x>=x0 and "
                "all W in {30,210,2310,30030}, the A1/A3 endpoint-transfer constants are common "
                "and satisfy a single uniform inequality family."
            ),
            "current_constant_candidates": {
                "a_ref": float(uc["a_ref"]),
                "b_ref": float(uc["b_ref"]),
                "C0_ref": float(uc["C0_ref"]),
                "C_delta": float(uc["C_delta"]),
                "tau_m_tail_fp": float(uc["tau_m_tail_fp"]),
                "C_H": float(uc["C_H"]),
            },
            "grid_uniformity_status": {
                "holds_on_grid": bool(tr["holds_on_grid"]),
                "ratio_max": float(tr["ratio_max"]),
                "max_gap_lhs_minus_rhs": float(tr["max_gap_lhs_minus_rhs"]),
            },
        },
        "dependency_merge": [
            {
                "from": "O1",
                "need": "C0_ref proven asymptotically and uniform in W.",
                "artifact": args.o1,
            },
            {
                "from": "O3",
                "need": "C_H, A_H proven asymptotically and uniform in W.",
                "artifact": args.o3,
            },
        ],
        "remaining_o4_blockers": [
            "Replace finite-grid spread checks with asymptotic uniform-in-W proofs.",
            "Prove no hidden W-dependence in constants as x->infinity.",
            "Finalize common x0 threshold valid simultaneously for all W in the wheel family.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    c = payload["lemma_o4_uniformity"]["current_constant_candidates"]
    g = payload["lemma_o4_uniformity"]["grid_uniformity_status"]
    lines = [
        "# O4 Uniformity Asymptotic Scaffold",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Quantified Statement",
        "",
        payload["lemma_o4_uniformity"]["quantified_statement"],
        "",
        "## Current Constant Candidates",
        "",
        f"- `a_ref = {c['a_ref']}`",
        f"- `b_ref = {c['b_ref']}`",
        f"- `C0_ref = {c['C0_ref']}`",
        f"- `C_delta = {c['C_delta']}`",
        f"- `tau_m_tail_fp = {c['tau_m_tail_fp']}`",
        f"- `C_H = {c['C_H']}`",
        "",
        "## Grid Uniformity Snapshot",
        "",
        f"- holds on grid: `{g['holds_on_grid']}`",
        f"- ratio max: `{g['ratio_max']}`",
        f"- max gap (lhs-rhs): `{g['max_gap_lhs_minus_rhs']}`",
        "",
        "## Remaining O4 Blockers",
        "",
    ]
    for x in payload["remaining_o4_blockers"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
