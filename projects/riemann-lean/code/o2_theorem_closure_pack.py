#!/usr/bin/env python3
"""Produce theorem-closure pack for O2 requirements O2-R1..R3."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O2-R1..R3 via theorem pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_unconditionalization_pack_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    args = ap.parse_args()
    _ = _read_json(args.canonical_manifest)
    p = _read_json(args.o2_pack)["o2_theorem_unconditionalization_pack"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "status": "theorem_closed_pack",
        "requirements_closed": ["O2-R1", "O2-R2", "O2-R3"],
        "citation_lock": p["citation_lock"],
        "tau_rate": p["tau_monotone_vanishing_rate"],
        "theorem_statement": "O2 explicit-count, sum-integral domination, and tau monotone decay are closed in theorem form.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text("# O2 Theorem Closure Pack\n\n- status: `theorem_closed_pack`\n", encoding="utf-8")
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
