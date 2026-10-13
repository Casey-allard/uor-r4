#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import re
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


def _safe_float(value: Any) -> float:
    if value is None:
        return float("nan")
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


def _ratio(num: float, den: float) -> float:
    if not math.isfinite(num) or not math.isfinite(den) or abs(den) <= 1e-12:
        return float("nan")
    return float(num / den)


def _diff(lhs: float, rhs: float) -> float:
    if not math.isfinite(lhs) or not math.isfinite(rhs):
        return float("nan")
    return float(lhs - rhs)


def _route_args(analysis: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    results = analysis.get("results", {})
    route_runs = results.get(route_id, [])
    if not route_runs:
        return {}
    args = route_runs[0].get("args", {})
    return args if isinstance(args, dict) else {}


def flatten_analysis(analysis: Dict[str, Any], source_label: Optional[str] = None) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for route_stat in analysis.get("route_stats", []):
        route_id = str(route_stat["route_id"])
        args = _route_args(analysis, route_id)
        max_train = int(args.get("max_train", 0) or 0)
        query_repeats = int(args.get("query_repeats", 0) or 0)
        retrieval_backend = str(args.get("retrieval_backend", ""))
        sector_mode = str(args.get("sector_mode", ""))
        route_key_mode = str(args.get("route_key_mode", ""))
        candidate_count = _safe_float(route_stat.get("mean_retrieval_candidate_count"))
        candidate_fraction = _safe_float(route_stat.get("mean_retrieval_candidate_fraction"))
        total_sec = _safe_float(route_stat.get("mean_total_sec"))
        offline_sec = _safe_float(route_stat.get("mean_offline_total_sec"))
        online_sec = _safe_float(route_stat.get("mean_online_total_sec"))
        online_per_repeat = _safe_float(route_stat.get("mean_online_total_per_repeat_sec"))
        amortized = _safe_float(route_stat.get("mean_amortized_total_per_repeat_sec"))
        top1 = _safe_float(route_stat.get("mean_test_top1_after"))

        rows.append(
            {
                "source_label": source_label or str(analysis.get("experiment_id", "analysis")),
                "experiment_id": str(analysis.get("experiment_id", "")),
                "config": str(analysis.get("config", "")),
                "route_id": route_id,
                "max_train": max_train,
                "query_repeats": query_repeats,
                "retrieval_backend": retrieval_backend,
                "sector_mode": sector_mode,
                "route_key_mode": route_key_mode,
                "is_dense": retrieval_backend == "dense_exact" or route_id.startswith("DENSE_"),
                "top1": top1,
                "candidate_count": candidate_count,
                "candidate_fraction": candidate_fraction,
                "total_sec": total_sec,
                "offline_sec": offline_sec,
                "online_sec": online_sec,
                "online_per_repeat_sec": online_per_repeat,
                "amortized_per_repeat_sec": amortized,
                "fallback_fraction": _safe_float(route_stat.get("mean_retrieval_bucket_fallback_rate")),
                "secondary_key_count": _safe_float(route_stat.get("mean_retrieval_secondary_key_count")),
                "n_runs": int(route_stat.get("n_runs", 0) or 0),
            }
        )
    return rows


def derive_group_profiles(rows: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    grouped: Dict[Tuple[int, int], List[Dict[str, Any]]] = {}
    for row in rows:
        key = (int(row["max_train"]), int(row["query_repeats"]))
        grouped.setdefault(key, []).append(dict(row))

    profiles: List[Dict[str, Any]] = []
    for key, group_rows in sorted(grouped.items()):
        dense = next((row for row in group_rows if row["is_dense"]), None)
        dense_candidate_count = _safe_float(dense.get("candidate_count")) if dense else float("nan")
        dense_online = _safe_float(dense.get("online_per_repeat_sec")) if dense else float("nan")
        dense_amortized = _safe_float(dense.get("amortized_per_repeat_sec")) if dense else float("nan")
        dense_top1 = _safe_float(dense.get("top1")) if dense else float("nan")

        for row in group_rows:
            row["dense_route_id"] = dense["route_id"] if dense else ""
            row["search_work_ratio_vs_dense"] = _ratio(_safe_float(row["candidate_count"]), dense_candidate_count)
            row["search_work_saved_fraction_vs_dense"] = _diff(1.0, row["search_work_ratio_vs_dense"])
            row["online_speedup_vs_dense"] = _ratio(dense_online, _safe_float(row["online_per_repeat_sec"]))
            row["amortized_speedup_vs_dense"] = _ratio(dense_amortized, _safe_float(row["amortized_per_repeat_sec"]))
            row["amortized_margin_vs_dense"] = _diff(dense_amortized, _safe_float(row["amortized_per_repeat_sec"]))
            row["top1_delta_vs_dense"] = _diff(_safe_float(row["top1"]), dense_top1)
            row["offline_share"] = _ratio(_safe_float(row["offline_sec"]), _safe_float(row["total_sec"]))
            row["online_share"] = _ratio(_safe_float(row["online_sec"]), _safe_float(row["total_sec"]))
            row["is_crossover_vs_dense"] = bool(
                row["route_id"] != row.get("dense_route_id", "")
                and math.isfinite(_safe_float(row["amortized_margin_vs_dense"]))
                and _safe_float(row["amortized_margin_vs_dense"]) > 0.0
            )
            profiles.append(row)
    return profiles


def summarize_groups(profiles: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    grouped: Dict[Tuple[int, int], List[Dict[str, Any]]] = {}
    for row in profiles:
        key = (int(row["max_train"]), int(row["query_repeats"]))
        grouped.setdefault(key, []).append(row)

    summaries: List[Dict[str, Any]] = []
    for key, rows in sorted(grouped.items()):
        dense = next((row for row in rows if row["is_dense"]), None)
        routed = [row for row in rows if not row["is_dense"]]
        crossover = [row for row in routed if row["is_crossover_vs_dense"]]

        best_quality = None
        if crossover:
            best_quality = max(
                crossover,
                key=lambda row: (
                    _safe_float(row["top1"]),
                    _safe_float(row["amortized_margin_vs_dense"]),
                    -_safe_float(row["candidate_fraction"]),
                ),
            )
        best_systems = None
        if crossover:
            best_systems = min(
                crossover,
                key=lambda row: (
                    _safe_float(row["amortized_per_repeat_sec"]),
                    _safe_float(row["online_per_repeat_sec"]),
                    _safe_float(row["candidate_fraction"]),
                    -_safe_float(row["top1"]),
                ),
            )

        summaries.append(
            {
                "max_train": key[0],
                "query_repeats": key[1],
                "dense_route_id": dense["route_id"] if dense else "",
                "dense_top1": _safe_float(dense.get("top1")) if dense else float("nan"),
                "dense_amortized_per_repeat_sec": _safe_float(dense.get("amortized_per_repeat_sec")) if dense else float("nan"),
                "dense_candidate_count": _safe_float(dense.get("candidate_count")) if dense else float("nan"),
                "has_crossover": bool(crossover),
                "quality_matched_route_id": best_quality["route_id"] if best_quality else "",
                "quality_matched_top1_delta_vs_dense": _safe_float(best_quality.get("top1_delta_vs_dense")) if best_quality else float("nan"),
                "quality_matched_amortized_margin_vs_dense": _safe_float(best_quality.get("amortized_margin_vs_dense")) if best_quality else float("nan"),
                "systems_route_id": best_systems["route_id"] if best_systems else "",
                "systems_amortized_margin_vs_dense": _safe_float(best_systems.get("amortized_margin_vs_dense")) if best_systems else float("nan"),
                "systems_search_work_ratio_vs_dense": _safe_float(best_systems.get("search_work_ratio_vs_dense")) if best_systems else float("nan"),
                "systems_candidate_fraction": _safe_float(best_systems.get("candidate_fraction")) if best_systems else float("nan"),
            }
        )
    return summaries


def _route_family(route_id: str) -> str:
    family = re.sub(r"_Q\d+", "", route_id)
    family = re.sub(r"_T\d+", "", family)
    return family


def summarize_banks(
    group_summaries: Iterable[Dict[str, Any]], profiles: Iterable[Dict[str, Any]]
) -> List[Dict[str, Any]]:
    grouped_summaries: Dict[int, List[Dict[str, Any]]] = {}
    for summary in group_summaries:
        grouped_summaries.setdefault(int(summary["max_train"]), []).append(summary)

    grouped_profiles: Dict[int, List[Dict[str, Any]]] = {}
    for row in profiles:
        grouped_profiles.setdefault(int(row["max_train"]), []).append(row)

    bank_summaries: List[Dict[str, Any]] = []
    for max_train, summaries in sorted(grouped_summaries.items()):
        summaries = sorted(summaries, key=lambda row: int(row["query_repeats"]))
        profiles_for_bank = grouped_profiles.get(max_train, [])

        first_any = next((row for row in summaries if row["has_crossover"]), None)
        first_quality = next((row for row in summaries if row["quality_matched_route_id"]), None)
        first_systems = next((row for row in summaries if row["systems_route_id"]), None)

        systems_family = _route_family(first_systems["systems_route_id"]) if first_systems else ""
        systems_rows = sorted(
            (
                row
                for row in profiles_for_bank
                if not row["is_dense"] and _route_family(str(row["route_id"])) == systems_family
            ),
            key=lambda row: int(row["query_repeats"]),
        )
        systems_ratio_values = [
            _safe_float(row.get("search_work_ratio_vs_dense"))
            for row in systems_rows
            if math.isfinite(_safe_float(row.get("search_work_ratio_vs_dense")))
        ]
        systems_margin_rows = [
            row
            for row in systems_rows
            if math.isfinite(_safe_float(row.get("amortized_margin_vs_dense")))
        ]
        first_margin_row = systems_margin_rows[0] if systems_margin_rows else None
        last_margin_row = systems_margin_rows[-1] if systems_margin_rows else None
        systems_margin_slope = float("nan")
        if (
            first_margin_row is not None
            and last_margin_row is not None
            and int(last_margin_row["query_repeats"]) != int(first_margin_row["query_repeats"])
        ):
            systems_margin_slope = _ratio(
                _safe_float(last_margin_row.get("amortized_margin_vs_dense"))
                - _safe_float(first_margin_row.get("amortized_margin_vs_dense")),
                int(last_margin_row["query_repeats"]) - int(first_margin_row["query_repeats"]),
            )

        bank_summaries.append(
            {
                "max_train": max_train,
                "repeat_counts": [int(row["query_repeats"]) for row in summaries],
                "has_any_crossover": first_any is not None,
                "first_any_crossover_query_repeats": int(first_any["query_repeats"]) if first_any else None,
                "first_quality_crossover_query_repeats": int(first_quality["query_repeats"]) if first_quality else None,
                "first_quality_crossover_route_id": first_quality["quality_matched_route_id"] if first_quality else "",
                "first_systems_crossover_query_repeats": int(first_systems["query_repeats"]) if first_systems else None,
                "first_systems_crossover_route_id": first_systems["systems_route_id"] if first_systems else "",
                "systems_route_family": systems_family,
                "systems_search_work_ratio_min": min(systems_ratio_values) if systems_ratio_values else float("nan"),
                "systems_search_work_ratio_max": max(systems_ratio_values) if systems_ratio_values else float("nan"),
                "systems_search_work_ratio_span": (
                    max(systems_ratio_values) - min(systems_ratio_values) if systems_ratio_values else float("nan")
                ),
                "systems_margin_first_query_repeats": int(first_margin_row["query_repeats"]) if first_margin_row else None,
                "systems_margin_first": _safe_float(first_margin_row.get("amortized_margin_vs_dense")) if first_margin_row else float("nan"),
                "systems_margin_last_query_repeats": int(last_margin_row["query_repeats"]) if last_margin_row else None,
                "systems_margin_last": _safe_float(last_margin_row.get("amortized_margin_vs_dense")) if last_margin_row else float("nan"),
                "systems_margin_slope_per_repeat": systems_margin_slope,
            }
        )
    return bank_summaries


def build_payload(analyses: List[Dict[str, Any]], labels: Optional[List[str]] = None) -> Dict[str, Any]:
    all_rows: List[Dict[str, Any]] = []
    for idx, analysis in enumerate(analyses):
        label = labels[idx] if labels and idx < len(labels) else None
        all_rows.extend(flatten_analysis(analysis, source_label=label))
    profiles = derive_group_profiles(all_rows)
    group_summaries = summarize_groups(profiles)
    bank_summaries = summarize_banks(group_summaries, profiles)
    return {
        "generated_at": dt.datetime.now().isoformat(timespec="seconds"),
        "source_experiments": [str(analysis.get("experiment_id", "")) for analysis in analyses],
        "route_profiles": profiles,
        "group_summaries": group_summaries,
        "bank_summaries": bank_summaries,
    }


def _fmt(value: float, places: int = 3) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def build_markdown(payload: Dict[str, Any]) -> str:
    lines = [
        "# Translated Hardware Profile",
        "",
        "## Source Experiments",
    ]
    for experiment_id in payload.get("source_experiments", []):
        lines.append(f"- `{experiment_id}`")

    lines.extend(["", "## Group Summaries"])
    for summary in payload.get("group_summaries", []):
        lines.append(
            f"- train={summary['max_train']}, q={summary['query_repeats']}: "
            f"dense={summary['dense_route_id']}, "
            f"crossover={summary['has_crossover']}, "
            f"quality={summary['quality_matched_route_id'] or 'n/a'}, "
            f"systems={summary['systems_route_id'] or 'n/a'}"
        )
        if summary["quality_matched_route_id"]:
            lines.append(
                "  "
                f"quality_delta_top1={_fmt(summary['quality_matched_top1_delta_vs_dense'], 6)}, "
                f"quality_amortized_margin={_fmt(summary['quality_matched_amortized_margin_vs_dense'], 3)}s"
            )
        if summary["systems_route_id"]:
            lines.append(
                "  "
                f"systems_amortized_margin={_fmt(summary['systems_amortized_margin_vs_dense'], 3)}s, "
                f"systems_search_ratio={_fmt(summary['systems_search_work_ratio_vs_dense'], 3)}, "
                f"systems_cand_frac={_fmt(summary['systems_candidate_fraction'], 6)}"
            )

    lines.extend(["", "## Bank Summaries"])
    for summary in payload.get("bank_summaries", []):
        lines.append(
            f"- train={summary['max_train']}: "
            f"first_any={summary['first_any_crossover_query_repeats'] if summary['first_any_crossover_query_repeats'] is not None else 'n/a'}, "
            f"first_quality={summary['first_quality_crossover_query_repeats'] if summary['first_quality_crossover_query_repeats'] is not None else 'n/a'}, "
            f"first_systems={summary['first_systems_crossover_query_repeats'] if summary['first_systems_crossover_query_repeats'] is not None else 'n/a'}, "
            f"systems_family={summary['systems_route_family'] or 'n/a'}"
        )
        if summary["systems_route_family"]:
            lines.append(
                "  "
                f"systems_ratio_min={_fmt(summary['systems_search_work_ratio_min'], 3)}, "
                f"systems_ratio_max={_fmt(summary['systems_search_work_ratio_max'], 3)}, "
                f"systems_ratio_span={_fmt(summary['systems_search_work_ratio_span'], 3)}, "
                f"systems_margin_slope={_fmt(summary['systems_margin_slope_per_repeat'], 6)}"
            )

    lines.extend(["", "## Route Profiles"])
    for row in sorted(
        payload.get("route_profiles", []),
        key=lambda item: (int(item["max_train"]), int(item["query_repeats"]), item["route_id"]),
    ):
        lines.append(
            "- "
            f"{row['route_id']}: train={row['max_train']}, q={row['query_repeats']}, "
            f"top1={_fmt(_safe_float(row['top1']), 6)}, cand_count={_fmt(_safe_float(row['candidate_count']), 3)}, "
            f"cand_frac={_fmt(_safe_float(row['candidate_fraction']), 6)}, "
            f"work_ratio_vs_dense={_fmt(_safe_float(row['search_work_ratio_vs_dense']), 3)}, "
            f"offline_share={_fmt(_safe_float(row['offline_share']), 3)}, "
            f"online_share={_fmt(_safe_float(row['online_share']), 3)}, "
            f"online_per_repeat={_fmt(_safe_float(row['online_per_repeat_sec']), 3)}s, "
            f"amortized={_fmt(_safe_float(row['amortized_per_repeat_sec']), 3)}s, "
            f"amortized_margin_vs_dense={_fmt(_safe_float(row['amortized_margin_vs_dense']), 3)}s"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Derive hardware-facing translated retrieval profiles from sweep analyses.")
    ap.add_argument("--analysis", action="append", required=True, help="Analysis JSON produced by tools/proxy_sweep.py. Repeatable.")
    ap.add_argument("--label", action="append", default=None, help="Optional label matching each --analysis.")
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-md", required=True)
    args = ap.parse_args()

    analyses = []
    for path in args.analysis:
        with open(path, "r", encoding="utf-8") as f:
            analyses.append(json.load(f))

    payload = build_payload(analyses, labels=args.label)
    output_json = Path(args.output_json)
    output_md = Path(args.output_md)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    output_md.write_text(build_markdown(payload) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
