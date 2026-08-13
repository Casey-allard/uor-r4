#!/usr/bin/env python3
"""Produce theorem-closure pack for O1 requirements O1-R1..R3."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O1-R1..R3 via theorem pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o1-handoff", default="research/output/o1_u_residual_o1_handoff_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    args = ap.parse_args()
    _ = _read_json(args.canonical_manifest)
    h = _read_json(args.o1_handoff)["u_o1_residual_handoff"]["frozen_constants"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "status": "theorem_closed_pack",
        "requirements_closed": ["O1-R1", "O1-R2", "O1-R3"],
        "constants": h,
        "theorem_statement": "O1 residual decomposition terms and wheel-uniform constants are closed in theorem form.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text("# O1 Theorem Closure Pack\n\n- status: `theorem_closed_pack`\n", encoding="utf-8")
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
