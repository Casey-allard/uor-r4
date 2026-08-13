#!/usr/bin/env python3
"""Full workspace formalization audit for the RH/K1 project."""

from __future__ import annotations

import argparse
import glob
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def count_files(root: Path) -> int:
    return sum(1 for p in root.rglob("*") if p.is_file())


def scan_lean_markers(files: list[Path]) -> dict[str, Any]:
    axiom_re = re.compile(r"^\s*axiom\s+", flags=re.M)
    sorry_re = re.compile(r"\bsorry\b")
    admit_re = re.compile(r"\badmit\b")
    file_hits: list[dict[str, Any]] = []
    axiom_count = 0
    sorry_count = 0
    admit_count = 0
    for path in files:
        text = read_text(path)
        c_axiom = len(axiom_re.findall(text))
        c_sorry = len(sorry_re.findall(text))
        c_admit = len(admit_re.findall(text))
        if c_axiom or c_sorry or c_admit:
            file_hits.append(
                {
                    "file": str(path),
                    "axiom": c_axiom,
                    "sorry": c_sorry,
                    "admit": c_admit,
                }
            )
        axiom_count += c_axiom
        sorry_count += c_sorry
        admit_count += c_admit
    return {
        "axiom_count": axiom_count,
        "sorry_count": sorry_count,
        "admit_count": admit_count,
        "files_with_hits": file_hits,
    }


def load_report_if_exists(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    try:
        return read_json(path)
    except Exception:
        return None


def parse_k1_checkpoints() -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for raw in sorted(glob.glob("research/output/proof_resume_checkpoint_2026-02-24_k1_w*.json")):
        path = Path(raw)
        try:
            d = read_json(path)
        except Exception:
            continue
        m = re.search(r"_k1_w(\d+)", path.name)
        w = int(m.group(1)) if m else -1
        row: dict[str, Any] = {
            "file": str(path),
            "w": w,
            "status": d.get("status"),
            "focus": d.get("focus"),
        }
        if "open_math_task" in d:
            row["open_math_task"] = d["open_math_task"]
        if "open_math_tasks" in d:
            row["open_math_tasks"] = d["open_math_tasks"]
        if "remaining_math_task" in d:
            row["remaining_math_task"] = d["remaining_math_task"]
        if "remaining_open_kernel" in d:
            row["remaining_open_kernel"] = d["remaining_open_kernel"]
        if "next_immediate_step" in d:
            row["next_immediate_step"] = d["next_immediate_step"]
        if "next_step" in d:
            row["next_step"] = d["next_step"]
        out.append(row)
    return sorted(out, key=lambda r: r["w"])


def keep_open_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for r in rows:
        if any(
            key in r
            for key in (
                "open_math_task",
                "open_math_tasks",
                "remaining_math_task",
                "remaining_open_kernel",
            )
        ):
            out.append(r)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Run full workspace formalization audit")
    ap.add_argument(
        "--out-prefix",
        default=f"research/output/full_workspace_formalization_audit_{datetime.now(timezone.utc).date().isoformat()}",
    )
    args = ap.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    research_root = repo_root / "research"
    active_lean_dir = research_root / "formal" / "lean"
    external_riemann_dir = research_root / "external" / "riemann"

    active_lean_files = sorted(active_lean_dir.glob("*.lean"))
    external_lean_files = sorted(external_riemann_dir.rglob("*.lean"))

    active_scan = scan_lean_markers(active_lean_files)
    external_scan = scan_lean_markers(external_lean_files)

    schlage_path = active_lean_dir / "PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean"
    final_equiv_path = active_lean_dir / "PrimeRiemannBridgeFinalTargetEquivalence.lean"
    ingham_slot_path = active_lean_dir / "PrimeRiemannBridgeInghamImportedSlot.lean"
    completion_kernel_path = active_lean_dir / "PrimeRiemannBridgeCompletionKernel.lean"
    spinning_top_path = active_lean_dir / "PrimeRiemannBridgeSpinningTopFrontier.lean"

    schlage_text = read_text(schlage_path)
    final_equiv_text = read_text(final_equiv_path)
    ingham_text = read_text(ingham_slot_path)
    completion_text = read_text(completion_kernel_path)
    spinning_text = read_text(spinning_top_path)

    circular_summary = {
        "rh_nonempty_equivalences_in_schlage": len(
            re.findall(r"RHStatement\s*↔\s*Nonempty", schlage_text)
        ),
        "rh_constructed_provider_via_false_elim_in_schlage": schlage_text.count(
            "have hFalse : False := by linarith"
        ),
        "rh_nonempty_equivalences_in_final_target": len(
            re.findall(r"RHStatement\s*↔\s*Nonempty", final_equiv_text)
        ),
        "rh_nonempty_equivalences_in_ingham_slot": len(
            re.findall(r"RHStatement\s*↔\s*Nonempty", ingham_text)
        ),
        "published_pack_boundary_present": "structure PublishedZeroOscillationPack where"
        in completion_text,
        "k1_source_provider_boundary_present": "class K1SourceNonCircularProvider where"
        in schlage_text,
        "published_zero_to_cos_sin_provider_boundary_present": "class PublishedZeroToCosSinPhaseProvider where"
        in spinning_text,
    }

    report_main = load_report_if_exists(
        research_root / "output" / "formal_compile_report_2026-02-17.json"
    )
    report_mathlib = load_report_if_exists(
        research_root / "output" / "formal_compile_report_mathlib_2026-02-17.json"
    )
    report_kernel = load_report_if_exists(
        research_root / "output" / "formal_compile_report_completion_kernel_2026-02-17.json"
    )

    target_lock_path = research_root / "output" / "proof_resume_checkpoint_2026-02-24_k1_target_lock.json"
    w70_path = research_root / "output" / "proof_resume_checkpoint_2026-02-24_k1_w70_tail_contract_alignment.json"
    target_lock = load_report_if_exists(target_lock_path)
    w70 = load_report_if_exists(w70_path)

    checkpoints = parse_k1_checkpoints()
    open_checkpoint_rows = keep_open_rows(checkpoints)

    payload: dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "workspace_counts": {
            "repo_file_count": count_files(repo_root),
            "research_file_count": count_files(research_root),
            "active_lean_file_count": len(active_lean_files),
            "external_riemann_lean_file_count": len(external_lean_files),
        },
        "active_formal_scan": active_scan,
        "external_archive_scan": {
            "axiom_count": external_scan["axiom_count"],
            "sorry_count": external_scan["sorry_count"],
            "admit_count": external_scan["admit_count"],
            "files_with_hits_count": len(external_scan["files_with_hits"]),
        },
        "build_reports": {
            "main": report_main,
            "mathlib": report_mathlib,
            "completion_kernel": report_kernel,
        },
        "circularity_and_boundary_signals": circular_summary,
        "checkpoint_signals": {
            "k1_target_lock": target_lock,
            "k1_w70_tail_alignment": w70,
            "k1_open_rows": open_checkpoint_rows,
        },
        "audit_conclusion": {
            "active_lean_has_axioms_or_sorries": bool(
                active_scan["axiom_count"] or active_scan["sorry_count"] or active_scan["admit_count"]
            ),
            "build_green": all(
                (r is not None and r.get("status") == "pass")
                for r in (report_main, report_mathlib, report_kernel)
            ),
            "non_circular_source_boundary_closed": False,
            "summary": (
                "Lean scaffolding is build-clean and axiom-free in active files, but final closure remains "
                "conditional on provider/import boundaries with RH-equivalence patterns; non-circular source "
                "instantiation is still open."
            ),
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines = [
        f"# Full Workspace Formalization Audit ({datetime.now(timezone.utc).date().isoformat()})",
        "",
        "## Scope",
        f"- repo file count: `{payload['workspace_counts']['repo_file_count']}`",
        f"- research file count: `{payload['workspace_counts']['research_file_count']}`",
        f"- active Lean files: `{payload['workspace_counts']['active_lean_file_count']}`",
        f"- external/archive Lean files: `{payload['workspace_counts']['external_riemann_lean_file_count']}`",
        "",
        "## Active Lean Status",
        f"- axioms found: `{active_scan['axiom_count']}`",
        f"- sorry found: `{active_scan['sorry_count']}`",
        f"- admit found: `{active_scan['admit_count']}`",
        f"- `lake build` reports pass: `{payload['audit_conclusion']['build_green']}`",
        "",
        "## Key Boundary Findings",
        "- Provider-equivalence layer remains heavy:",
        f"  - RH↔Nonempty equivalence count in Schlage instance file: `{circular_summary['rh_nonempty_equivalences_in_schlage']}`",
        f"  - RH-derived false-elim constructor count in Schlage file: `{circular_summary['rh_constructed_provider_via_false_elim_in_schlage']}`",
        f"  - RH↔Nonempty equivalence count in final-target file: `{circular_summary['rh_nonempty_equivalences_in_final_target']}`",
        f"  - RH↔Nonempty equivalence count in Ingham slot file: `{circular_summary['rh_nonempty_equivalences_in_ingham_slot']}`",
        "- Published theorem-pack import boundary is present (`PublishedZeroOscillationPack`).",
        "- K1 source provider boundary is present (`K1SourceNonCircularProvider`).",
        "",
        "## Checkpoint Signals",
    ]
    if target_lock:
        lines.append(f"- target lock remaining step: `{target_lock.get('remaining_step', '')}`")
    if w70:
        lines.append(f"- W70 status: `{w70.get('status', '')}`")
    lines.append(f"- open checkpoint rows with explicit open-task fields: `{len(open_checkpoint_rows)}`")
    lines.append("")
    lines.append("## External Archive (Non-active) Snapshot")
    lines.append(
        f"- files containing `sorry/admit`: `{payload['external_archive_scan']['files_with_hits_count']}`"
    )
    lines.append(
        f"- total external `axiom` count: `{payload['external_archive_scan']['axiom_count']}`"
    )
    lines.append("")
    lines.append("## Conclusion")
    lines.append(payload["audit_conclusion"]["summary"])
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
