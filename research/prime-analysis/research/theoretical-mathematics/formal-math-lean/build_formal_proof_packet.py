#!/usr/bin/env python3
"""Build consolidated formal proof packet from closed theorem artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _first_evidence(reqs: List[Dict[str, Any]], req_id: str) -> str:
    row = next(r for r in reqs if r["id"] == req_id)
    ev = row.get("evidence", [])
    return ev[0] if ev else ""


def main() -> None:
    ap = argparse.ArgumentParser(description="Build consolidated formal proof packet")
    ap.add_argument("--closure-table", default="research/output/proof_requirement_closure_table_2026-02-17.json")
    ap.add_argument("--o1-pack", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o2-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o3-r1", default="research/output/o3_l_offabs_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r2", default="research/output/o3_l_offsign_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r3", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-r4", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    ap.add_argument("--o4-pack", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o5-closure", default="research/output/o5_theorem_closure_2026-02-17.json")
    ap.add_argument("--manuscript", default="research/output/formal_proof_manuscript_2026-02-17.md")
    ap.add_argument("--dependency-map", default="research/output/formal_proof_dependency_map_2026-02-17.json")
    ap.add_argument("--packet-index", default="research/output/formal_proof_packet_index_2026-02-17.json")
    args = ap.parse_args()

    table = _read_json(args.closure_table)
    reqs = table["requirements"]
    gs = table["global_summary"]
    if int(gs["theorem_open_total"]) != 0:
        raise RuntimeError("cannot build final formal packet: theorem_open_total != 0")

    o1 = _read_json(args.o1_pack)
    o2 = _read_json(args.o2_pack)
    o3r1 = _read_json(args.o3_r1)
    o3r2 = _read_json(args.o3_r2)
    o3r3 = _read_json(args.o3_r3)
    o3r4 = _read_json(args.o3_r4)
    o4 = _read_json(args.o4_pack)
    o5 = _read_json(args.o5_closure)

    dep: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "theorem": {
            "id": "Main-Theorem",
            "statement": o5["theorem_statement"],
            "depends_on": ["O1", "O2", "O3", "O4"],
        },
        "obligations": {
            "O1": {
                "status": o1["status"],
                "requirements": o1["requirements_closed"],
                "artifact": args.o1_pack,
            },
            "O2": {
                "status": o2["status"],
                "requirements": o2["requirements_closed"],
                "artifact": args.o2_pack,
            },
            "O3": {
                "status": "theorem_closed_pack",
                "requirements": ["O3-R1", "O3-R2", "O3-R3", "O3-R4"],
                "artifacts": [args.o3_r1, args.o3_r2, args.o3_r3, args.o3_r4],
            },
            "O4": {
                "status": o4["status"],
                "requirements": o4["requirements_closed"],
                "artifact": args.o4_pack,
            },
            "O5": {
                "status": o5["status"],
                "requirements": ["O5-R1"],
                "artifact": args.o5_closure,
            },
        },
        "closure_table": args.closure_table,
    }

    dep_out = Path(args.dependency_map)
    dep_out.parent.mkdir(parents=True, exist_ok=True)
    dep_out.write_text(json.dumps(dep, indent=2) + "\n", encoding="utf-8")

    manuscript_lines = [
        "# Formal Proof Manuscript (Consolidated)",
        "",
        f"Generated: {dep['timestamp_utc']}",
        "",
        "## Main Theorem",
        "",
        o5["theorem_statement"],
        "",
        "## Dependency Chain",
        "",
        "- Main-Theorem depends on O1, O2, O3, O4.",
        "- O1 provides residual decomposition and uniform constants.",
        "- O2 provides explicit-count, sum-integral, and tau-decay closures.",
        "- O3 provides direct OFFABS/OFFSIGN/DIAG/ASM closures.",
        "- O4 provides wheel-uniform asymptotic closure.",
        "",
        "## Closed Requirements (Strict Table)",
        "",
        f"- total requirements: `{gs['total_requirements']}`",
        f"- theorem closed: `{gs['theorem_closed_total']}`",
        f"- theorem open: `{gs['theorem_open_total']}`",
        "",
        "## Requirement Evidence",
        "",
    ]
    for r in reqs:
        ev = r["evidence"][0] if r.get("evidence") else ""
        manuscript_lines.append(f"- `{r['id']}` ({r['obligation']}): `{r['theorem_status']}` via `{ev}`")

    manuscript_lines += [
        "",
        "## O1 Theorem Closure",
        "",
        f"- Artifact: `{args.o1_pack}`",
        f"- Statement: {o1['theorem_statement']}",
        "",
        "## O2 Theorem Closure",
        "",
        f"- Artifact: `{args.o2_pack}`",
        f"- Statement: {o2['theorem_statement']}",
        "",
        "## O3 Theorem Closures",
        "",
        f"- O3-R1: `{o3r1['theorem_statement']}` (`{args.o3_r1}`)",
        f"- O3-R2: `{o3r2['theorem_statement']}` (`{args.o3_r2}`)",
        f"- O3-R3: `{o3r3['theorem_statement']}` (`{args.o3_r3}`)",
        f"- O3-R4: `{o3r4['theorem_statement']}` (`{args.o3_r4}`)",
        "",
        "## O4 Theorem Closure",
        "",
        f"- Artifact: `{args.o4_pack}`",
        f"- Statement: {o4['theorem_statement']}",
        "",
        "## O5 Final Closure",
        "",
        f"- Artifact: `{args.o5_closure}`",
        f"- Status: `{o5['status']}`",
        "",
        "## Notes",
        "",
        "- This manuscript consolidates closed requirement artifacts into one dependency-indexed proof packet.",
        "- The strict closure table is treated as the authoritative closure ledger.",
        "",
    ]

    man_out = Path(args.manuscript)
    man_out.parent.mkdir(parents=True, exist_ok=True)
    man_out.write_text("\n".join(manuscript_lines) + "\n", encoding="utf-8")

    packet = {
        "timestamp_utc": dep["timestamp_utc"],
        "manuscript": args.manuscript,
        "dependency_map": args.dependency_map,
        "closure_table": args.closure_table,
        "primary_theorem_artifact": args.o5_closure,
        "all_requirements_closed": True,
    }
    idx_out = Path(args.packet_index)
    idx_out.parent.mkdir(parents=True, exist_ok=True)
    idx_out.write_text(json.dumps(packet, indent=2) + "\n", encoding="utf-8")

    print(
        json.dumps(
            {
                "manuscript": str(man_out),
                "dependency_map": str(dep_out),
                "packet_index": str(idx_out),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
