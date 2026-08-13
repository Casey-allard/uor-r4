#!/usr/bin/env python3
"""Draft assembly (GLUE) lemma for O3 E2 asymptotic proof chain."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build E2-GLUE symbolic draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--o3-target", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_glue_symbolic_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    full = _read_json(args.o3_e2_full)
    target = _read_json(args.o3_target)["o3_e2_target_pack"]

    ae2_max = float(target["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    ce2_min = float(target["e2_constant_target"]["C_E2_min_from_budget_chain"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_full": args.o3_e2_full,
            "o3_target": args.o3_target,
        },
        "lemma_id": "E2-GLUE",
        "target_theorem_goal": full["theorem_goal"]["statement"],
        "symbolic_draft": {
            "assembly_rule": (
                "If Diag <= C_diag (log x)^A_diag, Offdiag <= C_off (log x)^A_off, "
                "Rem <= C_rem (log x)^A_rem, then E2/x <= C_E2 (log x)^A_E2 where "
                "A_E2=max(A_diag,A_off,A_rem), C_E2=C_diag+C_off+C_rem."
            ),
            "target_constraints": {
                "A_E2_max_required": ae2_max,
                "C_E2_min_chain_compatibility": ce2_min,
            },
            "closure_condition": "Show A_E2 <= A_E2_max_required and C_E2 compatible with O3 chain.",
        },
        "remaining_to_formalize": [
            "Track exact exponents from E2-DIAG, E2-OFFDIAG-SIGN, E2-REM outputs.",
            "Establish common x0 where all three component inequalities hold simultaneously.",
            "Derive final C_E2 and verify compatibility with O3 target pack.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    tc = payload["symbolic_draft"]["target_constraints"]
    lines = [
        "# O3 E2 Glue Symbolic Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Lemma Target",
        "",
        f"- `{payload['lemma_id']}`",
        f"- {payload['target_theorem_goal']}",
        "",
        "## Assembly Rule",
        "",
        payload["symbolic_draft"]["assembly_rule"],
        "",
        "## Target Constraints",
        "",
        f"- `A_E2_max_required = {tc['A_E2_max_required']}`",
        f"- `C_E2_min_chain_compatibility = {tc['C_E2_min_chain_compatibility']}`",
        f"- {payload['symbolic_draft']['closure_condition']}",
        "",
        "## Remaining To Formalize",
        "",
    ]
    for x in payload["remaining_to_formalize"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
