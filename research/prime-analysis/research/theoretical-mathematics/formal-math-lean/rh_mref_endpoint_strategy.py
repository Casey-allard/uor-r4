#!/usr/bin/env python3
"""Audit direct endpoint strategy with M fixed to M_ref (truncation-free transfer)."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="RH direct M_ref endpoint strategy audit")
    ap.add_argument(
        "--a3-mref",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument(
        "--a1",
        default="research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json",
    )
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--target-log-exponent", type=float, default=2.0)
    ap.add_argument(
        "--output",
        default="research/output/rh_mref_endpoint_strategy_2026-02-17.json",
    )
    args = ap.parse_args()

    a3 = read_json(args.a3_mref)
    a1 = read_json(args.a1)

    ah = float(a3["h_transfer_envelope"]["A_H"])
    ch = float(a3["h_transfer_envelope"]["C_H_from_density_transfer"])
    a3_valid = bool(a3["checks"]["valid"]["holds"])
    c0 = float(a1["decomposition_constants"]["C0_uplifted"])
    a_ref = float(a1["decomposition_constants"]["a_ref"])
    b_ref = float(a1["decomposition_constants"]["b_ref"])

    target = float(args.target_log_exponent)
    reachable = ah <= target

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {"a3_mref": args.a3_mref, "a1": args.a1, "m_ref": int(args.m_ref)},
        "strategy": {
            "name": "direct_mref_endpoint",
            "idea": (
                "Fix M = M_ref in the endpoint transfer, so Delta_M term vanishes identically "
                "and A2 beta does not control endpoint exponent."
            ),
            "identity": "|E|/sqrt(x) <= |a_ref| |H^(M_ref)| + |b_ref| + C0",
        },
        "constants": {
            "A_H_mref": ah,
            "C_H_mref": ch,
            "C0_uplifted": c0,
            "a_ref": a_ref,
            "b_ref": b_ref,
        },
        "endpoint_target": {"A_target": target},
        "reachability": {
            "a3_valid_on_grid": a3_valid,
            "A_H_minus_target": ah - target,
            "target_class_reachable_if_A1_A3_asymptotic": bool(reachable),
        },
        "proof_requirements_remaining": [
            "Prove A1 asymptotically at M_ref (not finite-window).",
            "Prove A3 asymptotic bridge bound at M_ref with exponent A_H <= target.",
            "Map obtained endpoint bound to a standard RH-equivalent theorem statement.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# RH Direct M_ref Endpoint Strategy",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Strategy",
        "",
        f"- {payload['strategy']['idea']}",
        f"- identity: `{payload['strategy']['identity']}`",
        "",
        "## Constants",
        "",
        f"- `A_H(M_ref) = {ah}`",
        f"- `C_H(M_ref) = {ch}`",
        f"- `C0 = {c0}`",
        f"- `a_ref = {a_ref}`",
        f"- `b_ref = {b_ref}`",
        "",
        "## Endpoint Reachability",
        "",
        f"- target exponent: `{target}`",
        f"- `A_H - target = {ah - target}`",
        f"- A3 held-out valid on current grid: `{a3_valid}`",
        f"- target class reachable if A1/A3 are proved asymptotically: `{reachable}`",
        "",
        "## Remaining Proof Requirements",
        "",
    ]
    for r in payload["proof_requirements_remaining"]:
        lines.append(f"- {r}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
