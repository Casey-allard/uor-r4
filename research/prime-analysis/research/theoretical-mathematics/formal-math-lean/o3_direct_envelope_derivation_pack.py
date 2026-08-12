#!/usr/bin/env python3
"""Build direct-derivation O3 theorem pack for replacing interface handoffs."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 direct envelope derivation pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--glue-closure", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    off = _read_json(args.offdiag_closure)
    diag = _read_json(args.diag_closure)
    glue = _read_json(args.glue_closure)
    sign = _read_json(args.o3_sign)["theorem_assumption_candidates"]

    offb = off["derived_offdiag_bound"]
    diab = diag["derived_diag_bound"]
    asm = glue["assembly"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "offdiag_closure": args.offdiag_closure,
            "diag_closure": args.diag_closure,
            "glue_closure": args.glue_closure,
            "o3_sign": args.o3_sign,
        },
        "o3_direct_envelope_derivation_pack": {
            "status": "theorem_draft_ready_for_unconditionalization",
            "targets": {
                "offdiag_target": {
                    "A_offdiag": float(offb["A_offdiag"]),
                    "C_offdiag": float(offb["C_offdiag"]),
                    "statement": "Prove |Offdiag(x;W)| <= C_offdiag*(log x)^A_offdiag directly from decomposition + sign caps.",
                },
                "diag_target": {
                    "A_diag": float(diab["A_diag"]),
                    "C_diag": float(diab["C_diag"]),
                    "statement": "Prove Diag(x;W) <= C_diag*(log x)^A_diag directly from kernel/channel definitions.",
                },
                "assembly_target": {
                    "A_E2": float(asm["A_E2"]),
                    "C_E2": float(asm["C_E2"]),
                    "statement": "Assemble direct DIAG/OFFDIAG/REM derivations to recover E2/x bound without interface assumptions.",
                },
            },
            "frozen_sign_caps": {
                "eps_sign": float(sign["eps_sign"]),
                "neg_over_abs_cap": float(sign["neg_over_abs_cap"]),
                "pos_over_abs_floor": float(sign["pos_over_abs_floor"]),
            },
            "proof_lemmas_to_write": [
                "L-OFFABS: direct absolute offdiag envelope from explicit decomposition terms.",
                "L-OFFSIGN: direct sign-sensitive reduction using eps_sign/neg_over_abs bounds.",
                "L-DIAG: direct diagonal kernel majorant with wheel-uniform constants.",
                "L-ASM: final E2 assembly theorem with direct lemmas only.",
            ],
            "remaining_blockers": [
                "Replace interface-level offdiag envelope with direct decomposition proof.",
                "Replace interface-level diagonal envelope with direct kernel proof.",
                "Carry direct lemmas into O5 final theorem statement.",
            ],
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 Direct Envelope Derivation Pack",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['o3_direct_envelope_derivation_pack']['status']}`",
        "",
        "## Targets",
        "",
        f"- offdiag: `A={offb['A_offdiag']}`, `C={offb['C_offdiag']}`",
        f"- diag: `A={diab['A_diag']}`, `C={diab['C_diag']}`",
        f"- assembly: `A_E2={asm['A_E2']}`, `C_E2={asm['C_E2']}`",
        "",
        "## Lemmas To Write",
        "",
    ]
    for x in payload["o3_direct_envelope_derivation_pack"]["proof_lemmas_to_write"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
