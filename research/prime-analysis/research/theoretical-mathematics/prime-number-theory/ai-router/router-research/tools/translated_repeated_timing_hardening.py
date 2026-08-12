#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


EPS = 1e-9


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


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


def _parse_compare(value: str) -> Dict[str, str]:
    label, baseline_route_id, candidate_route_id = value.split(":", 2)
    return {
        "label": label.strip(),
        "baseline_route_id": baseline_route_id.strip(),
        "candidate_route_id": candidate_route_id.strip(),
    }


def _load_analysis(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    data["_analysis_path"] = path
    return data


def _runs_by_seed(analysis: Dict[str, Any], route_id: str) -> Dict[int, Dict[str, Any]]:
    route_runs = analysis.get("results", {}).get(route_id, [])
    result: Dict[int, Dict[str, Any]] = {}
    for run in route_runs:
        seed = run.get("args", {}).get("seed")
        if seed is None:
            continue
        result[int(seed)] = run
    return result


def _observation(source_label: str, seed: int, baseline: Dict[str, Any], candidate: Dict[str, Any]) -> Dict[str, Any]:
    baseline_metrics = baseline.get("metrics", {})
    candidate_metrics = candidate.get("metrics", {})
    return {
        "source_label": source_label,
        "seed": seed,
        "top1_delta_candidate_minus_baseline": _safe_float(candidate_metrics.get("test_top1_after"))
        - _safe_float(baseline_metrics.get("test_top1_after")),
        "candidate_fraction_delta": _safe_float(candidate_metrics.get("retrieval_candidate_fraction_mean"))
        - _safe_float(baseline_metrics.get("retrieval_candidate_fraction_mean")),
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


def _per_seed(observations: List[Dict[str, Any]], seed: int) -> Dict[str, Any]:
    seed_rows = [row for row in observations if int(row["seed"]) == int(seed)]
    metric_keys = [
        "top1_delta_candidate_minus_baseline",
        "candidate_fraction_delta",
        "amortized_delta_candidate_minus_baseline_sec",
        "offline_delta_candidate_minus_baseline_sec",
        "online_delta_candidate_minus_baseline_sec",
        "route_index_build_delta_candidate_minus_baseline_sec",
        "route_query_delta_candidate_minus_baseline_sec",
        "retrieval_search_delta_candidate_minus_baseline_sec",
    ]
    return {
        "seed": seed,
        "repeat_count": len(seed_rows),
        "source_labels": [row["source_label"] for row in seed_rows],
        "summaries": {
            key: _summary([row[key] for row in seed_rows])
            for key in metric_keys
        },
    }


def _interpretation(comparison: Dict[str, Any]) -> str:
    per_seed = comparison["per_seed"]
    mixed_amortized = [
        row["seed"]
        for row in per_seed
        if row["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"] == "mixed"
    ]
    stable_win = [
        row["seed"]
        for row in per_seed
        if row["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"] == "stable_improvement"
    ]
    stable_loss = [
        row["seed"]
        for row in per_seed
        if row["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"] == "stable_regression"
    ]
    route_query_mixed = [
        row["seed"]
        for row in per_seed
        if row["summaries"]["route_query_delta_candidate_minus_baseline_sec"]["sign_consistency"] == "mixed"
    ]
    search_mixed = [
        row["seed"]
        for row in per_seed
        if row["summaries"]["retrieval_search_delta_candidate_minus_baseline_sec"]["sign_consistency"] == "mixed"
    ]

    parts = []
    if mixed_amortized:
        parts.append(
            f"Amortized sign flips across repeats for seeds {', '.join(str(seed) for seed in mixed_amortized)}, so timing noise is still present."
        )
    elif stable_win and not stable_loss:
        parts.append("Every seed stays sign-stable improvement across repeats.")
    elif stable_loss and not stable_win:
        parts.append("Every seed stays sign-stable regression across repeats.")
    else:
        parts.append("Amortized behavior is repeat-stable within seed, but not uniform across seeds.")

    if route_query_mixed:
        parts.append(
            f"Route-query remains repeat-mixed for seeds {', '.join(str(seed) for seed in route_query_mixed)}."
        )
    if search_mixed:
        parts.append(
            f"Retrieval-search remains repeat-mixed for seeds {', '.join(str(seed) for seed in search_mixed)}."
        )
    if stable_win and stable_loss:
        parts.append("That pattern is more consistent with data-sensitive variance than a single clean systems law.")
    return " ".join(parts)


def build_payload(analyses: Iterable[Dict[str, Any]], comparison_specs: List[Dict[str, str]]) -> Dict[str, Any]:
    analyses = list(analyses)
    comparisons: List[Dict[str, Any]] = []
    metric_keys = [
        "top1_delta_candidate_minus_baseline",
        "candidate_fraction_delta",
        "amortized_delta_candidate_minus_baseline_sec",
        "offline_delta_candidate_minus_baseline_sec",
        "online_delta_candidate_minus_baseline_sec",
        "route_index_build_delta_candidate_minus_baseline_sec",
        "route_query_delta_candidate_minus_baseline_sec",
        "retrieval_search_delta_candidate_minus_baseline_sec",
    ]

    for spec in comparison_specs:
        observations: List[Dict[str, Any]] = []
        for analysis in analyses:
            baseline_runs = _runs_by_seed(analysis, spec["baseline_route_id"])
            candidate_runs = _runs_by_seed(analysis, spec["candidate_route_id"])
            source_label = str(analysis.get("experiment_id", Path(str(analysis.get("_analysis_path", ""))).stem))
            for seed in sorted(set(baseline_runs) & set(candidate_runs)):
                observations.append(_observation(source_label, seed, baseline_runs[seed], candidate_runs[seed]))

        aggregate = {
            key: _summary([row[key] for row in observations])
            for key in metric_keys
        }
        seeds = sorted({int(row["seed"]) for row in observations})
        per_seed = [_per_seed(observations, seed) for seed in seeds]
        comparison = {
            "label": spec["label"],
            "baseline_route_id": spec["baseline_route_id"],
            "candidate_route_id": spec["candidate_route_id"],
            "observation_count": len(observations),
            "source_labels": sorted({row["source_label"] for row in observations}),
            "observations": observations,
            "aggregate_summaries": aggregate,
            "per_seed": per_seed,
        }
        comparison["interpretation"] = _interpretation(comparison)
        comparisons.append(comparison)

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "source_experiments": [str(analysis.get("experiment_id", "")) for analysis in analyses],
        "source_configs": [str(analysis.get("config", "")) for analysis in analyses],
        "comparisons": comparisons,
    }


def write_report(payload: Dict[str, Any], output_path: Path) -> None:
    lines = [
        "# Translated Repeated Timing Hardening Audit",
        "",
        "## Source",
    ]
    for experiment_id in payload.get("source_experiments", []):
        lines.append(f"- source_experiment: `{experiment_id}`")
    lines.append("")
    lines.append("## Comparisons")
    for comparison in payload.get("comparisons", []):
        lines.append(
            f"- `{comparison['label']}`: baseline=`{comparison['baseline_route_id']}`, "
            f"candidate=`{comparison['candidate_route_id']}`, observations={comparison['observation_count']}"
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
            summary = comparison["aggregate_summaries"][key]
            lines.append(
                f"  aggregate {label}: mean={_fmt(summary['mean'])}, min={_fmt(summary['min'])}, "
                f"max={_fmt(summary['max'])}, std={_fmt(summary['stddev'])}, sign={summary['sign_consistency']}"
            )
        lines.append("  Per-seed repeat summaries:")
        for seed_row in comparison["per_seed"]:
            amort = seed_row["summaries"]["amortized_delta_candidate_minus_baseline_sec"]
            route_query = seed_row["summaries"]["route_query_delta_candidate_minus_baseline_sec"]
            search = seed_row["summaries"]["retrieval_search_delta_candidate_minus_baseline_sec"]
            lines.append(
                "    "
                f"seed={seed_row['seed']}: repeats={seed_row['repeat_count']}, "
                f"amort_mean={_fmt(amort['mean'])}, amort_std={_fmt(amort['stddev'])}, amort_sign={amort['sign_consistency']}, "
                f"route_query_sign={route_query['sign_consistency']}, "
                f"search_sign={search['sign_consistency']}"
            )
        lines.append("")
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Audit repeated timing stability for fixed translated route comparisons.")
    parser.add_argument("--analysis", action="append", required=True, help="Analysis JSON path. Repeat for multiple reruns.")
    parser.add_argument("--compare", action="append", required=True, help="Comparison as label:baseline_route_id:candidate_route_id")
    parser.add_argument("--output-json", required=True, help="Output JSON path.")
    parser.add_argument("--output-md", required=True, help="Output markdown path.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    analyses = [_load_analysis(path) for path in args.analysis]
    comparisons = [_parse_compare(value) for value in args.compare]
    payload = build_payload(analyses, comparisons)

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    output_md = Path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    write_report(payload, output_md)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
