#!/usr/bin/env python3
"""Create direct theorem-draft artifact for O3 lemma L-OFFABS."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 direct lemma draft L-OFFABS")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-direct-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_offabs_direct_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    pack = _read_json(args.o3_direct_pack)["o3_direct_envelope_derivation_pack"]
    off = _read_json(args.offdiag_closure)
    tgt = pack["targets"]["offdiag_target"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_direct_pack": args.o3_direct_pack,
            "offdiag_closure": args.offdiag_closure,
        },
        "lemma_id": "L-OFFABS",
        "status": "direct_theorem_draft_started",
        "target_statement": (
            "For all x>=x0 and W in {30,210,2310,30030}, Offdiag_abs(x;W) <= C_offabs*(log x)^A_offabs."
        ),
        "target_constants": {
            "A_offabs_target": float(tgt["A_offdiag"]),
            "C_offabs_target": float(tgt["C_offdiag"]),
        },
        "derivation_outline": [
            "Write offdiag decomposition as weighted sum over pair-correlation channels.",
            "Apply absolute majorant per channel with explicit kernel envelope.",
            "Use wheel-uniform counting control to bound channel sums.",
            "Aggregate into C_offabs*(log x)^A_offabs and match target constants.",
        ],
        "handoff_replacement_goal": (
            "Replace interface assumption Offdiag_abs<=C_abs*(log x)^A_abs used in U-OFFDIAG handoff."
        ),
        "source_coefficients": off["derived_offdiag_bound"]["coefficients"],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 L-OFFABS Direct Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['status']}`",
        f"- `A_offabs_target = {payload['target_constants']['A_offabs_target']}`",
        f"- `C_offabs_target = {payload['target_constants']['C_offabs_target']}`",
        "",
        "## Derivation Outline",
        "",
    ]
    for x in payload["derivation_outline"]:
        lines.append(f"- {x}")
    lines += ["", payload["handoff_replacement_goal"], ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
