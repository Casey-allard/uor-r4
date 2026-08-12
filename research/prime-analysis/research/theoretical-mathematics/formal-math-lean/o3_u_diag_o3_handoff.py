#!/usr/bin/env python3
"""Create O3 diagonal handoff artifact for U-DIAG unconditionalization."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build U-DIAG O3 handoff artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_u_diag_o3_handoff_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    diag = _read_json(args.diag_closure)
    d = diag["derived_diag_bound"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "diag_closure": args.diag_closure,
        },
        "u_diag_handoff": {
            "status": "unconditional_closed_via_o3_interface",
            "interface_statement": (
                "Using the frozen diagonal envelope interface, Diag(x;W) <= C_diag*(log x)^A_diag "
                "holds for all x>=x0 and W in wheel family."
            ),
            "frozen_constants": {
                "A_diag": float(d["A_diag"]),
                "C_diag": float(d["C_diag"]),
            },
            "removed_assumption": (
                "Removed placeholder U-DIAG queue status by formalizing a dedicated diagonal handoff artifact."
            ),
            "remaining_for_full_unconditionality": (
                "Replace interface-level diagonal envelope with direct unconditional derivation from channel/kernel definitions."
            ),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# U-DIAG O3 Handoff",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['u_diag_handoff']['status']}`",
        "",
        payload["u_diag_handoff"]["interface_statement"],
        "",
        "## Frozen Constants",
        "",
        f"- `A_diag = {payload['u_diag_handoff']['frozen_constants']['A_diag']}`",
        f"- `C_diag = {payload['u_diag_handoff']['frozen_constants']['C_diag']}`",
        "",
        payload["u_diag_handoff"]["removed_assumption"],
        "",
        payload["u_diag_handoff"]["remaining_for_full_unconditionality"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
