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


def _downstream_rules(base_rules: List[str]) -> List[str]:
    rules = list(base_rules)
    rules.append(
        "Downstream real-task branches should start from this exact manifest-backed "
        "extension artifact and must record any optional comparator reintroduction explicitly."
    )
    return rules


def build_payload(packet_manifest: Dict[str, Any], carry_forward: Dict[str, Any]) -> Dict[str, Any]:
    packet_defaults = list(packet_manifest.get("default_route_ids", []))
    packet_optional = list(packet_manifest.get("optional_comparator_route_ids", []))
    packet_excluded = list(packet_manifest.get("excluded_by_default_route_ids", []))

    carry_defaults = list(
        carry_forward.get("default_downstream_real_task_routes", {}).get("route_ids", [])
    )
    carry_optional = list(carry_forward.get("optional_comparators", {}).get("route_ids", []))
    carry_excluded = list(carry_forward.get("excluded_by_default", {}).get("route_ids", []))

    if packet_defaults != carry_defaults:
        raise ValueError("packet-manifest default route ids do not match carry-forward defaults")
    if packet_optional != carry_optional:
        raise ValueError("packet-manifest optional route ids do not match carry-forward defaults")
    if packet_excluded != carry_excluded:
        raise ValueError("packet-manifest excluded route ids do not match carry-forward defaults")

    lower = dict(carry_forward.get("lower_bank_default", {}))
    upper = dict(carry_forward.get("upper_bank_default", {}))
    comparator = dict(carry_forward.get("optional_upper_bank_comparator", {}))

    downstream_read = (
        "Downstream LM-proxy real-task inheritance is now explicit from the exact packet manifest: "
        f"lower bank stays {lower.get('classification')} by default via {lower.get('route_id')}, "
        f"upper bank stays {upper.get('classification')} by default via {upper.get('route_id')}, "
        f"and {comparator.get('route_id')} remains optional comparator-only."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "downstream_surface": "lm_proxy_real_task_downstream",
        "report_target_path": "docs/reports/REAL_TASK_COMPARISON.md",
        "packet_id": packet_manifest.get("packet_id", ""),
        "packet_mode": packet_manifest.get("packet_mode", ""),
        "source_packet_manifest": {
            "packet_id": packet_manifest.get("packet_id", ""),
            "packet_mode": packet_manifest.get("packet_mode", ""),
            "interpretation": packet_manifest.get("interpretation", ""),
        },
        "source_real_task_carry_forward": {
            "carry_forward_contract": carry_forward.get("carry_forward_contract", ""),
            "comparison_surface": carry_forward.get("comparison_surface", ""),
            "interpretation": carry_forward.get("interpretation", ""),
        },
        "downstream_extension_read": downstream_read,
        "default_downstream_route_ids": packet_defaults,
        "optional_downstream_comparator_route_ids": packet_optional,
        "excluded_by_default_route_ids": packet_excluded,
        "lower_anchor_downstream_default": {
            "route_id": lower.get("route_id", ""),
            "baseline_route_id": lower.get("baseline_route_id", ""),
            "classification": lower.get("classification", ""),
            "quality_read": lower.get("quality_read", ""),
            "recommendation": lower.get("recommendation", {}),
        },
        "upper_anchor_downstream_default": {
            "route_id": upper.get("route_id", ""),
            "baseline_route_id": upper.get("baseline_route_id", ""),
            "classification": upper.get("classification", ""),
            "quality_read": upper.get("quality_read", ""),
            "recommendation": upper.get("recommendation", {}),
        },
        "optional_upper_downstream_comparator": {
            "route_id": comparator.get("route_id", ""),
            "classification": comparator.get("classification", ""),
            "quality_read": comparator.get("quality_read", ""),
            "recommendation": comparator.get("recommendation", {}),
        },
        "branch_requirements": [
            "Start the next downstream real-task branch from the exact INC-0121 packet manifest.",
            "Record any optional comparator reintroduction explicitly.",
            "Keep packet scope fixed while testing the next downstream real-task question.",
        ],
        "inheritance_rules": _downstream_rules(list(packet_manifest.get("inheritance_rules", []))),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower = payload["lower_anchor_downstream_default"]
    upper = payload["upper_anchor_downstream_default"]
    comparator = payload["optional_upper_downstream_comparator"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Packet id: `{payload['packet_id']}`",
        f"- Downstream surface: `{payload['downstream_surface']}`",
        f"- Read: {payload['downstream_extension_read']}",
        "",
        "## Default Downstream Routes",
        f"- Route ids: `{', '.join(payload['default_downstream_route_ids'])}`",
        "",
        "## Lower Anchor",
        f"- Default routed route: `{lower['route_id']}`",
        f"- Baseline: `{lower['baseline_route_id']}`",
        f"- Classification: `{lower['classification']}`",
        f"- Recommendation: `{lower['recommendation'].get('verdict', '')}`",
        "",
        "## Upper Anchor",
        f"- Default routed route: `{upper['route_id']}`",
        f"- Baseline: `{upper['baseline_route_id']}`",
        f"- Classification: `{upper['classification']}`",
        f"- Recommendation: `{upper['recommendation'].get('verdict', '')}`",
        "",
        "## Optional Comparator",
        f"- Route: `{comparator['route_id']}`",
        f"- Classification: `{comparator['classification']}`",
        f"- Recommendation: `{comparator['recommendation'].get('verdict', '')}`",
        "",
        "## Branch Requirements",
    ]
    for item in payload["branch_requirements"]:
        lines.append(f"- {item}")
    lines.extend(["", "## Inheritance Rules"])
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--packet-manifest", required=True)
    ap.add_argument("--carry-forward", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument(
        "--report-title",
        default="Dual-Anchor Real-Task Downstream Extension",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.packet_manifest),
        _load_json(args.carry_forward),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
