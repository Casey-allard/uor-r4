#!/usr/bin/env python3
"""Build a theorem-facing asymptotic closure artifact for O3 lemma E2-DIAG."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 E2-DIAG asymptotic closure artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-diag-draft", default="research/output/o3_e2_diag_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-target-pack", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--diag-constant", type=float, default=1.0, help="C_diag in Diag <= C_diag*(log x)^A_diag")
    ap.add_argument("--diag-exponent", type=float, default=0.0, help="A_diag in Diag <= C_diag*(log x)^A_diag")
    ap.add_argument("--x0", type=float, default=3.0)
    ap.add_argument("--output", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    draft = _read_json(args.o3_e2_diag_draft)
    tpack = _read_json(args.o3_e2_target_pack)["o3_e2_target_pack"]

    a_diag = float(args.diag_exponent)
    c_diag = float(args.diag_constant)
    a_e2_max = float(tpack["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    compatible = a_diag <= a_e2_max

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_diag_draft": args.o3_e2_diag_draft,
            "o3_e2_target_pack": args.o3_e2_target_pack,
        },
        "lemma_id": "E2-DIAG",
        "status": "conditional_asymptotic_closed",
        "assumption_scope": (
            "assumes an explicit wheel-uniform diagonal envelope "
            "Diag(x;W) <= C_diag*(log x)^A_diag for all x>=x0"
        ),
        "assumptions": {
            "x0": float(args.x0),
            "diag_envelope": {
                "C_diag": c_diag,
                "A_diag": a_diag,
                "statement": "For all x>=x0 and W in {30,210,2310,30030}, Diag(x;W) <= C_diag*(log x)^A_diag.",
            },
        },
        "derived_diag_bound": {
            "C_diag": c_diag,
            "A_diag": a_diag,
            "A_E2_max_target": a_e2_max,
            "target_compatible": compatible,
            "source_slot": draft["symbolic_draft"]["decomposition_slot"],
        },
        "interpretation": (
            "This discharges the diagonal slot in theorem-facing conditional form; "
            "remaining O3 assembly work is concentrated in E2-GLUE."
        ),
        "remaining_to_unconditionalize": [
            "Prove the assumed diagonal envelope directly from channel/kernel definitions with explicit constants."
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Diagonal Asymptotic Closure",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Status",
        "",
        f"- `{payload['status']}`",
        f"- scope: {payload['assumption_scope']}",
        "",
        "## Assumed Diagonal Envelope",
        "",
        f"- `Diag <= {c_diag}*(log x)^{a_diag}` for `x>= {args.x0}`",
        "",
        "## Compatibility",
        "",
        f"- `A_E2_max_target = {a_e2_max}`",
        f"- target compatible: `{compatible}`",
        "",
        "## Remaining To Unconditionalize",
        "",
    ]
    for item in payload["remaining_to_unconditionalize"]:
        lines.append(f"- {item}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
