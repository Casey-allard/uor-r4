#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, List

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


DEFAULT_TOP1_TOLERANCE = 0.001
DEFAULT_STALE_AMORTIZED_RATIO = 4.0
DEFAULT_MIN_QUALITY_GAIN = 0.0005


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
        "online_per_repeat_sec": _safe_float(route_row.get("mean_online_total_per_repeat_sec")),
        "amortized_per_repeat_sec": _safe_float(route_row.get("mean_amortized_total_per_repeat_sec")),
    }


def _delta_metrics(candidate: Dict[str, float], baseline: Dict[str, float]) -> Dict[str, float]:
    return {
        "top1": candidate["top1"] - baseline["top1"],
        "candidate_fraction": candidate["candidate_fraction"] - baseline["candidate_fraction"],
        "online_per_repeat_sec": candidate["online_per_repeat_sec"] - baseline["online_per_repeat_sec"],
        "amortized_per_repeat_sec": candidate["amortized_per_repeat_sec"] - baseline["amortized_per_repeat_sec"],
    }


def _screen_confirm_delta(screen: Dict[str, float], confirm: Dict[str, float]) -> Dict[str, float]:
    return {
        "top1": confirm["top1"] - screen["top1"],
        "candidate_fraction": confirm["candidate_fraction"] - screen["candidate_fraction"],
        "online_per_repeat_sec": confirm["online_per_repeat_sec"] - screen["online_per_repeat_sec"],
        "amortized_per_repeat_sec": confirm["amortized_per_repeat_sec"] - screen["amortized_per_repeat_sec"],
    }


def build_payload(
    historical_backfill_analysis: Dict[str, Any],
    soft_bias_screen_analysis: Dict[str, Any],
    soft_bias_confirm_analysis: Dict[str, Any],
    *,
    historical_backfill_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
    near_hard_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
    balanced_quality_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
    quality_first_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
    dense_route_id: str = "DENSE_Q01_T2500",
    top1_tolerance: float = DEFAULT_TOP1_TOLERANCE,
    stale_amortized_ratio: float = DEFAULT_STALE_AMORTIZED_RATIO,
    min_quality_gain: float = DEFAULT_MIN_QUALITY_GAIN,
) -> Dict[str, Any]:
    historical_backfill = _route_metrics(_route_by_id(historical_backfill_analysis, historical_backfill_route_id))
    focused_backfill = _route_metrics(_route_by_id(soft_bias_confirm_analysis, historical_backfill_route_id))
    near_hard = _route_metrics(_route_by_id(soft_bias_confirm_analysis, near_hard_route_id))
    balanced = _route_metrics(_route_by_id(soft_bias_confirm_analysis, balanced_quality_route_id))
    quality_first = _route_metrics(_route_by_id(soft_bias_confirm_analysis, quality_first_route_id))
    dense = _route_metrics(_route_by_id(soft_bias_confirm_analysis, dense_route_id))
    balanced_screen = _route_metrics(_route_by_id(soft_bias_screen_analysis, balanced_quality_route_id))
    quality_first_screen = _route_metrics(_route_by_id(soft_bias_screen_analysis, quality_first_route_id))

    packet_scope_ratio = (
        focused_backfill["amortized_per_repeat_sec"] / historical_backfill["amortized_per_repeat_sec"]
        if historical_backfill["amortized_per_repeat_sec"] > 0.0
        else float("inf")
    )
    packet_scope_delta = focused_backfill["amortized_per_repeat_sec"] - historical_backfill["amortized_per_repeat_sec"]
    stale_backfill = (
        packet_scope_ratio >= stale_amortized_ratio
        and near_hard["amortized_per_repeat_sec"] < historical_backfill["amortized_per_repeat_sec"]
        and near_hard["top1"] + top1_tolerance >= focused_backfill["top1"]
    )

    default_systems = {
        "route_id": near_hard_route_id if stale_backfill else historical_backfill_route_id,
        "classification": "default_systems_reference" if stale_backfill else "historical_default_retained",
        "metrics": near_hard if stale_backfill else historical_backfill,
    }

    quality_candidates: List[Dict[str, Any]] = []
    for route_id, metrics in [
        (balanced_quality_route_id, balanced),
        (quality_first_route_id, quality_first),
    ]:
        if metrics["top1"] >= default_systems["metrics"]["top1"] + min_quality_gain:
            quality_candidates.append({"route_id": route_id, "metrics": metrics})

    if not quality_candidates:
        raise ValueError("no quality candidates improved top1 over the selected default")

    quality_candidates.sort(key=lambda row: (row["metrics"]["amortized_per_repeat_sec"], -row["metrics"]["top1"]))
    balanced_selection = quality_candidates[0]
    quality_first_selection = max(quality_candidates, key=lambda row: row["metrics"]["top1"])

    selection_mode = (
        "near_hard_promoted_over_stale_backfill"
        if stale_backfill
        else "historical_backfill_retained"
    )
    carry_forward_contract = (
        "single_lower_bank_default_plus_two_comparators"
        if stale_backfill
        else "historical_lower_bank_default_retained"
    )

    interpretation = (
        f"{default_systems['route_id']} becomes the explicit lower-bank default because it keeps the fastest "
        "stable translated sparse-event systems profile after the focused prewarmed confirm. "
        f"{balanced_selection['route_id']} carries the smallest quality lift over the default, while "
        f"{quality_first_selection['route_id']} remains the quality-first comparator. "
        f"{historical_backfill_route_id} is demoted because its focused amortized time inflated by "
        f"{_fmt(packet_scope_delta)}s ({_fmt(packet_scope_ratio, 3)}x historical), making it unsafe as the "
        "default carry-forward route."
    )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "sources": {
            "historical_backfill_analysis": historical_backfill_analysis.get("experiment_id", ""),
            "soft_bias_screen_analysis": soft_bias_screen_analysis.get("experiment_id", ""),
            "soft_bias_confirm_analysis": soft_bias_confirm_analysis.get("experiment_id", ""),
        },
        "tolerances": {
            "top1": top1_tolerance,
            "stale_amortized_ratio": stale_amortized_ratio,
            "min_quality_gain": min_quality_gain,
        },
        "selection_mode": selection_mode,
        "carry_forward_contract": carry_forward_contract,
        "default_systems_reference": default_systems,
        "balanced_quality_comparator": {
            "route_id": balanced_selection["route_id"],
            "classification": "balanced_quality_comparator",
            "metrics": balanced_selection["metrics"],
            "delta_vs_default": _delta_metrics(balanced_selection["metrics"], default_systems["metrics"]),
            "screen_confirm_delta": _screen_confirm_delta(balanced_screen, balanced),
        },
        "quality_first_comparator": {
            "route_id": quality_first_selection["route_id"],
            "classification": "quality_first_comparator",
            "metrics": quality_first_selection["metrics"],
            "delta_vs_default": _delta_metrics(quality_first_selection["metrics"], default_systems["metrics"]),
            "delta_vs_dense": _delta_metrics(quality_first_selection["metrics"], dense),
            "screen_confirm_delta": _screen_confirm_delta(quality_first_screen, quality_first),
        },
        "stale_historical_comparator": {
            "route_id": historical_backfill_route_id,
            "classification": "stale_historical_comparator" if stale_backfill else "retained_default",
            "historical_metrics": historical_backfill,
            "focused_confirm_metrics": focused_backfill,
            "packet_scope_amortized_delta_sec": packet_scope_delta,
            "packet_scope_amortized_ratio": packet_scope_ratio,
            "focused_confirm_delta_vs_default": _delta_metrics(focused_backfill, default_systems["metrics"]),
        },
        "dense_context": {
            "route_id": dense_route_id,
            "metrics": dense,
        },
        "interpretation": interpretation,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    default_route = payload["default_systems_reference"]
    balanced = payload["balanced_quality_comparator"]
    quality_first = payload["quality_first_comparator"]
    stale = payload["stale_historical_comparator"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Carry-forward contract: `{payload['carry_forward_contract']}`",
        f"- Selection mode: `{payload['selection_mode']}`",
        f"- Default lower-bank route: `{default_route['route_id']}`",
        f"- Balanced comparator: `{balanced['route_id']}`",
        f"- Quality-first comparator: `{quality_first['route_id']}`",
        f"- Historical comparator status: `{stale['classification']}`",
        "",
        "## Key Read",
        f"- Default systems reference `{default_route['route_id']}`",
        f"  - top1 `{_fmt(default_route['metrics']['top1'], 4)}`",
        f"  - cand_frac `{_fmt(default_route['metrics']['candidate_fraction'], 6)}`",
        f"  - amortized `{_fmt(default_route['metrics']['amortized_per_repeat_sec'], 4)}s`",
        f"- Balanced quality comparator `{balanced['route_id']}`",
        f"  - top1 delta vs default `{_fmt(balanced['delta_vs_default']['top1'], 4)}`",
        f"  - amortized delta vs default `{_fmt(balanced['delta_vs_default']['amortized_per_repeat_sec'], 4)}s`",
        f"- Quality-first comparator `{quality_first['route_id']}`",
        f"  - top1 delta vs default `{_fmt(quality_first['delta_vs_default']['top1'], 4)}`",
        f"  - amortized delta vs default `{_fmt(quality_first['delta_vs_default']['amortized_per_repeat_sec'], 4)}s`",
        f"  - top1 delta vs dense `{_fmt(quality_first['delta_vs_dense']['top1'], 4)}`",
        f"  - amortized delta vs dense `{_fmt(quality_first['delta_vs_dense']['amortized_per_repeat_sec'], 4)}s`",
        f"- Historical backfill packet-scope amortized delta `{_fmt(stale['packet_scope_amortized_delta_sec'], 4)}s`",
        f"- Historical backfill packet-scope amortized ratio `{_fmt(stale['packet_scope_amortized_ratio'], 3)}x`",
        "",
        "## Interpretation",
        f"- {payload['interpretation']}",
    ]
    output_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Select the lower-bank sparse-event translated carry-forward routes.")
    parser.add_argument("--historical-backfill-analysis", required=True)
    parser.add_argument("--soft-bias-screen-analysis", required=True)
    parser.add_argument("--soft-bias-confirm-analysis", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--report", default="")
    parser.add_argument("--title", default="INC0132 Product Phase Sparse Event Translation Lower-Bank Reference Reselection")
    args = parser.parse_args()

    payload = build_payload(
        _load_json(args.historical_backfill_analysis),
        _load_json(args.soft_bias_screen_analysis),
        _load_json(args.soft_bias_confirm_analysis),
    )

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.report:
        report_path = Path(args.report)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        write_report(payload, report_path, args.title)


if __name__ == "__main__":
    main()
