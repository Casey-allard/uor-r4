#!/usr/bin/env python3
"""Create direct theorem-draft artifact for O3 lemma L-ASM."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 direct lemma draft L-ASM")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-direct-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--o3-e2-glue-closure", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_asm_direct_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    pack = _read_json(args.o3_direct_pack)["o3_direct_envelope_derivation_pack"]
    glue = _read_json(args.o3_e2_glue_closure)["assembly"]
    tgt = pack["targets"]["assembly_target"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_direct_pack": args.o3_direct_pack,
            "o3_e2_glue_closure": args.o3_e2_glue_closure,
        },
        "lemma_id": "L-ASM",
        "status": "direct_theorem_draft_started",
        "target_statement": (
            "Assuming direct lemmas L-OFFABS, L-OFFSIGN, L-DIAG and O2 remainder theorem, "
            "derive E2/x <= C_E2*(log x)^A_E2 and then bridge bound for |H|."
        ),
        "target_constants": {
            "A_E2_target": float(tgt["A_E2"]),
            "C_E2_target": float(tgt["C_E2"]),
        },
        "reference_constants": {
            "A_E2_current": float(glue["A_E2"]),
            "C_E2_current": float(glue["C_E2"]),
        },
        "assembly_outline": [
            "Insert direct OFFDIAG and DIAG bounds plus O2 remainder theorem bound.",
            "Take max exponent and sum constants.",
            "Derive final E2/x theorem form with explicit x0 and wheel-uniform constants.",
            "Feed into bridge identity to recover final O3 theorem constants.",
        ],
        "replacement_goal": "Replace interface-level glue closure with direct theorem assembly.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 L-ASM Direct Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['status']}`",
        f"- `A_E2_target = {tgt['A_E2']}`",
        f"- `C_E2_target = {tgt['C_E2']}`",
        "",
        "## Assembly Outline",
        "",
    ]
    for x in payload["assembly_outline"]:
        lines.append(f"- {x}")
    lines += ["", payload["replacement_goal"], ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
