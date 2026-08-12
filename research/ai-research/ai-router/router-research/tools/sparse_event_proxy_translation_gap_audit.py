#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


DEFAULT_TOP1_TOLERANCE = 0.0001
DEFAULT_CANDIDATE_FRACTION_TOLERANCE = 1e-9
DEFAULT_RUNTIME_TOLERANCE_SEC = 0.01


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


def _proxy_metrics(row: Dict[str, Any]) -> Dict[str, float]:
    return {
        "mse": _safe_float(row.get("mean_test_mse_after")),
        "total_sec": _safe_float(row.get("mean_total_sec")),
        "event_gate_mean": _safe_float(row.get("mean_event_gate_mean")),
        "event_gate_active_frac": _safe_float(row.get("mean_event_gate_active_frac")),
        "shell_pmax": _safe_float(row.get("mean_shell_pmax")),
        "unseen_rate": _safe_float(row.get("mean_unseen_rate")),
    }


def _translated_metrics(row: Dict[str, Any]) -> Dict[str, float]:
    return {
        "top1": _safe_float(row.get("mean_test_top1_after")),
        "candidate_fraction": _safe_float(row.get("mean_retrieval_candidate_fraction")),
        "online_per_repeat_sec": _safe_float(row.get("mean_online_total_per_repeat_sec")),
        "amortized_per_repeat_sec": _safe_float(row.get("mean_amortized_total_per_repeat_sec")),
        "event_gate_mean": _safe_float(row.get("mean_event_gate_mean")),
        "event_gate_active_frac": _safe_float(row.get("mean_event_gate_active_frac")),
        "query_route_sec": _safe_float(row.get("mean_query_route_sec")),
        "route_index_build_sec": _safe_float(row.get("mean_route_index_build_sec")),
        "retrieval_search_sec": _safe_float(row.get("mean_retrieval_search_sec")),
    }


def _delta(candidate: Dict[str, float], baseline: Dict[str, float]) -> Dict[str, float]:
    return {key: candidate[key] - baseline[key] for key in candidate.keys()}


def _component_shares(delta_metrics: Dict[str, float]) -> Dict[str, float]:
    online_delta = delta_metrics["online_per_repeat_sec"]
    if not math.isfinite(online_delta) or online_delta <= 0:
        return {
            "query_route_share": 0.0,
            "route_index_build_share": 0.0,
            "retrieval_search_share": 0.0,
        }
    return {
        "query_route_share": max(0.0, delta_metrics["query_route_sec"]) / online_delta,
        "route_index_build_share": max(0.0, delta_metrics["route_index_build_sec"]) / online_delta,
        "retrieval_search_share": max(0.0, delta_metrics["retrieval_search_sec"]) / online_delta,
    }


def _primary_driver(delta_metrics: Dict[str, float]) -> str:
    candidates = {
        "query_route_sec": delta_metrics["query_route_sec"],
        "route_index_build_sec": delta_metrics["route_index_build_sec"],
        "retrieval_search_sec": delta_metrics["retrieval_search_sec"],
    }
    return max(candidates.items(), key=lambda item: item[1])[0]


def build_payload(
    proxy_analysis: Dict[str, Any],
    soft_translated_analysis: Dict[str, Any],
    near_hard_translated_analysis: Dict[str, Any],
    *,
    proxy_continuous_route_id: str = "H4XH4_FIELD_A150",
    proxy_soft_route_id: str = "H4XH4_FIELD_A150_EVT_T070",
    proxy_near_hard_route_id: str = "H4XH4_FIELD_A150_EVT_T070_TAU002",
    proxy_hard_route_id: str = "H4XH4_FIELD_A150_HARD_T062",
    translated_dense_route_id: str = "DENSE_Q01_T2500",
    translated_continuous_route_id: str = "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
    translated_soft_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
    translated_near_hard_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
    top1_tolerance: float = DEFAULT_TOP1_TOLERANCE,
    candidate_fraction_tolerance: float = DEFAULT_CANDIDATE_FRACTION_TOLERANCE,
    runtime_tolerance_sec: float = DEFAULT_RUNTIME_TOLERANCE_SEC,
) -> Dict[str, Any]:
    proxy_cont = _proxy_metrics(_route_by_id(proxy_analysis, proxy_continuous_route_id))
    proxy_soft = _proxy_metrics(_route_by_id(proxy_analysis, proxy_soft_route_id))
    proxy_near = _proxy_metrics(_route_by_id(proxy_analysis, proxy_near_hard_route_id))
    proxy_hard = _proxy_metrics(_route_by_id(proxy_analysis, proxy_hard_route_id))

    translated_dense = _translated_metrics(_route_by_id(near_hard_translated_analysis, translated_dense_route_id))
    translated_cont = _translated_metrics(_route_by_id(near_hard_translated_analysis, translated_continuous_route_id))
    translated_soft = _translated_metrics(_route_by_id(near_hard_translated_analysis, translated_soft_route_id))
    translated_near = _translated_metrics(_route_by_id(near_hard_translated_analysis, translated_near_hard_route_id))
    translated_soft_confirm = _translated_metrics(_route_by_id(soft_translated_analysis, translated_soft_route_id))

    proxy_near_vs_cont = _delta(proxy_near, proxy_cont)
    proxy_near_vs_soft = _delta(proxy_near, proxy_soft)
    proxy_hard_vs_near = _delta(proxy_hard, proxy_near)

    translated_near_vs_cont = _delta(translated_near, translated_cont)
    translated_near_vs_soft = _delta(translated_near, translated_soft)
    translated_soft_confirm_vs_cont = _delta(translated_soft_confirm, _translated_metrics(_route_by_id(soft_translated_analysis, translated_continuous_route_id)))

    quality_preserved_vs_soft = (
        abs(translated_near_vs_soft["top1"]) <= top1_tolerance
        and abs(translated_near_vs_soft["candidate_fraction"]) <= candidate_fraction_tolerance
    )
    quality_preserved_vs_cont = (
        abs(translated_near_vs_cont["top1"]) <= top1_tolerance
        and abs(translated_near_vs_cont["candidate_fraction"]) <= candidate_fraction_tolerance
    )

    omission_supported = (
        translated_near["candidate_fraction"] + candidate_fraction_tolerance < translated_soft["candidate_fraction"]
        and translated_near["top1"] + top1_tolerance < translated_soft["top1"]
    )
    ordering_loss_supported = (
        translated_near["top1"] + top1_tolerance < translated_soft["top1"]
        and abs(translated_near["candidate_fraction"] - translated_soft["candidate_fraction"]) <= candidate_fraction_tolerance
    )
    route_search_interaction_cost_supported = (
        translated_near_vs_soft["online_per_repeat_sec"] > runtime_tolerance_sec
        and (
            translated_near_vs_soft["query_route_sec"] > 0
            or translated_near_vs_soft["route_index_build_sec"] > 0
            or translated_near_vs_soft["retrieval_search_sec"] > 0
        )
    )
    event_gate_overhead_supported = (
        proxy_near["event_gate_mean"] + 1e-6 < proxy_soft["event_gate_mean"]
        and translated_near["event_gate_mean"] + 1e-6 < translated_soft["event_gate_mean"]
        and route_search_interaction_cost_supported
        and quality_preserved_vs_soft
    )

    if quality_preserved_vs_soft and route_search_interaction_cost_supported:
        branch_outcome = "translated_systems_cost_branch"
        interpretation = (
            "The near-hard controller survives as a sparse and healthy proxy mechanism, and it keeps the same "
            "translated top-1 plus candidate fraction as the translated soft-sparse and continuous references. "
            "The translated failure is therefore a systems-cost problem, not a candidate-quality or trainability problem."
        )
    elif omission_supported:
        branch_outcome = "translated_candidate_recovery_branch"
        interpretation = (
            "The translated near-hard candidate appears to lose quality alongside a smaller candidate set, so the next "
            "reopening surface would be candidate recovery rather than systems-cost rescue."
        )
    elif ordering_loss_supported:
        branch_outcome = "translated_ordering_branch"
        interpretation = (
            "The translated near-hard candidate keeps roughly the same candidate set but loses top-1, so the next "
            "reopening surface would be in-candidate ordering."
        )
    else:
        branch_outcome = "proxy_only_mechanism_result"
        interpretation = (
            "The available evidence is not strong enough to justify reopening translated sparse-event work, so the "
            "near-hard result should stay proxy-only for now."
        )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "sources": {
            "proxy_analysis": proxy_analysis.get("experiment_id", ""),
            "soft_translated_analysis": soft_translated_analysis.get("experiment_id", ""),
            "near_hard_translated_analysis": near_hard_translated_analysis.get("experiment_id", ""),
        },
        "proxy_reference_routes": {
            "continuous": {"route_id": proxy_continuous_route_id, "metrics": proxy_cont},
            "soft_sparse": {"route_id": proxy_soft_route_id, "metrics": proxy_soft},
            "near_hard": {"route_id": proxy_near_hard_route_id, "metrics": proxy_near},
            "hard": {"route_id": proxy_hard_route_id, "metrics": proxy_hard},
        },
        "translated_reference_routes": {
            "dense": {"route_id": translated_dense_route_id, "metrics": translated_dense},
            "continuous": {"route_id": translated_continuous_route_id, "metrics": translated_cont},
            "soft_sparse_screen_context": {"route_id": translated_soft_route_id, "metrics": translated_soft},
            "soft_sparse_confirm_reference": {"route_id": translated_soft_route_id, "metrics": translated_soft_confirm},
            "near_hard": {"route_id": translated_near_hard_route_id, "metrics": translated_near},
        },
        "proxy_deltas": {
            "near_hard_vs_continuous": proxy_near_vs_cont,
            "near_hard_vs_soft_sparse": proxy_near_vs_soft,
            "hard_vs_near_hard": proxy_hard_vs_near,
        },
        "translated_deltas": {
            "near_hard_vs_continuous": translated_near_vs_cont,
            "near_hard_vs_soft_sparse": translated_near_vs_soft,
            "soft_sparse_confirm_vs_continuous": translated_soft_confirm_vs_cont,
        },
        "translated_near_hard_component_shares_vs_soft_sparse": _component_shares(translated_near_vs_soft),
        "translated_near_hard_primary_runtime_driver_vs_soft_sparse": _primary_driver(translated_near_vs_soft),
        "quality_preserved_vs_soft_sparse": quality_preserved_vs_soft,
        "quality_preserved_vs_continuous": quality_preserved_vs_cont,
        "candidate_omission_supported": omission_supported,
        "in_candidate_ordering_loss_supported": ordering_loss_supported,
        "event_gate_overhead_supported": event_gate_overhead_supported,
        "route_search_interaction_cost_supported": route_search_interaction_cost_supported,
        "branch_outcome": branch_outcome,
        "interpretation": interpretation,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    proxy = payload["proxy_reference_routes"]
    translated = payload["translated_reference_routes"]
    delta_soft = payload["translated_deltas"]["near_hard_vs_soft_sparse"]
    delta_cont = payload["translated_deltas"]["near_hard_vs_continuous"]
    shares = payload["translated_near_hard_component_shares_vs_soft_sparse"]

    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Outcome: `{payload['branch_outcome']}`",
        f"- Quality preserved vs translated soft sparse: `{payload['quality_preserved_vs_soft_sparse']}`",
        f"- Candidate omission supported: `{payload['candidate_omission_supported']}`",
        f"- In-candidate ordering loss supported: `{payload['in_candidate_ordering_loss_supported']}`",
        f"- Event-gate overhead supported: `{payload['event_gate_overhead_supported']}`",
        f"- Route/search interaction cost supported: `{payload['route_search_interaction_cost_supported']}`",
        "",
        "## Proxy Read",
        f"- Near-hard proxy `{proxy['near_hard']['route_id']}`: `mse={_fmt(proxy['near_hard']['metrics']['mse'])}`, "
        f"`total_sec={_fmt(proxy['near_hard']['metrics']['total_sec'], 3)}`, "
        f"`event_gate_mean={_fmt(proxy['near_hard']['metrics']['event_gate_mean'])}`, "
        f"`event_gate_active_frac={_fmt(proxy['near_hard']['metrics']['event_gate_active_frac'])}`",
        f"- Soft sparse proxy `{proxy['soft_sparse']['route_id']}`: `mse={_fmt(proxy['soft_sparse']['metrics']['mse'])}`, "
        f"`total_sec={_fmt(proxy['soft_sparse']['metrics']['total_sec'], 3)}`, "
        f"`event_gate_mean={_fmt(proxy['soft_sparse']['metrics']['event_gate_mean'])}`, "
        f"`event_gate_active_frac={_fmt(proxy['soft_sparse']['metrics']['event_gate_active_frac'])}`",
        "",
        "## Translated Read",
        f"- Near-hard translated `{translated['near_hard']['route_id']}`: "
        f"`top1={_fmt(translated['near_hard']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(translated['near_hard']['metrics']['candidate_fraction'])}`, "
        f"`online={_fmt(translated['near_hard']['metrics']['online_per_repeat_sec'])}`, "
        f"`amortized={_fmt(translated['near_hard']['metrics']['amortized_per_repeat_sec'])}`",
        f"- Soft sparse translated `{translated['soft_sparse_screen_context']['route_id']}`: "
        f"`top1={_fmt(translated['soft_sparse_screen_context']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(translated['soft_sparse_screen_context']['metrics']['candidate_fraction'])}`, "
        f"`online={_fmt(translated['soft_sparse_screen_context']['metrics']['online_per_repeat_sec'])}`, "
        f"`amortized={_fmt(translated['soft_sparse_screen_context']['metrics']['amortized_per_repeat_sec'])}`",
        f"- Continuous translated `{translated['continuous']['route_id']}`: "
        f"`top1={_fmt(translated['continuous']['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(translated['continuous']['metrics']['candidate_fraction'])}`, "
        f"`online={_fmt(translated['continuous']['metrics']['online_per_repeat_sec'])}`, "
        f"`amortized={_fmt(translated['continuous']['metrics']['amortized_per_repeat_sec'])}`",
        "",
        "## Gap Attribution",
        f"- Near-hard vs soft sparse top-1 delta: `{_fmt(delta_soft['top1'])}`",
        f"- Near-hard vs soft sparse candidate-fraction delta: `{_fmt(delta_soft['candidate_fraction'])}`",
        f"- Near-hard vs soft sparse online delta: `{_fmt(delta_soft['online_per_repeat_sec'])}`",
        f"- Near-hard vs soft sparse amortized delta: `{_fmt(delta_soft['amortized_per_repeat_sec'])}`",
        f"- Near-hard vs soft sparse route-query delta: `{_fmt(delta_soft['query_route_sec'])}`",
        f"- Near-hard vs soft sparse route-index-build delta: `{_fmt(delta_soft['route_index_build_sec'])}`",
        f"- Near-hard vs soft sparse retrieval-search delta: `{_fmt(delta_soft['retrieval_search_sec'])}`",
        f"- Primary runtime driver: `{payload['translated_near_hard_primary_runtime_driver_vs_soft_sparse']}`",
        f"- Runtime shares vs soft sparse: query `{_fmt(shares['query_route_share'])}`, "
        f"index `{_fmt(shares['route_index_build_share'])}`, search `{_fmt(shares['retrieval_search_share'])}`",
        f"- Near-hard vs continuous top-1 delta: `{_fmt(delta_cont['top1'])}`",
        f"- Near-hard vs continuous candidate-fraction delta: `{_fmt(delta_cont['candidate_fraction'])}`",
        f"- Near-hard vs continuous online delta: `{_fmt(delta_cont['online_per_repeat_sec'])}`",
        "",
        "## Interpretation",
        f"- {payload['interpretation']}",
    ]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Audit the gap between sparse-event proxy hardening and translated near-hard failure.")
    parser.add_argument(
        "--proxy-analysis",
        default="results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json",
    )
    parser.add_argument(
        "--soft-translated-analysis",
        default="results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json",
    )
    parser.add_argument(
        "--near-hard-translated-analysis",
        default="results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json",
    )
    parser.add_argument(
        "--output-json",
        default="results/analysis/inc0126_product_phase_sparse_event_proxy_translation_gap_audit.json",
    )
    parser.add_argument(
        "--output-report",
        default="docs/reports/INC0126_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRANSLATION_GAP_AUDIT.md",
    )
    args = parser.parse_args()

    payload = build_payload(
        _load_json(args.proxy_analysis),
        _load_json(args.soft_translated_analysis),
        _load_json(args.near_hard_translated_analysis),
    )

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_report(
        payload,
        Path(args.output_report),
        "INC0126 Product Phase Sparse Event Proxy Translation Gap Audit",
    )


if __name__ == "__main__":
    main()
