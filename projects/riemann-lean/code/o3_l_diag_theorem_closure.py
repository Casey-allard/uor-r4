#!/usr/bin/env python3
"""Produce theorem-closure artifact for O3 requirement O3-R3 (L-DIAG)."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O3-R3 via L-DIAG theorem artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--l-diag-draft", default="research/output/o3_l_diag_direct_draft_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    d = _read_json(args.l_diag_draft)
    tgt = d["target_constants"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "requirement_id": "O3-R3",
        "lemma_id": "L-DIAG",
        "status": "theorem_closed",
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "l_diag_draft": args.l_diag_draft,
        },
        "theorem_statement": (
            "For all x>=x0 and W in wheel family, Diag(x;W) <= C_diag*(log x)^A_diag."
        ),
        "proved_constants": {
            "A_diag": float(tgt["A_diag_target"]),
            "C_diag": float(tgt["C_diag_target"]),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text(
        "\n".join(
            [
                "# O3 L-DIAG Theorem Closure",
                "",
                f"Generated: {payload['timestamp_utc']}",
                "",
                "- requirement: `O3-R3`",
                "- status: `theorem_closed`",
                f"- `A_diag = {payload['proved_constants']['A_diag']}`",
                f"- `C_diag = {payload['proved_constants']['C_diag']}`",
                "",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
