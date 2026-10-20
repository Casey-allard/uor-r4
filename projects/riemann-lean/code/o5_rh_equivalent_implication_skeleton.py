#!/usr/bin/env python3
"""Build a proof-facing O5 skeleton for RH-equivalent endpoint implication."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _pick_a3_constants(a3: Dict[str, Any]) -> Dict[str, float]:
    if "h_transfer_envelope" in a3:
        h = a3["h_transfer_envelope"]
        return {
            "A_H": float(h["A_H"]),
            "C_H": float(h["C_H_from_density_transfer"]),
        }
    if "uplift_constants" in a3:
        u = a3["uplift_constants"]
        return {
            "A_H": float(u["A_H_from_normalized_sum"]),
            "C_H": float(u["C_H_from_normalized_sum"]),
        }
    raise KeyError("unrecognized A3 artifact schema")


def main() -> None:
    ap = argparse.ArgumentParser(description="Construct O5 RH-equivalent implication skeleton")
    ap.add_argument(
        "--canonical-manifest",
        default="research/output/proof_canonical_manifest.json",
    )
    ap.add_argument(
        "--a3",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument(
        "--a4",
        default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json",
    )
    ap.add_argument("--target-log-exponent", type=float, default=2.0)
    ap.add_argument(
        "--output",
        default="research/output/o5_rh_equivalent_implication_skeleton_2026-02-17.json",
    )
    args = ap.parse_args()

    manifest = _read_json(args.canonical_manifest)
    canonical = manifest.get("canonical", {})
    a3_path = args.a3 or canonical.get("a3", "")
    a4_path = args.a4 or canonical.get("a4", "")
    a3 = _read_json(a3_path)
    a4 = _read_json(a4_path)

    a3c = _pick_a3_constants(a3)
    a_h = float(a3c["A_H"])
    c_h = float(a3c["C_H"])
    target = float(args.target_log_exponent)

    uc = a4["uniform_constants"]
    a_ref = float(uc["a_ref"])
    b_ref = float(uc["b_ref"])
    c0_ref = float(uc["C0_ref"])
    m_ref = int(a4["config"]["m_ref"])

    theorem_statement = (
        "Assume, for x >= x0 and W in {30,210,2310,30030}, the M_ref transfer inequality "
        "|E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref||H_W^(M_ref)(x)|, and bridge growth "
        "|H_W^(M_ref)(x)| <= C_H (log x)^A_H. Then |E(x)| <= C sqrt(x) (log x)^A with "
        "A=max(A_H,0), C depending on C0_ref,b_ref,a_ref,C_H."
    )

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a3": a3_path,
            "a4": a4_path,
        },
        "fixed_endpoint_criterion": {
            "name": "von Koch / Chebyshev endpoint class",
            "target_form": "|psi(x)-x| <= C sqrt(x) (log x)^2",
            "target_log_exponent": target,
            "equivalence_anchor": "AIM RH equivalence index (article 95a)",
            "schoenfeld_anchor": "Math. Comp. 30 (1976), Theorem 10",
        },
        "mref_route": {
            "m_ref": m_ref,
            "delta_m_term": "identically zero at M=M_ref",
            "a_ref": a_ref,
            "b_ref": b_ref,
            "C0_ref": c0_ref,
            "A_H": a_h,
            "C_H": c_h,
            "meets_target_exponent_class": bool(a_h <= target),
            "exponent_gap_AH_minus_target": a_h - target,
        },
        "theorem_skeleton": {
            "statement": theorem_statement,
            "required_assumptions": [
                "A1_ref: theorem-level uniform bound for reference residual at M_ref.",
                "A3_ref: theorem-level bound |H_W^(M_ref)(x)| <= C_H (log x)^A_H.",
                "A4_uniform: constants are uniform over W in {30,210,2310,30030}.",
            ],
            "proof_outline": [
                "Start from deterministic affine transfer inequality at M_ref.",
                "Substitute A3_ref bound for bridge term.",
                "Collect additive constants into C and exponent into A.",
                "Compare obtained endpoint class against fixed RH-equivalent target form.",
            ],
        },
        "remaining_o5_blockers": [
            "Convert grid-validated M_ref transfer inequality into an asymptotic theorem with explicit hypotheses.",
            "Prove A3_ref asymptotically without empirical safety-factor dependence.",
            "Provide final implication write-up to a standard RH-equivalent statement.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O5 RH-Equivalent Implication Skeleton",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Fixed Endpoint Criterion",
        "",
        f"- `{payload['fixed_endpoint_criterion']['target_form']}`",
        f"- target exponent: `{target}`",
        f"- anchor: `{payload['fixed_endpoint_criterion']['equivalence_anchor']}`",
        f"- explicit-bound anchor: `{payload['fixed_endpoint_criterion']['schoenfeld_anchor']}`",
        "",
        "## Mref Route",
        "",
        f"- `m_ref={m_ref}`",
        "- `Delta_M=0` at `M=M_ref` (A2 exponent does not drive endpoint class here).",
        f"- `A_H={a_h}`, `C_H={c_h}`",
        f"- meets target exponent class: `{payload['mref_route']['meets_target_exponent_class']}`",
        f"- exponent gap `A_H-target`: `{payload['mref_route']['exponent_gap_AH_minus_target']}`",
        "",
        "## Theorem Skeleton",
        "",
        payload["theorem_skeleton"]["statement"],
        "",
        "### Required Assumptions",
        "",
    ]
    for x in payload["theorem_skeleton"]["required_assumptions"]:
        lines.append(f"- {x}")
    lines += [
        "",
        "### Proof Outline",
        "",
    ]
    for x in payload["theorem_skeleton"]["proof_outline"]:
        lines.append(f"- {x}")
    lines += [
        "",
        "## Remaining O5 Blockers",
        "",
    ]
    for x in payload["remaining_o5_blockers"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
