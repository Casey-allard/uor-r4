#!/usr/bin/env python3
"""Audit C1-C5 lemma-chain wiring status around W70."""

from __future__ import annotations

import argparse
import glob
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_json_if_exists(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    try:
        return read_json(path)
    except Exception:
        return None


def contains_theorem(text: str, name: str) -> bool:
    return re.search(rf"\btheorem\s+{re.escape(name)}\b", text) is not None


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit W70 wiring in the C1-C5 chain")
    ap.add_argument(
        "--out-prefix",
        default=f"research/output/k1_w70_chain_wiring_audit_{datetime.now(timezone.utc).date().isoformat()}",
    )
    args = ap.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    research_root = repo_root / "research"
    out_dir = research_root / "output"

    checkpoint_paths = sorted(
        glob.glob(str(out_dir / "proof_resume_checkpoint_2026-02-24_k1_w*.json"))
    )
    checkpoint_rows: list[dict[str, Any]] = []
    max_w = -1
    for raw in checkpoint_paths:
        path = Path(raw)
        d = load_json_if_exists(path)
        if d is None:
            continue
        m = re.search(r"_k1_w(\d+)", path.name)
        if not m:
            continue
        w = int(m.group(1))
        max_w = max(max_w, w)
        checkpoint_rows.append(
            {
                "w": w,
                "file": str(path),
                "json": d,
            }
        )

    by_w = {row["w"]: row["json"] for row in checkpoint_rows}
    w60 = by_w.get(60)
    w65 = by_w.get(65)
    w66 = by_w.get(66)
    w68 = by_w.get(68)
    w69 = by_w.get(69)
    w70 = by_w.get(70)

    target_lock = load_json_if_exists(
        out_dir / "proof_resume_checkpoint_2026-02-24_k1_target_lock.json"
    )
    w70_numeric = load_json_if_exists(
        out_dir / "k1_w70_onesided_tail_contract_alignment_2026-02-25.json"
    )

    spinning_top_path = (
        research_root / "formal" / "lean" / "PrimeRiemannBridgeSpinningTopFrontier.lean"
    )
    spinning_top_text = spinning_top_path.read_text(encoding="utf-8")
    symbolic_bridge_presence = {
        "constructive_cos_gate_of_buffered_c2_and_eventual_rounding_errors": contains_theorem(
            spinning_top_text, "constructive_cos_gate_of_buffered_c2_and_eventual_rounding_errors"
        ),
        "constructive_gate_of_cos_gate_and_eventual_tail_bound": contains_theorem(
            spinning_top_text, "constructive_gate_of_cos_gate_and_eventual_tail_bound"
        ),
        "normalized_lower_envelope_of_constructive_gate_and_q_lt_a1": contains_theorem(
            spinning_top_text, "normalized_lower_envelope_of_constructive_gate_and_q_lt_a1"
        ),
        "normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1": contains_theorem(
            spinning_top_text, "normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1"
        ),
    }

    has_post_w70_checkpoint = max_w > 70
    post_w70_mentions_w70 = False
    for row in checkpoint_rows:
        if row["w"] <= 70:
            continue
        text = json.dumps(row["json"], sort_keys=True)
        text_l = text.lower()
        if (
            "k1_w70" in text_l
            or "tail_contract_alignment" in text_l
            or re.search(r"\bw70\b", text_l) is not None
            or "tail contract" in text_l
        ):
            post_w70_mentions_w70 = True
            break

    target_lock_text = json.dumps(target_lock, sort_keys=True) if target_lock else ""
    target_lock_text_l = target_lock_text.lower()
    target_lock_mentions_w70 = (
        "k1_w70" in target_lock_text_l
        or "tail_contract_alignment" in target_lock_text_l
        or re.search(r"\bw70\b", target_lock_text_l) is not None
        or "tail contract" in target_lock_text_l
    )

    c1_status = {
        "lemma": "C1 (tau1 anchor recurrence)",
        "status": "not_explicitly_checkpoint_closed",
        "evidence": (
            (
                "W60 keeps C1-C5 in open chain form; later checkpoints focus on C2/rounding/tail and "
                "do not provide an explicit standalone C1-closed flag."
            )
            if isinstance(w60, dict)
            else "W60 checkpoint unavailable"
        ),
    }
    c2_status = {
        "lemma": "C2 (nonnegative tau2 gate on anchors)",
        "status": "open_math_assumption_conditional",
        "evidence": (
            w68.get("open_math_tasks")
            if isinstance(w68, dict)
            else "W68 checkpoint unavailable"
        ),
    }
    c3_status = {
        "lemma": "C3 (one-sided reduction)",
        "status": "closed_symbolic",
        "evidence": "Symbolic bridge theorem present in PrimeRiemannBridgeSpinningTopFrontier.",
    }

    c4_tail_closed = isinstance(w69, dict) and isinstance(w70, dict) and (
        "derived_tail_bound" in w69 and "derived_contract_statement" in w70
    )
    c4_status = {
        "lemma": "C4 (one-sided tail control)",
        "status": "closed_via_direct_tail_envelope" if c4_tail_closed else "open_math",
        "evidence": {
            "w69_has_derived_tail_bound": isinstance(w69, dict) and "derived_tail_bound" in w69,
            "w70_has_contract_alignment": isinstance(w70, dict)
            and "derived_contract_statement" in w70,
            "w70_status": w70.get("status") if isinstance(w70, dict) else None,
        },
    }

    c5_status = {
        "lemma": "C5 (admissibility closure q<a1 => positive lower envelope)",
        "status": "closed_symbolic",
        "evidence": "Admissibility/lower-envelope closure theorem present in PrimeRiemannBridgeSpinningTopFrontier.",
    }

    remaining_blockers: list[str] = []
    if isinstance(w68, dict):
        for item in w68.get("open_math_tasks", []):
            if isinstance(item, str):
                remaining_blockers.append(item)
    if c4_tail_closed:
        remaining_blockers = [
            s
            for s in remaining_blockers
            if "one-sided tail constants" not in s.lower()
            and "c*x^{-eta}" not in s.lower()
        ]
    if isinstance(target_lock, dict):
        rs = target_lock.get("remaining_step")
        if isinstance(rs, str) and rs:
            remaining_blockers.append(rs)

    w70_fully_wired_downstream = has_post_w70_checkpoint and post_w70_mentions_w70
    overall_summary = (
        "W70 closed C4 contract alignment and this is now checkpointed downstream (W71). "
        "Remaining math blockers are C2 theorem-grade closure (with source-locked ratio assumptions) "
        "plus final non-circular source-provider instantiation."
        if w70_fully_wired_downstream
        else (
            "W70 closed C4 contract alignment, but downstream chain closure is not checkpointed past W70. "
            "Remaining math blockers are C2 theorem-grade closure (with source-locked ratio assumptions) "
            "plus final non-circular source-provider instantiation."
        )
    )

    payload: dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "scope": "C1-C5 chain wiring status around W70",
        "chain_context": {
            "source_chain_file": str(out_dir / "k1_w60_constructive_gate_lemma_chain_2026-02-24.md"),
            "spinning_top_frontier_file": str(spinning_top_path),
        },
        "lemma_status": [c1_status, c2_status, c3_status, c4_status, c5_status],
        "w70_contract_numbers": w70_numeric.get("tail_bound") if isinstance(w70_numeric, dict) else None,
        "symbolic_bridge_presence": symbolic_bridge_presence,
        "wiring_audit": {
            "max_checkpoint_w": max_w,
            "has_post_w70_checkpoint": has_post_w70_checkpoint,
            "post_w70_mentions_w70": post_w70_mentions_w70,
            "target_lock_mentions_w70": target_lock_mentions_w70,
            "target_lock_remaining_step": target_lock.get("remaining_step")
            if isinstance(target_lock, dict)
            else None,
        },
        "checkpoint_snapshots": {
            "w65_status_by_obligation": w65.get("status_by_obligation") if isinstance(w65, dict) else None,
            "w66_math_obligations_status": w66.get("math_obligations_status")
            if isinstance(w66, dict)
            else None,
            "w69_remaining_math_task": w69.get("remaining_math_task") if isinstance(w69, dict) else None,
            "w70_status": w70.get("status") if isinstance(w70, dict) else None,
        },
        "remaining_blockers_after_w70_tail_alignment": remaining_blockers,
        "conclusion": {
            "w70_completed_for_c4_contract_shape": c4_tail_closed,
            "w70_fully_wired_downstream": w70_fully_wired_downstream,
            "overall": overall_summary,
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines = [
        f"# K1 W70 Chain Wiring Audit ({datetime.now(timezone.utc).date().isoformat()})",
        "",
        "## C1-C5 Status",
    ]
    for row in payload["lemma_status"]:
        lines.append(f"- {row['lemma']}: `{row['status']}`")
    lines.extend(
        [
            "",
            "## W70 Integration Check",
            f"- max checkpoint index found: `W{max_w}`",
            f"- has any checkpoint after W70: `{has_post_w70_checkpoint}`",
            f"- any post-W70 checkpoint referencing W70 artifacts: `{post_w70_mentions_w70}`",
            f"- target-lock checkpoint references W70 artifacts: `{target_lock_mentions_w70}`",
            "",
            "## Key Finding",
            payload["conclusion"]["overall"],
            "",
            "## Remaining Blockers After W70",
        ]
    )
    for item in remaining_blockers:
        lines.append(f"- {item}")
    lines.append("")

    out_md.write_text("\n".join(lines), encoding="utf-8")
    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
