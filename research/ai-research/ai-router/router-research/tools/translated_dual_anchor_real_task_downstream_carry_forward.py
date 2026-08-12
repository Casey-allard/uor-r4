#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def build_payload(downstream_comparison: Dict[str, Any]) -> Dict[str, Any]:
    lower = dict(downstream_comparison.get("lower_anchor_downstream_read", {}))
    upper = dict(downstream_comparison.get("upper_anchor_downstream_read", {}))
    comparator = dict(downstream_comparison.get("optional_upper_downstream_comparator", {}))
    recommendation = dict(downstream_comparison.get("promotion_recommendation", {}))
    default_route_ids = list(downstream_comparison.get("default_downstream_route_ids", []))
    optional_route_ids = list(
        downstream_comparison.get("optional_downstream_comparator_route_ids", [])
    )
    excluded_route_ids = list(downstream_comparison.get("excluded_by_default_route_ids", []))

    expected_default = [
        "DENSE_Q01_T2500",
        "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
        "DENSE_Q01_T40000",
        "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
    ]
    if default_route_ids != expected_default:
        raise ValueError(f"unexpected default downstream route ids: {default_route_ids}")
    if optional_route_ids != ["CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"]:
        raise ValueError(f"unexpected optional downstream comparator ids: {optional_route_ids}")

    interpretation = (
        f"The explicit downstream LM-proxy real-task comparison now carries forward "
        f"{lower.get('route_id')} as the lower-bank systems-only default and "
        f"{upper.get('route_id')} as the promoted upper-bank downstream default. "
        f"{comparator.get('route_id')} remains available only when a downstream real-task branch "
        "explicitly needs a pruning or systems comparator."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "carry_forward_contract": "dual_anchor_downstream_real_task_default_comparison",
        "comparison_surface": downstream_comparison.get("comparison_surface", ""),
        "packet_id": downstream_comparison.get("packet_id", ""),
        "packet_mode": downstream_comparison.get("packet_mode", ""),
        "source_downstream_comparison": {
            "comparison_surface": downstream_comparison.get("comparison_surface", ""),
            "overall_downstream_comparison_read": downstream_comparison.get(
                "overall_downstream_comparison_read", ""
            ),
        },
        "default_downstream_real_task_routes": {
            "route_ids": default_route_ids,
            "description": (
                "Default downstream real-task comparison: lower-bank dense baseline plus the "
                "lower-bank systems-only routed point, and upper-bank dense baseline plus the "
                "promoted upper-bank downstream routed reference."
            ),
        },
        "optional_comparators": {
            "route_ids": optional_route_ids,
            "description": (
                "Routes kept available only for explicit upper-bank pruning or systems side-by-side "
                "downstream real-task comparisons."
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
        "inheritance_rules": list(downstream_comparison.get("inheritance_rules", [])),
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
    ap.add_argument("--downstream-analysis", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Dual-Anchor Real-Task Downstream Carry-Forward")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(_load_json(args.downstream_analysis))
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
