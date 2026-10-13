#!/usr/bin/env python3
"""Bounded reuse-recovery test for prime transport on real MUDBench signal."""

from __future__ import annotations

import csv
from pathlib import Path

from angular_hopf_benchmark_adapter import angular_route_key_for_trace_entry
from real_signal_benchmark_wrapper import ROOT, build_real_signal_traces
from run_mock_router_trace_eval import class_unresolved_fraction, mean


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_real_signal_reuse_recovery.csv"
PROMOTION_THRESHOLD = 0.30


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def gap_bucket(gap: int) -> int:
    return min(int(gap), 2)


def candidate_key(mode: str, *, b: int, gap: int) -> str:
    if mode == "base_gap":
        return f"base_gap:r=2:b={b}:gap={gap}"
    if mode == "base_gap_gapbucket":
        return f"base_gap_gapbucket:r=2:b={b}:gap_bucket={gap_bucket(gap)}"
    if mode == "base_bucket5_gapbucket":
        return f"base_bucket5_gapbucket:r=2:base_bucket={b // 5}:gap_bucket={gap_bucket(gap)}"
    raise ValueError(f"unknown mode: {mode}")


def angular_route_keys(trace) -> list[str]:
    return [
        angular_route_key_for_trace_entry(
            r_depth=2,
            base_phase=trace.base_phases[index],
            phase_tuple=trace.phi_tuples[index],
            layer_primes=(4, 4),
            next_return_gap=trace.next_return_gaps[index],
        )
        for index in range(trace.trace_length)
    ]


def evaluate_guarded_partition(trace, *, mode: str, route_keys: list[str]) -> dict[str, object]:
    unresolved_by_route = class_unresolved_fraction(route_keys, trace.route_full_keys)

    route_table: dict[str, bool] = {}
    route_decisions: dict[str, bool] = {}
    emitted_keys: set[str] = set()
    cache_hits = 0
    promoted_steps = 0
    resolved_scores: list[float] = []

    for index, route_key in enumerate(route_keys):
        if route_key in route_table:
            cache_hits += 1
        else:
            route_table[route_key] = float(unresolved_by_route[route_key]) > PROMOTION_THRESHOLD
        promote = route_table[route_key]
        if route_key in route_decisions and route_decisions[route_key] != promote:
            raise RuntimeError(f"decision instability in {mode} for {route_key}")
        route_decisions[route_key] = promote
        if promote:
            promoted_steps += 1
            emitted_keys.add(trace.route_full_keys[index])
            resolved_scores.append(1.0)
        else:
            emitted_keys.add(route_key)
            resolved_scores.append(1.0 - unresolved_by_route[route_key])

    promoted_routes = [key for key, promoted in route_decisions.items() if promoted]
    nonpromoted_routes = [key for key, promoted in route_decisions.items() if not promoted]
    return {
        "trace_source": trace.trace_source,
        "scenario_id": trace.scenario_id,
        "run_id": trace.run_id,
        "trace_length": trace.trace_length,
        "policy_name": mode,
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
        "benchmark_signal_pressure": trace.mean_signal_pressure,
    }


def evaluate_base_gap_family(trace, *, mode: str) -> dict[str, object]:
    route_keys = [
        candidate_key(mode, b=trace.base_phases[index], gap=trace.next_return_gaps[index])
        for index in range(trace.trace_length)
    ]
    return evaluate_guarded_partition(trace, mode=mode, route_keys=route_keys)


def evaluate_angular_guarded(trace) -> dict[str, object]:
    return evaluate_guarded_partition(
        trace,
        mode="angular_hopf_guarded",
        route_keys=angular_route_keys(trace),
    )


def summarize_policy(rows: list[dict[str, object]], policy_name: str) -> dict[str, object]:
    subset = [row for row in rows if row["policy_name"] == policy_name]
    return {
        "trace_source": "ALL",
        "scenario_id": "ALL",
        "run_id": "ALL",
        "trace_length": sum(int(row["trace_length"]) for row in subset),
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
        "benchmark_signal_pressure": mean([float(row["benchmark_signal_pressure"]) for row in subset]),
    }


def main() -> None:
    traces = build_real_signal_traces()
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(evaluate_base_gap_family(trace, mode="base_gap"))
        rows.append(evaluate_base_gap_family(trace, mode="base_gap_gapbucket"))
        rows.append(evaluate_base_gap_family(trace, mode="base_bucket5_gapbucket"))
        rows.append(evaluate_angular_guarded(trace))

    for policy_name in (
        "base_gap",
        "base_gap_gapbucket",
        "base_bucket5_gapbucket",
        "angular_hopf_guarded",
    ):
        rows.append(summarize_policy(rows, policy_name))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_real_signal_reuse_recovery_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
