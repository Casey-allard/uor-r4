#!/usr/bin/env python3
"""Current-branch wiring audit after W72/W73 tau12 chain closure."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_if_exists(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    try:
        return read_json(path)
    except Exception:
        return None


def main() -> None:
    ap = argparse.ArgumentParser(description="W73 chain wiring audit")
    ap.add_argument(
        "--out-prefix",
        default=(
            "research/output/"
            f"k1_w73_chain_wiring_audit_{datetime.now(timezone.utc).date().isoformat()}"
        ),
    )
    args = ap.parse_args()

    out_dir = Path(__file__).resolve().parent / "output"

    w68 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w68_rational_uniform_buffer.json")
    w69 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w69_onesided_tail_constants.json")
    w70 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w70_tail_contract_alignment.json")
    w71 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w71_chain_wiring_audit.json")
    w72 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w72_c2_source_locked_closure.json")
    w73 = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w73_tau12_chain_composition.json")
    target_lock = load_if_exists(out_dir / "proof_resume_checkpoint_2026-02-24_k1_target_lock.json")

    c2_closed = bool(w72 and "C2(1/2) theorem-style closure achieved" in str(w72.get("status", "")))
    tail_closed = bool(w70 and "derived_contract_statement" in w70)
    composed_closed = bool(w73 and "derived_contract_statement" in w73)

    w71_open = list((w71 or {}).get("open_math_tasks", [])) if w71 else []
    resolved_from_w71: list[str] = []
    unresolved_from_w71: list[str] = []
    for item in w71_open:
        s = str(item)
        if "C2(1/2)" in s or "tau ratio" in s:
            if c2_closed:
                resolved_from_w71.append(s)
            else:
                unresolved_from_w71.append(s)
        elif "combining C2 + rounding-preservation + W70 tail" in s:
            if composed_closed:
                resolved_from_w71.append(s)
            else:
                unresolved_from_w71.append(s)
        else:
            unresolved_from_w71.append(s)

    remaining_blockers: list[str] = []
    if target_lock and target_lock.get("remaining_step"):
        remaining_blockers.append(str(target_lock["remaining_step"]))

    payload: dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "scope": "Current-branch C1-C5 wiring status after W72/W73",
        "checkpoint_presence": {
            "w68": bool(w68),
            "w69": bool(w69),
            "w70": bool(w70),
            "w71": bool(w71),
            "w72": bool(w72),
            "w73": bool(w73),
        },
        "lemma_status": {
            "C1_tau1_anchor_recurrence": (
                "closed_via_buffered_c2_anchor_term" if composed_closed else "not_explicitly_checkpoint_closed"
            ),
            "C2_nonnegative_tau2_gate": (
                "closed_source_locked_assumptions" if c2_closed else "open_math_assumption_conditional"
            ),
            "C3_one_sided_reduction": "closed_symbolic",
            "C4_tail_control": (
                "closed_via_w69_w70_direct_envelope" if tail_closed else "open_math"
            ),
            "C5_admissibility_closure": "closed_symbolic",
        },
        "w71_open_tasks_resolution": {
            "resolved": resolved_from_w71,
            "unresolved": unresolved_from_w71,
        },
        "remaining_blockers": remaining_blockers,
        "conclusion": {
            "tau12_chain_rewired": c2_closed and composed_closed,
            "tail_contract_rewired": tail_closed,
            "overall": (
                "Tau12 branch is no longer broken: W72 closes C2 source-lock assumptions and W73 composes C2+rounding+W70 tail into the active q<a1 chain. "
                "Remaining blocker is the final non-circular K1 source-provider theorem term instantiation."
                if (c2_closed and composed_closed and tail_closed)
                else "Chain still has unresolved tau12/tail composition links."
            ),
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")

    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines = [
        f"# K1 W73 Chain Wiring Audit ({datetime.now(timezone.utc).date().isoformat()})",
        "",
        "## Lemma Status",
    ]
    for k, v in payload["lemma_status"].items():
        lines.append(f"- {k}: `{v}`")
    lines.extend(
        [
            "",
            "## W71 Task Resolution",
            f"- resolved count: `{len(resolved_from_w71)}`",
            f"- unresolved count: `{len(unresolved_from_w71)}`",
        ]
    )
    for item in resolved_from_w71:
        lines.append(f"- resolved: {item}")
    for item in unresolved_from_w71:
        lines.append(f"- unresolved: {item}")
    lines.extend(["", "## Remaining Blocker"])
    for b in remaining_blockers:
        lines.append(f"- {b}")
    lines.extend(["", "## Conclusion", payload["conclusion"]["overall"], ""])

    out_md.write_text("\n".join(lines), encoding="utf-8")
    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
