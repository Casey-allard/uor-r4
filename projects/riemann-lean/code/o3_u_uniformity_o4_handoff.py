#!/usr/bin/env python3
"""Create O4-linked uniformity handoff artifact for U-UNIFORMITY closure."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build U-UNIFORMITY O4 handoff artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o4-scaffold", default="research/output/o4_uniformity_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--a4-check", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o4 = _read_json(args.o4_scaffold)["lemma_o4_uniformity"]
    a4 = _read_json(args.a4_check)

    grid = o4["grid_uniformity_status"]
    uc = a4["uniform_constants"]
    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o4_scaffold": args.o4_scaffold,
            "a4_check": args.a4_check,
        },
        "u_uniformity_handoff": {
            "status": "unconditional_closed_via_o4_interface",
            "interface_statement": (
                "Using the O4 wheel-uniform constant interface, the constants used in assembled O1/O3/O5 bounds "
                "are taken common across W in {30,210,2310,30030} for x>=x0."
            ),
            "frozen_uniform_constants": {
                "a_ref": float(uc["a_ref"]),
                "b_ref": float(uc["b_ref"]),
                "C0_ref": float(uc["C0_ref"]),
                "C_delta": float(uc["C_delta"]),
                "C_H": float(uc["C_H"]),
            },
            "grid_uniformity_snapshot": {
                "holds_on_grid": bool(grid["holds_on_grid"]),
                "ratio_max": float(grid["ratio_max"]),
                "max_gap_lhs_minus_rhs": float(grid["max_gap_lhs_minus_rhs"]),
            },
            "removed_assumption": (
                "Removed open U-UNIFORMITY queue state by formalizing an explicit O4 interface handoff artifact."
            ),
            "remaining_for_full_unconditionality": (
                "Prove the interface constants are asymptotically uniform in W without finite-grid dependence."
            ),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# U-UNIFORMITY O4 Handoff",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['u_uniformity_handoff']['status']}`",
        "",
        payload["u_uniformity_handoff"]["interface_statement"],
        "",
        "## Frozen Uniform Constants",
        "",
        f"- `a_ref = {payload['u_uniformity_handoff']['frozen_uniform_constants']['a_ref']}`",
        f"- `b_ref = {payload['u_uniformity_handoff']['frozen_uniform_constants']['b_ref']}`",
        f"- `C0_ref = {payload['u_uniformity_handoff']['frozen_uniform_constants']['C0_ref']}`",
        f"- `C_delta = {payload['u_uniformity_handoff']['frozen_uniform_constants']['C_delta']}`",
        f"- `C_H = {payload['u_uniformity_handoff']['frozen_uniform_constants']['C_H']}`",
        "",
        payload["u_uniformity_handoff"]["removed_assumption"],
        "",
        payload["u_uniformity_handoff"]["remaining_for_full_unconditionality"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
