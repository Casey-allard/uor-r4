#!/usr/bin/env python3
"""Generate strict O1-O5 proof requirement closure table."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _exists(path: str) -> bool:
    return Path(path).exists()


def _status_interface(path: str, key_path: List[str], expected: str) -> bool:
    if not _exists(path):
        return False
    row: Any = _read_json(path)
    for k in key_path:
        if not isinstance(row, dict) or k not in row:
            return False
        row = row[k]
    return row == expected


def main() -> None:
    ap = argparse.ArgumentParser(description="Build strict proof requirement closure table")
    ap.add_argument("--o1-handoff", default="research/output/o1_u_residual_o1_handoff_2026-02-17.json")
    ap.add_argument("--o1-theorem-pack", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_unconditionalization_pack_2026-02-17.json")
    ap.add_argument("--o2-theorem-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o3-direct-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--o3-l-offabs", default="research/output/o3_l_offabs_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-offabs-theorem", default="research/output/o3_l_offabs_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-offsign", default="research/output/o3_l_offsign_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-offsign-theorem", default="research/output/o3_l_offsign_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-diag", default="research/output/o3_l_diag_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-diag-theorem", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-asm", default="research/output/o3_l_asm_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-asm-theorem", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    ap.add_argument("--o4-handoff", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    ap.add_argument("--o4-theorem-pack", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o5-integrated", default="research/output/o5_integrated_implication_draft_2026-02-17.json")
    ap.add_argument("--o5-theorem-closure", default="research/output/o5_theorem_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/proof_requirement_closure_table_2026-02-17.json")
    args = ap.parse_args()

    o1_iface = _status_interface(args.o1_handoff, ["u_o1_residual_handoff", "status"], "conditional_closed_via_o1_interface")
    o1_theorem = _status_interface(args.o1_theorem_pack, ["status"], "theorem_closed_pack")
    o2_iface = _status_interface(args.o2_pack, ["o2_theorem_unconditionalization_pack", "status"], "theorem_draft_strengthened")
    o2_theorem = _status_interface(args.o2_theorem_pack, ["status"], "theorem_closed_pack")
    o3_pack = _status_interface(args.o3_direct_pack, ["o3_direct_envelope_derivation_pack", "status"], "theorem_draft_ready_for_unconditionalization")
    o3_l_offabs_started = _status_interface(args.o3_l_offabs, ["status"], "direct_theorem_draft_started")
    o3_l_offabs_theorem = _status_interface(args.o3_l_offabs_theorem, ["status"], "theorem_closed")
    o3_l_offsign_started = _status_interface(args.o3_l_offsign, ["status"], "direct_theorem_draft_started")
    o3_l_offsign_theorem = _status_interface(args.o3_l_offsign_theorem, ["status"], "theorem_closed")
    o3_l_diag_started = _status_interface(args.o3_l_diag, ["status"], "direct_theorem_draft_started")
    o3_l_diag_theorem = _status_interface(args.o3_l_diag_theorem, ["status"], "theorem_closed")
    o3_l_asm_started = _status_interface(args.o3_l_asm, ["status"], "direct_theorem_draft_started")
    o3_l_asm_theorem = _status_interface(args.o3_l_asm_theorem, ["status"], "theorem_closed")
    o4_iface = _status_interface(args.o4_handoff, ["u_uniformity_handoff", "status"], "unconditional_closed_via_o4_interface")
    o4_theorem = _status_interface(args.o4_theorem_pack, ["status"], "theorem_closed_pack")
    o5_iface = _exists(args.o5_integrated)
    o5_theorem = _status_interface(args.o5_theorem_closure, ["status"], "theorem_closed")

    reqs: List[Dict[str, Any]] = [
        {
            "id": "O1-R1",
            "obligation": "O1",
            "requirement": "Prove smoothing term asymptotically (not grid-calibrated).",
            "interface_status": "closed" if o1_iface else "open",
            "theorem_status": "closed" if o1_theorem else "open",
            "closure_criterion": "Formal asymptotic smoothing inequality with explicit constants for all x>=x0 and wheel family.",
            "evidence": [args.o1_theorem_pack] if o1_theorem else ([args.o1_handoff] if o1_iface else []),
        },
        {
            "id": "O1-R2",
            "obligation": "O1",
            "requirement": "Prove link term asymptotically (not grid-calibrated).",
            "interface_status": "closed" if o1_iface else "open",
            "theorem_status": "closed" if o1_theorem else "open",
            "closure_criterion": "Formal asymptotic link-term bound independent of sampled finite windows.",
            "evidence": [args.o1_theorem_pack] if o1_theorem else ([args.o1_handoff] if o1_iface else []),
        },
        {
            "id": "O1-R3",
            "obligation": "O1",
            "requirement": "Prove wheel-uniform C0_ref constants for all W.",
            "interface_status": "closed" if o1_iface else "open",
            "theorem_status": "closed" if o1_theorem else "open",
            "closure_criterion": "Single uniform constant pack proven for W in {30,210,2310,30030}.",
            "evidence": [args.o1_theorem_pack] if o1_theorem else ([args.o1_handoff] if o1_iface else []),
        },
        {
            "id": "O2-R1",
            "obligation": "O2",
            "requirement": "Theorem-side explicit zero-count bound justification.",
            "interface_status": "closed" if o2_iface else "open",
            "theorem_status": "closed" if o2_theorem else "open",
            "closure_criterion": "Fully proved in-project proposition with citation-locked constants and applicability domain.",
            "evidence": [args.o2_theorem_pack] if o2_theorem else ([args.o2_pack] if o2_iface else []),
        },
        {
            "id": "O2-R2",
            "obligation": "O2",
            "requirement": "Formal sum-to-integral domination lemma.",
            "interface_status": "closed" if o2_iface else "open",
            "theorem_status": "closed" if o2_theorem else "open",
            "closure_criterion": "Asymptotic domination theorem replacing checker-only evidence.",
            "evidence": [args.o2_theorem_pack] if o2_theorem else ([args.o2_pack] if o2_iface else []),
        },
        {
            "id": "O2-R3",
            "obligation": "O2",
            "requirement": "Formal monotone-vanishing tau_infty(M) with explicit rate.",
            "interface_status": "closed" if o2_iface else "open",
            "theorem_status": "closed" if o2_theorem else "open",
            "closure_criterion": "Proved tau envelope and monotonicity theorem valid for all M>=M0.",
            "evidence": [args.o2_theorem_pack] if o2_theorem else ([args.o2_pack] if o2_iface else []),
        },
        {
            "id": "O3-R1",
            "obligation": "O3",
            "requirement": "Direct proof of offdiag absolute envelope (L-OFFABS).",
            "interface_status": "closed" if o3_l_offabs_theorem else ("in_progress" if o3_l_offabs_started else ("closed" if o3_pack else "open")),
            "theorem_status": "closed" if o3_l_offabs_theorem else "open",
            "closure_criterion": "Direct derivation from decomposition terms without interface assumption.",
            "evidence": [args.o3_l_offabs_theorem] if o3_l_offabs_theorem else ([args.o3_l_offabs] if o3_l_offabs_started else ([args.o3_direct_pack] if o3_pack else [])),
        },
        {
            "id": "O3-R2",
            "obligation": "O3",
            "requirement": "Direct proof of sign-sensitive offdiag reduction (L-OFFSIGN).",
            "interface_status": "closed" if o3_l_offsign_theorem else ("in_progress" if o3_l_offsign_started else ("closed" if o3_pack else "open")),
            "theorem_status": "closed" if o3_l_offsign_theorem else "open",
            "closure_criterion": "Direct sign-cancellation lemma from explicit caps and decomposition.",
            "evidence": [args.o3_l_offsign_theorem] if o3_l_offsign_theorem else ([args.o3_l_offsign] if o3_l_offsign_started else ([args.o3_direct_pack] if o3_pack else [])),
        },
        {
            "id": "O3-R3",
            "obligation": "O3",
            "requirement": "Direct proof of diagonal envelope (L-DIAG).",
            "interface_status": "closed" if o3_l_diag_theorem else ("in_progress" if o3_l_diag_started else ("closed" if o3_pack else "open")),
            "theorem_status": "closed" if o3_l_diag_theorem else "open",
            "closure_criterion": "Direct kernel/channel majorant without interface fallback.",
            "evidence": [args.o3_l_diag_theorem] if o3_l_diag_theorem else ([args.o3_l_diag] if o3_l_diag_started else ([args.o3_direct_pack] if o3_pack else [])),
        },
        {
            "id": "O3-R4",
            "obligation": "O3",
            "requirement": "Direct assembly lemma to final E2/x bound (L-ASM).",
            "interface_status": "closed" if o3_l_asm_theorem else ("in_progress" if o3_l_asm_started else ("closed" if o3_pack else "open")),
            "theorem_status": "closed" if o3_l_asm_theorem else "open",
            "closure_criterion": "Final O3 bridge bound derived from direct lemmas only.",
            "evidence": [args.o3_l_asm_theorem] if o3_l_asm_theorem else ([args.o3_l_asm] if o3_l_asm_started else ([args.o3_direct_pack] if o3_pack else [])),
        },
        {
            "id": "O4-R1",
            "obligation": "O4",
            "requirement": "Asymptotic wheel-uniform constants across all W.",
            "interface_status": "closed" if o4_iface else "open",
            "theorem_status": "closed" if o4_theorem else "open",
            "closure_criterion": "Uniform asymptotic proof, not finite-grid spread check.",
            "evidence": [args.o4_theorem_pack] if o4_theorem else ([args.o4_handoff] if o4_iface else []),
        },
        {
            "id": "O4-R2",
            "obligation": "O4",
            "requirement": "No hidden W-dependence in limits/constants.",
            "interface_status": "closed" if o4_iface else "open",
            "theorem_status": "closed" if o4_theorem else "open",
            "closure_criterion": "Explicit proof that constants are independent of W in the asymptotic regime.",
            "evidence": [args.o4_theorem_pack] if o4_theorem else ([args.o4_handoff] if o4_iface else []),
        },
        {
            "id": "O4-R3",
            "obligation": "O4",
            "requirement": "Common x0 valid simultaneously for all wheel bases.",
            "interface_status": "closed" if o4_iface else "open",
            "theorem_status": "closed" if o4_theorem else "open",
            "closure_criterion": "Single threshold theorem for all W in wheel family.",
            "evidence": [args.o4_theorem_pack] if o4_theorem else ([args.o4_handoff] if o4_iface else []),
        },
        {
            "id": "O5-R1",
            "obligation": "O5",
            "requirement": "Final RH-equivalent implication with only unconditional inputs.",
            "interface_status": "closed" if o5_iface else "open",
            "theorem_status": "closed" if o5_theorem else "open",
            "closure_criterion": "Final theorem statement using theorem_closed O1/O3/O4 requirements only.",
            "evidence": [args.o5_theorem_closure] if o5_theorem else ([args.o5_integrated] if o5_iface else []),
        },
    ]

    per_obligation: Dict[str, Dict[str, int]] = {}
    for row in reqs:
        ob = row["obligation"]
        if ob not in per_obligation:
            per_obligation[ob] = {
                "total": 0,
                "interface_closed": 0,
                "theorem_closed": 0,
                "theorem_open": 0,
            }
        per_obligation[ob]["total"] += 1
        if row["interface_status"] == "closed":
            per_obligation[ob]["interface_closed"] += 1
        if row["theorem_status"] == "closed":
            per_obligation[ob]["theorem_closed"] += 1
        else:
            per_obligation[ob]["theorem_open"] += 1

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "policy": {
            "fixed_requirement_set": True,
            "no_new_taxonomy": True,
            "progress_metric": "theorem_closed_count_only",
        },
        "requirements": reqs,
        "summary_by_obligation": per_obligation,
        "global_summary": {
            "total_requirements": len(reqs),
            "theorem_closed_total": sum(1 for r in reqs if r["theorem_status"] == "closed"),
            "theorem_open_total": sum(1 for r in reqs if r["theorem_status"] != "closed"),
            "interface_closed_total": sum(1 for r in reqs if r["interface_status"] == "closed"),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# Proof Requirement Closure Table",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Policy",
        "",
        "- Fixed O1-O5 requirement set only.",
        "- No new taxonomy names.",
        "- Progress measured by theorem-closed requirements, not event counts.",
        "",
        "## Global Summary",
        "",
        f"- total requirements: `{payload['global_summary']['total_requirements']}`",
        f"- theorem closed: `{payload['global_summary']['theorem_closed_total']}`",
        f"- theorem open: `{payload['global_summary']['theorem_open_total']}`",
        f"- interface closed: `{payload['global_summary']['interface_closed_total']}`",
        "",
        "## Requirements",
        "",
        "| id | O | interface | theorem | requirement |",
        "|---|---|---|---|---|",
    ]
    for r in reqs:
        lines.append(f"| {r['id']} | {r['obligation']} | {r['interface_status']} | {r['theorem_status']} | {r['requirement']} |")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
