#!/usr/bin/env python3
"""Validate canonical research state documents for queue drift."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _read(path: Path) -> str:
    return path.read_text()


def _extract_label(text: str, label: str) -> str | None:
    pattern = re.compile(
        rf"^- {re.escape(label)}:\s*(?:\n\s*)?`([^`]+)`",
        re.MULTILINE,
    )
    match = pattern.search(text)
    return match.group(1).strip() if match else None


def _parse_active_state(path: Path) -> dict[str, object]:
    text = _read(path)
    deferred = {}
    deferred_section = re.search(
        r"## Deferred Branches\n(?P<body>.*?)(?:\n## |\Z)", text, re.S
    )
    if deferred_section:
        for inc, doc in re.findall(
            r"- `(INC-\d+)`:\s*\n\s*`([^`]+)`", deferred_section.group("body")
        ):
            deferred[inc] = doc
    return {
        "rr": _extract_label(text, "Current primary RR"),
        "inc": _extract_label(text, "Current primary INC"),
        "stage": _extract_label(text, "Kill-list stage"),
        "doc": _extract_label(text, "Current primary increment doc"),
        "has_earlier_stage_justification": "- Justification:" in text,
        "deferred": deferred,
        "text": text,
    }


def _parse_kill_list_tracker(path: Path) -> dict[str, object]:
    text = _read(path)
    sections = []
    for match in re.finditer(
        r"^## (?P<index>\d+)\. (?P<name>.+?)\n(?P<body>.*?)(?=^## |\Z)",
        text,
        re.M | re.S,
    ):
        body = match.group("body")
        status = _extract_label(body, "Status")
        next_branch = None
        next_match = re.search(r"- Next branch:\s*\n\s*- `?([^`\n]+)`?", body)
        if next_match:
            next_branch = next_match.group(1).strip()
        sections.append(
            {
                "index": int(match.group("index")),
                "name": match.group("name").strip(),
                "status": status,
                "next_branch": next_branch,
            }
        )
    return {
        "rr": _extract_label(text, "Current primary RR"),
        "inc": _extract_label(text, "Current primary INC"),
        "sections": sections,
    }


def _parse_primary_queue(path: Path) -> tuple[str | None, str | None]:
    text = _read(path)
    return (
        _extract_label(text, "Current primary RR"),
        _extract_label(text, "Current primary INC"),
    )


def _validate_branch_doc(path: Path) -> list[str]:
    text = _read(path)
    errors = []
    required_markers = {
        "Kill-List Stage": "## Kill-List Stage",
        "Mathematical Object": None,
        "Success Condition": "## Success Condition",
        "Falsification Condition": "## Falsification Condition",
    }
    if required_markers["Kill-List Stage"] not in text:
        errors.append(f"{path}: missing `## Kill-List Stage`")
    if "## Mathematical Object" not in text and "## Mathematical Object Under Test" not in text:
        errors.append(f"{path}: missing mathematical object section")
    if required_markers["Success Condition"] not in text:
        errors.append(f"{path}: missing `## Success Condition`")
    if required_markers["Falsification Condition"] not in text:
        errors.append(f"{path}: missing `## Falsification Condition`")
    return errors


def validate_state(root: Path = ROOT) -> list[str]:
    errors: list[str] = []

    active_state_path = root / "docs/research/ACTIVE_STATE.md"
    kill_list_path = root / "docs/research/KILL_LIST_TRACKER.md"
    route_matrix_path = root / "docs/routes/ROUTE_MATRIX.md"
    project_board_path = root / "docs/program/PROJECT_BOARD.md"
    issue_registry_path = root / "docs/program/ISSUE_REGISTRY.md"

    active = _parse_active_state(active_state_path)
    kill_list = _parse_kill_list_tracker(kill_list_path)
    route_rr, route_inc = _parse_primary_queue(route_matrix_path)
    board_rr, board_inc = _parse_primary_queue(project_board_path)
    issue_rr, issue_inc = _parse_primary_queue(issue_registry_path)

    rr_values = {
        "ACTIVE_STATE": active["rr"],
        "KILL_LIST_TRACKER": kill_list["rr"],
        "ROUTE_MATRIX": route_rr,
        "PROJECT_BOARD": board_rr,
        "ISSUE_REGISTRY": issue_rr,
    }
    inc_values = {
        "ACTIVE_STATE": active["inc"],
        "KILL_LIST_TRACKER": kill_list["inc"],
        "ROUTE_MATRIX": route_inc,
        "PROJECT_BOARD": board_inc,
        "ISSUE_REGISTRY": issue_inc,
    }

    if len(set(rr_values.values())) != 1:
        errors.append(f"primary RR mismatch: {rr_values}")
    if len(set(inc_values.values())) != 1:
        errors.append(f"primary INC mismatch: {inc_values}")

    queued_branch_docs = []
    if active["doc"]:
        queued_branch_docs.append((active["inc"], active["doc"]))
    queued_count = 0
    for inc, doc in [(active["inc"], active["doc"]), *active["deferred"].items()]:
        if not doc:
            continue
        doc_path = root / doc
        if not doc_path.exists():
            errors.append(f"missing branch doc for {inc}: {doc}")
            continue
        errors.extend(_validate_branch_doc(doc_path))
        text = _read(doc_path)
        if re.search(r"## Status\s*\nQueued next\.", text):
            queued_count += 1
    if queued_count > 1:
        errors.append(f"more than one queued-next branch in canonical state set: {queued_count}")

    active_stage = (active["stage"] or "").strip().lower()
    active_index = None
    earliest_unresolved = None
    for section in kill_list["sections"]:
        name_norm = section["name"].strip().lower()
        if name_norm == active_stage:
            active_index = section["index"]
        if section["status"] in {"open", "partial"} and earliest_unresolved is None:
            earliest_unresolved = section["index"]
    if active_index is None:
        errors.append(
            f"active kill-list stage `{active['stage']}` not found in KILL_LIST_TRACKER"
        )
    elif earliest_unresolved is not None and active_index > earliest_unresolved:
        if not active["has_earlier_stage_justification"]:
            errors.append(
                "later-stage branch is active while earlier unresolved stage exists and no justification is recorded in ACTIVE_STATE"
            )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate canonical research state files.")
    parser.add_argument(
        "--root",
        type=Path,
        default=ROOT,
        help="Repo root to validate.",
    )
    args = parser.parse_args()

    errors = validate_state(args.root)
    if errors:
        print("research state check failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print("research state check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
