#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from tools import translated_cost_accounting as accounting


EPS = 1e-9
DEFAULT_NEGLIGIBLE_GAP = 0.002


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


def _parse_compare(value: str) -> Dict[str, str]:
    label, baseline_route_id, candidate_route_id = value.split(":", 2)
    return {
        "label": label.strip(),
        "baseline_route_id": baseline_route_id.strip(),
        "candidate_route_id": candidate_route_id.strip(),
    }


def _runs_by_seed(analysis: Dict[str, Any], route_id: str) -> Dict[int, Dict[str, Any]]:
    results = analysis.get("results", {}).get(route_id, [])
    by_seed: Dict[int, Dict[str, Any]] = {}
    for run in results:
        seed = run.get("args", {}).get("seed")
        if seed is None:
            continue
        by_seed[int(seed)] = run
    return by_seed


def _load_query_audit(path: str) -> Dict[str, np.ndarray]:
    payload = np.load(path)
    return {key: payload[key] for key in payload.files}


def _aligned_query_pairs(
    baseline_audit: Dict[str, np.ndarray],
    candidate_audit: Dict[str, np.ndarray],
) -> Dict[str, np.ndarray]:
    baseline_idx = np.asarray(baseline_audit["eval_source_idx"], dtype=np.int64)
    candidate_idx = np.asarray(candidate_audit["eval_source_idx"], dtype=np.int64)
    common, baseline_pos, candidate_pos = np.intersect1d(
        baseline_idx,
        candidate_idx,
        assume_unique=False,
        return_indices=True,
    )
    return {
        "eval_source_idx": common,
        "baseline_pos": baseline_pos,
        "candidate_pos": candidate_pos,
    }


def _seed_gap_entry(seed: int, baseline_run: Dict[str, Any], candidate_run: Dict[str, Any]) -> Dict[str, Any]:
    baseline_artifacts = baseline_run.get("artifacts", {})
    candidate_artifacts = candidate_run.get("artifacts", {})
    baseline_path = str(baseline_artifacts.get("retrieval_query_audit_file", ""))
    candidate_path = str(candidate_artifacts.get("retrieval_query_audit_file", ""))
    if not baseline_path or not candidate_path:
        raise ValueError(f"missing retrieval_query_audit_file for seed {seed}")
    baseline_audit = _load_query_audit(baseline_path)
    candidate_audit = _load_query_audit(candidate_path)
    aligned = _aligned_query_pairs(baseline_audit, candidate_audit)
    baseline_pos = aligned["baseline_pos"]
    candidate_pos = aligned["candidate_pos"]

    baseline_correct = np.asarray(baseline_audit["correct"], dtype=np.int8)[baseline_pos] == 1
    candidate_correct = np.asarray(candidate_audit["correct"], dtype=np.int8)[candidate_pos] == 1
    candidate_target_present = np.asarray(candidate_audit["target_present"], dtype=np.int8)[candidate_pos] == 1
    candidate_topk_target_present = np.asarray(candidate_audit["topk_target_present"], dtype=np.int8)[candidate_pos] == 1

    dense_only = baseline_correct & (~candidate_correct)
    candidate_only = (~baseline_correct) & candidate_correct
    shared_correct = baseline_correct & candidate_correct
    shared_error = (~baseline_correct) & (~candidate_correct)

    omission = dense_only & (~candidate_target_present)
    present_outside_topk = dense_only & candidate_target_present & (~candidate_topk_target_present)
    present_inside_topk_not_top1 = dense_only & candidate_topk_target_present & (~candidate_correct)
    present_but_not_top1 = dense_only & candidate_target_present & (~candidate_correct)

    total_queries = int(aligned["eval_source_idx"].shape[0])
    dense_only_count = int(np.sum(dense_only))
    candidate_only_count = int(np.sum(candidate_only))
    net_dense_advantage_rate = float((dense_only_count - candidate_only_count) / max(1, total_queries))
    dense_only_gap_rate = float(dense_only_count / max(1, total_queries))
    return {
        "seed": int(seed),
        "joined_queries": total_queries,
        "shared_correct_count": int(np.sum(shared_correct)),
        "shared_error_count": int(np.sum(shared_error)),
        "dense_only_win_count": dense_only_count,
        "candidate_only_win_count": candidate_only_count,
        "dense_only_gap_rate": dense_only_gap_rate,
        "net_dense_advantage_rate": net_dense_advantage_rate,
        "omission_count": int(np.sum(omission)),
        "present_but_not_top1_count": int(np.sum(present_but_not_top1)),
        "present_outside_topk_count": int(np.sum(present_outside_topk)),
        "present_inside_topk_not_top1_count": int(np.sum(present_inside_topk_not_top1)),
        "omission_rate_within_gap": float(np.sum(omission) / max(1, dense_only_count)),
        "present_but_not_top1_rate_within_gap": float(np.sum(present_but_not_top1) / max(1, dense_only_count)),
        "present_outside_topk_rate_within_gap": float(np.sum(present_outside_topk) / max(1, dense_only_count)),
        "present_inside_topk_not_top1_rate_within_gap": float(np.sum(present_inside_topk_not_top1) / max(1, dense_only_count)),
        "baseline_audit_file": baseline_path,
        "candidate_audit_file": candidate_path,
    }


def _mean(entries: Iterable[float]) -> float:
    values = [float(value) for value in entries if math.isfinite(float(value))]
    if not values:
        return float("nan")
    return float(sum(values) / len(values))


def _classify_next_bias(seed_entries: List[Dict[str, Any]], negligible_gap_threshold: float) -> str:
    net_dense_advantage_rate = _mean(entry["net_dense_advantage_rate"] for entry in seed_entries)
    omission_rate = _mean(entry["omission_rate_within_gap"] for entry in seed_entries)
    present_rate = _mean(entry["present_but_not_top1_rate_within_gap"] for entry in seed_entries)
    if math.isfinite(net_dense_advantage_rate) and net_dense_advantage_rate <= negligible_gap_threshold + EPS:
        return "operationally_negligible"
    if omission_rate >= present_rate:
        return "candidate_recovery"
    return "in_candidate_ordering"


def _interpretation(next_bias: str, seed_entries: List[Dict[str, Any]], negligible_gap_threshold: float) -> str:
    net_dense_advantage_rate = _mean(entry["net_dense_advantage_rate"] for entry in seed_entries)
    omission_rate = _mean(entry["omission_rate_within_gap"] for entry in seed_entries)
    present_rate = _mean(entry["present_but_not_top1_rate_within_gap"] for entry in seed_entries)
    present_inside_topk_rate = _mean(entry["present_inside_topk_not_top1_rate_within_gap"] for entry in seed_entries)
    if next_bias == "operationally_negligible":
        return (
            f"The net dense advantage rate stays within the configured operational-negligibility band "
            f"({_fmt(net_dense_advantage_rate)} <= {_fmt(negligible_gap_threshold)}), so the residual gap is too small "
            "to justify another rescue branch."
        )
    if next_bias == "candidate_recovery":
        return (
            f"Dense-only wins are omission-led on average: omission explains {_fmt(omission_rate)} of the routed gap, "
            f"while present-but-not-top1 explains {_fmt(present_rate)}. If we reopen rescue, it should target candidate recovery."
        )
    return (
        f"Dense-only wins are mostly inside the routed candidate set: present-but-not-top1 explains {_fmt(present_rate)} "
        f"of the routed gap, with {_fmt(present_inside_topk_rate)} already inside routed top-k. If we reopen rescue, "
        "it should target in-candidate ordering rather than candidate expansion."
    )


def build_payload(
    analyses: Iterable[Dict[str, Any]],
    comparison_specs: List[Dict[str, str]],
    negligible_gap_threshold: float = DEFAULT_NEGLIGIBLE_GAP,
) -> Dict[str, Any]:
    merged = accounting._merge_analyses(list(analyses))
    comparisons: List[Dict[str, Any]] = []
    for spec in comparison_specs:
        baseline_runs = _runs_by_seed(merged, spec["baseline_route_id"])
        candidate_runs = _runs_by_seed(merged, spec["candidate_route_id"])
        common_seeds = sorted(set(baseline_runs) & set(candidate_runs))
        seed_entries = [
            _seed_gap_entry(seed, baseline_runs[seed], candidate_runs[seed])
            for seed in common_seeds
        ]
        next_bias = _classify_next_bias(seed_entries, negligible_gap_threshold=negligible_gap_threshold)
        comparisons.append(
            {
                "label": spec["label"],
                "baseline_route_id": spec["baseline_route_id"],
                "candidate_route_id": spec["candidate_route_id"],
                "seed_count": len(seed_entries),
                "seed_entries": seed_entries,
                "mean_dense_only_gap_rate": _mean(entry["dense_only_gap_rate"] for entry in seed_entries),
                "mean_net_dense_advantage_rate": _mean(entry["net_dense_advantage_rate"] for entry in seed_entries),
                "mean_omission_rate_within_gap": _mean(entry["omission_rate_within_gap"] for entry in seed_entries),
                "mean_present_but_not_top1_rate_within_gap": _mean(
                    entry["present_but_not_top1_rate_within_gap"] for entry in seed_entries
                ),
                "mean_present_outside_topk_rate_within_gap": _mean(
                    entry["present_outside_topk_rate_within_gap"] for entry in seed_entries
                ),
                "mean_present_inside_topk_not_top1_rate_within_gap": _mean(
                    entry["present_inside_topk_not_top1_rate_within_gap"] for entry in seed_entries
                ),
                "next_branch_bias": next_bias,
                "interpretation": _interpretation(
                    next_bias,
                    seed_entries,
                    negligible_gap_threshold=negligible_gap_threshold,
                ),
            }
        )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "negligible_gap_threshold": negligible_gap_threshold,
        "experiment_id": merged.get("experiment_id", ""),
        "config": merged.get("config", ""),
        "source_experiments": merged.get("source_experiments", []),
        "source_configs": merged.get("source_configs", []),
        "comparisons": comparisons,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Operational-negligibility threshold: `{_fmt(payload['negligible_gap_threshold'])}`",
        "",
        "## Comparisons",
    ]
    for comparison in payload["comparisons"]:
        lines.extend(
            [
                f"### {comparison['label']}",
                f"- Baseline: `{comparison['baseline_route_id']}`",
                f"- Candidate: `{comparison['candidate_route_id']}`",
                f"- Mean dense-only gap rate: `{_fmt(comparison['mean_dense_only_gap_rate'])}`",
                f"- Mean net dense advantage rate: `{_fmt(comparison['mean_net_dense_advantage_rate'])}`",
                f"- Mean omission share within dense-only wins: `{_fmt(comparison['mean_omission_rate_within_gap'])}`",
                f"- Mean present-but-not-top1 share within dense-only wins: `{_fmt(comparison['mean_present_but_not_top1_rate_within_gap'])}`",
                f"- Mean present-outside-topk share within dense-only wins: `{_fmt(comparison['mean_present_outside_topk_rate_within_gap'])}`",
                f"- Mean present-inside-topk-not-top1 share within dense-only wins: `{_fmt(comparison['mean_present_inside_topk_not_top1_rate_within_gap'])}`",
                f"- Next-branch bias: `{comparison['next_branch_bias']}`",
                f"- Read: {comparison['interpretation']}",
                "",
            ]
        )

    lines.append("## Seed Detail")
    for comparison in payload["comparisons"]:
        lines.append(f"### {comparison['label']}")
        for entry in comparison["seed_entries"]:
            lines.append(
                "- "
                f"seed{entry['seed']}: "
                f"dense_only={entry['dense_only_win_count']}, "
                f"candidate_only={entry['candidate_only_win_count']}, "
                f"omission={entry['omission_count']}, "
                f"present_not_top1={entry['present_but_not_top1_count']}, "
                f"present_outside_topk={entry['present_outside_topk_count']}, "
                f"present_inside_topk_not_top1={entry['present_inside_topk_not_top1_count']}"
            )
        lines.append("")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--analysis", action="append", required=True)
    ap.add_argument("--compare", action="append", required=True)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Translated Dense Gap Decomposition")
    ap.add_argument("--negligible-gap-threshold", type=float, default=DEFAULT_NEGLIGIBLE_GAP)
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    analyses = [_load_json(path) for path in args.analysis]
    specs = [_parse_compare(value) for value in args.compare]
    payload = build_payload(
        analyses,
        specs,
        negligible_gap_threshold=float(args.negligible_gap_threshold),
    )
    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_report(payload, Path(args.output_report), title=args.report_title)


if __name__ == "__main__":
    main()
