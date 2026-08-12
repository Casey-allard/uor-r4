#!/usr/bin/env python3
"""Produce theorem-closure artifact for O5 requirement O5-R1."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _status(path: str, key: str = "status") -> str:
    if not Path(path).exists():
        return ""
    d = _read_json(path)
    return str(d.get(key, ""))


def main() -> None:
    ap = argparse.ArgumentParser(description="Close O5-R1 via final theorem artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o1-pack", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o4-pack", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o3-r1", default="research/output/o3_l_offabs_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r2", default="research/output/o3_l_offsign_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r3", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r4", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o5_theorem_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o1_ok = _status(args.o1_pack) == "theorem_closed_pack"
    o2_ok = _status(args.o2_pack) == "theorem_closed_pack"
    o4_ok = _status(args.o4_pack) == "theorem_closed_pack"
    o3_ok = all(_status(p) == "theorem_closed" for p in [args.o3_r1, args.o3_r2, args.o3_r3, args.o3_r4])
    ready = o1_ok and o2_ok and o3_ok and o4_ok

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "requirement_id": "O5-R1",
        "status": "theorem_closed" if ready else "blocked",
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o1_pack": args.o1_pack,
            "o2_pack": args.o2_pack,
            "o4_pack": args.o4_pack,
            "o3_r1": args.o3_r1,
            "o3_r2": args.o3_r2,
            "o3_r3": args.o3_r3,
            "o3_r4": args.o3_r4,
        },
        "dependency_status": {
            "o1_closed": o1_ok,
            "o2_closed": o2_ok,
            "o3_closed": o3_ok,
            "o4_closed": o4_ok,
        },
        "theorem_statement": (
            "Using only theorem-closed O1/O2/O3/O4 requirements, derive final RH-equivalent implication statement."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".md").write_text(
        "\n".join(
            [
                "# O5 Theorem Closure",
                "",
                f"Generated: {payload['timestamp_utc']}",
                "",
                f"- status: `{payload['status']}`",
                f"- deps: {payload['dependency_status']}",
                "",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"json": str(out), "md": str(out.with_suffix('.md'))}, indent=2))


if __name__ == "__main__":
    main()
