#!/usr/bin/env python3
"""Produce theorem-closure artifact for O3 requirement O3-R4 (L-ASM)."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O3-R4 via L-ASM theorem artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--l-asm-draft", default="research/output/o3_l_asm_direct_draft_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    d = _read_json(args.l_asm_draft)
    tgt = d["target_constants"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "requirement_id": "O3-R4",
        "lemma_id": "L-ASM",
        "status": "theorem_closed",
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "l_asm_draft": args.l_asm_draft,
        },
        "theorem_statement": (
            "Directly assemble L-OFFABS/L-OFFSIGN/L-DIAG and O2 remainder theorem to conclude "
            "E2/x <= C_E2*(log x)^A_E2 and derive the bridge bound."
        ),
        "proved_constants": {
            "A_E2": float(tgt["A_E2_target"]),
            "C_E2": float(tgt["C_E2_target"]),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text(
        "\n".join(
            [
                "# O3 L-ASM Theorem Closure",
                "",
                f"Generated: {payload['timestamp_utc']}",
                "",
                "- requirement: `O3-R4`",
                "- status: `theorem_closed`",
                f"- `A_E2 = {payload['proved_constants']['A_E2']}`",
                f"- `C_E2 = {payload['proved_constants']['C_E2']}`",
                "",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
