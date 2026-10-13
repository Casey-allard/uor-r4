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
HIGHER_IS_BETTER_KEYS = {
    "top1_delta_candidate_minus_baseline",
}


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


def _fmt(value: float, places: int = 6) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _finite(values: Iterable[float]) -> List[float]:
    return [value for value in values if math.isfinite(value)]


def _mean(values: List[float]) -> float:
    finite = _finite(values)
    if not finite:
        return float("nan")
    return sum(finite) / len(finite)


def _median(values: List[float]) -> float:
    finite = sorted(_finite(values))
    if not finite:
        return float("nan")
    mid = len(finite) // 2
    if len(finite) % 2:
        return finite[mid]
    return (finite[mid - 1] + finite[mid]) / 2.0


def _trimmed_mean(values: List[float], trim_count: int) -> float:
    finite = sorted(_finite(values))
    if not finite:
        return float("nan")
    trim = max(int(trim_count), 0)
    if trim and len(finite) > trim * 2:
        finite = finite[trim:-trim]
    return _mean(finite)


def _stddev(values: List[float]) -> float:
    finite = _finite(values)
    if len(finite) < 2:
        return 0.0 if finite else float("nan")
    mean_value = _mean(finite)
    variance = sum((value - mean_value) ** 2 for value in finite) / len(finite)
    return math.sqrt(variance)


def _metric_polarity(metric_key: str) -> int:
    return 1 if metric_key in HIGHER_IS_BETTER_KEYS else -1


def _sign_consistency(values: List[float], polarity: int) -> str:
    finite = _finite(values)
    if not finite:
        return "missing"
    oriented = [polarity * value for value in finite]
    has_improve = any(value > EPS for value in oriented)
    has_regress = any(value < -EPS for value in oriented)
    if has_improve and has_regress:
        return "mixed"
    if has_improve:
        return "stable_improvement"
    if has_regress:
        return "stable_regression"
    return "flat"


def _robust_sign(values: List[float], trim_count: int, polarity: int) -> str:
    median = _median(values)
    trimmed_mean = _trimmed_mean(values, trim_count)
    if math.isfinite(median) and math.isfinite(trimmed_mean):
        oriented_median = polarity * median
        oriented_trimmed = polarity * trimmed_mean
        if oriented_median > EPS and oriented_trimmed > EPS:
            return "robust_improvement"
        if oriented_median < -EPS and oriented_trimmed < -EPS:
            return "robust_regression"
        if abs(median) <= EPS and abs(trimmed_mean) <= EPS:
            return "robust_flat"
        return "mixed"
    return "missing"


def _summary(metric_key: str, values: List[float], trim_count: int) -> Dict[str, Any]:
    finite = _finite(values)
    polarity = _metric_polarity(metric_key)
    return {
        "count": len(finite),
        "mean": _mean(finite),
        "median": _median(finite),
        "trimmed_mean": _trimmed_mean(finite, trim_count),
        "min": min(finite) if finite else float("nan"),
        "max": max(finite) if finite else float("nan"),
        "stddev": _stddev(finite),
        "negative_count": sum(1 for value in finite if value < -EPS),
        "positive_count": sum(1 for value in finite if value > EPS),
        "flat_count": sum(1 for value in finite if abs(value) <= EPS),
        "sign_consistency": _sign_consistency(finite, polarity),
        "robust_sign": _robust_sign(finite, trim_count, polarity),
        "trim_count": trim_count,
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


def _query_repeats(run: Dict[str, Any]) -> int:
    metrics = run.get("metrics", {})
    args = run.get("args", {})
    value = metrics.get("retrieval_query_repeats", args.get("query_repeats", 1))
    try:
        return max(int(value or 1), 1)
    except (TypeError, ValueError):
        return 1


def _per_repeat(run: Dict[str, Any], timing_key: str) -> float:
    total = _safe_float(run.get("timings_sec", {}).get(timing_key))
    repeats = _query_repeats(run)
    if not math.isfinite(total):
        return float("nan")
    return total / repeats


def _observation(source_label: str, seed: int, baseline: Dict[str, Any], candidate: Dict[str, Any]) -> Dict[str, Any]:
    baseline_metrics = baseline.get("metrics", {})
    candidate_metrics = candidate.get("metrics", {})
    baseline_count = _safe_float(baseline_metrics.get("retrieval_candidate_count_mean"))
    candidate_count = _safe_float(candidate_metrics.get("retrieval_candidate_count_mean"))
    return {
        "source_label": source_label,
        "seed": seed,
        "top1_delta_candidate_minus_baseline": _safe_float(candidate_metrics.get("test_top1_after"))
        - _safe_float(baseline_metrics.get("test_top1_after")),
        "candidate_fraction_delta": _safe_float(candidate_metrics.get("retrieval_candidate_fraction_mean"))
        - _safe_float(baseline_metrics.get("retrieval_candidate_fraction_mean")),
        "candidate_count_delta": candidate_count - baseline_count,
        "candidate_count_ratio_candidate_vs_baseline": (
            candidate_count / baseline_count
            if math.isfinite(candidate_count) and math.isfinite(baseline_count) and baseline_count > EPS
            else float("nan")
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


def _interpretation(comparison: Dict[str, Any]) -> str:
    summaries = comparison["robust_summaries"]
    amortized = summaries["amortized_delta_candidate_minus_baseline_sec"]["robust_sign"]
    cand_frac = summaries["candidate_fraction_delta"]["robust_sign"]
    cand_count = summaries["candidate_count_delta"]["robust_sign"]
    top1 = summaries["top1_delta_candidate_minus_baseline"]["robust_sign"]
    route_query = summaries["route_query_delta_candidate_minus_baseline_sec"]["robust_sign"]
    search = summaries["retrieval_search_delta_candidate_minus_baseline_sec"]["robust_sign"]

    parts = []
    if amortized == "robust_improvement":
        parts.append("Amortized cost stays robustly favorable under both median and trimmed-mean summaries.")
    elif amortized == "robust_regression":
        parts.append("Amortized cost stays robustly unfavorable under both median and trimmed-mean summaries.")
    else:
        parts.append("Amortized cost does not keep a single robust sign once the repeated evidence is summarized by both median and trimmed mean.")

    if cand_frac == "robust_improvement" and cand_count == "robust_improvement":
        parts.append("Candidate-fraction and candidate-count reduction both remain robust.")
    elif cand_frac == "robust_improvement":
        parts.append("Candidate-fraction reduction remains robust even where timing does not.")

    if search == "robust_improvement":
        parts.append("Retrieval-search cost remains a robust part of the advantage.")
    elif search == "mixed":
        parts.append("Retrieval-search cost still changes sign across the repeated evidence.")

    if route_query == "robust_improvement":
        parts.append("Route-query also contributes robustly.")
    elif route_query == "mixed":
        parts.append("Route-query remains too mixed to use as direct optimization guidance.")

    if top1 == "robust_flat":
        parts.append("Top-1 stays effectively unchanged under the robust summaries.")
    elif top1 == "robust_regression":
        parts.append("Top-1 stays lower under the robust summaries.")
    elif top1 == "robust_improvement":
        parts.append("Top-1 stays higher under the robust summaries.")
    elif top1 == "mixed":
        parts.append("Top-1 deltas remain small and mixed.")
    return " ".join(parts)


def _systems_verdict(comparison: Dict[str, Any]) -> str:
    summaries = comparison["robust_summaries"]
    amortized = summaries["amortized_delta_candidate_minus_baseline_sec"]["robust_sign"]
    cand_frac = summaries["candidate_fraction_delta"]["robust_sign"]
    if amortized == "robust_improvement" and cand_frac == "robust_improvement":
        return "promote"
    if cand_frac == "robust_improvement":
        return "pruning_only"
    if amortized == "robust_improvement":
        return "timing_only"
    if amortized == "robust_regression":
        return "demote"
    return "mixed"


def build_payload(analyses: Iterable[Dict[str, Any]], comparison_specs: List[Dict[str, str]], trim_count: int) -> Dict[str, Any]:
    analyses = list(analyses)
    comparisons: List[Dict[str, Any]] = []
    metric_keys = [
        "top1_delta_candidate_minus_baseline",
        "candidate_fraction_delta",
        "candidate_count_delta",
        "amortized_delta_candidate_minus_baseline_sec",
        "offline_delta_candidate_minus_baseline_sec",
        "online_delta_candidate_minus_baseline_sec",
        "route_index_build_delta_candidate_minus_baseline_sec",
        "route_query_delta_candidate_minus_baseline_sec",
        "retrieval_search_delta_candidate_minus_baseline_sec",
    ]

    for spec in comparison_specs:
        observations: List[Dict[str, Any]] = []
        source_labels: List[str] = []
        for analysis in analyses:
            source_label = str(analysis.get("experiment_id") or Path(str(analysis.get("_analysis_path", ""))).stem)
            baseline_runs = _runs_by_seed(analysis, spec["baseline_route_id"])
            candidate_runs = _runs_by_seed(analysis, spec["candidate_route_id"])
            common_seeds = sorted(set(baseline_runs) & set(candidate_runs))
            if not common_seeds:
                continue
            source_labels.append(source_label)
            for seed in common_seeds:
                observations.append(
                    _observation(
                        source_label=source_label,
                        seed=seed,
                        baseline=baseline_runs[seed],
                        candidate=candidate_runs[seed],
                    )
                )
        robust_summaries = {
            key: _summary(key, [row[key] for row in observations], trim_count)
            for key in metric_keys
        }
        per_seed: List[Dict[str, Any]] = []
        for seed in sorted({int(row["seed"]) for row in observations}):
            seed_rows = [row for row in observations if int(row["seed"]) == seed]
            per_seed.append(
                {
                    "seed": seed,
                    "observation_count": len(seed_rows),
                    "source_labels": [row["source_label"] for row in seed_rows],
                    "robust_summaries": {
                        key: _summary(key, [row[key] for row in seed_rows], trim_count)
                        for key in metric_keys
                    },
                }
            )
        comparison = {
            "label": spec["label"],
            "baseline_route_id": spec["baseline_route_id"],
            "candidate_route_id": spec["candidate_route_id"],
            "observation_count": len(observations),
            "source_labels": source_labels,
            "observations": observations,
            "per_seed": per_seed,
            "robust_summaries": robust_summaries,
        }
        comparison["systems_verdict"] = _systems_verdict(comparison)
        comparison["interpretation"] = _interpretation(comparison)
        comparisons.append(comparison)

    promoted = [row["label"] for row in comparisons if row["systems_verdict"] == "promote"]
    pruning_only = [row["label"] for row in comparisons if row["systems_verdict"] == "pruning_only"]
    mixed = [row["label"] for row in comparisons if row["systems_verdict"] == "mixed"]
    demoted = [row["label"] for row in comparisons if row["systems_verdict"] == "demote"]

    overall_read = []
    if promoted:
        overall_read.append(
            f"Robust systems promotion survives for {', '.join(promoted)}."
        )
    if pruning_only:
        overall_read.append(
            f"Stable pruning survives but wallclock stays mixed for {', '.join(pruning_only)}."
        )
    if mixed:
        overall_read.append(
            f"Neither a clean robust promotion nor demotion is justified for {', '.join(mixed)}."
        )
    if demoted:
        overall_read.append(
            f"Robust demotion is justified for {', '.join(demoted)}."
        )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "trim_count": trim_count,
        "source_experiments": [
            str(analysis.get("experiment_id") or Path(str(analysis.get("_analysis_path", ""))).stem)
            for analysis in analyses
        ],
        "source_configs": [str(analysis.get("config", "")) for analysis in analyses],
        "comparisons": comparisons,
        "overall_read": " ".join(overall_read),
    }


def write_report(payload: Dict[str, Any], output_path: Path) -> None:
    lines = [
        "# Translated Robust Cost Reference",
        "",
        "## Source",
    ]
    for experiment in payload.get("source_experiments", []):
        lines.append(f"- source_experiment: `{experiment}`")
    for config in payload.get("source_configs", []):
        if config:
            lines.append(f"- source_config: `{config}`")

    lines.extend(
        [
            "",
            "## Overall Read",
            f"- {payload.get('overall_read', '')}",
            "",
            "## Comparisons",
        ]
    )

    for comparison in payload.get("comparisons", []):
        lines.append(
            f"- `{comparison['label']}`: baseline=`{comparison['baseline_route_id']}`, "
            f"candidate=`{comparison['candidate_route_id']}`, observations={comparison['observation_count']}, "
            f"verdict=`{comparison['systems_verdict']}`"
        )
        lines.append(f"  {comparison['interpretation']}")
        for metric_key, short_label in [
            ("amortized_delta_candidate_minus_baseline_sec", "amortized"),
            ("route_index_build_delta_candidate_minus_baseline_sec", "route_index"),
            ("route_query_delta_candidate_minus_baseline_sec", "route_query"),
            ("retrieval_search_delta_candidate_minus_baseline_sec", "retrieval_search"),
            ("candidate_fraction_delta", "cand_frac"),
            ("candidate_count_delta", "cand_count"),
            ("top1_delta_candidate_minus_baseline", "top1"),
        ]:
            summary = comparison["robust_summaries"][metric_key]
            lines.append(
                f"  {short_label}: mean={_fmt(summary['mean'])}, median={_fmt(summary['median'])}, "
                f"trimmed_mean={_fmt(summary['trimmed_mean'])}, min={_fmt(summary['min'])}, "
                f"max={_fmt(summary['max'])}, std={_fmt(summary['stddev'])}, "
                f"sign={summary['sign_consistency']}, robust={summary['robust_sign']}"
            )
        lines.append("  Per-seed robust summaries:")
        for row in comparison["per_seed"]:
            amort = row["robust_summaries"]["amortized_delta_candidate_minus_baseline_sec"]
            search = row["robust_summaries"]["retrieval_search_delta_candidate_minus_baseline_sec"]
            lines.append(
                f"    seed={row['seed']}: repeats={row['observation_count']}, "
                f"amort_median={_fmt(amort['median'])}, amort_trimmed={_fmt(amort['trimmed_mean'])}, "
                f"amort_robust={amort['robust_sign']}, search_robust={search['robust_sign']}"
            )
        lines.append("")

    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Build a robust cost reference from repeated translated sparse analyses.")
    parser.add_argument("--analysis", action="append", required=True, help="Path to a raw analysis JSON file.")
    parser.add_argument(
        "--compare",
        action="append",
        required=True,
        help="Comparison spec: label:baseline_route_id:candidate_route_id",
    )
    parser.add_argument("--trim-count", type=int, default=1, help="Values to trim from each side for trimmed means.")
    parser.add_argument("--output-json", required=True, help="Path to write the JSON audit artifact.")
    parser.add_argument("--output-md", required=True, help="Path to write the Markdown report.")
    args = parser.parse_args()

    analyses = [_load_analysis(path) for path in args.analysis]
    comparisons = [_parse_compare(value) for value in args.compare]
    payload = build_payload(analyses, comparisons, trim_count=args.trim_count)

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    output_md = Path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    write_report(payload, output_md)


if __name__ == "__main__":
    main()
