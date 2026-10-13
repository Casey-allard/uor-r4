#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _task_side_rules(base_rules: List[str]) -> List[str]:
    rules = list(base_rules)
    rules.append(
        "Task-side comparisons should update docs/reports/REAL_TASK_COMPARISON.md "
        "from this exact inherited packet instead of rebuilding lower-bank or "
        "upper-bank sparse route forks."
    )
    return rules


def build_payload(packet: Dict[str, Any], broader: Dict[str, Any]) -> Dict[str, Any]:
    if list(packet.get("default_route_ids", [])) != list(broader.get("default_packet_route_ids", [])):
        raise ValueError("default packet route ids do not match broader-comparison defaults")
    if list(packet.get("optional_comparator_route_ids", [])) != list(
        broader.get("optional_comparator_route_ids", [])
    ):
        raise ValueError("optional comparator route ids do not match broader-comparison defaults")
    if list(packet.get("excluded_by_default_route_ids", [])) != list(
        broader.get("excluded_by_default_route_ids", [])
    ):
        raise ValueError("excluded-by-default route ids do not match broader-comparison defaults")

    lower = dict(broader.get("lower_anchor_default_read", {}))
    upper = dict(broader.get("upper_anchor_default_read", {}))
    comparator = dict(broader.get("optional_upper_comparator", {}))

    task_side_read = (
        "Task-side dual-anchor inheritance is now explicit: lower bank stays "
        f"{lower.get('classification')} by default via {lower.get('route_id')}, "
        "upper bank stays "
        f"{upper.get('classification')} by default via {upper.get('route_id')}, "
        f"and {comparator.get('route_id')} remains optional comparator-only."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "task_side_surface": "real_task_comparison",
        "task_side_report_path": "docs/reports/REAL_TASK_COMPARISON.md",
        "packet_id": packet.get("packet_id", ""),
        "packet_mode": packet.get("packet_mode", ""),
        "source_packet": packet.get("sources", {}),
        "source_broader_comparison": {
            "packet_id": broader.get("packet_id", ""),
            "overall_dense_frontier_claim": broader.get("overall_dense_frontier_claim", ""),
            "overall_broader_comparison_read": broader.get("overall_broader_comparison_read", ""),
        },
        "task_side_extension_read": task_side_read,
        "lower_anchor_default_task_side_read": {
            "route_id": lower.get("route_id", ""),
            "baseline_route_id": lower.get("baseline_route_id", ""),
            "classification": lower.get("classification", ""),
            "systems_verdict": lower.get("systems_verdict", ""),
            "quality_read": lower.get("quality_read", ""),
            "top1_tolerance_gap_abs": lower.get("top1_tolerance_gap_abs"),
        },
        "upper_anchor_default_task_side_read": {
            "route_id": upper.get("route_id", ""),
            "baseline_route_id": upper.get("baseline_route_id", ""),
            "classification": upper.get("classification", ""),
            "systems_verdict": upper.get("systems_verdict", ""),
            "quality_read": upper.get("quality_read", ""),
            "top1_tolerance_gap_abs": upper.get("top1_tolerance_gap_abs"),
        },
        "optional_upper_task_side_comparator": {
            "route_id": comparator.get("route_id", ""),
            "classification": comparator.get("classification", ""),
            "systems_verdict": comparator.get("systems_verdict", ""),
            "quality_read": comparator.get("quality_read", ""),
            "inclusion_mode": comparator.get("inclusion_mode", ""),
        },
        "default_task_side_route_ids": list(broader.get("default_packet_route_ids", [])),
        "optional_task_side_comparator_route_ids": list(
            broader.get("optional_comparator_route_ids", [])
        ),
        "excluded_by_default_route_ids": list(broader.get("excluded_by_default_route_ids", [])),
        "inheritance_rules": _task_side_rules(list(broader.get("inheritance_rules", []))),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_anchor_default_task_side_read"]
    upper = payload["upper_anchor_default_task_side_read"]
    comparator = payload["optional_upper_task_side_comparator"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Packet id: `{payload['packet_id']}`",
        f"- Task-side surface: `{payload['task_side_surface']}`",
        f"- Read: {payload['task_side_extension_read']}",
        "",
        "## Default Task-Side Routes",
        f"- Route ids: `{', '.join(payload['default_task_side_route_ids'])}`",
        "",
        "## Lower Anchor",
        f"- Default routed route: `{lower['route_id']}`",
        f"- Baseline: `{lower['baseline_route_id']}`",
        f"- Classification: `{lower['classification']}`",
        f"- Quality read: `{lower['quality_read']}`",
        "",
        "## Upper Anchor",
        f"- Default routed route: `{upper['route_id']}`",
        f"- Baseline: `{upper['baseline_route_id']}`",
        f"- Classification: `{upper['classification']}`",
        f"- Quality read: `{upper['quality_read']}`",
        "",
        "## Optional Comparator",
        f"- Route: `{comparator['route_id']}`",
        f"- Inclusion mode: `{comparator['inclusion_mode']}`",
        f"- Classification: `{comparator['classification']}`",
        "",
        "## Inheritance Rules",
    ]
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--packet", required=True)
    ap.add_argument("--broader-analysis", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument(
        "--report-title",
        default="Dual-Anchor Task-Side Extension",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.packet),
        _load_json(args.broader_analysis),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
