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

from tasks.router_retrieval_eval import event_gate_changes_retrieval_surface


EVENT_GATE_KEYS = {
    "event_gate_mode",
    "event_gate_threshold",
    "event_gate_tau",
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


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _merged_route_args(config: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    common = dict(config.get("common_args", {}))
    for route in config.get("routes", []):
        if route.get("route_id") == route_id:
            merged = dict(common)
            merged.update(route.get("args", {}))
            return merged
    raise KeyError(f"route_id not found in config: {route_id}")


def _diff_args(a: Dict[str, Any], b: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    diff: Dict[str, Dict[str, Any]] = {}
    for key in sorted(set(a) | set(b)):
        if a.get(key) != b.get(key):
            diff[key] = {
                "baseline": a.get(key),
                "candidate": b.get(key),
            }
    return diff


def _route_summary(analysis: Dict[str, Any], route_id: str) -> Dict[str, float]:
    runs = analysis.get("results", {}).get(route_id, [])
    if not runs:
        raise KeyError(f"route_id not found in analysis: {route_id}")
    keys = {
        "test_top1_after": ("metrics", "test_top1_after"),
        "retrieval_candidate_fraction_mean": ("metrics", "retrieval_candidate_fraction_mean"),
        "retrieval_online_total_per_repeat_sec": ("metrics", "retrieval_online_total_per_repeat_sec"),
        "retrieval_total_amortized_per_repeat_sec": ("metrics", "retrieval_total_amortized_per_repeat_sec"),
        "event_gate_mean": ("metrics", "event_gate_mean"),
        "event_gate_active_frac": ("metrics", "event_gate_active_frac"),
        "event_gate_retrieval_surface_active": ("metrics", "event_gate_retrieval_surface_active"),
        "query_route_sec": ("timings_sec", "query_route"),
        "route_index_build_sec": ("timings_sec", "route_index_build"),
        "retrieval_search_sec": ("timings_sec", "retrieval_search"),
    }
    summary: Dict[str, float] = {}
    for out_key, (parent_key, metric_key) in keys.items():
        values = [
            _safe_float(run.get(parent_key, {}).get(metric_key))
            for run in runs
        ]
        finite = [value for value in values if math.isfinite(value)]
        summary[out_key] = (sum(finite) / len(finite)) if finite else float("nan")
    summary["seed_count"] = float(len(runs))
    return summary


def _delta(candidate: Dict[str, float], baseline: Dict[str, float], key: str) -> float:
    return _safe_float(candidate.get(key)) - _safe_float(baseline.get(key))


def build_payload(
    translation_config: Dict[str, Any],
    translation_analysis: Dict[str, Any],
    proxy_analysis: Dict[str, Any],
    soft_route_id: str,
    near_hard_route_id: str,
    proxy_soft_route_id: str,
    proxy_near_hard_route_id: str,
) -> Dict[str, Any]:
    soft_args = _merged_route_args(translation_config, soft_route_id)
    near_hard_args = _merged_route_args(translation_config, near_hard_route_id)
    arg_diff = _diff_args(soft_args, near_hard_args)
    diff_keys = sorted(arg_diff)
    diff_is_event_gate_only = set(diff_keys).issubset(EVENT_GATE_KEYS)
    retrieval_surface_active = bool(
        event_gate_changes_retrieval_surface(argparse.Namespace(**near_hard_args))
    )

    soft_summary = _route_summary(translation_analysis, soft_route_id)
    near_hard_summary = _route_summary(translation_analysis, near_hard_route_id)
    proxy_soft_summary = _route_summary(proxy_analysis, proxy_soft_route_id)
    proxy_near_hard_summary = _route_summary(proxy_analysis, proxy_near_hard_route_id)

    deltas = {
        "top1_delta_candidate_minus_baseline": _delta(
            near_hard_summary, soft_summary, "test_top1_after"
        ),
        "candidate_fraction_delta_candidate_minus_baseline": _delta(
            near_hard_summary, soft_summary, "retrieval_candidate_fraction_mean"
        ),
        "online_delta_candidate_minus_baseline_sec": _delta(
            near_hard_summary, soft_summary, "retrieval_online_total_per_repeat_sec"
        ),
        "amortized_delta_candidate_minus_baseline_sec": _delta(
            near_hard_summary, soft_summary, "retrieval_total_amortized_per_repeat_sec"
        ),
        "route_query_delta_candidate_minus_baseline_sec": _delta(
            near_hard_summary, soft_summary, "query_route_sec"
        ),
        "route_index_build_delta_candidate_minus_baseline_sec": _delta(
            near_hard_summary, soft_summary, "route_index_build_sec"
        ),
        "retrieval_search_delta_candidate_minus_baseline_sec": _delta(
            near_hard_summary, soft_summary, "retrieval_search_sec"
        ),
        "proxy_event_gate_mean_delta_candidate_minus_baseline": _delta(
            proxy_near_hard_summary, proxy_soft_summary, "event_gate_mean"
        ),
        "proxy_event_gate_active_frac_delta_candidate_minus_baseline": _delta(
            proxy_near_hard_summary, proxy_soft_summary, "event_gate_active_frac"
        ),
    }

    rescue_surface_supported = bool(diff_is_event_gate_only and retrieval_surface_active)
    if diff_is_event_gate_only and not retrieval_surface_active:
        branch_outcome = "no_selective_retrieval_rescue_surface"
        interpretation = (
            "The translated near-hard and translated soft-sparse routes differ only "
            "in sparse-event audit knobs, and those knobs are not wired into the "
            "translated retrieval surface in the current harness. The observed "
            "runtime gap is therefore not a selective retrieval-rescue branch."
        )
    elif rescue_surface_supported:
        branch_outcome = "rescue_surface_supported"
        interpretation = (
            "Sparse-event knobs do change the translated retrieval surface, so a "
            "selective systems-cost rescue remains a valid next branch."
        )
    else:
        branch_outcome = "mixed_contract"
        interpretation = (
            "The compared routes differ on non-event-gate knobs, so this is not a "
            "clean sparse-event-only rescue surface."
        )

    return {
        "experiment_id": "inc0127_product_phase_sparse_event_translation_systems_cost_rescue",
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "translation": {
            "config_experiment_id": translation_config.get("experiment_id"),
            "analysis_experiment_id": translation_analysis.get("experiment_id"),
            "baseline_route_id": soft_route_id,
            "candidate_route_id": near_hard_route_id,
            "baseline_summary": soft_summary,
            "candidate_summary": near_hard_summary,
            "deltas_candidate_minus_baseline": deltas,
        },
        "proxy_reference": {
            "analysis_experiment_id": proxy_analysis.get("experiment_id"),
            "baseline_route_id": proxy_soft_route_id,
            "candidate_route_id": proxy_near_hard_route_id,
            "baseline_summary": proxy_soft_summary,
            "candidate_summary": proxy_near_hard_summary,
        },
        "contract_check": {
            "route_arg_differences": arg_diff,
            "route_arg_difference_keys": diff_keys,
            "difference_is_event_gate_only": diff_is_event_gate_only,
            "event_gate_retrieval_surface_active": retrieval_surface_active,
            "event_gate_retrieval_surface_reason": (
                "Current routed retrieval uses fixed routed train/eval embeddings; "
                "event-gate knobs are consumed only by sparse_event_training_audit."
            ),
            "rescue_surface_supported": rescue_surface_supported,
        },
        "branch_outcome": branch_outcome,
        "interpretation": interpretation,
        "next_honest_branch": {
            "increment": "INC-0128",
            "title": "Product Phase Sparse Event Translation Route-Coupled Pilot",
            "reason": (
                "If sparse-event translated behavior should matter downstream, the "
                "event controller must couple into the translated route or "
                "retrieval surface instead of remaining audit-only."
            ),
        },
    }


def _report(payload: Dict[str, Any]) -> str:
    contract = payload["contract_check"]
    deltas = payload["translation"]["deltas_candidate_minus_baseline"]
    return "\n".join(
        [
            "# INC-0127 Product Phase Sparse Event Translation Systems Cost Rescue",
            "",
            f"- Branch outcome: `{payload['branch_outcome']}`",
            f"- Interpretation: {payload['interpretation']}",
            "",
            "## Contract Check",
            f"- Translation route diff keys: `{', '.join(contract['route_arg_difference_keys']) or 'none'}`",
            f"- Event-gate-only difference: `{contract['difference_is_event_gate_only']}`",
            f"- Event gate changes translated retrieval surface: `{contract['event_gate_retrieval_surface_active']}`",
            f"- Reason: {contract['event_gate_retrieval_surface_reason']}",
            "",
            "## Observed Translated Deltas (Near-Hard Minus Soft Sparse)",
            f"- top-1 delta: `{_fmt(deltas['top1_delta_candidate_minus_baseline'])}`",
            f"- candidate-fraction delta: `{_fmt(deltas['candidate_fraction_delta_candidate_minus_baseline'])}`",
            f"- online delta: `{_fmt(deltas['online_delta_candidate_minus_baseline_sec'])}s`",
            f"- amortized delta: `{_fmt(deltas['amortized_delta_candidate_minus_baseline_sec'])}s`",
            f"- route-query delta: `{_fmt(deltas['route_query_delta_candidate_minus_baseline_sec'])}s`",
            f"- route-index-build delta: `{_fmt(deltas['route_index_build_delta_candidate_minus_baseline_sec'])}s`",
            f"- retrieval-search delta: `{_fmt(deltas['retrieval_search_delta_candidate_minus_baseline_sec'])}s`",
            "",
            "## Decision",
            "- Close `INC-0127` as a rescue-feasibility branch, not as a translated tuning branch.",
            "- Do not reopen candidate recovery or quality rescue from this surface.",
            "- Move next to a route-coupled sparse-event translated pilot if we want downstream sparse-event behavior to matter.",
        ]
    ) + "\n"


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--translation-config",
        type=str,
        default="/Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_screen.json",
    )
    ap.add_argument(
        "--translation-analysis",
        type=str,
        default="/Users/adminamn/ai-router/router-research/results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json",
    )
    ap.add_argument(
        "--proxy-analysis",
        type=str,
        default="/Users/adminamn/ai-router/router-research/results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json",
    )
    ap.add_argument(
        "--soft-route-id",
        type=str,
        default="CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
    )
    ap.add_argument(
        "--near-hard-route-id",
        type=str,
        default="CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
    )
    ap.add_argument(
        "--proxy-soft-route-id",
        type=str,
        default="H4XH4_FIELD_A150_EVT_T070",
    )
    ap.add_argument(
        "--proxy-near-hard-route-id",
        type=str,
        default="H4XH4_FIELD_A150_EVT_T070_TAU002",
    )
    ap.add_argument(
        "--output-json",
        type=str,
        default="/Users/adminamn/ai-router/router-research/results/analysis/inc0127_product_phase_sparse_event_translation_systems_cost_rescue.json",
    )
    ap.add_argument(
        "--output-md",
        type=str,
        default="/Users/adminamn/ai-router/router-research/docs/reports/INC0127_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SYSTEMS_COST_RESCUE.md",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        translation_config=_load_json(args.translation_config),
        translation_analysis=_load_json(args.translation_analysis),
        proxy_analysis=_load_json(args.proxy_analysis),
        soft_route_id=args.soft_route_id,
        near_hard_route_id=args.near_hard_route_id,
        proxy_soft_route_id=args.proxy_soft_route_id,
        proxy_near_hard_route_id=args.proxy_near_hard_route_id,
    )
    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    output_md = Path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_md.write_text(_report(payload), encoding="utf-8")
    print(json.dumps({
        "output_json": str(output_json),
        "output_md": str(output_md),
        "branch_outcome": payload["branch_outcome"],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
