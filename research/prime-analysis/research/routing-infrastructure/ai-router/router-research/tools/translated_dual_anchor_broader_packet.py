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


def _route_by_id(config: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    for row in config.get("routes", []):
        if str(row.get("route_id")) == route_id:
            return row
    raise KeyError(f"route_id not found in config: {route_id}")


def _resolved_args(config: Dict[str, Any], route_row: Dict[str, Any]) -> Dict[str, Any]:
    merged = dict(config.get("common_args", {}))
    merged.update(route_row.get("args", {}))
    return merged


def _route_entry(
    config: Dict[str, Any],
    *,
    config_path: str,
    route_id: str,
    anchor: str,
    role: str,
    inclusion_mode: str,
) -> Dict[str, Any]:
    route_row = _route_by_id(config, route_id)
    return {
        "route_id": route_id,
        "anchor": anchor,
        "role": role,
        "inclusion_mode": inclusion_mode,
        "source_experiment_id": config.get("experiment_id", ""),
        "source_config": config_path,
        "resolved_args": _resolved_args(config, route_row),
        "route_args": dict(route_row.get("args", {})),
    }


def build_payload(
    contract: Dict[str, Any],
    lower_config: Dict[str, Any],
    upper_config: Dict[str, Any],
    *,
    lower_config_path: str,
    upper_config_path: str,
) -> Dict[str, Any]:
    default_route_ids = list(contract.get("default_broader_comparison_packet", {}).get("route_ids", []))
    optional_comparator_ids = list(contract.get("optional_comparators", {}).get("route_ids", []))
    excluded_ids = list(contract.get("excluded_by_default", {}).get("route_ids", []))

    expected_default = [
        "DENSE_Q01_T2500",
        "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
        "DENSE_Q01_T40000",
        "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
    ]
    if default_route_ids != expected_default:
        raise ValueError(f"unexpected default packet route ids: {default_route_ids}")

    default_routes = [
        _route_entry(
            lower_config,
            config_path=lower_config_path,
            route_id="DENSE_Q01_T2500",
            anchor="lower_bank",
            role="dense_baseline",
            inclusion_mode="default",
        ),
        _route_entry(
            lower_config,
            config_path=lower_config_path,
            route_id="CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
            anchor="lower_bank",
            role="default_routed_systems_reference",
            inclusion_mode="default",
        ),
        _route_entry(
            upper_config,
            config_path=upper_config_path,
            route_id="DENSE_Q01_T40000",
            anchor="upper_bank",
            role="dense_baseline",
            inclusion_mode="default",
        ),
        _route_entry(
            upper_config,
            config_path=upper_config_path,
            route_id="CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            anchor="upper_bank",
            role="promoted_dense_near_reference",
            inclusion_mode="default",
        ),
    ]
    optional_comparators = [
        _route_entry(
            upper_config,
            config_path=upper_config_path,
            route_id="CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            anchor="upper_bank",
            role="supporting_comparator",
            inclusion_mode="optional",
        )
    ]
    excluded_by_default = [
        _route_entry(
            lower_config,
            config_path=lower_config_path,
            route_id="CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
            anchor="lower_bank",
            role="nondefault_pruning_quality_reference",
            inclusion_mode="excluded",
        ),
        _route_entry(
            upper_config,
            config_path=upper_config_path,
            route_id="CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            anchor="upper_bank",
            role="supporting_comparator",
            inclusion_mode="excluded",
        ),
    ]

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "packet_id": "inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet",
        "packet_mode": "dual_anchor_default_packet",
        "contract_ref": {
            "carry_forward_contract": contract.get("carry_forward_contract", ""),
            "selection_mode": contract.get("selection_mode", ""),
            "promoted_upper_bank_route_id": contract.get("promoted_upper_bank_route_id", ""),
            "supporting_upper_bank_route_id": contract.get("supporting_upper_bank_route_id", ""),
        },
        "sources": {
            "contract": contract.get("sources", {}),
            "lower_config_experiment_id": lower_config.get("experiment_id", ""),
            "upper_config_experiment_id": upper_config.get("experiment_id", ""),
            "lower_config_path": lower_config_path,
            "upper_config_path": upper_config_path,
        },
        "default_route_ids": default_route_ids,
        "optional_comparator_route_ids": optional_comparator_ids,
        "excluded_by_default_route_ids": excluded_ids,
        "default_routes": default_routes,
        "optional_comparators": optional_comparators,
        "excluded_by_default_routes": excluded_by_default,
        "inheritance_rules": list(contract.get("inclusion_rules", [])),
        "interpretation": (
            "This packet freezes the dual-anchor broader-comparison inheritance set. Later hardware-side "
            "or task-side branches should start from these four default routes and only reintroduce the "
            "upper-bank bounded-backfill comparator when the branch explicitly needs a pruning/systems "
            "side-by-side check."
        ),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lines: List[str] = [
        f"# {title}",
        "",
        "## Summary",
        f"- Packet id: `{payload['packet_id']}`",
        f"- Mode: `{payload['packet_mode']}`",
        f"- Contract ref: `{payload['contract_ref']['carry_forward_contract']}`",
        f"- Read: {payload['interpretation']}",
        "",
        "## Default Routes",
    ]
    for route in payload["default_routes"]:
        lines.append(
            f"- `{route['route_id']}` ({route['anchor']}, {route['role']}) from "
            f"`{route['source_experiment_id']}`"
        )
    lines.extend(
        [
            "",
            "## Optional Comparators",
        ]
    )
    for route in payload["optional_comparators"]:
        lines.append(
            f"- `{route['route_id']}` ({route['anchor']}, {route['role']}) from "
            f"`{route['source_experiment_id']}`"
        )
    lines.extend(
        [
            "",
            "## Excluded By Default",
        ]
    )
    for route in payload["excluded_by_default_routes"]:
        lines.append(
            f"- `{route['route_id']}` ({route['anchor']}, {route['role']})"
        )
    lines.extend(
        [
            "",
            "## Inheritance Rules",
        ]
    )
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--contract", required=True)
    ap.add_argument("--lower-config", required=True)
    ap.add_argument("--upper-config", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-packet", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Dual-Anchor Broader Comparison Packet")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.contract),
        _load_json(args.lower_config),
        _load_json(args.upper_config),
        lower_config_path=args.lower_config,
        upper_config_path=args.upper_config,
    )
    _write_json(args.output_json, payload)
    _write_json(args.output_packet, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
