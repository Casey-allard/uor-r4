#!/usr/bin/env python3
"""Build a full proof-facing draft for O3 E2/x asymptotic closure."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Create full O3 E2 proof draft artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-lemma", default="research/output/o3_e2_asymptotic_lemma_scaffold_2026-02-17.json")
    ap.add_argument("--o3-target", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--o3-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-e2-rem-closure", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-glue-closure", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    lemma = _read_json(args.o3_lemma)["lemma_o3_e2_asymptotic"]
    target = _read_json(args.o3_target)["o3_e2_target_pack"]
    sign = _read_json(args.o3_sign)["theorem_assumption_candidates"]
    budget = _read_json(args.o3_budget)["o3_theorem_budget"]
    rem_closure: Dict[str, Any] = {}
    rem_closed = False
    rem_path = Path(args.o3_e2_rem_closure)
    if rem_path.exists():
        rem_closure = _read_json(args.o3_e2_rem_closure)
        rem_closed = rem_closure.get("status") == "conditional_asymptotic_closed"
    offdiag_closure: Dict[str, Any] = {}
    offdiag_closed = False
    offdiag_path = Path(args.o3_e2_offdiag_closure)
    if offdiag_path.exists():
        offdiag_closure = _read_json(args.o3_e2_offdiag_closure)
        offdiag_closed = (
            offdiag_closure.get("status") == "conditional_asymptotic_closed"
            and bool(offdiag_closure.get("derived_offdiag_bound", {}).get("target_compatible", False))
        )
    diag_closure: Dict[str, Any] = {}
    diag_closed = False
    diag_path = Path(args.o3_e2_diag_closure)
    if diag_path.exists():
        diag_closure = _read_json(args.o3_e2_diag_closure)
        diag_closed = (
            diag_closure.get("status") == "conditional_asymptotic_closed"
            and bool(diag_closure.get("derived_diag_bound", {}).get("target_compatible", False))
        )
    glue_closure: Dict[str, Any] = {}
    glue_closed = False
    glue_path = Path(args.o3_e2_glue_closure)
    if glue_path.exists():
        glue_closure = _read_json(args.o3_e2_glue_closure)
        glue_closed = (
            glue_closure.get("status") == "conditional_asymptotic_closed"
            and bool(glue_closure.get("target_check", {}).get("A_E2_target_compatible", False))
        )

    a_e2_max = float(lemma["target_thresholds"]["A_E2_max"])
    c_e2_min = float(lemma["target_thresholds"]["C_E2_min_chain_compatibility"])
    eps_sign = float(sign["eps_sign"])
    neg_cap = float(sign["neg_over_abs_cap"])
    c_eta = float(budget["C_eta_theorem_budget"])
    a_eta = float(budget["A_eta"])

    unresolved = [
        {
            "id": "E2-DIAG",
            "need": "Asymptotic diagonal bound with explicit exponent A_diag.",
        },
        {
            "id": "E2-OFFDIAG-SIGN",
            "need": "Asymptotic sign-sensitive offdiag inequality meeting eps_sign/neg_over_abs caps.",
        },
        {
            "id": "E2-REM",
            "need": "Uniform remainder majorant for all x>=x0 and W in wheel family.",
        },
        {
            "id": "E2-GLUE",
            "need": "Assembly lemma yielding A_E2<=A_E2_max and explicit C_E2.",
        },
    ]
    if rem_closed:
        unresolved = [x for x in unresolved if x["id"] != "E2-REM"]
    if offdiag_closed:
        unresolved = [x for x in unresolved if x["id"] != "E2-OFFDIAG-SIGN"]
    if diag_closed:
        unresolved = [x for x in unresolved if x["id"] != "E2-DIAG"]
    if glue_closed:
        unresolved = [x for x in unresolved if x["id"] != "E2-GLUE"]

    next_lemma = unresolved[0]["id"] if unresolved else "none"
    proof_complete = len(unresolved) == 0

    draft: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_lemma": args.o3_lemma,
            "o3_target": args.o3_target,
            "o3_sign": args.o3_sign,
            "o3_budget": args.o3_budget,
            "o3_e2_rem_closure": args.o3_e2_rem_closure,
            "o3_e2_offdiag_closure": args.o3_e2_offdiag_closure,
            "o3_e2_diag_closure": args.o3_e2_diag_closure,
            "o3_e2_glue_closure": args.o3_e2_glue_closure,
        },
        "theorem_goal": {
            "name": "O3 E2/x Asymptotic Lemma (full draft)",
            "statement": lemma["quantified_statement"],
            "required_thresholds": {
                "A_E2_max": a_e2_max,
                "C_E2_min_chain_compatibility": c_e2_min,
            },
        },
        "proof_outline": {
            "step_1_decomposition": (
                "Write E2/x as diagonal + offdiag + remainder with explicit indexing over wheel-filtered sequence."
            ),
            "step_2_diagonal_bound": (
                "Prove diagonal term <= C_diag (log x)^A_diag using channel smoothness and finite-zero truncation structure."
            ),
            "step_3_offdiag_bound": (
                "Use sign-sensitive caps eps_sign and neg_over_abs_cap to bound offdiag without full absolute-value loss."
            ),
            "step_4_remainder_bound": (
                "Bound truncation/remainder terms uniformly in W by explicit asymptotic majorants."
            ),
            "step_5_assemble": (
                "Set A_E2 = max(A_diag, A_offdiag, A_rem) and C_E2 = C_diag + C_offdiag + C_rem, verify A_E2<=A_E2_max."
            ),
            "step_6_o3_link": (
                "Combine with eta_+(x)<=C_eta(log x)^A_eta and |H|<=sqrt((1+eta_+)E2/x) to close O3 bridge bound."
            ),
        },
        "explicit_assumptions": {
            "sign_sensitive_caps": {
                "eps_sign": eps_sign,
                "neg_over_abs_cap": neg_cap,
                "template": sign["assumption_template"],
            },
            "eta_budget": {
                "A_eta": a_eta,
                "C_eta_budget": c_eta,
            },
            "remainder_closure": {
                "present": rem_path.exists(),
                "status": rem_closure.get("status", "missing"),
                "A_rem_if_closed": rem_closure.get("asymptotic_closure", {}).get("A_rem"),
                "C_rem_if_closed": rem_closure.get("asymptotic_closure", {}).get("C_rem_uniform"),
            },
            "offdiag_closure": {
                "present": offdiag_path.exists(),
                "status": offdiag_closure.get("status", "missing"),
                "A_offdiag_if_closed": offdiag_closure.get("derived_offdiag_bound", {}).get("A_offdiag"),
                "C_offdiag_if_closed": offdiag_closure.get("derived_offdiag_bound", {}).get("C_offdiag"),
                "target_compatible_if_closed": offdiag_closure.get("derived_offdiag_bound", {}).get("target_compatible"),
            },
            "diag_closure": {
                "present": diag_path.exists(),
                "status": diag_closure.get("status", "missing"),
                "A_diag_if_closed": diag_closure.get("derived_diag_bound", {}).get("A_diag"),
                "C_diag_if_closed": diag_closure.get("derived_diag_bound", {}).get("C_diag"),
                "target_compatible_if_closed": diag_closure.get("derived_diag_bound", {}).get("target_compatible"),
            },
            "glue_closure": {
                "present": glue_path.exists(),
                "status": glue_closure.get("status", "missing"),
                "A_E2_if_closed": glue_closure.get("assembly", {}).get("A_E2"),
                "C_E2_if_closed": glue_closure.get("assembly", {}).get("C_E2"),
                "target_compatible_if_closed": glue_closure.get("target_check", {}).get("A_E2_target_compatible"),
            },
        },
        "unresolved_lemmas": unresolved,
        "closure_status": {
            "proof_complete": proof_complete,
            "blocking_lemma_count": len(unresolved),
            "next_lemma_to_attack": next_lemma,
        },
        "next_action": (
            "Propagate closed E2 assembly into O3 bridge theorem statement and then advance O5 implication closure."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(draft, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Full Proof Draft",
        "",
        f"Generated: {draft['timestamp_utc']}",
        "",
        "## Theorem Goal",
        "",
        f"- {draft['theorem_goal']['statement']}",
        f"- `A_E2_max = {a_e2_max}`",
        f"- `C_E2_min_chain_compatibility = {c_e2_min}`",
        "",
        "## Proof Outline",
        "",
    ]
    for k in [
        "step_1_decomposition",
        "step_2_diagonal_bound",
        "step_3_offdiag_bound",
        "step_4_remainder_bound",
        "step_5_assemble",
        "step_6_o3_link",
    ]:
        lines.append(f"- {draft['proof_outline'][k]}")
    lines += [
        "",
        "## Explicit Assumptions",
        "",
        f"- `eps_sign = {eps_sign}`",
        f"- `neg_over_abs_cap = {neg_cap}`",
        f"- `A_eta = {a_eta}`",
        f"- `C_eta_budget = {c_eta}`",
        "",
        "## Unresolved Lemmas",
        "",
    ]
    for row in draft["unresolved_lemmas"]:
        lines.append(f"- `{row['id']}`: {row['need']}")
    lines += [
        "",
        "## Status",
        "",
        f"- proof complete: `{draft['closure_status']['proof_complete']}`",
        f"- blocking lemmas: `{draft['closure_status']['blocking_lemma_count']}`",
        f"- next lemma to attack: `{draft['closure_status']['next_lemma_to_attack']}`",
        "",
        draft["next_action"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
