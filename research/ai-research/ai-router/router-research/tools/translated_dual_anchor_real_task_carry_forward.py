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


def build_payload(real_task_comparison: Dict[str, Any]) -> Dict[str, Any]:
    lower = dict(real_task_comparison.get("lower_anchor_real_task_read", {}))
    upper = dict(real_task_comparison.get("upper_anchor_real_task_read", {}))
    comparator = dict(real_task_comparison.get("optional_upper_real_task_comparator", {}))
    recommendation = dict(real_task_comparison.get("promotion_recommendation", {}))
    default_route_ids = list(real_task_comparison.get("default_real_task_route_ids", []))
    optional_route_ids = list(real_task_comparison.get("optional_real_task_comparator_route_ids", []))
    excluded_route_ids = list(real_task_comparison.get("excluded_by_default_route_ids", []))

    expected_default = [
        "DENSE_Q01_T2500",
        "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
        "DENSE_Q01_T40000",
        "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
    ]
    if default_route_ids != expected_default:
        raise ValueError(f"unexpected default real-task route ids: {default_route_ids}")
    if optional_route_ids != ["CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"]:
        raise ValueError(f"unexpected optional real-task comparator ids: {optional_route_ids}")

    interpretation = (
        f"The explicit LM-proxy real-task comparison now carries forward {lower.get('route_id')} "
        f"as the lower-bank systems-only default and {upper.get('route_id')} as the promoted "
        f"upper-bank real-task default. {comparator.get('route_id')} remains available only when "
        "a downstream real-task branch explicitly needs a pruning or systems comparator."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "carry_forward_contract": "dual_anchor_real_task_default_comparison",
        "comparison_surface": real_task_comparison.get("comparison_surface", ""),
        "packet_id": real_task_comparison.get("packet_id", ""),
        "packet_mode": real_task_comparison.get("packet_mode", ""),
        "source_real_task_comparison": {
            "comparison_surface": real_task_comparison.get("comparison_surface", ""),
            "overall_real_task_comparison_read": real_task_comparison.get(
                "overall_real_task_comparison_read", ""
            ),
        },
        "default_downstream_real_task_routes": {
            "route_ids": default_route_ids,
            "description": (
                "Default downstream real-task comparison: lower-bank dense baseline plus the "
                "lower-bank systems-only routed point, and upper-bank dense baseline plus the "
                "promoted upper-bank real-task routed reference."
            ),
        },
        "optional_comparators": {
            "route_ids": optional_route_ids,
            "description": (
                "Routes kept available only for explicit upper-bank pruning or systems side-by-side "
                "real-task comparisons."
            ),
        },
        "excluded_by_default": {
            "route_ids": excluded_route_ids,
            "description": (
                "Routes excluded from the default downstream real-task comparison because they are "
                "either comparator-only or carry a nondefault lower-bank pruning/quality read."
            ),
        },
        "lower_bank_default": {
            "route_id": lower.get("route_id", ""),
            "baseline_route_id": lower.get("baseline_route_id", ""),
            "classification": lower.get("classification", ""),
            "quality_read": lower.get("quality_read", ""),
            "recommendation": recommendation.get("lower_bank_default", {}),
        },
        "upper_bank_default": {
            "route_id": upper.get("route_id", ""),
            "baseline_route_id": upper.get("baseline_route_id", ""),
            "classification": upper.get("classification", ""),
            "quality_read": upper.get("quality_read", ""),
            "recommendation": recommendation.get("upper_bank_default", {}),
        },
        "optional_upper_bank_comparator": {
            "route_id": comparator.get("route_id", ""),
            "classification": comparator.get("classification", ""),
            "quality_read": comparator.get("quality_read", ""),
            "recommendation": recommendation.get("upper_bank_optional_comparator", {}),
        },
        "inheritance_rules": list(real_task_comparison.get("inheritance_rules", [])),
        "interpretation": interpretation,
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_bank_default"]
    upper = payload["upper_bank_default"]
    comparator = payload["optional_upper_bank_comparator"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Contract: `{payload['carry_forward_contract']}`",
        f"- Surface: `{payload['comparison_surface']}`",
        f"- Read: {payload['interpretation']}",
        "",
        "## Default Downstream Real-Task Routes",
        f"- Route ids: `{', '.join(payload['default_downstream_real_task_routes']['route_ids'])}`",
        "",
        "## Lower-Bank Default",
        f"- Route: `{lower['route_id']}`",
        f"- Classification: `{lower['classification']}`",
        f"- Recommendation: `{lower['recommendation'].get('verdict', '')}`",
        "",
        "## Upper-Bank Default",
        f"- Route: `{upper['route_id']}`",
        f"- Classification: `{upper['classification']}`",
        f"- Recommendation: `{upper['recommendation'].get('verdict', '')}`",
        "",
        "## Optional Comparator",
        f"- Route: `{comparator['route_id']}`",
        f"- Recommendation: `{comparator['recommendation'].get('verdict', '')}`",
        "",
        "## Inheritance Rules",
    ]
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--real-task-analysis", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Dual-Anchor Real-Task Carry-Forward")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(_load_json(args.real_task_analysis))
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
