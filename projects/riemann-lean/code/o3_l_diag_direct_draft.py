#!/usr/bin/env python3
"""Create direct theorem-draft artifact for O3 lemma L-DIAG."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 direct lemma draft L-DIAG")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-direct-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_diag_direct_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    pack = _read_json(args.o3_direct_pack)["o3_direct_envelope_derivation_pack"]
    diag = _read_json(args.diag_closure)["derived_diag_bound"]
    tgt = pack["targets"]["diag_target"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_direct_pack": args.o3_direct_pack,
            "diag_closure": args.diag_closure,
        },
        "lemma_id": "L-DIAG",
        "status": "direct_theorem_draft_started",
        "target_statement": "For all x>=x0 and W in wheel family, Diag(x;W) <= C_diag*(log x)^A_diag.",
        "target_constants": {
            "A_diag_target": float(tgt["A_diag"]),
            "C_diag_target": float(tgt["C_diag"]),
        },
        "reference_constants": {
            "A_diag_current": float(diag["A_diag"]),
            "C_diag_current": float(diag["C_diag"]),
        },
        "derivation_outline": [
            "Express diagonal term in kernel/channel form.",
            "Apply channel-wise majorants with explicit kernel bounds.",
            "Use wheel-uniform counting bound to remove base dependence.",
            "Conclude global diagonal envelope with frozen constants.",
        ],
        "replacement_goal": "Replace U-DIAG interface argument by direct diagonal theorem lemma.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 L-DIAG Direct Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['status']}`",
        f"- `A_diag_target = {tgt['A_diag']}`",
        f"- `C_diag_target = {tgt['C_diag']}`",
        "",
        "## Derivation Outline",
        "",
    ]
    for x in payload["derivation_outline"]:
        lines.append(f"- {x}")
    lines += ["", payload["replacement_goal"], ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
