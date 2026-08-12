#!/usr/bin/env python3
"""Guarded comparison: angular Hopf with transferred discipline vs prime transport."""

from __future__ import annotations

import csv
from pathlib import Path

from angular_hopf_benchmark_adapter import angular_route_key_for_trace_entry
from base_gap_benchmark_wrapper import ROOT, build_bounded_traces
from run_guarded_benchmark_harness_eval import evaluate_base_gap_policy, full_route_keys_for_trace
from run_mock_router_trace_eval import class_unresolved_fraction, mean


OUT_CSV = (
    ROOT
    / "results"
    / "prime_transport_recursive_system"
    / "angular_hopf_vs_prime_transport_with_transfers.csv"
)
THRESHOLD_SUMMARY = ROOT / "results" / "prime_transport_recursive_system" / "threshold_law_summary.csv"
PROMOTION_THRESHOLD = 0.30


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def angular_route_keys_for_trace(trace) -> list[str]:
    return [
        angular_route_key_for_trace_entry(
            r_depth=trace.r_depth,
            base_phase=j % 35,
            phase_tuple=trace.phases[j],
            layer_primes=trace.layer_primes,
            next_return_gap=trace.gaps[j],
        )
        for j in range(trace.trace_length)
    ]


def _with_fallback_metric(row: dict[str, object]) -> dict[str, object]:
    out = dict(row)
    out["fallback_step_fraction"] = float(row["promotion_step_fraction"])
    return out


def evaluate_angular_hopf_baseline(trace) -> dict[str, object]:
    route_keys = angular_route_keys_for_trace(trace)
    full_keys = full_route_keys_for_trace(trace)
    unresolved_by_route = class_unresolved_fraction(route_keys, full_keys)
    unique_route_keys = len(set(route_keys))
    cache_hits = trace.trace_length - unique_route_keys
    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "r_depth": trace.r_depth,
        "policy_name": "angular_hopf",
        "unique_route_keys": unique_route_keys,
        "unique_emitted_route_keys": unique_route_keys,
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": 0.0,
        "promotion_step_fraction": 0.0,
        "fallback_step_fraction": 0.0,
        "effective_resolved_fraction": 1.0 - mean(list(unresolved_by_route.values())),
        "mean_unresolved_among_nonpromoted_routes": mean(list(unresolved_by_route.values())),
        "route_decision_instability": 0.0,
    }


def evaluate_angular_hopf_guarded(trace) -> dict[str, object]:
    route_keys = angular_route_keys_for_trace(trace)
    full_keys = full_route_keys_for_trace(trace)
    unresolved_by_route = class_unresolved_fraction(route_keys, full_keys)

    route_table: dict[str, bool] = {}
    cache_hits = 0
    promoted_steps = 0
    emitted_keys: set[str] = set()
    route_decisions: dict[str, bool] = {}
    resolved_scores: list[float] = []

    for j, route_key in enumerate(route_keys):
        if route_key in route_table:
            cache_hits += 1
        else:
            route_table[route_key] = float(unresolved_by_route[route_key]) > PROMOTION_THRESHOLD
        promote = route_table[route_key]
        if route_key in route_decisions and route_decisions[route_key] != promote:
            raise RuntimeError(f"decision instability in angular_hopf_guarded for {route_key}")
        route_decisions[route_key] = promote
        if promote:
            promoted_steps += 1
            emitted_keys.add(full_keys[j])
            resolved_scores.append(1.0)
        else:
            emitted_keys.add(route_key)
            resolved_scores.append(1.0 - unresolved_by_route[route_key])

    promoted_routes = [key for key, promoted in route_decisions.items() if promoted]
    nonpromoted_routes = [key for key, promoted in route_decisions.items() if not promoted]
    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "r_depth": trace.r_depth,
        "policy_name": "angular_hopf_guarded",
        "unique_route_keys": len(set(route_keys)),
        "unique_emitted_route_keys": len(emitted_keys),
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": len(promoted_routes) / len(route_decisions) if route_decisions else 0.0,
        "promotion_step_fraction": promoted_steps / trace.trace_length,
        "fallback_step_fraction": promoted_steps / trace.trace_length,
        "effective_resolved_fraction": mean(resolved_scores),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [unresolved_by_route[key] for key in nonpromoted_routes]
        ),
        "route_decision_instability": 0.0,
    }


def summarize_policy(rows: list[dict[str, object]], policy_name: str) -> dict[str, object]:
    subset = [row for row in rows if row["policy_name"] == policy_name]
    return {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in subset),
        "r_depth": "",
        "policy_name": policy_name,
        "unique_route_keys": mean([float(row["unique_route_keys"]) for row in subset]),
        "unique_emitted_route_keys": mean([float(row["unique_emitted_route_keys"]) for row in subset]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in subset]),
        "promotion_route_fraction": mean([float(row["promotion_route_fraction"]) for row in subset]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in subset]),
        "fallback_step_fraction": mean([float(row["fallback_step_fraction"]) for row in subset]),
        "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in subset]),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [float(row["mean_unresolved_among_nonpromoted_routes"]) for row in subset]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces(source_paths=(THRESHOLD_SUMMARY,))
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(evaluate_angular_hopf_baseline(trace))
        rows.append(evaluate_angular_hopf_guarded(trace))
        rows.append(_with_fallback_metric(evaluate_base_gap_policy(trace)))

    rows.append(summarize_policy(rows, "angular_hopf"))
    rows.append(summarize_policy(rows, "angular_hopf_guarded"))
    rows.append(summarize_policy(rows, "base_gap"))
    write_rows(OUT_CSV, rows)
    print(f"angular_hopf_vs_prime_transport_with_transfers_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
