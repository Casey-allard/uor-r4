#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


def _fmt(value: float, places: int = 6) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _route_by_id(analysis: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    for row in analysis.get("route_stats", []):
        if str(row.get("route_id")) == route_id:
            return row
    raise KeyError(f"route_id not found: {route_id}")


def _route_metrics(route_row: Dict[str, Any]) -> Dict[str, float]:
    return {
        "top1": _safe_float(route_row.get("mean_test_top1_after")),
        "candidate_fraction": _safe_float(route_row.get("mean_retrieval_candidate_fraction")),
        "amortized_per_repeat_sec": _safe_float(route_row.get("mean_amortized_total_per_repeat_sec")),
    }


def _route_snapshot(analysis: Dict[str, Any], route_id: str, role: str) -> Dict[str, Any]:
    return {
        "role": role,
        "route_id": route_id,
        "metrics": _route_metrics(_route_by_id(analysis, route_id)),
    }


def build_payload(
    selection_analysis: Dict[str, Any],
    lower_confirm: Dict[str, Any],
    upper_confirm: Dict[str, Any],
    *,
    lower_dense_route_id: str = "DENSE_Q01_T2500",
    lower_systems_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
    lower_soft_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
    upper_dense_route_id: str = "DENSE_Q01_T40000",
) -> Dict[str, Any]:
    if str(selection_analysis.get("carry_forward_contract")) != "single_promoted_reference":
        raise ValueError("selection analysis did not promote a single upper-bank reference")

    promoted_route_id = str(selection_analysis.get("promoted_route_id") or "")
    supporting_route_id = str(selection_analysis.get("supporting_route_id") or "")
    if not promoted_route_id or not supporting_route_id:
        raise ValueError("selection analysis is missing promoted or supporting route ids")

    lower_dense = _route_snapshot(lower_confirm, lower_dense_route_id, "lower_bank_dense_baseline")
    lower_systems = _route_snapshot(lower_confirm, lower_systems_route_id, "lower_bank_default_routed_systems_reference")
    lower_soft = _route_snapshot(lower_confirm, lower_soft_route_id, "lower_bank_nondefault_quality_reference")
    upper_dense = _route_snapshot(upper_confirm, upper_dense_route_id, "upper_bank_dense_baseline")
    upper_promoted = _route_snapshot(upper_confirm, promoted_route_id, "upper_bank_promoted_dense_near_reference")
    upper_supporting = _route_snapshot(upper_confirm, supporting_route_id, "upper_bank_supporting_comparator")

    default_packet_route_ids = [
        lower_dense_route_id,
        lower_systems_route_id,
        upper_dense_route_id,
        promoted_route_id,
    ]
    optional_comparator_route_ids = [supporting_route_id]
    excluded_by_default_route_ids = [lower_soft_route_id, supporting_route_id]

    interpretation = (
        f"The default broader-comparison packet now carries {lower_systems_route_id} as the lower-bank "
        f"systems-only routed point and {promoted_route_id} as the sole upper-bank dense-near routed "
        f"reference. {supporting_route_id} remains available only when an explicit pruning/systems "
        "comparator is needed, and the lower-bank soft sparse point stays out of the default packet "
        "because the lower-bank dense read remains systems-only."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "sources": {
            "selection_analysis": selection_analysis.get("sources", {}),
            "lower_confirm": lower_confirm.get("experiment_id", ""),
            "upper_confirm": upper_confirm.get("experiment_id", ""),
        },
        "carry_forward_contract": "promoted_upper_bank_single_reference",
        "selection_mode": selection_analysis.get("selection_mode", ""),
        "promoted_upper_bank_route_id": promoted_route_id,
        "supporting_upper_bank_route_id": supporting_route_id,
        "lower_bank_mode": "systems_only_anchor",
        "upper_bank_mode": "single_promoted_reference",
        "route_roles": {
            "lower_bank_dense_baseline": lower_dense,
            "lower_bank_default_routed_systems_reference": lower_systems,
            "lower_bank_nondefault_quality_reference": lower_soft,
            "upper_bank_dense_baseline": upper_dense,
            "upper_bank_promoted_dense_near_reference": upper_promoted,
            "upper_bank_supporting_comparator": upper_supporting,
        },
        "default_broader_comparison_packet": {
            "route_ids": default_packet_route_ids,
            "description": (
                "Default dual-anchor broader-comparison packet: lower-bank dense baseline plus the "
                "lower-bank systems-only routed point, and upper-bank dense baseline plus the promoted "
                "upper-bank dense-near routed reference."
            ),
        },
        "optional_comparators": {
            "route_ids": optional_comparator_route_ids,
            "description": "Routes kept available only for explicit pruning/systems side-by-side comparisons.",
        },
        "excluded_by_default": {
            "route_ids": excluded_by_default_route_ids,
            "description": (
                "Routes excluded from the default broader packet because they are either comparator-only "
                "or carry a nondefault lower-bank quality/pruning read."
            ),
        },
        "pair_deltas": selection_analysis.get("pair_deltas", {}),
        "inclusion_rules": [
            f"Always carry {promoted_route_id} as the sole upper-bank routed representative in broader comparisons.",
            f"Only add {supporting_route_id} when an explicit upper-bank pruning or systems comparator is required.",
            f"Keep {lower_systems_route_id} as the default lower-bank routed representative because the lower-bank dense story remains systems-only.",
            f"Do not carry {lower_soft_route_id} by default; it remains a lower-bank pruning/quality reference rather than the lower-bank default routed point.",
        ],
        "interpretation": interpretation,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    roles = payload["route_roles"]
    packet = payload["default_broader_comparison_packet"]
    optional = payload["optional_comparators"]
    excluded = payload["excluded_by_default"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Contract: `{payload['carry_forward_contract']}`",
        f"- Selection mode carried forward from `INC-0114`: `{payload['selection_mode']}`",
        f"- Lower-bank mode: `{payload['lower_bank_mode']}`",
        f"- Upper-bank mode: `{payload['upper_bank_mode']}`",
        f"- Read: {payload['interpretation']}",
        "",
        "## Default Broader Packet",
        f"- Route ids: `{', '.join(packet['route_ids'])}`",
        f"- Lower-bank default routed point: `{roles['lower_bank_default_routed_systems_reference']['route_id']}`",
        f"- Upper-bank promoted routed point: `{roles['upper_bank_promoted_dense_near_reference']['route_id']}`",
        "",
        "## Comparator Handling",
        f"- Optional comparator ids: `{', '.join(optional['route_ids'])}`",
        f"- Excluded-by-default ids: `{', '.join(excluded['route_ids'])}`",
        "",
        "## Anchors",
        f"- Lower dense `{roles['lower_bank_dense_baseline']['route_id']}`: "
        f"`top1={_fmt(roles['lower_bank_dense_baseline']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['lower_bank_dense_baseline']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['lower_bank_dense_baseline']['metrics']['amortized_per_repeat_sec'])}s`",
        f"- Lower systems `{roles['lower_bank_default_routed_systems_reference']['route_id']}`: "
        f"`top1={_fmt(roles['lower_bank_default_routed_systems_reference']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['lower_bank_default_routed_systems_reference']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['lower_bank_default_routed_systems_reference']['metrics']['amortized_per_repeat_sec'])}s`",
        f"- Lower nondefault `{roles['lower_bank_nondefault_quality_reference']['route_id']}`: "
        f"`top1={_fmt(roles['lower_bank_nondefault_quality_reference']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['lower_bank_nondefault_quality_reference']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['lower_bank_nondefault_quality_reference']['metrics']['amortized_per_repeat_sec'])}s`",
        f"- Upper dense `{roles['upper_bank_dense_baseline']['route_id']}`: "
        f"`top1={_fmt(roles['upper_bank_dense_baseline']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['upper_bank_dense_baseline']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['upper_bank_dense_baseline']['metrics']['amortized_per_repeat_sec'])}s`",
        f"- Upper promoted `{roles['upper_bank_promoted_dense_near_reference']['route_id']}`: "
        f"`top1={_fmt(roles['upper_bank_promoted_dense_near_reference']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['upper_bank_promoted_dense_near_reference']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['upper_bank_promoted_dense_near_reference']['metrics']['amortized_per_repeat_sec'])}s`",
        f"- Upper supporting `{roles['upper_bank_supporting_comparator']['route_id']}`: "
        f"`top1={_fmt(roles['upper_bank_supporting_comparator']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(roles['upper_bank_supporting_comparator']['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(roles['upper_bank_supporting_comparator']['metrics']['amortized_per_repeat_sec'])}s`",
        "",
        "## Inclusion Rules",
    ]
    for rule in payload["inclusion_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--selection-analysis", required=True)
    ap.add_argument("--lower-confirm-analysis", required=True)
    ap.add_argument("--upper-confirm-analysis", required=True)
    ap.add_argument("--lower-dense-route-id", default="DENSE_Q01_T2500")
    ap.add_argument("--lower-systems-route-id", default="CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500")
    ap.add_argument("--lower-soft-route-id", default="CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500")
    ap.add_argument("--upper-dense-route-id", default="DENSE_Q01_T40000")
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Promoted Upper-Bank Carry-Forward Contract")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.selection_analysis),
        _load_json(args.lower_confirm_analysis),
        _load_json(args.upper_confirm_analysis),
        lower_dense_route_id=args.lower_dense_route_id,
        lower_systems_route_id=args.lower_systems_route_id,
        lower_soft_route_id=args.lower_soft_route_id,
        upper_dense_route_id=args.upper_dense_route_id,
    )
    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_report(payload, Path(args.output_report), title=args.report_title)


if __name__ == "__main__":
    main()
