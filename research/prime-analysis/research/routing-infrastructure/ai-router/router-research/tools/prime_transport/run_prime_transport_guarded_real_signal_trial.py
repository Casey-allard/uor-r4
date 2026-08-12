#!/usr/bin/env python3
"""First guarded real-signal research-side integration trial for prime transport."""

from __future__ import annotations

import csv
from pathlib import Path

from angular_hopf_benchmark_adapter import angular_route_key_for_trace_entry
from base_gap_routing_adapter import adapter_output, initialize_adapter, initialize_entry
from real_signal_benchmark_wrapper import ROOT, build_real_signal_traces
from run_mock_router_trace_eval import class_unresolved_fraction, mean


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_guarded_real_signal_trial.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


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


def evaluate_angular_baseline(trace) -> dict[str, object]:
    route_keys = angular_route_keys(trace)
    unresolved_by_route = class_unresolved_fraction(route_keys, trace.route_full_keys)
    unique_route_keys = len(set(route_keys))
    cache_hits = trace.trace_length - unique_route_keys
    return {
        "trace_source": trace.trace_source,
        "scenario_id": trace.scenario_id,
        "run_id": trace.run_id,
        "trace_length": trace.trace_length,
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
        "benchmark_signal_pressure": trace.mean_signal_pressure,
    }


def evaluate_angular_guarded(trace) -> dict[str, object]:
    route_keys = angular_route_keys(trace)
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
            route_table[route_key] = float(unresolved_by_route[route_key]) > 0.30
        promote = route_table[route_key]
        if route_key in route_decisions and route_decisions[route_key] != promote:
            raise RuntimeError(f"decision instability in angular_hopf_guarded for {route_key}")
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
        "benchmark_signal_pressure": trace.mean_signal_pressure,
    }


def evaluate_prime_transport(trace) -> dict[str, object]:
    adapter = initialize_adapter()
    route_keys = [
        f"base_gap:r=2:b={trace.base_phases[index]}:gap={trace.next_return_gaps[index]}"
        for index in range(trace.trace_length)
    ]
    unresolved_by_route = class_unresolved_fraction(route_keys, trace.route_full_keys)

    route_decisions: dict[str, bool] = {}
    emitted_keys: set[str] = set()
    cache_hits = 0
    promoted_steps = 0
    resolved_scores: list[float] = []

    for index in range(trace.trace_length):
        route_key = route_keys[index]
        if route_key in adapter.route_policy_cache:
            cache_hits += 1
        state = initialize_entry(
            b=trace.base_phases[index],
            phi=trace.phi_tuples[index],
            r=2,
            next_return_gap=trace.next_return_gaps[index],
        )
        result = adapter_output(
            adapter,
            state,
            unresolved_fraction=unresolved_by_route[route_key],
            spin_H=trace.route_full_keys[index],
        )
        promoted = bool(result["promoted"])
        if route_key in route_decisions and route_decisions[route_key] != promoted:
            raise RuntimeError(f"decision instability in base_gap for {route_key}")
        route_decisions[route_key] = promoted
        emitted_keys.add(str(result["route_key"]))
        if promoted:
            promoted_steps += 1
            resolved_scores.append(1.0)
        else:
            resolved_scores.append(1.0 - unresolved_by_route[route_key])

    promoted_routes = [key for key, promoted in route_decisions.items() if promoted]
    nonpromoted_routes = [key for key, promoted in route_decisions.items() if not promoted]
    return {
        "trace_source": trace.trace_source,
        "scenario_id": trace.scenario_id,
        "run_id": trace.run_id,
        "trace_length": trace.trace_length,
        "policy_name": "base_gap",
        "unique_route_keys": len(adapter.route_policy_cache),
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
        rows.append(evaluate_angular_baseline(trace))
        rows.append(evaluate_angular_guarded(trace))
        rows.append(evaluate_prime_transport(trace))

    rows.append(summarize_policy(rows, "angular_hopf"))
    rows.append(summarize_policy(rows, "angular_hopf_guarded"))
    rows.append(summarize_policy(rows, "base_gap"))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_guarded_real_signal_trial_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
