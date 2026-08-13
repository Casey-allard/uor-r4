#!/usr/bin/env python3
"""Build O3 scaffold for asymptotic bridge-growth control at M_ref."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Create O3 A3_ref asymptotic scaffold")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument(
        "--a3",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument("--output", default="research/output/o3_ref_bridge_asymptotic_scaffold_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    a3 = _read_json(args.a3)

    cfg = a3["config"]
    eta = a3["eta_envelope"]
    h = a3["h_transfer_envelope"]
    valid = a3["checks"]["valid"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a3": args.a3,
        },
        "lemma_o3_a3_ref": {
            "name": "Bridge-Growth Asymptotic Control at M_ref",
            "quantified_statement": (
                "There exist x0>=3 and constants C_H>=0, A_H>=0 such that for all x>=x0 "
                "and W in {30,210,2310,30030}, |H_W^(M_ref)(x)| <= C_H (log x)^A_H."
            ),
            "proof_chain_template": [
                "Bound eta_+(x;W) by C_eta (log x)^A_eta uniformly in W and x>=x0.",
                "Use deterministic identity |H| <= sqrt((1+eta_+) E2/x).",
                "Bound E2/x uniformly and combine exponents/constants.",
            ],
            "current_constant_candidates": {
                "m_ref": int(cfg["m_zero"]),
                "A_eta": float(eta["A_eta"]),
                "C_eta_uplifted": float(eta["C_eta_uplifted"]),
                "A_H": float(h["A_H"]),
                "C_H": float(h["C_H_from_density_transfer"]),
            },
            "grid_validation_status": {
                "valid_holds": bool(valid["holds"]),
                "valid_ratio_max": float(valid["ratio_max"]),
                "valid_max_gap_h_minus_rhs": float(valid["max_gap_h_minus_rhs"]),
                "deterministic_eta_holds": bool(valid["deterministic_dynamic_eta_holds"]),
                "deterministic_eta_max_gap": float(valid["deterministic_dynamic_eta_max_gap"]),
            },
        },
        "remaining_o3_blockers": [
            "Replace safety-factor calibrated C_eta with theorem-side sign-sensitive offdiag constants.",
            "Prove eta_+ upper bound uniformly for all x>=x0 and all W in the wheel family.",
            "Prove asymptotic bound for E2/x independent of sampled grids.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 A3_ref Asymptotic Scaffold",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Quantified Statement",
        "",
        payload["lemma_o3_a3_ref"]["quantified_statement"],
        "",
        "## Proof-Chain Template",
        "",
    ]
    for x in payload["lemma_o3_a3_ref"]["proof_chain_template"]:
        lines.append(f"- {x}")
    cc = payload["lemma_o3_a3_ref"]["current_constant_candidates"]
    gs = payload["lemma_o3_a3_ref"]["grid_validation_status"]
    lines += [
        "",
        "## Current Constant Candidates",
        "",
        f"- `m_ref = {cc['m_ref']}`",
        f"- `A_eta = {cc['A_eta']}`",
        f"- `C_eta_uplifted = {cc['C_eta_uplifted']}`",
        f"- `A_H = {cc['A_H']}`",
        f"- `C_H = {cc['C_H']}`",
        "",
        "## Grid Validation Snapshot",
        "",
        f"- valid holds: `{gs['valid_holds']}`",
        f"- valid ratio max: `{gs['valid_ratio_max']}`",
        f"- valid max gap h-rhs: `{gs['valid_max_gap_h_minus_rhs']}`",
        f"- deterministic eta holds: `{gs['deterministic_eta_holds']}`",
        f"- deterministic eta max gap: `{gs['deterministic_eta_max_gap']}`",
        "",
        "## Remaining O3 Blockers",
        "",
    ]
    for x in payload["remaining_o3_blockers"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
