#!/usr/bin/env python3
"""Audit remaining formal blockers in Lean modules by extracting axioms."""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


AXIOM_RE = re.compile(r"^\s*axiom\s+([A-Za-z0-9_']+)")


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit remaining axioms in Lean scaffold")
    ap.add_argument(
        "--lean-files",
        default="research/formal/lean/PrimeRiemannBridge.lean,"
        "research/formal/lean/PrimeRiemannBridgeMathlib.lean,"
        "research/formal/lean/PrimeRiemannBridgeCompletionKernel.lean,"
        "research/formal/lean/PrimeRiemannBridgeImportedResults.lean,"
        "research/formal/lean/PrimeRiemannBridgeImportedInstance.lean,"
        "research/formal/lean/PrimeRiemannBridgeZeroOscillationProgram.lean,"
        "research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean,"
        "research/formal/lean/PrimeRiemannBridgeConcretePackInstantiation.lean,"
        "research/formal/lean/PrimeRiemannBridgeW2bImportedInstance.lean,"
        "research/formal/lean/PrimeRiemannBridgeW2bFinalSlot.lean,"
        "research/formal/lean/PrimeRiemannBridgeInghamImportedSlot.lean,"
        "research/formal/lean/PrimeRiemannBridgeSpinningTopFrontier.lean,"
        "research/formal/lean/PrimeRiemannBridgeNearStrictTailToPintz.lean,"
        "research/formal/lean/PrimeRiemannBridgeFinalTargetEquivalence.lean,"
        "research/formal/lean/PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean",
        help="Comma-separated Lean files to scan.",
    )
    ap.add_argument("--output-json", default="research/output/formal_axiom_audit_2026-02-17.json")
    ap.add_argument("--output-md", default="research/output/formal_axiom_audit_2026-02-17.md")
    ap.add_argument(
        "--proof-status-json",
        default="research/output/proof_resume_checkpoint_2026-02-18.json",
        help="Optional status snapshot with verified_truth.remaining_item_count.",
    )
    args = ap.parse_args()

    lean_files = [Path(p.strip()) for p in args.lean_files.split(",") if p.strip()]
    axioms: List[Dict[str, str]] = []
    for lean_file in lean_files:
        text = lean_file.read_text(encoding="utf-8")
        for i, line in enumerate(text.splitlines(), start=1):
            m = AXIOM_RE.match(line)
            if not m:
                continue
            axioms.append(
                {
                    "id": m.group(1),
                    "line": i,
                    "file": str(lean_file),
                    "status": "open_formal_blocker",
                }
            )

    # Fixed blocker taxonomy: no dynamic expansion.
    catalog = {
        "L0_log_sq_ge_one": "Prove analytic side condition log(x)^2 >= 1 for x >= e in Lean.",
        "L0_two_le_exp1": "Discharge numeric side condition 2 <= exp(1) in Lean.",
        "L1_contract_from_o1_o2_o4": "Replace artifact-import axiom with theorem deriving L1 contract from O1/O2/O4 formal statements.",
        "L2_contract_from_o3": "Replace artifact-import axiom with theorem deriving L2 contract from O3 formal statements.",
    }

    blockers: List[Dict[str, str]] = []
    for a in axioms:
        blockers.append(
            {
                "id": a["id"],
                "file": a["file"],
                "line": a["line"],
                "status": a["status"],
                "resolution_target": catalog.get(a["id"], "Resolve this axiom by formal theorem proof."),
            }
        )

    remaining_item_count = None
    proof_finished = None
    status_path = Path(args.proof_status_json)
    if status_path.exists():
        try:
            status_payload = json.loads(status_path.read_text(encoding="utf-8"))
            truth_block = status_payload.get("verified_truth")
            if not isinstance(truth_block, dict):
                truth_block = status_payload.get("truth", {})
            ric = truth_block.get("remaining_item_count")
            if isinstance(ric, int):
                remaining_item_count = ric
        except json.JSONDecodeError:
            remaining_item_count = None
    formal_finished = len(axioms) == 0
    if remaining_item_count is not None:
        proof_finished = formal_finished and remaining_item_count == 0

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "lean_files": [str(p) for p in lean_files],
        "policy": {
            "fixed_blocker_taxonomy": True,
            "no_goalpost_shift": True,
            "completion_rule": "proof_finished_when_axiom_count_is_zero_and_remaining_item_count_is_zero_if_available",
        },
        "summary": {
            "axiom_count": len(axioms),
            "formal_finished": formal_finished,
            "proof_remaining_item_count": remaining_item_count,
            "proof_finished": proof_finished,
        },
        "blockers": blockers,
    }

    out_json = Path(args.output_json)
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    lines = [
        "# Formal Axiom Audit",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- Lean files scanned: `{len(lean_files)}`",
        f"- Axiom blockers: `{payload['summary']['axiom_count']}`",
        f"- Formal finished (axioms): `{payload['summary']['formal_finished']}`",
        f"- Proof remaining items (from status): `{payload['summary']['proof_remaining_item_count']}`",
        f"- Proof finished: `{payload['summary']['proof_finished']}`",
        "",
        "## Blockers",
        "",
        "| id | file | line | status | resolution_target |",
        "|---|---|---:|---|---|",
    ]
    for b in blockers:
        lines.append(
            f"| {b['id']} | {b['file']} | {b['line']} | {b['status']} | {b['resolution_target']} |"
        )
    lines.append("")
    Path(args.output_md).write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": args.output_json, "md": args.output_md, "axiom_count": len(axioms)}, indent=2))


if __name__ == "__main__":
    main()
