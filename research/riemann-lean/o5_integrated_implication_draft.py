#!/usr/bin/env python3
"""Create integrated O5 implication draft from O1/O3/O5 scaffolds."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build integrated O5 implication draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o1", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o1-handoff", default="research/output/o1_u_residual_o1_handoff_2026-02-17.json")
    ap.add_argument("--o3", default="research/output/o3_ref_bridge_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o3-derived-bridge", default="research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json")
    ap.add_argument("--o3-unconditionalization-contract", default="research/output/o3_unconditionalization_contract_2026-02-17.json")
    ap.add_argument("--o3-direct-envelope-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--o3-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--o4-handoff", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    ap.add_argument("--proof-requirement-table", default="research/output/proof_requirement_closure_table_2026-02-17.json")
    ap.add_argument("--o5-mref", default="research/output/o5_mref_transfer_lemma_scaffold_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o5_integrated_implication_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o1 = _read_json(args.o1)
    o1h = _read_json(args.o1_handoff) if Path(args.o1_handoff).exists() else {}
    o3 = _read_json(args.o3)
    o3d = _read_json(args.o3_derived_bridge) if Path(args.o3_derived_bridge).exists() else {}
    o3u = _read_json(args.o3_unconditionalization_contract) if Path(args.o3_unconditionalization_contract).exists() else {}
    o3dp = _read_json(args.o3_direct_envelope_pack) if Path(args.o3_direct_envelope_pack).exists() else {}
    o3b = _read_json(args.o3_budget) if Path(args.o3_budget).exists() else {}
    o3e2 = _read_json(args.o3_e2_full) if Path(args.o3_e2_full).exists() else {}
    o4h = _read_json(args.o4_handoff) if Path(args.o4_handoff).exists() else {}
    reqt = _read_json(args.proof_requirement_table) if Path(args.proof_requirement_table).exists() else {}
    o5m = _read_json(args.o5_mref)

    c1 = o1["lemma_o1_a1_ref"]["current_constant_candidates"]
    c3 = o3["lemma_o3_a3_ref"]["current_constant_candidates"]
    c5 = o5m["lemma_o5_mref_transfer"]["constant_pack"]
    tgt = o5m["lemma_o5_mref_transfer"]["rh_endpoint_target"]["target_exponent"]
    derived_row = o3d.get("derived_o3_bridge", {}).get("derived_constants", {}) if o3d else {}
    using_derived = bool(derived_row)
    a_h = float(derived_row["A_H_derived"]) if using_derived else float(c3["A_H"])
    c_h = float(derived_row["C_H_derived"]) if using_derived else float(c3["C_H"])
    c_h_theorem_budget = (
        float(o3b["o3_theorem_budget"]["C_H_theorem_budget"])
        if o3b
        else None
    )
    o3e2_closed = bool(o3e2) and bool(o3e2.get("closure_status", {}).get("proof_complete", False))
    o3e2_assume = o3e2.get("explicit_assumptions", {}).get("glue_closure", {}) if o3e2 else {}
    o1_interface_closed = bool(o1h) and o1h.get("u_o1_residual_handoff", {}).get("status") == "conditional_closed_via_o1_interface"
    o4_interface_closed = bool(o4h) and o4h.get("u_uniformity_handoff", {}).get("status") == "unconditional_closed_via_o4_interface"

    theorem = (
        "Assume A1_ref and A3_ref hold asymptotically with wheel-uniform constants at M_ref. "
        "Then for all x>=x0 and W in {30,210,2310,30030}, "
        "|E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref| C_H (log x)^A_H. "
        "Hence |E(x)| <= C sqrt(x) (log x)^A with A=max(A_H,0)."
    )

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o1_scaffold": args.o1,
            "o1_handoff": args.o1_handoff if o1h else "",
            "o3_scaffold": args.o3,
            "o3_derived_bridge": args.o3_derived_bridge if o3d else "",
            "o3_unconditionalization_contract": args.o3_unconditionalization_contract if o3u else "",
            "o3_direct_envelope_pack": args.o3_direct_envelope_pack if o3dp else "",
            "o3_budget": args.o3_budget if o3b else "",
            "o3_e2_full": args.o3_e2_full if o3e2 else "",
            "o4_handoff": args.o4_handoff if o4h else "",
            "proof_requirement_table": args.proof_requirement_table if reqt else "",
            "o5_mref_scaffold": args.o5_mref,
        },
        "integrated_o5_theorem": {
            "statement": theorem,
            "constants_snapshot": {
                "a_ref": float(c5["a_ref"]),
                "b_ref": float(c5["b_ref"]),
                "C0_ref_from_o5": float(c5["C0_ref"]),
                "C0_uplifted_from_o1": float(c1["C0_uplifted_sum"]),
                "C_H": c_h,
                "C_H_theorem_budget": c_h_theorem_budget,
                "A_H": a_h,
                "A_E2_if_closed": o3e2_assume.get("A_E2_if_closed"),
                "C_E2_if_closed": o3e2_assume.get("C_E2_if_closed"),
                "H_constants_source": "o3_derived_bridge_from_e2" if using_derived else "o3_ref_scaffold",
            },
            "target_endpoint_exponent": float(tgt),
            "meets_target_exponent_class": bool(a_h <= float(tgt)),
            "exponent_gap_AH_minus_target": a_h - float(tgt),
        },
        "assumption_discharge_status": [
            {
                "assumption": "A1_ref",
                "obligation": "O1",
                "status": "conditional_residual_interface_closed" if o1_interface_closed else "open_theorem_work",
                "artifact": args.o1_handoff if o1_interface_closed else args.o1,
            },
            {
                "assumption": "A3_ref",
                "obligation": "O3",
                "status": "conditional_bridge_derived" if using_derived else ("conditional_e2_chain_closed" if o3e2_closed else "open_theorem_work"),
                "artifact": args.o3_derived_bridge if using_derived else (args.o3_e2_full if o3e2_closed else args.o3),
            },
            {
                "assumption": "wheel_uniformity",
                "obligation": "O4",
                "status": "conditional_uniformity_interface_closed" if o4_interface_closed else "open_theorem_work",
                "artifact": args.o4_handoff if o4_interface_closed else "research/output/a4_uniform_assumption_check_refresh_2026-02-17.json",
            },
        ],
        "next_blocker_order": (
            [f"Complete {x} from O3 unconditionalization contract." for x in o3u.get("o3_unconditionalization_contract", {}).get("dependency_order", [])]
            if o3u
            else ["Unconditionalize O3 E2 component envelopes inside A3_ref (remove conditional assumptions)."]
        ) + ([] if o1_interface_closed else [
            "Discharge O1 asymptotically (A1_ref theorem constants).",
        ]) + ([] if o4_interface_closed else [
            "Discharge O4 asymptotic wheel-uniformity and finalize RH-equivalent implication statement.",
        ]) + ([
            "Write direct O3 envelope lemmas (L-OFFABS/L-OFFSIGN/L-DIAG/L-ASM) to replace interface-level closures."
        ] if o3dp else []),
    }

    if reqt:
        open_reqs = [r for r in reqt.get("requirements", []) if r.get("theorem_status") != "closed"]
        # Prefer explicit O3/O1/O4/O2/O5 ordering for proof closure sequencing.
        priority = {"O3": 0, "O1": 1, "O4": 2, "O2": 3, "O5": 4}
        open_reqs.sort(key=lambda r: (priority.get(r.get("obligation", "O5"), 9), r.get("id", "")))
        if open_reqs:
            payload["next_blocker_order"] = [
                f"{r['id']}: {r['requirement']}" for r in open_reqs
            ]
        else:
            payload["next_blocker_order"] = [
                "All strict theorem requirements are closed."
            ]

    if not payload["next_blocker_order"]:
        payload["next_blocker_order"] = [
            "Finalize unconditional theorem proofs for O1/O3/O4 interfaces and restate the RH-equivalent implication without interface assumptions."
        ]

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O5 Integrated Implication Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Integrated Theorem Statement",
        "",
        payload["integrated_o5_theorem"]["statement"],
        "",
        "## Constants Snapshot",
        "",
        f"- `a_ref = {payload['integrated_o5_theorem']['constants_snapshot']['a_ref']}`",
        f"- `b_ref = {payload['integrated_o5_theorem']['constants_snapshot']['b_ref']}`",
        f"- `C0_ref (o5) = {payload['integrated_o5_theorem']['constants_snapshot']['C0_ref_from_o5']}`",
        f"- `C0_uplifted (o1) = {payload['integrated_o5_theorem']['constants_snapshot']['C0_uplifted_from_o1']}`",
        f"- `C_H = {payload['integrated_o5_theorem']['constants_snapshot']['C_H']}`",
        f"- `C_H_theorem_budget = {payload['integrated_o5_theorem']['constants_snapshot']['C_H_theorem_budget']}`",
        f"- `A_H = {payload['integrated_o5_theorem']['constants_snapshot']['A_H']}`",
        f"- `H constants source = {payload['integrated_o5_theorem']['constants_snapshot']['H_constants_source']}`",
        "",
        "## Endpoint Class Check",
        "",
        f"- target exponent: `{payload['integrated_o5_theorem']['target_endpoint_exponent']}`",
        f"- meets class now: `{payload['integrated_o5_theorem']['meets_target_exponent_class']}`",
        f"- gap `A_H-target`: `{payload['integrated_o5_theorem']['exponent_gap_AH_minus_target']}`",
        "",
        "## Assumption Discharge Status",
        "",
        "| assumption | obligation | status | artifact |",
        "|---|---|---|---|",
    ]
    for row in payload["assumption_discharge_status"]:
        lines.append(f"| {row['assumption']} | {row['obligation']} | {row['status']} | `{row['artifact']}` |")
    lines += [
        "",
        "## Next Blocker Order",
        "",
    ]
    for x in payload["next_blocker_order"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
