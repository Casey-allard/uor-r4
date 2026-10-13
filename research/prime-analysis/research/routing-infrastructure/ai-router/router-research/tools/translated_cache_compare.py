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

from tools import translated_hardware_profile as profile


def _safe_float(value: Any) -> float:
    return profile._safe_float(value)


def _fmt(value: float, places: int = 3) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _profiles_by_id(analysis: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    rows = profile.flatten_analysis(analysis, source_label=str(analysis.get("experiment_id", "analysis")))
    return {str(row["route_id"]): row for row in profile.derive_group_profiles(rows)}


def build_payload(cold: Dict[str, Any], warm: Dict[str, Any], route_ids: List[str], tolerance: float) -> Dict[str, Any]:
    cold_profiles = _profiles_by_id(cold)
    warm_profiles = _profiles_by_id(warm)
    cold_stats = {str(row.get("route_id", "")): row for row in cold.get("route_stats", [])}
    warm_stats = {str(row.get("route_id", "")): row for row in warm.get("route_stats", [])}
    comparisons: List[Dict[str, Any]] = []
    for route_id in route_ids:
        cold_row = cold_profiles.get(route_id)
        warm_row = warm_profiles.get(route_id)
        if cold_row is None or warm_row is None:
            continue
        cold_stat = cold_stats.get(route_id, {})
        warm_stat = warm_stats.get(route_id, {})
        cold_repeats = max(int(cold_row.get("query_repeats", 0) or 0), 1)
        warm_repeats = max(int(warm_row.get("query_repeats", 0) or 0), 1)
        comparison = {
            "route_id": route_id,
            "is_dense": bool(warm_row.get("is_dense")),
            "max_train": int(warm_row.get("max_train", 0) or 0),
            "query_repeats": int(warm_row.get("query_repeats", 0) or 0),
            "cold_top1": _safe_float(cold_row.get("top1")),
            "warm_top1": _safe_float(warm_row.get("top1")),
            "cold_candidate_fraction": _safe_float(cold_row.get("candidate_fraction")),
            "warm_candidate_fraction": _safe_float(warm_row.get("candidate_fraction")),
            "cold_offline_per_repeat_sec": _safe_float(cold_row.get("offline_sec")) / cold_repeats,
            "warm_offline_per_repeat_sec": _safe_float(warm_row.get("offline_sec")) / warm_repeats,
            "cold_online_per_repeat_sec": _safe_float(cold_row.get("online_per_repeat_sec")),
            "warm_online_per_repeat_sec": _safe_float(warm_row.get("online_per_repeat_sec")),
            "cold_amortized_per_repeat_sec": _safe_float(cold_row.get("amortized_per_repeat_sec")),
            "warm_amortized_per_repeat_sec": _safe_float(warm_row.get("amortized_per_repeat_sec")),
            "cold_route_index_build_per_repeat_sec": _safe_float(cold_stat.get("mean_route_index_build_sec")) / cold_repeats,
            "warm_route_index_build_per_repeat_sec": _safe_float(warm_stat.get("mean_route_index_build_sec")) / warm_repeats,
            "cold_chart_opt_per_repeat_sec": _safe_float(cold_stat.get("mean_offline_total_sec") - cold_stat.get("mean_route_index_build_sec", 0.0)) / cold_repeats,
            "warm_chart_opt_per_repeat_sec": _safe_float(warm_stat.get("mean_offline_total_sec") - warm_stat.get("mean_route_index_build_sec", 0.0)) / warm_repeats,
            "warm_chart_cache_hit": _safe_float(warm_stat.get("mean_retrieval_chart_cache_hit")),
            "warm_route_cache_hit": _safe_float(warm_stat.get("mean_retrieval_route_cache_hit")),
            "top1_stable": abs(_safe_float(cold_row.get("top1")) - _safe_float(warm_row.get("top1"))) <= tolerance,
            "candidate_fraction_stable": abs(_safe_float(cold_row.get("candidate_fraction")) - _safe_float(warm_row.get("candidate_fraction"))) <= tolerance,
            "warm_margin_vs_dense_sec": _safe_float(warm_row.get("amortized_margin_vs_dense")),
            "cold_margin_vs_dense_sec": _safe_float(cold_row.get("amortized_margin_vs_dense")),
        }
        comparison["offline_savings_per_repeat_sec"] = (
            comparison["cold_offline_per_repeat_sec"] - comparison["warm_offline_per_repeat_sec"]
        )
        comparison["amortized_savings_per_repeat_sec"] = (
            comparison["cold_amortized_per_repeat_sec"] - comparison["warm_amortized_per_repeat_sec"]
        )
        comparisons.append(comparison)

    return {
        "generated_at": dt.datetime.now().isoformat(timespec="seconds"),
        "cold_experiment_id": str(cold.get("experiment_id", "")),
        "warm_experiment_id": str(warm.get("experiment_id", "")),
        "route_ids": route_ids,
        "tolerance": tolerance,
        "comparisons": comparisons,
    }


def build_markdown(payload: Dict[str, Any]) -> str:
    lines = [
        "# Translated Cache Compare",
        "",
        "## Source",
        f"- cold: `{payload.get('cold_experiment_id', '')}`",
        f"- warm: `{payload.get('warm_experiment_id', '')}`",
        f"- tolerance: `{payload.get('tolerance', 0.0)}`",
        "",
        "## Route Comparisons",
    ]
    for row in payload.get("comparisons", []):
        lines.append(
            f"- `{row['route_id']}`: cold amortized `{_fmt(row['cold_amortized_per_repeat_sec'])}s`, "
            f"warm amortized `{_fmt(row['warm_amortized_per_repeat_sec'])}s`, "
            f"savings `{_fmt(row['amortized_savings_per_repeat_sec'])}s`"
        )
        lines.append(
            "  "
            f"cold offline `{_fmt(row['cold_offline_per_repeat_sec'])}s`, "
            f"warm offline `{_fmt(row['warm_offline_per_repeat_sec'])}s`, "
            f"offline savings `{_fmt(row['offline_savings_per_repeat_sec'])}s`"
        )
        lines.append(
            "  "
            f"top1 stable={row['top1_stable']}, cand_frac stable={row['candidate_fraction_stable']}, "
            f"warm margin vs dense `{_fmt(row['warm_margin_vs_dense_sec'])}s`"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Compare cold and warm translated cache analyses.")
    ap.add_argument("--cold-analysis", required=True)
    ap.add_argument("--warm-analysis", required=True)
    ap.add_argument("--route-id", action="append", required=True)
    ap.add_argument("--tolerance", type=float, default=1e-12)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-md", required=True)
    args = ap.parse_args()

    with open(args.cold_analysis, "r", encoding="utf-8") as f:
        cold = json.load(f)
    with open(args.warm_analysis, "r", encoding="utf-8") as f:
        warm = json.load(f)

    payload = build_payload(cold, warm, route_ids=list(args.route_id), tolerance=float(args.tolerance))
    output_json = Path(args.output_json)
    output_md = Path(args.output_md)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    output_md.write_text(build_markdown(payload) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
