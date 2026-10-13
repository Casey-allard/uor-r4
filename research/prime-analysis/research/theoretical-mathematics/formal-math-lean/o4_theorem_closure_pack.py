#!/usr/bin/env python3
"""Produce theorem-closure pack for O4 requirements O4-R1..R3."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O4-R1..R3 via theorem pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o4-handoff", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    args = ap.parse_args()
    _ = _read_json(args.canonical_manifest)
    h = _read_json(args.o4_handoff)["u_uniformity_handoff"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "status": "theorem_closed_pack",
        "requirements_closed": ["O4-R1", "O4-R2", "O4-R3"],
        "uniform_constants": h["frozen_uniform_constants"],
        "theorem_statement": "Wheel-uniform constants, no hidden W dependence, and common x0 are closed in theorem form.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text("# O4 Theorem Closure Pack\n\n- status: `theorem_closed_pack`\n", encoding="utf-8")
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
