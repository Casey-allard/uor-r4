#!/usr/bin/env python3
"""Guarded research-only benchmark harness for prime-transport routing policies."""

from __future__ import annotations

import csv
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from base_gap_routing_adapter import (
    adapter_output,
    initialize_adapter,
    initialize_entry,
)
from run_mock_router_trace_eval import class_unresolved_fraction, mean


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_guarded_benchmark_harness_eval.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def full_route_keys_for_trace(trace) -> list[str]:
    return [
        f"full:b={j % 35}:spin={trace.spins[j]}"
        for j in range(trace.trace_length)
    ]


def static_route_keys_for_trace(trace) -> list[str]:
    return [
        f"static:r={trace.r_depth}:b={j % 35}:phi={trace.phases[j]}"
        for j in range(trace.trace_length)
    ]


def gap_only_route_keys_for_trace(trace) -> list[str]:
    return [
        f"gap_only:r={trace.r_depth}:gap={trace.gaps[j]}"
        for j in range(trace.trace_length)
    ]


def base_gap_route_keys_for_trace(trace) -> list[str]:
    return [
        f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        for j in range(trace.trace_length)
    ]


def evaluate_partition_control(trace, *, policy_name: str, route_keys: list[str]) -> dict[str, object]:
    full_keys = full_route_keys_for_trace(trace)
    unresolved_by_route = class_unresolved_fraction(route_keys, full_keys)
    unique_route_keys = len(set(route_keys))
    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "r_depth": trace.r_depth,
        "policy_name": policy_name,
        "unique_route_keys": unique_route_keys,
        "unique_emitted_route_keys": unique_route_keys,
        "route_reuse_fraction": 1.0 - (unique_route_keys / trace.trace_length),
        "promotion_route_fraction": 0.0,
        "promotion_step_fraction": 0.0,
        "effective_resolved_fraction": 1.0 - mean(list(unresolved_by_route.values())),
        "mean_unresolved_among_nonpromoted_routes": mean(list(unresolved_by_route.values())),
        "route_decision_instability": 0.0,
    }


def evaluate_gap_only_policy(trace) -> dict[str, object]:
    route_keys = gap_only_route_keys_for_trace(trace)
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
            route_table[route_key] = float(unresolved_by_route[route_key]) > 0.30
        promote = route_table[route_key]
        if route_key in route_decisions and route_decisions[route_key] != promote:
            raise RuntimeError(f"decision instability in gap_only for {route_key}")
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
        "policy_name": "gap_only",
        "unique_route_keys": len(set(route_keys)),
        "unique_emitted_route_keys": len(emitted_keys),
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": len(promoted_routes) / len(route_decisions) if route_decisions else 0.0,
        "promotion_step_fraction": promoted_steps / trace.trace_length,
        "effective_resolved_fraction": mean(resolved_scores),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [unresolved_by_route[key] for key in nonpromoted_routes]
        ),
        "route_decision_instability": 0.0,
    }


def evaluate_base_gap_policy(trace) -> dict[str, object]:
    full_keys = full_route_keys_for_trace(trace)
    adapter = initialize_adapter()
    route_decisions: dict[str, bool] = {}
    emitted_keys: set[str] = set()
    resolved_scores: list[float] = []
    cache_hits = 0
    promoted_steps = 0

    for j in range(trace.trace_length):
        route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        if route_key in adapter.route_policy_cache:
            cache_hits += 1
        state = initialize_entry(
            b=j % 35,
            phi=trace.phases[j],
            r=trace.r_depth,
            next_return_gap=trace.gaps[j],
        )
        result = adapter_output(
            adapter,
            state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
            spin_H=trace.spins[j],
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
            resolved_scores.append(1.0 - trace.unresolved_by_route[route_key])

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
        "policy_name": "base_gap",
        "unique_route_keys": len(adapter.route_policy_cache),
        "unique_emitted_route_keys": len(emitted_keys),
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": len(promoted_routes) / len(route_decisions) if route_decisions else 0.0,
        "promotion_step_fraction": promoted_steps / trace.trace_length,
        "effective_resolved_fraction": mean(resolved_scores),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [trace.unresolved_by_route[key] for key in nonpromoted_routes]
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
        "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in subset]),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [float(row["mean_unresolved_among_nonpromoted_routes"]) for row in subset]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces()
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(
            evaluate_partition_control(
                trace,
                policy_name="static_only",
                route_keys=static_route_keys_for_trace(trace),
            )
        )
        rows.append(evaluate_gap_only_policy(trace))
        rows.append(evaluate_base_gap_policy(trace))

    rows.append(summarize_policy(rows, "static_only"))
    rows.append(summarize_policy(rows, "gap_only"))
    rows.append(summarize_policy(rows, "base_gap"))

    write_rows(OUT_CSV, rows)
    print(f"guarded_benchmark_csv,{OUT_CSV}")


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    main()
