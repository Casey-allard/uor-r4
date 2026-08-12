#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from tools import translated_hardware_profile as profile


def _safe_float(value: Any) -> float:
    return profile._safe_float(value)


def _ratio(num: float, den: float) -> float:
    return profile._ratio(num, den)


def _diff(lhs: float, rhs: float) -> float:
    return profile._diff(lhs, rhs)


def _fmt(value: float, places: int = 3) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _load_meta(path: Optional[str]) -> Dict[str, Any]:
    if not path:
        return {}
    with open(path, "r", encoding="utf-8") as f:
        obj = json.load(f)
    return obj if isinstance(obj, dict) else {}


def _merge_analyses(analyses: Iterable[Dict[str, Any]]) -> Dict[str, Any]:
    route_stats: List[Dict[str, Any]] = []
    results: Dict[str, Any] = {}
    source_experiments: List[str] = []
    source_configs: List[str] = []
    seen_route_ids: Dict[str, str] = {}

    for analysis in analyses:
        experiment_id = str(analysis.get("experiment_id", ""))
        config = str(analysis.get("config", ""))
        source_experiments.append(experiment_id)
        source_configs.append(config)
        for route_stat in analysis.get("route_stats", []):
            route_id = str(route_stat.get("route_id", ""))
            if route_id in seen_route_ids:
                raise ValueError(
                    f"duplicate route_id '{route_id}' across analyses '{seen_route_ids[route_id]}' and '{experiment_id}'"
                )
            seen_route_ids[route_id] = experiment_id
            route_stats.append(dict(route_stat))
        analysis_results = analysis.get("results", {})
        if isinstance(analysis_results, dict):
            for route_id, runs in analysis_results.items():
                route_id = str(route_id)
                if route_id in results:
                    raise ValueError(
                        f"duplicate results entry for route_id '{route_id}' across merged analyses"
                    )
                results[route_id] = runs

    return {
        "experiment_id": " + ".join([exp for exp in source_experiments if exp]),
        "config": " + ".join([cfg for cfg in source_configs if cfg]),
        "route_stats": route_stats,
        "results": results,
        "source_experiments": source_experiments,
        "source_configs": source_configs,
    }


def _build_profiles(analysis: Dict[str, Any], feature_dim: int, bytes_per_float: int) -> Dict[str, Dict[str, Any]]:
    rows = profile.flatten_analysis(analysis, source_label=str(analysis.get("experiment_id", "analysis")))
    route_profiles = profile.derive_group_profiles(rows)
    stats_by_id = {
        str(route_stat.get("route_id", "")): route_stat for route_stat in analysis.get("route_stats", [])
    }

    by_id: Dict[str, Dict[str, Any]] = {}
    for row in route_profiles:
        route_id = str(row["route_id"])
        stat = stats_by_id.get(route_id, {})
        query_repeats = max(int(row.get("query_repeats", 0) or 0), 1)
        candidate_count = _safe_float(row.get("candidate_count"))
        route_query_total_sec = _safe_float(stat.get("mean_query_route_sec"))
        retrieval_search_total_sec = _safe_float(stat.get("mean_retrieval_search_sec"))
        route_index_build_sec = _safe_float(stat.get("mean_route_index_build_sec"))
        candidate_vector_floats = (
            float(candidate_count * feature_dim) if math.isfinite(candidate_count) and feature_dim > 0 else float("nan")
        )
        candidate_vector_bytes = (
            float(candidate_vector_floats * bytes_per_float)
            if math.isfinite(candidate_vector_floats) and bytes_per_float > 0
            else float("nan")
        )
        route_query_per_repeat_sec = _ratio(route_query_total_sec, query_repeats)
        retrieval_search_per_repeat_sec = _ratio(retrieval_search_total_sec, query_repeats)
        route_index_build_per_repeat_sec = _ratio(route_index_build_sec, query_repeats)
        offline_per_repeat_sec = _ratio(_safe_float(row.get("offline_sec")), query_repeats)
        online_per_repeat_sec = _safe_float(row.get("online_per_repeat_sec"))
        offline_residual_per_repeat_sec = _diff(offline_per_repeat_sec, route_index_build_per_repeat_sec)
        online_residual_per_repeat_sec = _diff(
            online_per_repeat_sec,
            route_query_per_repeat_sec + retrieval_search_per_repeat_sec,
        )
        residual_per_repeat_sec = (
            offline_residual_per_repeat_sec + online_residual_per_repeat_sec
            if math.isfinite(offline_residual_per_repeat_sec) and math.isfinite(online_residual_per_repeat_sec)
            else float("nan")
        )
        enriched = dict(row)
        enriched.update(
            {
                "route_query_total_sec": route_query_total_sec,
                "route_query_per_repeat_sec": route_query_per_repeat_sec,
                "retrieval_search_total_sec": retrieval_search_total_sec,
                "retrieval_search_per_repeat_sec": retrieval_search_per_repeat_sec,
                "route_index_build_sec": route_index_build_sec,
                "route_index_build_per_repeat_sec": route_index_build_per_repeat_sec,
                "offline_per_repeat_sec": offline_per_repeat_sec,
                "offline_residual_per_repeat_sec": offline_residual_per_repeat_sec,
                "online_residual_per_repeat_sec": online_residual_per_repeat_sec,
                "residual_per_repeat_sec": residual_per_repeat_sec,
                "candidate_vector_floats_proxy": candidate_vector_floats,
                "candidate_vector_bytes_proxy": candidate_vector_bytes,
            }
        )
        by_id[route_id] = enriched
    return by_id


def _pair_explanation(pair: Dict[str, Any]) -> str:
    if pair["has_crossover"]:
        return (
            f"Online savings of {_fmt(pair['online_gain_per_repeat_vs_dense_sec'], 3)}s per repeat "
            f"outpace the offline penalty of {_fmt(pair['offline_penalty_per_repeat_vs_dense_sec'], 3)}s."
        )
    return (
        f"The offline penalty of {_fmt(pair['offline_penalty_per_repeat_vs_dense_sec'], 3)}s per repeat "
        f"still exceeds the online savings of {_fmt(pair['online_gain_per_repeat_vs_dense_sec'], 3)}s."
    )


def _dominant_component(components: Dict[str, float]) -> str:
    best_key = ""
    best_value = float("-inf")
    for key, value in components.items():
        if not math.isfinite(value):
            continue
        if value > best_value:
            best_key = key
            best_value = value
    return best_key


def _build_comparison(
    profiles_by_id: Dict[str, Dict[str, Any]],
    label: str,
    baseline_route_id: str,
    candidate_route_id: str,
) -> Optional[Dict[str, Any]]:
    baseline = profiles_by_id.get(str(baseline_route_id))
    candidate = profiles_by_id.get(str(candidate_route_id))
    if baseline is None or candidate is None:
        return None

    route_index_build_penalty = _diff(
        _safe_float(candidate.get("route_index_build_per_repeat_sec")),
        _safe_float(baseline.get("route_index_build_per_repeat_sec")),
    )
    route_query_penalty = _diff(
        _safe_float(candidate.get("route_query_per_repeat_sec")),
        _safe_float(baseline.get("route_query_per_repeat_sec")),
    )
    retrieval_search_penalty = _diff(
        _safe_float(candidate.get("retrieval_search_per_repeat_sec")),
        _safe_float(baseline.get("retrieval_search_per_repeat_sec")),
    )
    residual_penalty = _diff(
        _safe_float(candidate.get("residual_per_repeat_sec")),
        _safe_float(baseline.get("residual_per_repeat_sec")),
    )
    component_penalties = {
        "route_index_build": route_index_build_penalty,
        "route_query": route_query_penalty,
        "retrieval_search": retrieval_search_penalty,
        "residual": residual_penalty,
    }
    dominant_component = _dominant_component(component_penalties)
    amortized_delta = _diff(
        _safe_float(candidate.get("amortized_per_repeat_sec")),
        _safe_float(baseline.get("amortized_per_repeat_sec")),
    )

    explanation = (
        f"{candidate_route_id} relative to {baseline_route_id}: "
        f"route_index_build={_fmt(route_index_build_penalty, 3)}s, "
        f"route_query={_fmt(route_query_penalty, 3)}s, "
        f"retrieval_search={_fmt(retrieval_search_penalty, 3)}s, "
        f"residual={_fmt(residual_penalty, 3)}s, "
        f"amortized_delta={_fmt(amortized_delta, 3)}s."
    )

    return {
        "label": label,
        "baseline_route_id": str(baseline_route_id),
        "candidate_route_id": str(candidate_route_id),
        "baseline_source_label": str(baseline.get("source_label", "")),
        "candidate_source_label": str(candidate.get("source_label", "")),
        "baseline_max_train": int(baseline.get("max_train", 0) or 0),
        "candidate_max_train": int(candidate.get("max_train", 0) or 0),
        "baseline_query_repeats": int(baseline.get("query_repeats", 0) or 0),
        "candidate_query_repeats": int(candidate.get("query_repeats", 0) or 0),
        "baseline_top1": _safe_float(baseline.get("top1")),
        "candidate_top1": _safe_float(candidate.get("top1")),
        "top1_delta_candidate_minus_baseline": _diff(
            _safe_float(candidate.get("top1")),
            _safe_float(baseline.get("top1")),
        ),
        "baseline_candidate_fraction": _safe_float(baseline.get("candidate_fraction")),
        "candidate_candidate_fraction": _safe_float(candidate.get("candidate_fraction")),
        "candidate_fraction_delta": _diff(
            _safe_float(candidate.get("candidate_fraction")),
            _safe_float(baseline.get("candidate_fraction")),
        ),
        "baseline_offline_per_repeat_sec": _safe_float(baseline.get("offline_per_repeat_sec")),
        "candidate_offline_per_repeat_sec": _safe_float(candidate.get("offline_per_repeat_sec")),
        "offline_penalty_candidate_vs_baseline_sec": _diff(
            _safe_float(candidate.get("offline_per_repeat_sec")),
            _safe_float(baseline.get("offline_per_repeat_sec")),
        ),
        "baseline_online_per_repeat_sec": _safe_float(baseline.get("online_per_repeat_sec")),
        "candidate_online_per_repeat_sec": _safe_float(candidate.get("online_per_repeat_sec")),
        "online_penalty_candidate_vs_baseline_sec": _diff(
            _safe_float(candidate.get("online_per_repeat_sec")),
            _safe_float(baseline.get("online_per_repeat_sec")),
        ),
        "baseline_route_index_build_per_repeat_sec": _safe_float(baseline.get("route_index_build_per_repeat_sec")),
        "candidate_route_index_build_per_repeat_sec": _safe_float(candidate.get("route_index_build_per_repeat_sec")),
        "route_index_build_penalty_candidate_vs_baseline_sec": route_index_build_penalty,
        "baseline_route_query_per_repeat_sec": _safe_float(baseline.get("route_query_per_repeat_sec")),
        "candidate_route_query_per_repeat_sec": _safe_float(candidate.get("route_query_per_repeat_sec")),
        "route_query_penalty_candidate_vs_baseline_sec": route_query_penalty,
        "baseline_retrieval_search_per_repeat_sec": _safe_float(baseline.get("retrieval_search_per_repeat_sec")),
        "candidate_retrieval_search_per_repeat_sec": _safe_float(candidate.get("retrieval_search_per_repeat_sec")),
        "retrieval_search_penalty_candidate_vs_baseline_sec": retrieval_search_penalty,
        "baseline_residual_per_repeat_sec": _safe_float(baseline.get("residual_per_repeat_sec")),
        "candidate_residual_per_repeat_sec": _safe_float(candidate.get("residual_per_repeat_sec")),
        "residual_penalty_candidate_vs_baseline_sec": residual_penalty,
        "baseline_amortized_per_repeat_sec": _safe_float(baseline.get("amortized_per_repeat_sec")),
        "candidate_amortized_per_repeat_sec": _safe_float(candidate.get("amortized_per_repeat_sec")),
        "amortized_delta_candidate_minus_baseline_sec": amortized_delta,
        "amortized_savings_vs_baseline_sec": _diff(
            _safe_float(baseline.get("amortized_per_repeat_sec")),
            _safe_float(candidate.get("amortized_per_repeat_sec")),
        ),
        "dominant_penalty_component": dominant_component,
        "explanation": explanation,
    }


def build_payload(
    analysis: Any,
    route_ids: Iterable[str],
    meta: Optional[Dict[str, Any]] = None,
    comparison_specs: Optional[Iterable[Dict[str, str]]] = None,
    bytes_per_float: int = 4,
) -> Dict[str, Any]:
    meta = meta or {}
    analyses = analysis if isinstance(analysis, list) else [analysis]
    merged_analysis = _merge_analyses(analyses)
    feature_dim = int(meta.get("d", 0) or 0)
    profiles_by_id = _build_profiles(merged_analysis, feature_dim=feature_dim, bytes_per_float=bytes_per_float)

    route_audits: List[Dict[str, Any]] = []
    pair_audits: List[Dict[str, Any]] = []
    comparison_audits: List[Dict[str, Any]] = []
    selected_ids = [str(route_id) for route_id in route_ids]
    for route_id in selected_ids:
        row = profiles_by_id.get(route_id)
        if row is None:
            continue
        route_audits.append(row)
        if row.get("is_dense"):
            continue
        dense_id = str(row.get("dense_route_id", ""))
        dense = profiles_by_id.get(dense_id)
        if dense is None:
            continue
        candidate_bytes_routed = _safe_float(row.get("candidate_vector_bytes_proxy"))
        candidate_bytes_dense = _safe_float(dense.get("candidate_vector_bytes_proxy"))
        pair = {
            "dense_route_id": dense_id,
            "routed_route_id": route_id,
            "max_train": int(row.get("max_train", 0) or 0),
            "query_repeats": int(row.get("query_repeats", 0) or 0),
            "has_crossover": bool(_safe_float(row.get("amortized_margin_vs_dense")) > 0.0),
            "top1_dense": _safe_float(dense.get("top1")),
            "top1_routed": _safe_float(row.get("top1")),
            "top1_delta_vs_dense": _safe_float(row.get("top1_delta_vs_dense")),
            "candidate_fraction_routed": _safe_float(row.get("candidate_fraction")),
            "search_work_ratio_vs_dense": _safe_float(row.get("search_work_ratio_vs_dense")),
            "search_work_saved_fraction_vs_dense": _safe_float(row.get("search_work_saved_fraction_vs_dense")),
            "candidate_vector_bytes_proxy_dense": candidate_bytes_dense,
            "candidate_vector_bytes_proxy_routed": candidate_bytes_routed,
            "candidate_vector_bytes_saved_fraction_vs_dense": _diff(
                1.0, _ratio(candidate_bytes_routed, candidate_bytes_dense)
            ),
            "dense_offline_per_repeat_sec": _safe_float(dense.get("offline_per_repeat_sec")),
            "routed_offline_per_repeat_sec": _safe_float(row.get("offline_per_repeat_sec")),
            "offline_penalty_per_repeat_vs_dense_sec": _diff(
                _safe_float(row.get("offline_per_repeat_sec")),
                _safe_float(dense.get("offline_per_repeat_sec")),
            ),
            "dense_online_per_repeat_sec": _safe_float(dense.get("online_per_repeat_sec")),
            "routed_online_per_repeat_sec": _safe_float(row.get("online_per_repeat_sec")),
            "online_gain_per_repeat_vs_dense_sec": _diff(
                _safe_float(dense.get("online_per_repeat_sec")),
                _safe_float(row.get("online_per_repeat_sec")),
            ),
            "dense_search_per_repeat_sec": _safe_float(dense.get("retrieval_search_per_repeat_sec")),
            "routed_search_per_repeat_sec": _safe_float(row.get("retrieval_search_per_repeat_sec")),
            "search_gain_per_repeat_vs_dense_sec": _diff(
                _safe_float(dense.get("retrieval_search_per_repeat_sec")),
                _safe_float(row.get("retrieval_search_per_repeat_sec")),
            ),
            "dense_route_query_per_repeat_sec": _safe_float(dense.get("route_query_per_repeat_sec")),
            "routed_route_query_per_repeat_sec": _safe_float(row.get("route_query_per_repeat_sec")),
            "route_lookup_penalty_per_repeat_sec": _diff(
                _safe_float(row.get("route_query_per_repeat_sec")),
                _safe_float(dense.get("route_query_per_repeat_sec")),
            ),
            "dense_amortized_per_repeat_sec": _safe_float(dense.get("amortized_per_repeat_sec")),
            "routed_amortized_per_repeat_sec": _safe_float(row.get("amortized_per_repeat_sec")),
            "amortized_margin_vs_dense_sec": _safe_float(row.get("amortized_margin_vs_dense")),
            "secondary_key_count": _safe_float(row.get("secondary_key_count")),
        }
        pair["explanation"] = _pair_explanation(pair)
        pair_audits.append(pair)

    comparison_specs = list(comparison_specs or [])
    for spec in comparison_specs:
        comparison = _build_comparison(
            profiles_by_id,
            str(spec.get("label", "")),
            str(spec.get("baseline_route_id", "")),
            str(spec.get("candidate_route_id", "")),
        )
        if comparison is not None:
            comparison_audits.append(comparison)

    q04_pairs = sorted(
        [pair for pair in pair_audits if int(pair["query_repeats"]) == 4],
        key=lambda pair: int(pair["max_train"]),
    )
    q04_cross = next((pair for pair in q04_pairs if pair["has_crossover"]), None)
    q04_miss = next((pair for pair in q04_pairs if not pair["has_crossover"]), None)
    threshold_finding: Dict[str, Any] = {}
    if q04_cross and q04_miss:
        threshold_finding = {
            "q04_crossing_route_id": q04_cross["routed_route_id"],
            "q04_crossing_max_train": q04_cross["max_train"],
            "q04_crossing_amortized_margin_vs_dense_sec": q04_cross["amortized_margin_vs_dense_sec"],
            "q04_crossing_online_gain_per_repeat_vs_dense_sec": q04_cross["online_gain_per_repeat_vs_dense_sec"],
            "q04_crossing_offline_penalty_per_repeat_vs_dense_sec": q04_cross["offline_penalty_per_repeat_vs_dense_sec"],
            "q04_miss_route_id": q04_miss["routed_route_id"],
            "q04_miss_max_train": q04_miss["max_train"],
            "q04_miss_amortized_margin_vs_dense_sec": q04_miss["amortized_margin_vs_dense_sec"],
            "q04_miss_online_gain_per_repeat_vs_dense_sec": q04_miss["online_gain_per_repeat_vs_dense_sec"],
            "q04_miss_offline_penalty_per_repeat_vs_dense_sec": q04_miss["offline_penalty_per_repeat_vs_dense_sec"],
            "explanation": (
                f"Q04 crosses at train={q04_cross['max_train']} because online savings "
                f"({_fmt(q04_cross['online_gain_per_repeat_vs_dense_sec'], 3)}s/repeat) exceed the offline penalty "
                f"({_fmt(q04_cross['offline_penalty_per_repeat_vs_dense_sec'], 3)}s/repeat), "
                f"but misses at train={q04_miss['max_train']} because the offline penalty "
                f"({_fmt(q04_miss['offline_penalty_per_repeat_vs_dense_sec'], 3)}s/repeat) stays larger than the online savings "
                f"({_fmt(q04_miss['online_gain_per_repeat_vs_dense_sec'], 3)}s/repeat)."
            ),
        }

    return {
        "generated_at": dt.datetime.now().isoformat(timespec="seconds"),
        "experiment_id": str(merged_analysis.get("experiment_id", "")),
        "config": str(merged_analysis.get("config", "")),
        "source_experiments": list(merged_analysis.get("source_experiments", [])),
        "feature_dim": feature_dim,
        "bytes_per_float": bytes_per_float,
        "selected_route_ids": selected_ids,
        "route_audits": route_audits,
        "pair_audits": pair_audits,
        "comparison_audits": comparison_audits,
        "threshold_finding": threshold_finding,
    }


def build_markdown(payload: Dict[str, Any]) -> str:
    lines = [
        "# Translated Cost Accounting Audit",
        "",
        "## Source",
        f"- experiment: `{payload.get('experiment_id', '')}`",
        f"- config: `{payload.get('config', '')}`",
        f"- feature_dim: `{payload.get('feature_dim', 0)}`",
        f"- bytes_per_float: `{payload.get('bytes_per_float', 0)}`",
    ]
    for experiment_id in payload.get("source_experiments", []):
        if experiment_id:
            lines.append(f"- source_experiment: `{experiment_id}`")

    lines.extend(["", "## Pair Audits"])
    for pair in sorted(
        payload.get("pair_audits", []),
        key=lambda item: (int(item["max_train"]), int(item["query_repeats"]), str(item["routed_route_id"])),
    ):
        lines.append(
            f"- train={pair['max_train']}, q={pair['query_repeats']}: "
            f"dense={pair['dense_route_id']}, routed={pair['routed_route_id']}, "
            f"crossover={pair['has_crossover']}"
        )
        lines.append(
            "  "
            f"top1_delta={_fmt(pair['top1_delta_vs_dense'], 6)}, "
            f"cand_frac={_fmt(pair['candidate_fraction_routed'], 6)}, "
            f"search_ratio={_fmt(pair['search_work_ratio_vs_dense'], 6)}, "
            f"bytes_saved={_fmt(pair['candidate_vector_bytes_saved_fraction_vs_dense'], 6)}"
        )
        lines.append(
            "  "
            f"offline_penalty_per_repeat={_fmt(pair['offline_penalty_per_repeat_vs_dense_sec'], 3)}s, "
            f"online_gain_per_repeat={_fmt(pair['online_gain_per_repeat_vs_dense_sec'], 3)}s, "
            f"search_gain_per_repeat={_fmt(pair['search_gain_per_repeat_vs_dense_sec'], 3)}s, "
            f"route_lookup_penalty_per_repeat={_fmt(pair['route_lookup_penalty_per_repeat_sec'], 3)}s, "
            f"amortized_margin={_fmt(pair['amortized_margin_vs_dense_sec'], 3)}s"
        )
        lines.append(f"  {pair['explanation']}")

    comparison_audits = payload.get("comparison_audits", [])
    if comparison_audits:
        lines.extend(["", "## Comparison Audits"])
        for pair in comparison_audits:
            label = pair.get("label") or f"{pair['candidate_route_id']} vs {pair['baseline_route_id']}"
            lines.append(
                f"- `{label}`: baseline=`{pair['baseline_route_id']}`, candidate=`{pair['candidate_route_id']}`"
            )
            lines.append(
                "  "
                f"top1_delta={_fmt(pair['top1_delta_candidate_minus_baseline'], 6)}, "
                f"cand_frac_delta={_fmt(pair['candidate_fraction_delta'], 6)}, "
                f"offline_penalty={_fmt(pair['offline_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"online_penalty={_fmt(pair['online_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"amortized_delta={_fmt(pair['amortized_delta_candidate_minus_baseline_sec'], 3)}s"
            )
            lines.append(
                "  "
                f"route_index_build_penalty={_fmt(pair['route_index_build_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"route_query_penalty={_fmt(pair['route_query_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"retrieval_search_penalty={_fmt(pair['retrieval_search_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"residual_penalty={_fmt(pair['residual_penalty_candidate_vs_baseline_sec'], 3)}s, "
                f"dominant={pair['dominant_penalty_component'] or 'n/a'}"
            )
            lines.append(f"  {pair['explanation']}")

    threshold_finding = payload.get("threshold_finding", {})
    if threshold_finding:
        lines.extend(["", "## Threshold Finding"])
        lines.append(
            f"- `Q04` crossing route: `{threshold_finding['q04_crossing_route_id']}` "
            f"(train={threshold_finding['q04_crossing_max_train']}, "
            f"margin={_fmt(_safe_float(threshold_finding['q04_crossing_amortized_margin_vs_dense_sec']), 3)}s)"
        )
        lines.append(
            f"- `Q04` miss route: `{threshold_finding['q04_miss_route_id']}` "
            f"(train={threshold_finding['q04_miss_max_train']}, "
            f"margin={_fmt(_safe_float(threshold_finding['q04_miss_amortized_margin_vs_dense_sec']), 3)}s)"
        )
        lines.append(f"- {threshold_finding['explanation']}")

    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit translated retrieval cost composition for selected routes.")
    ap.add_argument("--analysis", action="append", required=True, help="Analysis JSON produced by tools/proxy_sweep.py. Repeatable.")
    ap.add_argument("--meta", default=None, help="Optional proxy-meta JSON used for simple memory-traffic proxies.")
    ap.add_argument("--route-id", action="append", required=True, help="Route id to include. Repeatable.")
    ap.add_argument(
        "--compare",
        action="append",
        default=[],
        help="Explicit comparison in the form label:baseline_route_id:candidate_route_id. Repeatable.",
    )
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-md", required=True)
    args = ap.parse_args()

    analyses = []
    for path in args.analysis:
        with open(path, "r", encoding="utf-8") as f:
            analyses.append(json.load(f))
    meta = _load_meta(args.meta)
    comparison_specs: List[Dict[str, str]] = []
    for raw_spec in args.compare:
        parts = raw_spec.split(":", 2)
        if len(parts) != 3:
            raise SystemExit(
                f"invalid --compare value '{raw_spec}'; expected label:baseline_route_id:candidate_route_id"
            )
        label, baseline_route_id, candidate_route_id = parts
        comparison_specs.append(
            {
                "label": label,
                "baseline_route_id": baseline_route_id,
                "candidate_route_id": candidate_route_id,
            }
        )

    payload = build_payload(analyses, route_ids=args.route_id, meta=meta, comparison_specs=comparison_specs)
    output_json = Path(args.output_json)
    output_md = Path(args.output_md)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    output_md.write_text(build_markdown(payload) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
