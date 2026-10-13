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


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


def _route_by_id(analysis: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    for row in analysis.get("route_stats", []):
        if str(row.get("route_id")) == route_id:
            return row
    raise KeyError(f"route_id not found: {route_id}")


def _metrics(row: Dict[str, Any]) -> Dict[str, float]:
    return {
        "top1": _safe_float(row.get("mean_test_top1_after")),
        "candidate_fraction": _safe_float(row.get("mean_retrieval_candidate_fraction")),
        "online_per_repeat_sec": _safe_float(row.get("mean_online_total_per_repeat_sec")),
        "amortized_per_repeat_sec": _safe_float(row.get("mean_amortized_total_per_repeat_sec")),
    }


def _delta(candidate: Dict[str, float], baseline: Dict[str, float]) -> Dict[str, float]:
    return {
        "top1": candidate["top1"] - baseline["top1"],
        "candidate_fraction": candidate["candidate_fraction"] - baseline["candidate_fraction"],
        "online_per_repeat_sec": candidate["online_per_repeat_sec"] - baseline["online_per_repeat_sec"],
        "amortized_per_repeat_sec": candidate["amortized_per_repeat_sec"] - baseline["amortized_per_repeat_sec"],
    }


def _required_outputs() -> List[str]:
    return [
        "refreshed real-task comparison artifact",
        "update to docs/reports/REAL_TASK_COMPARISON.md",
        "carry-forward recommendation from refreshed lower-bank evidence",
    ]


def build_payload(
    refreshed_contract: Dict[str, Any],
    lower_confirm: Dict[str, Any],
    upper_confirm: Dict[str, Any],
    upper_selection: Dict[str, Any],
) -> Dict[str, Any]:
    real_task_contract = dict(refreshed_contract.get("real_task_contract", {}))
    lower_contract = dict(refreshed_contract.get("lower_bank_contract", {}))
    upper_contract = dict(refreshed_contract.get("upper_bank_contract", {}))

    default_route_ids = list(real_task_contract.get("default_route_ids", []))
    optional_route_ids = list(real_task_contract.get("optional_comparator_route_ids", []))
    excluded_route_ids = list(real_task_contract.get("excluded_by_default_route_ids", []))

    lower_default_route_id = lower_contract["default_systems_reference"]["route_id"]
    lower_balanced_route_id = lower_contract["balanced_quality_comparator"]["route_id"]
    lower_quality_first_route_id = lower_contract["quality_first_comparator"]["route_id"]
    lower_historical_route_id = lower_contract["stale_historical_comparator"]["route_id"]
    dense_lower_route_id = lower_contract["default_systems_reference"]["baseline_route_id"]

    upper_default_route_id = upper_contract["default_reference"]["route_id"]
    upper_optional_route_id = upper_contract["optional_comparator"]["route_id"]
    dense_upper_route_id = upper_contract["default_reference"]["baseline_route_id"]

    if default_route_ids != [
        dense_lower_route_id,
        lower_default_route_id,
        dense_upper_route_id,
        upper_default_route_id,
    ]:
        raise ValueError("refreshed contract default route ids are inconsistent with its lower/upper blocks")

    lower_dense = _metrics(_route_by_id(lower_confirm, dense_lower_route_id))
    lower_default = _metrics(_route_by_id(lower_confirm, lower_default_route_id))
    lower_balanced = _metrics(_route_by_id(lower_confirm, lower_balanced_route_id))
    lower_quality_first = _metrics(_route_by_id(lower_confirm, lower_quality_first_route_id))
    lower_historical = _metrics(_route_by_id(lower_confirm, lower_historical_route_id))

    upper_dense = _metrics(_route_by_id(upper_confirm, dense_upper_route_id))
    upper_default = _metrics(_route_by_id(upper_confirm, upper_default_route_id))
    upper_optional = _metrics(_route_by_id(upper_confirm, upper_optional_route_id))

    lower_default_read = {
        "route_id": lower_default_route_id,
        "baseline_route_id": dense_lower_route_id,
        "classification": "systems-only",
        "metrics": lower_default,
        "delta_vs_dense": _delta(lower_default, lower_dense),
        "recommendation": {
            "verdict": "carry_as_systems_only_default",
            "reason": "best lower-bank amortized profile, but still materially below dense top-1",
        },
    }
    lower_balanced_read = {
        "route_id": lower_balanced_route_id,
        "baseline_route_id": dense_lower_route_id,
        "classification": "balanced_quality_comparator",
        "metrics": lower_balanced,
        "delta_vs_default": _delta(lower_balanced, lower_default),
        "delta_vs_dense": _delta(lower_balanced, lower_dense),
        "recommendation": {
            "verdict": "optional_balanced_quality_comparator",
            "reason": "smallest confirmed quality lift over the lower-bank default with a small amortized penalty",
        },
    }
    lower_quality_first_read = {
        "route_id": lower_quality_first_route_id,
        "baseline_route_id": dense_lower_route_id,
        "classification": "quality_first_comparator",
        "metrics": lower_quality_first,
        "delta_vs_default": _delta(lower_quality_first, lower_default),
        "delta_vs_dense": _delta(lower_quality_first, lower_dense),
        "recommendation": {
            "verdict": "optional_quality_first_comparator",
            "reason": "only lower-bank point that edges dense on top-1, but it gives back the systems advantage",
        },
    }
    lower_historical_read = {
        "route_id": lower_historical_route_id,
        "baseline_route_id": dense_lower_route_id,
        "classification": "historical_only",
        "metrics": lower_historical,
        "delta_vs_default": _delta(lower_historical, lower_default),
        "recommendation": {
            "verdict": "historical_only",
            "reason": "focused prewarmed packet keeps this route well outside the active lower-bank carry-forward set",
        },
    }

    upper_default_read = {
        "route_id": upper_default_route_id,
        "baseline_route_id": dense_upper_route_id,
        "classification": upper_contract["default_reference"]["classification"],
        "metrics": upper_default,
        "delta_vs_dense": _delta(upper_default, upper_dense),
        "recommendation": {
            "verdict": upper_contract["default_reference"]["recommendation"]["verdict"],
            "reason": "upper-bank route stays inside the dense-near quality band while preserving a large systems win",
        },
    }
    upper_optional_read = {
        "route_id": upper_optional_route_id,
        "baseline_route_id": dense_upper_route_id,
        "classification": upper_contract["optional_comparator"]["classification"],
        "metrics": upper_optional,
        "delta_vs_upper_default": _delta(upper_optional, upper_default),
        "delta_vs_dense": _delta(upper_optional, upper_dense),
        "selection_context": {
            "selection_mode": upper_selection.get("selection_mode", ""),
            "interpretation": upper_selection.get("interpretation", ""),
        },
        "recommendation": {
            "verdict": upper_contract["optional_comparator"]["recommendation"]["verdict"],
            "reason": "keep only as an explicit upper-bank pruning/systems comparator inside the existing tolerance band",
        },
    }

    overall = (
        f"Refreshed LM-proxy dual-anchor real-task comparison now uses {lower_default_route_id} as the lower-bank "
        f"systems-only default, keeps {lower_balanced_route_id} as the balanced lower-bank quality comparator, "
        f"keeps {lower_quality_first_route_id} as the lower-bank quality-first comparator, and leaves "
        f"{upper_default_route_id} as the upper-bank quality-near systems default. "
        f"{lower_historical_route_id} is no longer an active default route."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "branch_status": "positive_explanatory",
        "comparison_surface": "lm_proxy_real_task_refreshed",
        "report_target_path": "docs/reports/REAL_TASK_COMPARISON.md",
        "contract_id": refreshed_contract.get("contract_id", ""),
        "default_real_task_route_ids": default_route_ids,
        "optional_real_task_comparator_route_ids": optional_route_ids,
        "excluded_by_default_route_ids": excluded_route_ids,
        "lower_anchor_default": lower_default_read,
        "lower_anchor_balanced_quality_comparator": lower_balanced_read,
        "lower_anchor_quality_first_comparator": lower_quality_first_read,
        "lower_anchor_historical_comparator": lower_historical_read,
        "upper_anchor_default": upper_default_read,
        "optional_upper_comparator": upper_optional_read,
        "overall_real_task_comparison_read": overall,
        "required_outputs": _required_outputs(),
        "promotion_recommendation": {
            "lower_bank_default": lower_default_read["recommendation"],
            "lower_bank_balanced_quality_comparator": lower_balanced_read["recommendation"],
            "lower_bank_quality_first_comparator": lower_quality_first_read["recommendation"],
            "upper_bank_default": upper_default_read["recommendation"],
            "upper_bank_optional_comparator": upper_optional_read["recommendation"],
        },
        "source_contract": {
            "contract_id": refreshed_contract.get("contract_id", ""),
            "refresh_mode": refreshed_contract.get("refresh_mode", ""),
            "interpretation": refreshed_contract.get("interpretation", ""),
        },
        "source_analyses": {
            "lower_confirm": lower_confirm.get("experiment_id", ""),
            "upper_confirm": upper_confirm.get("experiment_id", ""),
            "upper_selection_mode": upper_selection.get("selection_mode", ""),
        },
        "inheritance_rules": list(refreshed_contract.get("inheritance_rules", [])),
    }


def _write_json(path_str: str, payload: Dict[str, Any]) -> None:
    path = Path(path_str)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lower_default = payload["lower_anchor_default"]
    lower_balanced = payload["lower_anchor_balanced_quality_comparator"]
    lower_quality = payload["lower_anchor_quality_first_comparator"]
    upper_default = payload["upper_anchor_default"]
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
        "## Lower-Bank Default",
        f"- Route: `{lower_default['route_id']}`",
        f"- Top-1: `{lower_default['metrics']['top1']:.4f}`",
        f"- Cand frac: `{lower_default['metrics']['candidate_fraction']:.6f}`",
        f"- Amortized: `{lower_default['metrics']['amortized_per_repeat_sec']:.4f}s`",
        f"- Top-1 delta vs dense: `{lower_default['delta_vs_dense']['top1']:.4f}`",
        f"- Recommendation: `{lower_default['recommendation']['verdict']}`",
        "",
        "## Lower-Bank Comparators",
        f"- Balanced: `{lower_balanced['route_id']}`",
        f"  - top-1 delta vs default: `{lower_balanced['delta_vs_default']['top1']:.4f}`",
        f"  - amortized delta vs default: `{lower_balanced['delta_vs_default']['amortized_per_repeat_sec']:.4f}s`",
        f"  - recommendation: `{lower_balanced['recommendation']['verdict']}`",
        f"- Quality-first: `{lower_quality['route_id']}`",
        f"  - top-1 delta vs dense: `{lower_quality['delta_vs_dense']['top1']:.4f}`",
        f"  - amortized delta vs dense: `{lower_quality['delta_vs_dense']['amortized_per_repeat_sec']:.4f}s`",
        f"  - recommendation: `{lower_quality['recommendation']['verdict']}`",
        "",
        "## Upper-Bank Default",
        f"- Route: `{upper_default['route_id']}`",
        f"- Top-1 delta vs dense: `{upper_default['delta_vs_dense']['top1']:.4f}`",
        f"- Cand frac delta vs dense: `{upper_default['delta_vs_dense']['candidate_fraction']:.6f}`",
        f"- Amortized delta vs dense: `{upper_default['delta_vs_dense']['amortized_per_repeat_sec']:.4f}s`",
        f"- Recommendation: `{upper_default['recommendation']['verdict']}`",
        "",
        "## Inheritance Rules",
    ]
    for rule in payload["inheritance_rules"]:
        lines.append(f"- {rule}")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--refreshed-contract", required=True)
    ap.add_argument("--lower-confirm", required=True)
    ap.add_argument("--upper-confirm", required=True)
    ap.add_argument("--upper-selection", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument(
        "--report-title",
        default="INC0134 Product Phase Sparse Event Translation Dual-Anchor Real-Task Refresh Comparison",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.refreshed_contract),
        _load_json(args.lower_confirm),
        _load_json(args.upper_confirm),
        _load_json(args.upper_selection),
    )
    _write_json(args.output_json, payload)
    write_report(payload, Path(args.output_report), args.report_title)


if __name__ == "__main__":
    main()
