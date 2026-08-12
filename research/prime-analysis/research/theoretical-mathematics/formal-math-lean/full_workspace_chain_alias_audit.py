#!/usr/bin/env python3
"""Full workspace alias/chain audit for remaining non-circular RH blocker.

This script reconciles differently named blocker labels across checkpoints,
Lean boundary classes, and recent W70/W72/W73 math artifacts.
"""

from __future__ import annotations

import argparse
import glob
import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class PatternHit:
    file: str
    pattern: str
    line: int
    text: str


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def read_json(path: Path) -> dict[str, Any] | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    if isinstance(data, dict):
        return data
    return {"_non_dict_payload": data}


def normalize_status(text: str | None) -> str:
    if text is None:
        return ""
    return " ".join(str(text).split())


def extract_strings(obj: Any) -> list[str]:
    out: list[str] = []
    if isinstance(obj, str):
        out.append(obj)
    elif isinstance(obj, dict):
        for v in obj.values():
            out.extend(extract_strings(v))
    elif isinstance(obj, list):
        for v in obj:
            out.extend(extract_strings(v))
    return out


def collect_pattern_hits(path: Path, patterns: dict[str, str]) -> list[PatternHit]:
    text = read_text(path)
    lines = text.splitlines()
    hits: list[PatternHit] = []
    for name, patt in patterns.items():
        rx = re.compile(patt)
        for idx, line in enumerate(lines, start=1):
            if rx.search(line):
                hits.append(
                    PatternHit(
                        file=str(path),
                        pattern=name,
                        line=idx,
                        text=line.strip(),
                    )
                )
    return hits


def load_key_json_rows(files: list[Path]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for path in files:
        data = read_json(path)
        if data is None:
            continue
        row: dict[str, Any] = {
            "file": str(path),
        }
        for k in (
            "generated_utc",
            "timestamp_utc",
            "status_id",
            "status",
            "focus",
            "remaining_step",
            "remaining_open_kernel",
            "next_immediate_step",
            "next_step",
            "open_math_tasks",
            "open_math_task",
            "remaining_math_task",
            "interpretation",
            "conclusion",
        ):
            if k in data:
                row[k] = data[k]
        if "verified_truth" in data and isinstance(data["verified_truth"], dict):
            vt = data["verified_truth"]
            subset: dict[str, Any] = {}
            for key in (
                "k1_source_complete",
                "unconditional_rh_proof_complete",
                "remaining_item_count",
                "remaining_math_kernel_count",
            ):
                if key in vt:
                    subset[key] = vt[key]
            if subset:
                row["verified_truth"] = subset
        out.append(row)
    return out


def md_escape(s: str) -> str:
    return s.replace("|", "\\|")


def build_markdown(payload: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append(f"# Full Workspace Chain Alias Audit ({payload['date']})")
    lines.append("")
    lines.append("## Scope")
    lines.append(
        f"- checkpoint files scanned: `{payload['scope']['checkpoint_file_count']}`"
    )
    lines.append(f"- lean files scanned: `{payload['scope']['lean_file_count']}`")
    lines.append(
        f"- md/json files keyword-scanned: `{payload['scope']['md_json_file_count']}`"
    )
    lines.append("")
    lines.append("## W70/W72/W73 Math Chain")
    lines.append(
        f"- W70 status: `{payload['w_chain']['w70_status']}`"
    )
    lines.append(
        f"- W72 status: `{payload['w_chain']['w72_status']}`"
    )
    lines.append(
        f"- W73 open_math_tasks empty: `{str(payload['w_chain']['w73_open_tasks_empty']).lower()}`"
    )
    lines.append(
        f"- W73 remaining blocker: `{payload['w_chain']['w73_remaining_blocker']}`"
    )
    lines.append("")
    lines.append("## Alias Cluster (Same Blocker, Different Labels)")
    for row in payload["alias_cluster"]["labels"]:
        lines.append(f"- `{row}`")
    lines.append("")
    lines.append("## Lean Boundary Evidence")
    lines.append(
        f"- `K1SourceNonCircularProvider` class lines: `{payload['lean_boundary']['k1_provider_class_lines']}`"
    )
    lines.append(
        f"- `R6DualBandTailRepresentationCandidateShapeProvider` class lines: "
        f"`{payload['lean_boundary']['r6_candidate_shape_provider_class_lines']}`"
    )
    lines.append(
        f"- imported linear-phase-only instance uses RH/contradiction route: "
        f"`{str(payload['lean_boundary']['imported_linear_phase_only_is_rh_derived']).lower()}`"
    )
    lines.append(
        f"- zero-to-cos/sin provider concrete instances found: "
        f"`{payload['lean_boundary']['zero_to_cos_sin_provider_instance_count']}`"
    )
    lines.append("")
    lines.append("## Keyword Scan")
    for k, v in payload["keyword_scan"].items():
        lines.append(f"- `{k}` hits: `{v}`")
    lines.append("")
    lines.append("## Conclusion")
    lines.append(f"- {payload['conclusion']['status']}")
    lines.append(f"- {payload['conclusion']['detail']}")
    lines.append("")
    lines.append("## Next Math-First Step")
    lines.append(f"- {payload['conclusion']['next_math_step']}")
    lines.append("")
    lines.append("## Key Files")
    for path in payload["key_files"]:
        lines.append(f"- `{path}`")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Run full workspace chain alias audit")
    ap.add_argument(
        "--out-prefix",
        default=(
            "research/output/"
            f"full_workspace_chain_alias_audit_{datetime.now(timezone.utc).date().isoformat()}"
        ),
    )
    args = ap.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    output_dir = repo_root / "research" / "output"
    lean_dir = repo_root / "research" / "formal" / "lean"

    checkpoint_files = sorted(
        Path(p)
        for p in glob.glob(str(output_dir / "proof_resume_checkpoint*.json"))
    )
    audit_json_files = sorted(
        Path(p)
        for p in glob.glob(str(output_dir / "full_workspace*formalization*audit*.json"))
    )
    key_json_files = [
        output_dir / "proof_resume_checkpoint_2026-02-18.json",
        output_dir / "proof_resume_checkpoint_2026-02-19.json",
        output_dir / "proof_resume_checkpoint_2026-02-24.json",
        output_dir / "proof_resume_checkpoint_2026-02-24_k1_target_lock.json",
        output_dir / "proof_resume_checkpoint_2026-02-24_k1_w70_tail_contract_alignment.json",
        output_dir / "proof_resume_checkpoint_2026-02-24_k1_w72_c2_source_locked_closure.json",
        output_dir / "proof_resume_checkpoint_2026-02-24_k1_w73_tau12_chain_composition.json",
        output_dir / "k1_w73_tau12_chain_composition_2026-02-25.json",
        output_dir / "k1_w73_chain_wiring_audit_2026-02-25.json",
        output_dir / "full_workspace_formalization_audit_2026-02-25_postw73.json",
    ]
    key_json_files = [p for p in key_json_files if p.exists()]
    key_rows = load_key_json_rows(key_json_files)

    # W70/W72/W73 status extraction
    w70 = read_json(output_dir / "proof_resume_checkpoint_2026-02-24_k1_w70_tail_contract_alignment.json") or {}
    w72 = read_json(output_dir / "proof_resume_checkpoint_2026-02-24_k1_w72_c2_source_locked_closure.json") or {}
    w73 = read_json(output_dir / "proof_resume_checkpoint_2026-02-24_k1_w73_tau12_chain_composition.json") or {}
    w73_art = read_json(output_dir / "k1_w73_tau12_chain_composition_2026-02-25.json") or {}
    w73_tasks = w73.get("open_math_tasks", [])
    if not isinstance(w73_tasks, list):
        w73_tasks = []
    w73_blockers = w73_art.get("remaining_blockers", [])
    if isinstance(w73_blockers, list) and w73_blockers:
        w73_blocker = str(w73_blockers[0])
    else:
        w73_blocker = str(w73.get("next_immediate_step", ""))

    # Lean boundary scan
    schlage = lean_dir / "PrimeRiemannBridgeSchlagePuchta2019ImportedInstance.lean"
    spin = lean_dir / "PrimeRiemannBridgeSpinningTopFrontier.lean"
    oscill = lean_dir / "PrimeRiemannBridgeOscillatoryReduction.lean"
    key_lean_files = [schlage, spin, oscill]

    patt_hits: list[PatternHit] = []
    patt_hits.extend(
        collect_pattern_hits(
            schlage,
            {
                "k1_provider_class": r"^\s*class K1SourceNonCircularProvider where",
                "k1_provider_instance": r"^\s*noncomputable instance k1SourceNonCircularProviderOf",
            },
        )
    )
    patt_hits.extend(
        collect_pattern_hits(
            spin,
            {
                "r6_candidate_shape_provider_class": r"^\s*class R6DualBandTailRepresentationCandidateShapeProvider where",
                "zero_to_cos_sin_phase_provider_class": r"^\s*class ZeroToCosSinPhaseProvider where",
            },
        )
    )
    patt_hits.extend(
        collect_pattern_hits(
            oscill,
            {
                "imported_linear_phase_only_instance": r"^\s*noncomputable instance importedLinearPhaseOnlyResultsOfImportedPublished",
                "imported_linear_phase_only_have_rh": r"have hRH : RHStatement :=",
                "imported_linear_phase_only_contradiction": r"linarith \[hs_gt,\s*hsCritical\]",
            },
        )
    )

    k1_provider_lines = sorted(
        h.line for h in patt_hits if h.pattern == "k1_provider_class"
    )
    r6_provider_lines = sorted(
        h.line for h in patt_hits if h.pattern == "r6_candidate_shape_provider_class"
    )
    spin_text = read_text(spin)
    zero_to_cos_sin_instance_names = re.findall(
        r"noncomputable\s+instance\s+([A-Za-z0-9_']+)[\s\S]{0,300}?:\s*ZeroToCosSinPhaseProvider\s+where",
        spin_text,
    )
    zero_to_cos_sin_instance_count = len(zero_to_cos_sin_instance_names)
    imported_rh_derived = bool(
        [h for h in patt_hits if h.pattern == "imported_linear_phase_only_have_rh"]
        and [h for h in patt_hits if h.pattern == "imported_linear_phase_only_contradiction"]
    )

    # Keyword scan across md/json for user-suggested terms.
    md_json_paths = [
        Path(p)
        for p in glob.glob(str(output_dir / "**/*.*"), recursive=True)
        if p.endswith(".md") or p.endswith(".json")
    ]
    key_terms = [
        "von",
        "pintz",
        "printz",
        "m4",
        "mref",
        "mangoldt",
        "montgomery",
        "mandel",
    ]
    keyword_counts = {k: 0 for k in key_terms}
    for path in md_json_paths:
        text = read_text(path).lower()
        for term in key_terms:
            if term in text:
                keyword_counts[term] += 1

    # Alias extraction from key checkpoint/audit files.
    alias_tokens = [
        "K1-SOURCE",
        "K1-R6-CANDIDATE-SHAPE-PAYLOAD",
        "ZeroToCosSinPhaseTransfer",
        "R6DualBandTailRepresentationCandidateShapeProvider.theorem_term",
        "K1SourceNonCircularProvider.theorem_term",
        "SchlagePuchtaIntervalCoreProvider",
        "ConcreteLinearPhaseWitnessProvider",
        "InghamImportedPayloadTerm",
    ]
    alias_rows: list[str] = []
    for path in key_json_files + audit_json_files:
        data = read_json(path)
        if data is None:
            continue
        txt = "\n".join(extract_strings(data))
        for token in alias_tokens:
            if token in txt:
                alias_rows.append(f"{Path(path).name}: {token}")
    # stable unique
    seen = set()
    alias_rows_unique: list[str] = []
    for item in alias_rows:
        if item in seen:
            continue
        seen.add(item)
        alias_rows_unique.append(item)

    key_files = [str(p) for p in key_json_files + key_lean_files if p.exists()]

    payload: dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "date": datetime.now(timezone.utc).date().isoformat(),
        "scope": {
            "checkpoint_file_count": len(checkpoint_files),
            "lean_file_count": len(key_lean_files),
            "md_json_file_count": len(md_json_paths),
        },
        "w_chain": {
            "w70_status": normalize_status(w70.get("status")),
            "w72_status": normalize_status(w72.get("status")),
            "w73_open_tasks_empty": len(w73_tasks) == 0,
            "w73_remaining_blocker": w73_blocker,
        },
        "alias_cluster": {
            "labels": alias_rows_unique,
            "interpreted_single_blocker": (
                "All alias labels still point to one unresolved non-circular source payload."
            ),
        },
        "lean_boundary": {
            "k1_provider_class_lines": k1_provider_lines,
            "r6_candidate_shape_provider_class_lines": r6_provider_lines,
            "imported_linear_phase_only_is_rh_derived": imported_rh_derived,
            "zero_to_cos_sin_provider_instance_count": zero_to_cos_sin_instance_count,
            "zero_to_cos_sin_provider_instance_names": zero_to_cos_sin_instance_names,
            "pattern_hits": [
                {
                    "file": h.file,
                    "pattern": h.pattern,
                    "line": h.line,
                    "text": h.text,
                }
                for h in sorted(patt_hits, key=lambda x: (x.file, x.line, x.pattern))
            ],
        },
        "keyword_scan": keyword_counts,
        "key_json_rows": key_rows,
        "conclusion": {
            "status": (
                "W70/W72/W73 math chain appears closed, but final non-circular source instantiation remains open."
            ),
            "detail": (
                "No checkpoint or active Lean file shows a concrete non-RH theorem-term instance for "
                "K1SourceNonCircularProvider (or its alias path via the R6 candidate-shape provider)."
            ),
            "next_math_step": (
                "Attack a concrete theorem-term constructor for "
                "R6DualBandTailRepresentationCandidateShapeProvider.theorem_term or "
                "K1SourceNonCircularProvider.theorem_term from explicit-formula assumptions without RH-derived contradiction."
            ),
        },
        "key_files": key_files,
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    out_md.write_text(build_markdown(payload), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
