#!/usr/bin/env python3
"""Rank A3 candidate artifacts by asymptotic and practical envelope scores."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple


def read_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def extract_constants(doc: Dict[str, Any]) -> Tuple[float, float, bool]:
    if "h_transfer_envelope" in doc:
        h = doc["h_transfer_envelope"]
        c_h = float(h["C_H_from_density_transfer"])
        a_h = float(h["A_H"])
    elif "uplift_constants" in doc:
        u = doc["uplift_constants"]
        c_h = float(u["C_H_from_normalized_sum"])
        a_h = float(u["A_H_from_normalized_sum"])
    else:
        raise KeyError("unknown A3 schema")

    checks = doc.get("checks", {})
    valid = checks.get("valid", {})
    if "holds" in valid:
        ok = bool(valid["holds"])
    else:
        ok = bool(valid.get("deterministic_check_holds", False)) and bool(valid.get("log_envelope_holds", False))
    return c_h, a_h, ok


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 branch leaderboard")
    ap.add_argument(
        "--artifacts",
        type=str,
        default="research/output/a3_channel_energy_uplift_refresh_2026-02-17.json,"
        "research/output/a3_density_transfer_majorant_refresh_2026-02-17.json,"
        "research/output/a3_quadratic_energy_majorant_refresh_2026-02-17.json,"
        "research/output/a3_mean_square_majorant_refresh_2026-02-17.json,"
        "research/output/a3_offdiag_dynamic_majorant_refresh_2026-02-17.json",
    )
    ap.add_argument("--x-ref", type=float, default=1e12)
    ap.add_argument("--output", type=str, default="research/output/a3_branch_leaderboard_2026-02-17.json")
    args = ap.parse_args()

    entries: List[Dict[str, Any]] = []
    for raw in [x.strip() for x in args.artifacts.split(",") if x.strip()]:
        p = Path(raw)
        if not p.exists():
            continue
        doc = read_json(p)
        c_h, a_h, ok = extract_constants(doc)
        lhs = c_h * (math.log(max(3.0, args.x_ref)) ** a_h)
        entries.append(
            {
                "artifact": str(p),
                "C_H": c_h,
                "A_H": a_h,
                "valid_holds": ok,
                "rhs_at_x_ref": lhs,
            }
        )

    entries = [e for e in entries if e["valid_holds"]]
    entries.sort(key=lambda e: (e["A_H"], e["rhs_at_x_ref"], e["C_H"]))

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "x_ref": args.x_ref,
        "ranking": entries,
        "winner_by_exponent": entries[0] if entries else None,
        "winner_by_rhs_at_x_ref": min(entries, key=lambda e: e["rhs_at_x_ref"]) if entries else None,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    md = out.with_suffix(".md")
    lines = [
        "# A3 Branch Leaderboard",
        "",
        f"- Timestamp (UTC): `{payload['timestamp_utc']}`",
        f"- Reference scale: `x={args.x_ref:.0f}`",
        "",
        "| Rank | Artifact | A_H | C_H | RHS(x_ref) |",
        "|---:|---|---:|---:|---:|",
    ]
    for i, e in enumerate(entries, start=1):
        lines.append(
            f"| {i} | `{e['artifact']}` | {e['A_H']:.6g} | {e['C_H']:.6g} | {e['rhs_at_x_ref']:.6g} |"
        )
    md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps({"json": str(out), "md": str(md), "ranked": len(entries)}, indent=2))


if __name__ == "__main__":
    main()
