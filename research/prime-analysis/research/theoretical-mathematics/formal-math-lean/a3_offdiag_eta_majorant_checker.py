#!/usr/bin/env python3
"""Fast consistency checker for O3 offdiag-eta majorant artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def branch_row(name: str, doc: Dict[str, Any]) -> Dict[str, Any]:
    checks = doc["checks"]
    h = doc["h_transfer_envelope"]
    eta = doc["eta_envelope"]
    return {
        "name": name,
        "A_eta": float(eta["A_eta"]),
        "C_eta_uplifted": float(eta["C_eta_uplifted"]),
        "A_H": float(h["A_H"]),
        "C_H": float(h["C_H_from_density_transfer"]),
        "train_holds": bool(checks["train"]["holds"]),
        "valid_holds": bool(checks["valid"]["holds"]),
        "valid_ratio_max": float(checks["valid"]["ratio_max"]),
        "valid_max_gap_h_minus_rhs": float(checks["valid"]["max_gap_h_minus_rhs"]),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 offdiag-eta majorant checker")
    ap.add_argument(
        "--primary",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json",
    )
    ap.add_argument(
        "--stress",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json",
    )
    ap.add_argument(
        "--output",
        default="research/output/a3_offdiag_eta_majorant_checker_2026-02-17.json",
    )
    args = ap.parse_args()

    primary = read_json(args.primary)
    stress = read_json(args.stress)
    rows = [branch_row("primary", primary), branch_row("stress", stress)]

    all_hold = all(bool(r["train_holds"] and r["valid_holds"]) for r in rows)
    worst_ratio = max(float(r["valid_ratio_max"]) for r in rows)
    worst_gap = max(float(r["valid_max_gap_h_minus_rhs"]) for r in rows)

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {"primary": args.primary, "stress": args.stress},
        "rows": rows,
        "checks": {
            "all_hold": bool(all_hold),
            "worst_valid_ratio_max": float(worst_ratio),
            "worst_valid_max_gap_h_minus_rhs": float(worst_gap),
        },
        "cache_reuse_note": "Reads cached A3 majorant artifacts only; no recomputation.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# A3 Offdiag-Eta Majorant Checker",
        "",
        f"Generated: {payload['timestamp_utc']}",
        f"- all_hold: `{payload['checks']['all_hold']}`",
        f"- worst_valid_ratio_max: `{payload['checks']['worst_valid_ratio_max']:.12g}`",
        f"- worst_valid_max_gap_h_minus_rhs: `{payload['checks']['worst_valid_max_gap_h_minus_rhs']:.12g}`",
        "",
        "| branch | A_eta | C_eta_uplifted | A_H | C_H | valid_ratio_max | valid_max_gap_h_minus_rhs | holds |",
        "|---|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for r in rows:
        lines.append(
            f"| {r['name']} | {r['A_eta']:.6g} | {r['C_eta_uplifted']:.12g} | "
            f"{r['A_H']:.6g} | {r['C_H']:.12g} | {r['valid_ratio_max']:.12g} | "
            f"{r['valid_max_gap_h_minus_rhs']:.12g} | {r['train_holds'] and r['valid_holds']} |"
        )
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
