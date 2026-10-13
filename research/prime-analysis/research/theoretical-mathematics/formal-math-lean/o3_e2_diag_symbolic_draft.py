#!/usr/bin/env python3
"""Draft symbolic diagonal bound for O3 E2 asymptotic proof chain."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build E2-DIAG symbolic draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--o3-target", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_diag_symbolic_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    full = _read_json(args.o3_e2_full)
    target = _read_json(args.o3_target)["o3_e2_target_pack"]

    a_e2_max = float(target["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    c_e2_min = float(target["e2_constant_target"]["C_E2_min_from_budget_chain"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_full": args.o3_e2_full,
            "o3_target": args.o3_target,
        },
        "lemma_id": "E2-DIAG",
        "target_theorem_goal": full["theorem_goal"]["statement"],
        "symbolic_draft": {
            "decomposition_slot": "diagonal contribution in E2/x decomposition",
            "proposed_inequality": "Diag(x;W) <= C_diag * (log x)^A_diag for all x>=x0 and W in wheel family.",
            "compatibility_conditions": {
                "A_diag_upper_target": a_e2_max,
                "C_diag_target_scale_hint": c_e2_min,
            },
            "interpretation": (
                "Diagonal bound should be established with explicit channel smoothness and wheel-density terms, "
                "kept separate from offdiag cancellation machinery."
            ),
        },
        "remaining_to_formalize": [
            "Write explicit formula for diagonal summand and its wheel-filtered normalization.",
            "Prove asymptotic majorant for diagonal term uniformly in W.",
            "Extract explicit C_diag, A_diag constants compatible with E2 target pack.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    cc = payload["symbolic_draft"]["compatibility_conditions"]
    lines = [
        "# O3 E2 Diagonal Symbolic Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Lemma Target",
        "",
        f"- `{payload['lemma_id']}`",
        f"- {payload['target_theorem_goal']}",
        "",
        "## Proposed Diagonal Inequality",
        "",
        payload["symbolic_draft"]["proposed_inequality"],
        "",
        "## Compatibility Conditions",
        "",
        f"- `A_diag_upper_target = {cc['A_diag_upper_target']}`",
        f"- `C_diag_target_scale_hint = {cc['C_diag_target_scale_hint']}`",
        "",
        payload["symbolic_draft"]["interpretation"],
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
