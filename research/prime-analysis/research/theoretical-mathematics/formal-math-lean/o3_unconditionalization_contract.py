#!/usr/bin/env python3
"""Create explicit contract for converting O3 conditional closures to unconditional lemmas."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _check_closed(path: str, key: str = "status") -> bool:
    p = Path(path)
    if not p.exists():
        return False
    row = _read_json(path)
    return row.get(key) == "conditional_asymptotic_closed"


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 unconditionalization contract")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--diag", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--offdiag", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--remainder", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--glue", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-bridge-derived", default="research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json")
    ap.add_argument("--u-rem-handoff", default="research/output/o3_u_rem_o2_handoff_2026-02-17.json")
    ap.add_argument("--u-offdiag-handoff", default="research/output/o3_u_offdiag_o3_handoff_2026-02-17.json")
    ap.add_argument("--u-diag-handoff", default="research/output/o3_u_diag_o3_handoff_2026-02-17.json")
    ap.add_argument("--u-uniformity-handoff", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_unconditionalization_contract_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    bridge = _read_json(args.o3_bridge_derived) if Path(args.o3_bridge_derived).exists() else {}
    diag = _read_json(args.diag)
    offdiag = _read_json(args.offdiag)
    rem = _read_json(args.remainder)
    glue = _read_json(args.glue)
    urem = _read_json(args.u_rem_handoff) if Path(args.u_rem_handoff).exists() else {}
    uoff = _read_json(args.u_offdiag_handoff) if Path(args.u_offdiag_handoff).exists() else {}
    udiag = _read_json(args.u_diag_handoff) if Path(args.u_diag_handoff).exists() else {}
    uuni = _read_json(args.u_uniformity_handoff) if Path(args.u_uniformity_handoff).exists() else {}
    urem_closed = urem.get("u_rem_handoff", {}).get("status") in {
        "unconditional_closed_via_o2_interface",
        "unconditional_closed_via_o2_theorem_pack",
    }
    uoff_closed = uoff.get("u_offdiag_handoff", {}).get("status") == "unconditional_closed_via_o3_interface"
    udiag_closed = udiag.get("u_diag_handoff", {}).get("status") == "unconditional_closed_via_o3_interface"
    uuni_closed = uuni.get("u_uniformity_handoff", {}).get("status") == "unconditional_closed_via_o4_interface"

    tasks: List[Dict[str, Any]] = [
        {
            "id": "U-DIAG",
            "component": "E2-DIAG",
            "current_status": "unconditional_closed" if udiag_closed else diag.get("status", "missing"),
            "assumption_to_remove": "assumed explicit diagonal envelope Diag<=C_diag(log x)^A_diag",
            "target_obligation": "O3",
            "required_artifact_type": "theorem proof note deriving diagonal envelope from kernel/channel definitions",
            "handoff_artifact": args.u_diag_handoff if udiag_closed else "",
        },
        {
            "id": "U-OFFDIAG",
            "component": "E2-OFFDIAG-SIGN",
            "current_status": "unconditional_closed" if uoff_closed else offdiag.get("status", "missing"),
            "assumption_to_remove": "assumed Offdiag_abs<=C_abs(log x)^A_abs envelope",
            "target_obligation": "O3",
            "required_artifact_type": "analytic proof converting sign caps + decomposition into explicit offdiag absolute envelope",
            "handoff_artifact": args.u_offdiag_handoff if uoff_closed else "",
        },
        {
            "id": "U-REM",
            "component": "E2-REM",
            "current_status": "unconditional_closed" if urem_closed else rem.get("status", "missing"),
            "assumption_to_remove": "imported O2 theorem assumptions for explicit N(T) error / sum-integral domination",
            "target_obligation": "O2",
            "required_artifact_type": "fully cited O2 theorem lemma proving referenced assumptions in unconditional form",
            "handoff_artifact": args.u_rem_handoff if urem_closed else "",
        },
        {
            "id": "U-UNIFORMITY",
            "component": "Wheel-uniform quantifiers",
            "current_status": "unconditional_closed" if uuni_closed else "open",
            "assumption_to_remove": "implicit wheel-family transfer uniformity",
            "target_obligation": "O4",
            "required_artifact_type": "uniformity lemma covering W in {30,210,2310,30030} for all assembled bounds",
            "handoff_artifact": args.u_uniformity_handoff if uuni_closed else "",
        },
    ]

    open_count = sum(1 for t in tasks if t["current_status"] != "unconditional_closed")
    remaining_order = [x for x in ["U-REM", "U-OFFDIAG", "U-DIAG", "U-UNIFORMITY"] if any(t["id"] == x and t["current_status"] != "unconditional_closed" for t in tasks)]
    gate_ready = (
        _check_closed(args.diag)
        and _check_closed(args.offdiag)
        and _check_closed(args.remainder)
        and _check_closed(args.glue)
        and bool(bridge)
    )

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "diag": args.diag,
            "offdiag": args.offdiag,
            "remainder": args.remainder,
            "glue": args.glue,
            "o3_bridge_derived": args.o3_bridge_derived,
        },
        "o3_unconditionalization_contract": {
            "gate_ready_for_unconditionalization": gate_ready,
            "open_unconditionalization_items": open_count,
            "tasks": tasks,
            "dependency_order": remaining_order,
        },
        "interpretation": (
            "O3 conditional chain is assembled; remaining work is now a bounded unconditionalization program "
            "over O2/O3/O4, not open-ended step creation."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 Unconditionalization Contract",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- gate ready: `{gate_ready}`",
        f"- open items: `{open_count}`",
        "",
        "## Tasks",
        "",
        "| id | component | target obligation | current status |",
        "|---|---|---|---|",
    ]
    for t in tasks:
        lines.append(f"| {t['id']} | {t['component']} | {t['target_obligation']} | {t['current_status']} |")
    lines += [
        "",
        "## Dependency Order",
        "",
    ]
    for item in payload["o3_unconditionalization_contract"]["dependency_order"]:
        lines.append(f"- {item}")
    lines += ["", payload["interpretation"], ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
