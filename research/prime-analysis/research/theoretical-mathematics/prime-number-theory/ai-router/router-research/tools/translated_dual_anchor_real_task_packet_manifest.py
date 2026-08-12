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


def build_payload(real_task_carry_forward: Dict[str, Any], packet: Dict[str, Any]) -> Dict[str, Any]:
    default_route_ids = list(real_task_carry_forward.get("default_downstream_real_task_routes", {}).get("route_ids", []))
    optional_route_ids = list(real_task_carry_forward.get("optional_comparators", {}).get("route_ids", []))
    excluded_route_ids = list(real_task_carry_forward.get("excluded_by_default", {}).get("route_ids", []))

    if default_route_ids != list(packet.get("default_route_ids", [])):
        raise ValueError("carry-forward default route ids do not match source packet defaults")
    if optional_route_ids != list(packet.get("optional_comparator_route_ids", [])):
        raise ValueError("carry-forward optional route ids do not match source packet defaults")
    if excluded_route_ids != list(packet.get("excluded_by_default_route_ids", [])):
        raise ValueError("carry-forward excluded route ids do not match source packet defaults")

    inheritance_rules = list(real_task_carry_forward.get("inheritance_rules", []))
    inheritance_rules.append(
        "Downstream real-task branches should inherit this exact packet manifest rather than reconstructing route ids and args from older broader or task-side artifacts."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "packet_id": "inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest",
        "packet_mode": "downstream_dual_anchor_real_task_default_packet",
        "source_packet": {
            "packet_id": packet.get("packet_id", ""),
            "packet_mode": packet.get("packet_mode", ""),
            "sources": packet.get("sources", {}),
        },
        "source_carry_forward": {
            "carry_forward_contract": real_task_carry_forward.get("carry_forward_contract", ""),
            "comparison_surface": real_task_carry_forward.get("comparison_surface", ""),
            "interpretation": real_task_carry_forward.get("interpretation", ""),
        },
        "default_route_ids": default_route_ids,
        "optional_comparator_route_ids": optional_route_ids,
        "excluded_by_default_route_ids": excluded_route_ids,
        "default_routes": list(packet.get("default_routes", [])),
        "optional_comparators": list(packet.get("optional_comparators", [])),
        "excluded_by_default_routes": list(packet.get("excluded_by_default_routes", [])),
        "inheritance_rules": inheritance_rules,
        "interpretation": (
            "This packet freezes the downstream LM-proxy real-task inheritance set. Later downstream "
            "branches should start from these four default routes and only reintroduce the upper-bank "
            "bounded-backfill comparator when the branch explicitly needs a pruning or systems side-by-side comparison."
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
        f"- Read: {payload['interpretation']}",
        "",
        "## Default Routes",
    ]
    for route in payload["default_routes"]:
        lines.append(
            f"- `{route['route_id']}` ({route['anchor']}, {route['role']}) from "
            f"`{route['source_experiment_id']}`"
        )
    lines.extend(["", "## Optional Comparators"])
    for route in payload["optional_comparators"]:
        lines.append(
            f"- `{route['route_id']}` ({route['anchor']}, {route['role']}) from "
            f"`{route['source_experiment_id']}`"
        )
    lines.extend(["", "## Excluded By Default"])
    for route in payload["excluded_by_default_routes"]:
        lines.append(f"- `{route['route_id']}` ({route['anchor']}, {route['role']})")
    lines.extend(["", "## Inheritance Rules"])
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--real-task-carry-forward", required=True)
    ap.add_argument("--source-packet", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Dual-Anchor Real-Task Packet Manifest")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.real_task_carry_forward),
        _load_json(args.source_packet),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
