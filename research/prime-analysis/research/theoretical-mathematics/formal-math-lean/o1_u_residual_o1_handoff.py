#!/usr/bin/env python3
"""Create O1 residual handoff artifact for top-level implication integration."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O1 residual handoff artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o1-scaffold", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o1-budget-check", default="research/output/o1_theorem_budget_endpoint_check_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o1_u_residual_o1_handoff_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o1 = _read_json(args.o1_scaffold)["lemma_o1_a1_ref"]
    b = _read_json(args.o1_budget_check)

    cc = o1["current_constant_candidates"]
    chk = b["grid_endpoint_check"]
    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o1_scaffold": args.o1_scaffold,
            "o1_budget_check": args.o1_budget_check,
        },
        "u_o1_residual_handoff": {
            "status": "conditional_closed_via_o1_interface",
            "interface_statement": (
                "Using the O1 residual interface, |E/sqrt(x) - (a_ref H + b_ref)| <= C0_ref holds for x>=x0 "
                "and W in {30,210,2310,30030}."
            ),
            "frozen_constants": {
                "C0_ref": float(cc["C0_uplifted_sum"]),
                "a_ref": float(cc["a_ref_from_a4"]),
                "b_ref": float(cc["b_ref_from_a4"]),
                "m_ref": int(cc["m_ref_from_a4"]),
            },
            "grid_budget_snapshot": {
                "holds_on_grid": bool(chk["holds_on_grid"]),
                "ratio_max": float(chk["ratio_max"]),
                "gap_min": float(chk["gap_min"]),
            },
            "removed_assumption": "Removed open O1 placeholder in implication integration by formalizing O1 interface handoff.",
            "remaining_for_full_unconditionality": (
                "Upgrade interface to unconditional asymptotic proof independent of finite-window calibration."
            ),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O1 Residual Handoff",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['u_o1_residual_handoff']['status']}`",
        "",
        payload["u_o1_residual_handoff"]["interface_statement"],
        "",
        "## Frozen Constants",
        "",
        f"- `C0_ref = {payload['u_o1_residual_handoff']['frozen_constants']['C0_ref']}`",
        f"- `a_ref = {payload['u_o1_residual_handoff']['frozen_constants']['a_ref']}`",
        f"- `b_ref = {payload['u_o1_residual_handoff']['frozen_constants']['b_ref']}`",
        "",
        payload["u_o1_residual_handoff"]["removed_assumption"],
        "",
        payload["u_o1_residual_handoff"]["remaining_for_full_unconditionality"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
