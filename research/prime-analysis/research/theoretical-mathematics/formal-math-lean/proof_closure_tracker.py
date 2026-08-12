#!/usr/bin/env python3
"""Proof-closure tracker for Lemma A-E + A1-A4 theorem bridge."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def read_optional_json(path: str) -> Optional[Dict[str, Any]]:
    p = Path(path)
    if not p.exists():
        return None
    return json.loads(p.read_text(encoding="utf-8"))


def resolve_path(
    cli_value: Optional[str],
    canonical: Optional[Dict[str, Any]],
    key: str,
    fallback: str,
) -> str:
    if cli_value:
        return cli_value
    if canonical is not None:
        v = canonical.get(key)
        if isinstance(v, str) and v.strip():
            return v
    return fallback


def status(validated: bool, analytic_proved: bool = False) -> str:
    if analytic_proved:
        return "analytic_proved"
    if validated:
        return "validated_on_grid"
    return "blocked"


def main() -> None:
    ap = argparse.ArgumentParser(description="Build proof-closure status from existing artifacts.")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--lemma-a", default=None)
    ap.add_argument("--lemma-b", default=None)
    ap.add_argument("--lemma-c-triangle", default=None)
    ap.add_argument("--lemma-c-ineq", default=None)
    ap.add_argument("--lemma-d", default=None)
    ap.add_argument("--lemma-e", default=None)
    ap.add_argument("--theorem-pack", default=None)
    ap.add_argument("--draft", default=None)
    ap.add_argument("--skeleton", default=None)
    ap.add_argument("--output", default=None)
    args = ap.parse_args()

    manifest = read_optional_json(args.canonical_manifest)
    canonical = manifest.get("canonical", {}) if isinstance(manifest, dict) else None

    lemma_a = resolve_path(
        args.lemma_a,
        canonical,
        "lemma_a",
        "research/output/lemma_a_channel_derivation_check_none_z64_n1m.json",
    )
    lemma_b = resolve_path(
        args.lemma_b,
        canonical,
        "lemma_b",
        "research/output/lemma_b_truncation_program_gauss100_ref512.json",
    )
    lemma_c_triangle = resolve_path(
        args.lemma_c_triangle,
        canonical,
        "lemma_c_triangle",
        "research/output/lemma_c_triangle_transfer_refresh_2026-02-17.json",
    )
    lemma_c_ineq = resolve_path(
        args.lemma_c_ineq,
        canonical,
        "lemma_c_ineq",
        "research/output/lemma_c_inequality_probe_refresh_2026-02-17.json",
    )
    lemma_d = resolve_path(
        args.lemma_d,
        canonical,
        "lemma_d",
        "research/output/lemma_d_base_uniformity_probe_refresh_2026-02-17.json",
    )
    lemma_e = resolve_path(
        args.lemma_e,
        canonical,
        "lemma_e",
        "research/output/lemma_e_endpoint_probe_refresh_2026-02-17.json",
    )
    theorem_pack = resolve_path(
        args.theorem_pack,
        canonical,
        "theorem_pack",
        "research/output/uplift_theorem_pack_refresh_2026-02-17.json",
    )
    draft = resolve_path(
        args.draft,
        canonical,
        "draft",
        "research/output/rh_assumption_theorem_draft.md",
    )
    skeleton = resolve_path(
        args.skeleton,
        canonical,
        "proof_skeleton",
        "research/output/rh_bridge_candidate_b_proof_skeleton.md",
    )
    output = resolve_path(
        args.output,
        canonical,
        "proof_closure_tracker",
        "research/output/proof_closure_tracker_2026-02-17.json",
    )

    a = read_json(lemma_a)
    b = read_json(lemma_b)
    c_tri = read_json(lemma_c_triangle)
    c_in = read_json(lemma_c_ineq)
    d = read_json(lemma_d)
    e = read_json(lemma_e)
    pack = read_json(theorem_pack)

    a_ok = max(float(v) for v in a["worst_case_max_abs_diff"].values()) <= 1e-9
    b_ok = bool(b.get("all_monotone_nonincreasing", False))
    c_tri_hold_key = "holds" if "holds" in c_tri["inequality"] else "holds_on_test_grid"
    c_tri_gap_key = (
        "max_gap_lhs_minus_rhs"
        if "max_gap_lhs_minus_rhs" in c_tri["inequality"]
        else "global_max_gap_res_minus_rhs"
    )
    c_in_hold_key = "holds" if "holds" in c_in["inequality_check"] else "holds_on_test_grid"
    c_in_gap_key = (
        "max_gap_lhs_minus_rhs"
        if "max_gap_lhs_minus_rhs" in c_in["inequality_check"]
        else "global_max_gap_res_minus_rhs"
    )
    c_tri_ok = bool(c_tri["inequality"][c_tri_hold_key])
    c_in_ok = bool(c_in["inequality_check"][c_in_hold_key])
    d_hold_key = "holds" if "holds" in d["uniformity"] else "holds_uniform_triangle_on_grid"
    d_ok = bool(d["uniformity"][d_hold_key])
    e_ok = bool(e["rh_endpoint_indicator"]["residual_subpoly_support"]) and bool(
        e["rh_endpoint_indicator"]["triangle_subpoly_support"]
    )
    a1a4_ok = bool(pack["checks"]["unified_pack_holds"])

    obligations: List[Dict[str, Any]] = [
        {
            "id": "O1",
            "title": "A1 residual bound analytic uplift",
            "depends_on": ["Lemma C", "Lemma D"],
            "description": "Prove uniform C0 bound for all x>=x0 (not only sampled windows).",
            "state": "open",
        },
        {
            "id": "O2",
            "title": "A2 infinite-tail truncation proof",
            "depends_on": ["Lemma B"],
            "description": "Replace finite-grid tau(M) calibration with explicit asymptotic majorant tau_infty(M).",
            "state": "open",
        },
        {
            "id": "O3",
            "title": "A3 bridge growth/offdiag analytic closure",
            "depends_on": ["Lemma A", "Lemma B"],
            "description": "Derive C_H(log x)^A_H bound analytically with sign-sensitive control where needed.",
            "state": "open",
        },
        {
            "id": "O4",
            "title": "A4 base-uniform asymptotic constants",
            "depends_on": ["A1", "A2", "A3", "Lemma D"],
            "description": "Prove constants are wheel-family uniform asymptotically, not just on tested grids.",
            "state": "open",
        },
        {
            "id": "O5",
            "title": "RH-equivalent endpoint implication",
            "depends_on": ["A1", "A2", "A3", "A4", "Lemma E"],
            "description": "Map final bound to a standard RH-equivalent theorem with explicit hypotheses.",
            "state": "open",
        },
    ]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "lemma_a": lemma_a,
            "lemma_b": lemma_b,
            "lemma_c_triangle": lemma_c_triangle,
            "lemma_c_ineq": lemma_c_ineq,
            "lemma_d": lemma_d,
            "lemma_e": lemma_e,
            "theorem_pack": theorem_pack,
            "draft": draft,
            "skeleton": skeleton,
            "canonical_manifest": args.canonical_manifest,
        },
        "closure_status": {
            "Lemma A": {
                "status": status(a_ok),
                "evidence": {"max_channel_identity_diff": max(float(v) for v in a["worst_case_max_abs_diff"].values())},
            },
            "Lemma B": {
                "status": status(b_ok),
                "evidence": {
                    "all_monotone_nonincreasing": bool(b.get("all_monotone_nonincreasing", False)),
                    "global_worst_max_abs_diff_by_M": b.get("global_worst_max_abs_diff_by_M", {}),
                },
            },
            "Lemma C (triangle)": {
                "status": status(c_tri_ok),
                "evidence": {
                    "holds": bool(c_tri["inequality"][c_tri_hold_key]),
                    "max_gap_lhs_minus_rhs": float(c_tri["inequality"][c_tri_gap_key]),
                },
            },
            "Lemma C (inequality)": {
                "status": status(c_in_ok),
                "evidence": {
                    "holds": bool(c_in["inequality_check"][c_in_hold_key]),
                    "max_gap_lhs_minus_rhs": float(c_in["inequality_check"][c_in_gap_key]),
                },
            },
            "Lemma D": {
                "status": status(d_ok),
                "evidence": {
                    "holds": bool(d["uniformity"][d_hold_key]),
                    "uniformity_score": float(d["uniformity"]["uniformity_score"]),
                },
            },
            "Lemma E (indicator)": {
                "status": status(e_ok),
                "evidence": {
                    "A_residual_best_cv": float(e["rh_endpoint_indicator"]["A_residual_best_cv"]),
                    "A_rhs_best_cv": float(e["rh_endpoint_indicator"]["A_rhs_best_cv"]),
                },
            },
            "A1-A4 Unified Pack": {
                "status": status(a1a4_ok),
                "evidence": {
                    "unified_pack_holds": bool(pack["checks"]["unified_pack_holds"]),
                    "unified_ratio_max": float(pack["checks"]["unified_ratio_max"]),
                    "unified_gap_min": float(pack["checks"]["unified_gap_min"]),
                },
            },
        },
        "remaining_obligations": obligations,
        "overall_assessment": {
            "stage": "assumption_to_theorem_bridge",
            "summary": (
                "Lemma and A1-A4 stacks are validated on tested grids; "
                "remaining work is analytic/asymptotic closure and RH-equivalent implication."
            ),
        },
    }

    out = Path(output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# Proof Closure Tracker",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Closure Status",
        "",
        "| Item | Status | Key Evidence |",
        "|---|---|---|",
    ]
    for k, v in payload["closure_status"].items():
        ev = v["evidence"]
        if "unified_ratio_max" in ev:
            key = f"ratio_max={ev['unified_ratio_max']:.12g}"
        elif "max_channel_identity_diff" in ev:
            key = f"max_diff={ev['max_channel_identity_diff']:.3e}"
        elif "uniformity_score" in ev:
            key = f"score={ev['uniformity_score']:.6g}"
        elif "max_gap_lhs_minus_rhs" in ev:
            key = f"max_gap={ev['max_gap_lhs_minus_rhs']:.3e}"
        elif "A_residual_best_cv" in ev:
            key = f"A_res={ev['A_residual_best_cv']}, A_rhs={ev['A_rhs_best_cv']}"
        else:
            key = "see json"
        lines.append(f"| {k} | {v['status']} | {key} |")

    lines += ["", "## Remaining Proof Obligations", ""]
    for ob in payload["remaining_obligations"]:
        lines.append(f"- `{ob['id']}` {ob['title']}: {ob['description']}")
        lines.append(f"  depends_on={', '.join(ob['depends_on'])}; state={ob['state']}")

    lines += [
        "",
        "## Overall",
        "",
        payload["overall_assessment"]["summary"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
