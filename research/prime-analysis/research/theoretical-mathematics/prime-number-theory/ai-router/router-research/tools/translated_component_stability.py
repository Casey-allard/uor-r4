#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from tools import translated_cost_accounting as accounting


EPS = 1e-9


def _safe_float(value: Any) -> float:
    return accounting._safe_float(value)


def _fmt(value: float, places: int = 6) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _mean(values: List[float]) -> float:
    finite = [value for value in values if math.isfinite(value)]
    if not finite:
        return float("nan")
    return sum(finite) / len(finite)


def _stddev(values: List[float]) -> float:
    finite = [value for value in values if math.isfinite(value)]
    if len(finite) < 2:
        return 0.0 if finite else float("nan")
    mean_value = _mean(finite)
    variance = sum((value - mean_value) ** 2 for value in finite) / len(finite)
    return math.sqrt(variance)


def _sign_consistency(values: List[float]) -> str:
    finite = [value for value in values if math.isfinite(value)]
    if not finite:
        return "missing"
    has_neg = any(value < -EPS for value in finite)
    has_pos = any(value > EPS for value in finite)
    if has_neg and has_pos:
        return "mixed"
    if has_neg:
        return "stable_improvement"
    if has_pos:
        return "stable_regression"
    return "flat"


def _summary(values: List[float]) -> Dict[str, Any]:
    finite = [value for value in values if math.isfinite(value)]
    return {
        "mean": _mean(finite),
        "min": min(finite) if finite else float("nan"),
        "max": max(finite) if finite else float("nan"),
        "stddev": _stddev(finite),
        "negative_count": sum(1 for value in finite if value < -EPS),
        "positive_count": sum(1 for value in finite if value > EPS),
        "flat_count": sum(1 for value in finite if abs(value) <= EPS),
        "sign_consistency": _sign_consistency(finite),
    }


def _parse_compare(value: str) -> Dict[str, str]:
    label, baseline_route_id, candidate_route_id = value.split(":", 2)
    return {
        "label": label.strip(),
        "baseline_route_id": baseline_route_id.strip(),
        "candidate_route_id": candidate_route_id.strip(),
    }


def _load_analysis(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _query_repeats(run: Dict[str, Any]) -> int:
    metrics = run.get("metrics", {})
    args = run.get("args", {})
    value = metrics.get("retrieval_query_repeats", args.get("query_repeats", 1))
    try:
        return max(int(value or 1), 1)
    except (TypeError, ValueError):
        return 1


def _per_repeat(run: Dict[str, Any], timing_key: str) -> float:
    timings = run.get("timings_sec", {})
    total = _safe_float(timings.get(timing_key))
    repeats = _query_repeats(run)
    if not math.isfinite(total):
        return float("nan")
    return total / repeats


def _runs_by_seed(analysis: Dict[str, Any], route_id: str) -> Dict[int, Dict[str, Any]]:
    results = analysis.get("results", {}).get(route_id, [])
    by_seed: Dict[int, Dict[str, Any]] = {}
    for run in results:
        seed = run.get("args", {}).get("seed")
        if seed is None:
            continue
        by_seed[int(seed)] = run
    return by_seed


def _seed_entry(seed: int, baseline: Dict[str, Any], candidate: Dict[str, Any]) -> Dict[str, Any]:
    baseline_metrics = baseline.get("metrics", {})
    candidate_metrics = candidate.get("metrics", {})
    baseline_q = _query_repeats(baseline)
    candidate_q = _query_repeats(candidate)
    return {
        "seed": seed,
        "baseline_query_repeats": baseline_q,
        "candidate_query_repeats": candidate_q,
        "baseline_top1": _safe_float(baseline_metrics.get("test_top1_after")),
        "candidate_top1": _safe_float(candidate_metrics.get("test_top1_after")),
        "top1_delta_candidate_minus_baseline": _safe_float(candidate_metrics.get("test_top1_after"))
        - _safe_float(baseline_metrics.get("test_top1_after")),
        "baseline_candidate_fraction": _safe_float(baseline_metrics.get("retrieval_candidate_fraction_mean")),
        "candidate_candidate_fraction": _safe_float(candidate_metrics.get("retrieval_candidate_fraction_mean")),
        "candidate_fraction_delta": _safe_float(candidate_metrics.get("retrieval_candidate_fraction_mean"))
        - _safe_float(baseline_metrics.get("retrieval_candidate_fraction_mean")),
        "baseline_amortized_per_repeat_sec": _safe_float(
            baseline_metrics.get("retrieval_total_amortized_per_repeat_sec")
        ),
        "candidate_amortized_per_repeat_sec": _safe_float(
            candidate_metrics.get("retrieval_total_amortized_per_repeat_sec")
        ),
        "amortized_delta_candidate_minus_baseline_sec": _safe_float(
            candidate_metrics.get("retrieval_total_amortized_per_repeat_sec")
        )
        - _safe_float(baseline_metrics.get("retrieval_total_amortized_per_repeat_sec")),
        "offline_delta_candidate_minus_baseline_sec": _per_repeat(candidate, "offline_total")
        - _per_repeat(baseline, "offline_total"),
        "online_delta_candidate_minus_baseline_sec": _per_repeat(candidate, "online_total")
        - _per_repeat(baseline, "online_total"),
        "route_index_build_delta_candidate_minus_baseline_sec": _per_repeat(candidate, "route_index_build")
        - _per_repeat(baseline, "route_index_build"),
        "route_query_delta_candidate_minus_baseline_sec": _per_repeat(candidate, "query_route")
        - _per_repeat(baseline, "query_route"),
        "retrieval_search_delta_candidate_minus_baseline_sec": _per_repeat(candidate, "retrieval_search")
        - _per_repeat(baseline, "retrieval_search"),
    }


def _interpretation(label: str, summaries: Dict[str, Dict[str, Any]], win_count: int, loss_count: int) -> str:
    amortized = summaries["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"]
    route_query = summaries["route_query_delta_candidate_minus_baseline_sec"]["sign_consistency"]
    search = summaries["retrieval_search_delta_candidate_minus_baseline_sec"]["sign_consistency"]
    search_mean = summaries["retrieval_search_delta_candidate_minus_baseline_sec"]["mean"]
    route_query_mean = summaries["route_query_delta_candidate_minus_baseline_sec"]["mean"]
    route_index_mean = summaries["route_index_build_delta_candidate_minus_baseline_sec"]["mean"]

    parts = [
        f"{label}: {win_count} seed wins, {loss_count} seed losses on amortized cost."
    ]
    if amortized == "mixed":
        parts.append("Amortized behavior is seed-fragile rather than sign-stable.")
    elif amortized == "stable_improvement":
        parts.append("Amortized behavior is sign-stable improvement across seeds.")
    elif amortized == "stable_regression":
        parts.append("Amortized behavior is sign-stable regression across seeds.")

    if route_query == "mixed":
        parts.append("Route-query deltas change sign across seeds, so they are not safe as optimization guidance yet.")
    elif route_query == "stable_improvement":
        parts.append("Route-query deltas are consistently favorable across seeds.")
    elif route_query == "stable_regression":
        parts.append("Route-query deltas are consistently unfavorable across seeds.")

    if search == "stable_improvement":
        parts.append("Retrieval-search deltas are consistently favorable across seeds.")
    elif search == "mixed" and math.isfinite(search_mean) and search_mean < -EPS:
        parts.append("Retrieval-search savings dominate on average, but the sign is not fully stable.")

    if math.isfinite(route_index_mean) and route_index_mean > EPS:
        parts.append("Route-index build remains a cost penalty on average.")
    if math.isfinite(route_query_mean) and route_query_mean < -EPS and route_query != "mixed":
        parts.append("Route-query is a real part of the average gain.")
    return " ".join(parts)


def build_payload(
    analyses: Iterable[Dict[str, Any]],
    comparison_specs: List[Dict[str, str]],
) -> Dict[str, Any]:
    merged = accounting._merge_analyses(list(analyses))
    comparisons: List[Dict[str, Any]] = []

    for spec in comparison_specs:
        baseline_route_id = spec["baseline_route_id"]
        candidate_route_id = spec["candidate_route_id"]
        baseline_runs = _runs_by_seed(merged, baseline_route_id)
        candidate_runs = _runs_by_seed(merged, candidate_route_id)
        common_seeds = sorted(set(baseline_runs) & set(candidate_runs))
        seed_entries = [_seed_entry(seed, baseline_runs[seed], candidate_runs[seed]) for seed in common_seeds]
        summaries = {
            key: _summary([entry[key] for entry in seed_entries])
            for key in [
                "top1_delta_candidate_minus_baseline",
                "candidate_fraction_delta",
                "amortized_delta_candidate_minus_baseline_sec",
                "offline_delta_candidate_minus_baseline_sec",
                "online_delta_candidate_minus_baseline_sec",
                "route_index_build_delta_candidate_minus_baseline_sec",
                "route_query_delta_candidate_minus_baseline_sec",
                "retrieval_search_delta_candidate_minus_baseline_sec",
            ]
        }
        win_count = sum(
            1
            for entry in seed_entries
            if entry["amortized_delta_candidate_minus_baseline_sec"] < -EPS
        )
        loss_count = sum(
            1
            for entry in seed_entries
            if entry["amortized_delta_candidate_minus_baseline_sec"] > EPS
        )
        comparisons.append(
            {
                "label": spec["label"],
                "baseline_route_id": baseline_route_id,
                "candidate_route_id": candidate_route_id,
                "seed_count": len(seed_entries),
                "seed_entries": seed_entries,
                "summaries": summaries,
                "win_count": win_count,
                "loss_count": loss_count,
                "interpretation": _interpretation(spec["label"], summaries, win_count, loss_count),
            }
        )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "experiment_id": merged.get("experiment_id", ""),
        "config": merged.get("config", ""),
        "source_experiments": merged.get("source_experiments", []),
        "source_configs": merged.get("source_configs", []),
        "comparisons": comparisons,
    }


def write_report(payload: Dict[str, Any], output_path: Path) -> None:
    lines = [
        "# Translated Component Stability Audit",
        "",
        "## Source",
        f"- experiment: `{payload.get('experiment_id', '')}`",
        f"- config: `{payload.get('config', '')}`",
    ]
    for experiment_id in payload.get("source_experiments", []):
        lines.append(f"- source_experiment: `{experiment_id}`")
    lines.append("")
    lines.append("## Comparisons")
    for comparison in payload.get("comparisons", []):
        lines.append(
            f"- `{comparison['label']}`: baseline=`{comparison['baseline_route_id']}`, "
            f"candidate=`{comparison['candidate_route_id']}`, seeds={comparison['seed_count']}"
        )
        lines.append(f"  {comparison['interpretation']}")
        for key, label in [
            ("amortized_delta_candidate_minus_baseline_sec", "amortized"),
            ("route_index_build_delta_candidate_minus_baseline_sec", "route_index"),
            ("route_query_delta_candidate_minus_baseline_sec", "route_query"),
            ("retrieval_search_delta_candidate_minus_baseline_sec", "retrieval_search"),
            ("candidate_fraction_delta", "cand_frac"),
            ("top1_delta_candidate_minus_baseline", "top1"),
        ]:
            summary = comparison["summaries"][key]
            lines.append(
                f"  {label}: mean={_fmt(summary['mean'])}, min={_fmt(summary['min'])}, "
                f"max={_fmt(summary['max'])}, std={_fmt(summary['stddev'])}, "
                f"sign={summary['sign_consistency']}"
            )
        lines.append("  Seed rows:")
        for entry in comparison["seed_entries"]:
            lines.append(
                "    "
                f"seed={entry['seed']}: amort={_fmt(entry['amortized_delta_candidate_minus_baseline_sec'])}, "
                f"route_index={_fmt(entry['route_index_build_delta_candidate_minus_baseline_sec'])}, "
                f"route_query={_fmt(entry['route_query_delta_candidate_minus_baseline_sec'])}, "
                f"search={_fmt(entry['retrieval_search_delta_candidate_minus_baseline_sec'])}, "
                f"cand_frac={_fmt(entry['candidate_fraction_delta'])}, "
                f"top1={_fmt(entry['top1_delta_candidate_minus_baseline'])}"
            )
        lines.append("")
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Audit per-seed component stability for translated route comparisons.")
    parser.add_argument(
        "--analysis",
        action="append",
        required=True,
        help="Path to an analysis JSON. Repeat to merge multiple analyses.",
    )
    parser.add_argument(
        "--compare",
        action="append",
        required=True,
        help="Comparison spec as label:baseline_route_id:candidate_route_id",
    )
    parser.add_argument("--output-json", required=True, help="Path to write the JSON payload.")
    parser.add_argument("--output-md", required=True, help="Path to write the markdown report.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    analyses = [_load_analysis(path) for path in args.analysis]
    comparison_specs = [_parse_compare(value) for value in args.compare]
    payload = build_payload(analyses, comparison_specs)

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    output_md = Path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    write_report(payload, output_md)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
