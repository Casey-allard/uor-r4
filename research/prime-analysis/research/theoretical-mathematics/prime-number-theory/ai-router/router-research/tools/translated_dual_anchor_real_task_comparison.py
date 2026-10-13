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


def _required_outputs() -> List[str]:
    return [
        "transfer comparison artifact",
        "update to docs/reports/REAL_TASK_COMPARISON.md",
        "recommendation on whether transfer evidence is strong enough for promotion",
    ]


def build_payload(packet: Dict[str, Any], task_side: Dict[str, Any]) -> Dict[str, Any]:
    if list(packet.get("default_route_ids", [])) != list(task_side.get("default_task_side_route_ids", [])):
        raise ValueError("default task-side route ids do not match packet defaults")
    if list(packet.get("optional_comparator_route_ids", [])) != list(
        task_side.get("optional_task_side_comparator_route_ids", [])
    ):
        raise ValueError("optional task-side comparator route ids do not match packet defaults")
    if list(packet.get("excluded_by_default_route_ids", [])) != list(
        task_side.get("excluded_by_default_route_ids", [])
    ):
        raise ValueError("excluded-by-default task-side route ids do not match packet defaults")

    lower = dict(task_side.get("lower_anchor_default_task_side_read", {}))
    upper = dict(task_side.get("upper_anchor_default_task_side_read", {}))
    comparator = dict(task_side.get("optional_upper_task_side_comparator", {}))

    overall = (
        "Explicit LM-proxy dual-anchor real-task comparison is now fixed: lower bank remains "
        f"{lower.get('classification')} by default via {lower.get('route_id')}, "
        "upper bank remains "
        f"{upper.get('classification')} by default via {upper.get('route_id')}, "
        f"and {comparator.get('route_id')} remains optional comparator-only."
    )

    recommendation = {
        "lower_bank_default": {
            "route_id": lower.get("route_id", ""),
            "verdict": "carry_as_systems_only_default",
            "reason": "lower-bank dense replacement remains systems-first rather than near-frontier quality",
        },
        "upper_bank_default": {
            "route_id": upper.get("route_id", ""),
            "verdict": "carry_as_promoted_real_task_default",
            "reason": "upper-bank routed point remains the default quality-near systems promotion",
        },
        "upper_bank_optional_comparator": {
            "route_id": comparator.get("route_id", ""),
            "verdict": "optional_only",
            "reason": "only reintroduce when an explicit pruning or systems comparator is required",
        },
    }

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "comparison_surface": "lm_proxy_real_task",
        "report_target_path": "docs/reports/REAL_TASK_COMPARISON.md",
        "packet_id": packet.get("packet_id", ""),
        "packet_mode": packet.get("packet_mode", ""),
        "source_packet": packet.get("sources", {}),
        "source_task_side_extension": {
            "packet_id": task_side.get("packet_id", ""),
            "task_side_surface": task_side.get("task_side_surface", ""),
            "task_side_extension_read": task_side.get("task_side_extension_read", ""),
        },
        "default_real_task_route_ids": list(task_side.get("default_task_side_route_ids", [])),
        "optional_real_task_comparator_route_ids": list(
            task_side.get("optional_task_side_comparator_route_ids", [])
        ),
        "excluded_by_default_route_ids": list(task_side.get("excluded_by_default_route_ids", [])),
        "lower_anchor_real_task_read": lower,
        "upper_anchor_real_task_read": upper,
        "optional_upper_real_task_comparator": comparator,
        "overall_real_task_comparison_read": overall,
        "required_outputs": _required_outputs(),
        "promotion_recommendation": recommendation,
        "inheritance_rules": list(task_side.get("inheritance_rules", [])),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_anchor_real_task_read"]
    upper = payload["upper_anchor_real_task_read"]
    comparator = payload["optional_upper_real_task_comparator"]
    recommendation = payload["promotion_recommendation"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Surface: `{payload['comparison_surface']}`",
        f"- Read: {payload['overall_real_task_comparison_read']}",
        "",
        "## Default Real-Task Routes",
        f"- Route ids: `{', '.join(payload['default_real_task_route_ids'])}`",
        "",
        "## Lower Anchor",
        f"- Default routed route: `{lower['route_id']}`",
        f"- Baseline: `{lower['baseline_route_id']}`",
        f"- Classification: `{lower['classification']}`",
        f"- Recommendation: `{recommendation['lower_bank_default']['verdict']}`",
        "",
        "## Upper Anchor",
        f"- Default routed route: `{upper['route_id']}`",
        f"- Baseline: `{upper['baseline_route_id']}`",
        f"- Classification: `{upper['classification']}`",
        f"- Recommendation: `{recommendation['upper_bank_default']['verdict']}`",
        "",
        "## Optional Comparator",
        f"- Route: `{comparator['route_id']}`",
        f"- Inclusion mode: `{comparator['inclusion_mode']}`",
        f"- Recommendation: `{recommendation['upper_bank_optional_comparator']['verdict']}`",
        "",
        "## Required Outputs",
    ]
    for item in payload["required_outputs"]:
        lines.append(f"- {item}")
    lines.extend(["", "## Inheritance Rules"])
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--packet", required=True)
    ap.add_argument("--task-side-analysis", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument(
        "--report-title",
        default="Dual-Anchor Real-Task Comparison",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.packet),
        _load_json(args.task_side_analysis),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
