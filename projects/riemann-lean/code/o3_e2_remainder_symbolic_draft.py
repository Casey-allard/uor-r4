#!/usr/bin/env python3
"""Draft symbolic remainder bound for O3 E2 asymptotic proof chain."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build E2-REM symbolic draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--a2", default="research/output/a2_infinite_tail_uplift_2026-02-17_n_diff_explicit_hsw2022_sf3p5.json")
    ap.add_argument("--output", default="research/output/o3_e2_remainder_symbolic_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    full = _read_json(args.o3_e2_full)
    a2 = _read_json(args.a2)

    c_delta = float(a2["uplift_constant"]["C_delta_uplifted"])
    beta = float(a2["config"]["beta_fixed"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_full": args.o3_e2_full,
            "a2": args.a2,
        },
        "lemma_id": "E2-REM",
        "target_theorem_goal": full["theorem_goal"]["statement"],
        "symbolic_draft": {
            "decomposition_slot": "remainder/truncation contribution in E2/x decomposition",
            "proposed_inequality": (
                "Rem(x;W) <= C_rem * (log x)^A_rem with C_rem assembled from truncation/tail majorants."
            ),
            "candidate_constants_from_a2": {
                "C_delta_uplifted": c_delta,
                "beta_fixed": beta,
            },
            "interpretation": (
                "Remainder control should inherit explicit tail bounds from O2 and be uniform in W."
            ),
        },
        "remaining_to_formalize": [
            "Define remainder term in E2 decomposition with exact dependency on truncation level.",
            "Map O2 tail majorant into E2 remainder bound without losing uniformity in W.",
            "Extract explicit C_rem and A_rem for E2 assembly lemma.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    cc = payload["symbolic_draft"]["candidate_constants_from_a2"]
    lines = [
        "# O3 E2 Remainder Symbolic Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Lemma Target",
        "",
        f"- `{payload['lemma_id']}`",
        f"- {payload['target_theorem_goal']}",
        "",
        "## Proposed Remainder Inequality",
        "",
        payload["symbolic_draft"]["proposed_inequality"],
        "",
        "## Candidate Constants From O2",
        "",
        f"- `C_delta_uplifted = {cc['C_delta_uplifted']}`",
        f"- `beta_fixed = {cc['beta_fixed']}`",
        "",
        payload["symbolic_draft"]["interpretation"],
        "",
        "## Remaining To Formalize",
        "",
    ]
    for x in payload["remaining_to_formalize"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
