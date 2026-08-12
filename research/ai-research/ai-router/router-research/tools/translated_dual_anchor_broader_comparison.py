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


def _comparison_by_label(analysis: Dict[str, Any], label: str) -> Dict[str, Any]:
    for row in analysis.get("comparisons", []):
        if str(row.get("label")) == label:
            return row
    raise KeyError(f"comparison label not found: {label}")


def _extract_robust(row: Dict[str, Any], key: str) -> Dict[str, Any]:
    return dict(row.get("robust_summaries", {}).get(key, {}))


def build_payload(
    packet: Dict[str, Any],
    frontier: Dict[str, Any],
    *,
    lower_label: str = "lower_dense_vs_backfill",
    upper_label: str = "upper_dense_vs_soft_sparse",
    upper_comparator_label: str = "upper_dense_vs_backfill",
) -> Dict[str, Any]:
    lower = _comparison_by_label(frontier, lower_label)
    upper = _comparison_by_label(frontier, upper_label)
    upper_comparator = _comparison_by_label(frontier, upper_comparator_label)

    lower_default = next(
        row for row in packet.get("default_routes", [])
        if row.get("anchor") == "lower_bank" and row.get("role") == "default_routed_systems_reference"
    )
    upper_default = next(
        row for row in packet.get("default_routes", [])
        if row.get("anchor") == "upper_bank" and row.get("role") == "promoted_dense_near_reference"
    )
    upper_optional = next(
        row for row in packet.get("optional_comparators", [])
        if row.get("anchor") == "upper_bank"
    )

    if lower_default["route_id"] != lower.get("candidate_route_id"):
        raise ValueError("lower default packet route does not match frontier lower comparison")
    if upper_default["route_id"] != upper.get("candidate_route_id"):
        raise ValueError("upper default packet route does not match frontier upper comparison")
    if upper_optional["route_id"] != upper_comparator.get("candidate_route_id"):
        raise ValueError("upper optional comparator does not match frontier upper comparator comparison")

    broader_read = (
        f"Default dual-anchor broader comparison is now explicit: lower bank stays "
        f"{lower.get('classification')} via {lower_default['route_id']}, while upper bank stays "
        f"{upper.get('classification')} via {upper_default['route_id']}. "
        f"{upper_optional['route_id']} remains optional comparator-only."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "packet_id": packet.get("packet_id", ""),
        "packet_mode": packet.get("packet_mode", ""),
        "source_packet": packet.get("sources", {}),
        "source_frontier_analysis": frontier.get("source_robust_analysis", []),
        "overall_dense_frontier_claim": frontier.get("overall_dense_frontier_claim", ""),
        "overall_broader_comparison_read": broader_read,
        "lower_anchor_default_read": {
            "route_id": lower_default["route_id"],
            "baseline_route_id": lower.get("baseline_route_id", ""),
            "classification": lower.get("classification", ""),
            "systems_verdict": lower.get("systems_verdict", ""),
            "quality_read": lower.get("quality_read", ""),
            "interpretation": lower.get("interpretation", ""),
            "top1_tolerance_gap_abs": lower.get("top1_gap_abs_for_tolerance"),
            "candidate_fraction_delta": _extract_robust(lower, "candidate_fraction_delta"),
            "amortized_delta_sec": _extract_robust(lower, "amortized_delta_candidate_minus_baseline_sec"),
        },
        "upper_anchor_default_read": {
            "route_id": upper_default["route_id"],
            "baseline_route_id": upper.get("baseline_route_id", ""),
            "classification": upper.get("classification", ""),
            "systems_verdict": upper.get("systems_verdict", ""),
            "quality_read": upper.get("quality_read", ""),
            "interpretation": upper.get("interpretation", ""),
            "top1_tolerance_gap_abs": upper.get("top1_gap_abs_for_tolerance"),
            "candidate_fraction_delta": _extract_robust(upper, "candidate_fraction_delta"),
            "amortized_delta_sec": _extract_robust(upper, "amortized_delta_candidate_minus_baseline_sec"),
        },
        "optional_upper_comparator": {
            "route_id": upper_optional["route_id"],
            "classification": upper_comparator.get("classification", ""),
            "systems_verdict": upper_comparator.get("systems_verdict", ""),
            "quality_read": upper_comparator.get("quality_read", ""),
            "interpretation": upper_comparator.get("interpretation", ""),
            "inclusion_mode": upper_optional.get("inclusion_mode", ""),
        },
        "default_packet_route_ids": list(packet.get("default_route_ids", [])),
        "optional_comparator_route_ids": list(packet.get("optional_comparator_route_ids", [])),
        "excluded_by_default_route_ids": list(packet.get("excluded_by_default_route_ids", [])),
        "inheritance_rules": list(packet.get("inheritance_rules", [])),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_anchor_default_read"]
    upper = payload["upper_anchor_default_read"]
    comparator = payload["optional_upper_comparator"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Packet id: `{payload['packet_id']}`",
        f"- Overall dense frontier claim: `{payload['overall_dense_frontier_claim']}`",
        f"- Read: {payload['overall_broader_comparison_read']}",
        "",
        "## Default Packet",
        f"- Route ids: `{', '.join(payload['default_packet_route_ids'])}`",
        "",
        "## Lower Anchor",
        f"- Default routed route: `{lower['route_id']}`",
        f"- Baseline: `{lower['baseline_route_id']}`",
        f"- Classification: `{lower['classification']}`",
        f"- Systems verdict: `{lower['systems_verdict']}`",
        f"- Quality read: `{lower['quality_read']}`",
        f"- Top-1 tolerance gap abs: `{lower['top1_tolerance_gap_abs']}`",
        "",
        "## Upper Anchor",
        f"- Default routed route: `{upper['route_id']}`",
        f"- Baseline: `{upper['baseline_route_id']}`",
        f"- Classification: `{upper['classification']}`",
        f"- Systems verdict: `{upper['systems_verdict']}`",
        f"- Quality read: `{upper['quality_read']}`",
        f"- Top-1 tolerance gap abs: `{upper['top1_tolerance_gap_abs']}`",
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
    ap.add_argument("--frontier-analysis", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Dual-Anchor Broader Comparison")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.packet),
        _load_json(args.frontier_analysis),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
