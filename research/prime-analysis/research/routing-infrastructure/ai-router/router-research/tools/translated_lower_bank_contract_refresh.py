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


LEGACY_LOWER_NONDEFAULT_ROUTE_ID = "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500"


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _classification_read(classification: str) -> str:
    mapping = {
        "systems-only": "systems-first lower-bank routed point",
        "balanced_quality_comparator": "smallest confirmed quality lift over the lower-bank default",
        "quality_first_comparator": "largest confirmed lower-bank quality lift",
        "quality-near systems promotion": "upper-bank dense-near routed promotion",
        "historical_only": "historical-only route kept for provenance and explicit regression checks",
    }
    return mapping.get(classification, classification)


def _recommendation(verdict: str, reason: str) -> Dict[str, str]:
    return {"verdict": verdict, "reason": reason}


def build_payload(
    lower_bank_selection: Dict[str, Any],
    broader_packet: Dict[str, Any],
    task_side_extension: Dict[str, Any],
    real_task_carry_forward: Dict[str, Any],
    downstream_extension: Dict[str, Any],
    downstream_carry_forward: Dict[str, Any],
) -> Dict[str, Any]:
    lower_default = dict(lower_bank_selection.get("default_systems_reference", {}))
    balanced = dict(lower_bank_selection.get("balanced_quality_comparator", {}))
    quality_first = dict(lower_bank_selection.get("quality_first_comparator", {}))
    stale = dict(lower_bank_selection.get("stale_historical_comparator", {}))

    if lower_default.get("route_id", "") == "":
        raise ValueError("lower-bank selection is missing the default systems reference")

    old_broader_defaults = list(broader_packet.get("default_route_ids", []))
    if len(old_broader_defaults) != 4:
        raise ValueError("broader packet must contain exactly four default route ids")

    dense_lower_route_id = old_broader_defaults[0]
    dense_upper_route_id = old_broader_defaults[2]
    upper_default_route_id = old_broader_defaults[3]

    old_lower_default_route_id = old_broader_defaults[1]
    stale_lower_route_id = stale.get("route_id", "")
    if stale_lower_route_id and old_lower_default_route_id != stale_lower_route_id:
        raise ValueError("broader packet lower-bank default does not match the stale historical lower route")

    upper_task_side = dict(task_side_extension.get("upper_anchor_task_side_default", {}))
    if not upper_task_side:
        upper_task_side = {
            "route_id": upper_default_route_id,
            "baseline_route_id": dense_upper_route_id,
            "classification": "quality-near systems promotion",
            "quality_read": _classification_read("quality-near systems promotion"),
            "recommendation": _recommendation(
                "carry_as_task_side_default",
                "upper-bank default remains unchanged by the lower-bank refresh",
            ),
        }

    upper_real_task = dict(real_task_carry_forward.get("upper_bank_default", {}))
    upper_downstream = dict(downstream_extension.get("upper_anchor_downstream_default", {}))
    upper_downstream_cf = dict(downstream_carry_forward.get("upper_bank_default", {}))
    upper_optional = dict(real_task_carry_forward.get("optional_upper_bank_comparator", {}))
    if not upper_optional:
        upper_optional = dict(downstream_extension.get("optional_upper_downstream_comparator", {}))

    if upper_default_route_id != upper_real_task.get("route_id", upper_default_route_id):
        raise ValueError("real-task carry-forward upper-bank default drifted from the broader packet")
    if upper_default_route_id != upper_downstream.get("route_id", upper_default_route_id):
        raise ValueError("downstream extension upper-bank default drifted from the broader packet")
    if upper_default_route_id != upper_downstream_cf.get("route_id", upper_default_route_id):
        raise ValueError("downstream carry-forward upper-bank default drifted from the broader packet")

    optional_route_ids = [
        balanced.get("route_id", ""),
        quality_first.get("route_id", ""),
        upper_optional.get("route_id", ""),
    ]
    optional_route_ids = [route_id for route_id in optional_route_ids if route_id]

    default_route_ids = [
        dense_lower_route_id,
        lower_default["route_id"],
        dense_upper_route_id,
        upper_default_route_id,
    ]

    excluded_route_ids = [LEGACY_LOWER_NONDEFAULT_ROUTE_ID]
    if stale_lower_route_id:
        excluded_route_ids.append(stale_lower_route_id)

    lower_default_block = {
        "route_id": lower_default["route_id"],
        "baseline_route_id": dense_lower_route_id,
        "classification": "systems-only",
        "quality_read": _classification_read("systems-only"),
        "recommendation": _recommendation(
            "carry_as_systems_only_default",
            "lower-bank default now inherits the explicit INC-0132 near-hard systems reference",
        ),
    }
    balanced_block = {
        "route_id": balanced.get("route_id", ""),
        "baseline_route_id": dense_lower_route_id,
        "classification": "balanced_quality_comparator",
        "quality_read": _classification_read("balanced_quality_comparator"),
        "recommendation": _recommendation(
            "optional_lower_bank_quality_comparator",
            "keep the smallest lower-bank quality lift explicit without promoting it to default",
        ),
    }
    quality_first_block = {
        "route_id": quality_first.get("route_id", ""),
        "baseline_route_id": dense_lower_route_id,
        "classification": "quality_first_comparator",
        "quality_read": _classification_read("quality_first_comparator"),
        "recommendation": _recommendation(
            "optional_lower_bank_quality_comparator",
            "keep the strongest lower-bank quality-first point explicit without making it default",
        ),
    }
    stale_block = {
        "route_id": stale_lower_route_id,
        "baseline_route_id": dense_lower_route_id,
        "classification": "historical_only",
        "quality_read": _classification_read("historical_only"),
        "recommendation": _recommendation(
            "historical_only",
            "retain the stale lower-bank backfill route only for provenance and explicit regression checks",
        ),
    }

    broader_read = (
        f"Broader dual-anchor comparison now inherits {lower_default_block['route_id']} as the lower-bank "
        f"default, keeps {balanced_block['route_id']} and {quality_first_block['route_id']} as explicit "
        f"lower-bank comparators, leaves the upper-bank default at {upper_default_route_id}, and demotes "
        f"{stale_lower_route_id} to historical-only status."
    )
    task_side_read = (
        f"Task-side dual-anchor inheritance now uses {lower_default_block['route_id']} as the default "
        "lower-bank systems route, keeps the two soft-bias lower-bank routes as comparator-only, and "
        f"leaves the upper-bank task-side default at {upper_task_side.get('route_id', upper_default_route_id)}."
    )
    real_task_read = (
        f"LM-proxy real-task carry-forward now uses {lower_default_block['route_id']} as the lower-bank "
        f"default, keeps {balanced_block['route_id']} and {quality_first_block['route_id']} explicit as "
        f"lower-bank comparators, and leaves the promoted upper-bank route at {upper_real_task.get('route_id', upper_default_route_id)}."
    )
    downstream_read = (
        f"Downstream real-task inheritance now uses {lower_default_block['route_id']} as the lower-bank "
        f"default, keeps {balanced_block['route_id']} and {quality_first_block['route_id']} optional, and "
        f"leaves the upper-bank default at {upper_downstream.get('route_id', upper_default_route_id)}."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "contract_id": "inc0133_product_phase_sparse_event_translation_lower_bank_contract_refresh",
        "refresh_mode": "single_contract_refresh_from_inc0132",
        "sources": {
            "lower_bank_selection": lower_bank_selection.get("sources", {}),
            "broader_packet_id": broader_packet.get("packet_id", ""),
            "task_side_surface": task_side_extension.get("task_side_surface", "lm_proxy_real_task"),
            "real_task_contract": real_task_carry_forward.get("carry_forward_contract", ""),
            "downstream_surface": downstream_extension.get("downstream_surface", ""),
            "downstream_contract": downstream_carry_forward.get("carry_forward_contract", ""),
        },
        "lower_bank_contract": {
            "default_systems_reference": lower_default_block,
            "balanced_quality_comparator": balanced_block,
            "quality_first_comparator": quality_first_block,
            "stale_historical_comparator": stale_block,
        },
        "upper_bank_contract": {
            "default_reference": upper_real_task,
            "optional_comparator": upper_optional,
        },
        "broader_comparison_contract": {
            "default_route_ids": default_route_ids,
            "optional_comparator_route_ids": optional_route_ids,
            "excluded_by_default_route_ids": excluded_route_ids,
            "read": broader_read,
        },
        "task_side_contract": {
            "default_route_ids": default_route_ids,
            "optional_comparator_route_ids": optional_route_ids,
            "excluded_by_default_route_ids": excluded_route_ids,
            "read": task_side_read,
        },
        "real_task_contract": {
            "default_route_ids": default_route_ids,
            "optional_comparator_route_ids": optional_route_ids,
            "excluded_by_default_route_ids": excluded_route_ids,
            "read": real_task_read,
        },
        "downstream_contract": {
            "default_route_ids": default_route_ids,
            "optional_comparator_route_ids": optional_route_ids,
            "excluded_by_default_route_ids": excluded_route_ids,
            "read": downstream_read,
        },
        "inheritance_rules": [
            f"Use `{lower_default_block['route_id']}` as the only default lower-bank routed route on current dual-anchor surfaces.",
            f"Keep `{balanced_block['route_id']}` and `{quality_first_block['route_id']}` as explicit lower-bank comparators rather than defaults.",
            f"Keep `{stale_lower_route_id}` out of default inheritance; only use it when a branch explicitly needs the stale historical comparator.",
            f"Leave the upper-bank promoted default at `{upper_default_route_id}` and the upper-bank optional comparator at `{upper_optional.get('route_id', '')}`.",
        ],
        "interpretation": (
            f"INC-0132 moved the real lower-bank science: {lower_default_block['route_id']} is now the explicit "
            "default systems reference, while the two soft-bias routes are explicit comparator-only options. "
            "INC-0133 applies that result once across broader, task-side, and downstream contracts so later "
            "branches stop inheriting the stale lower-bank backfill default."
        ),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_bank_contract"]
    upper = payload["upper_bank_contract"]
    broader = payload["broader_comparison_contract"]
    task_side = payload["task_side_contract"]
    downstream = payload["downstream_contract"]
    lines: List[str] = [
        f"# {title}",
        "",
        "## Summary",
        f"- Contract id: `{payload['contract_id']}`",
        f"- Mode: `{payload['refresh_mode']}`",
        f"- Read: {payload['interpretation']}",
        "",
        "## Lower-Bank Contract",
        f"- Default: `{lower['default_systems_reference']['route_id']}`",
        f"- Balanced comparator: `{lower['balanced_quality_comparator']['route_id']}`",
        f"- Quality-first comparator: `{lower['quality_first_comparator']['route_id']}`",
        f"- Historical-only comparator: `{lower['stale_historical_comparator']['route_id']}`",
        "",
        "## Upper-Bank Contract",
        f"- Default: `{upper['default_reference']['route_id']}`",
        f"- Optional comparator: `{upper['optional_comparator']['route_id']}`",
        "",
        "## Refreshed Surfaces",
        f"- Broader default routes: `{', '.join(broader['default_route_ids'])}`",
        f"- Broader optional comparators: `{', '.join(broader['optional_comparator_route_ids'])}`",
        f"- Task-side read: {task_side['read']}",
        f"- Downstream read: {downstream['read']}",
        "",
        "## Inheritance Rules",
    ]
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lower-bank-selection", required=True)
    ap.add_argument("--broader-packet", required=True)
    ap.add_argument("--task-side-extension", required=True)
    ap.add_argument("--real-task-carry-forward", required=True)
    ap.add_argument("--downstream-extension", required=True)
    ap.add_argument("--downstream-carry-forward", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument(
        "--report-title",
        default="INC0133 Product Phase Sparse Event Translation Lower-Bank Contract Refresh",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.lower_bank_selection),
        _load_json(args.broader_packet),
        _load_json(args.task_side_extension),
        _load_json(args.real_task_carry_forward),
        _load_json(args.downstream_extension),
        _load_json(args.downstream_carry_forward),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
