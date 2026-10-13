#!/usr/bin/env python3
"""Bounded stateful task-loop evaluation for the prime-transport router memory layer."""

from __future__ import annotations

import csv
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from mock_router_module import FullState, score_or_route
from router_memory_layer import (
    initialize_memory_layer,
    initialize_memory_state,
    query_memory,
    update_memory_state,
    write_memory,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_task_loop.csv"

FIELDS = ("goal", "world", "combat", "pressure")


def mean(values: list[float]) -> float:
    return sum(values) / len(values) if values else 0.0


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def full_key_for_step(base_phase: int, spin_H: tuple[int, ...]) -> str:
    return str(score_or_route(FullState(b=base_phase, spin_H=spin_H))["route_key"])


def task_payload(trace, step_index: int) -> str:
    field = FIELDS[step_index % len(FIELDS)]
    phi_head = trace.phases[step_index][0] if trace.phases[step_index] else 0
    spin_mass = sum(trace.spins[step_index][: min(3, len(trace.spins[step_index]))])
    return f"{field}|gap={trace.gaps[step_index]}|phi0={phi_head}|spin3={spin_mass}"


def run_trace(trace) -> dict[str, object]:
    layer = initialize_memory_layer()
    state = initialize_memory_state(
        b=0,
        phi=trace.phases[0],
        r=trace.r_depth,
        next_return_gap=trace.gaps[0],
    )

    oracle_route_payload: dict[str, str] = {}
    oracle_full_payload: dict[str, str] = {}
    route_decisions: dict[str, bool] = {}

    query_attempts = 0
    query_hits = 0
    correct_queries = 0
    promoted_queries = 0
    promoted_writes = 0
    route_reuse_events = 0

    for j in range(trace.trace_length):
        route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        full_key = full_key_for_step(j % 35, trace.spins[j])
        unresolved_fraction = trace.unresolved_by_route[route_key]

        read_result = query_memory(
            layer,
            state,
            unresolved_fraction=unresolved_fraction,
            full_key=full_key,
        )

        if route_key in oracle_route_payload:
            query_attempts += 1
            route_reuse_events += 1
            query_hits += int(bool(read_result["read_hit"]))
            if read_result["read_mode"] == "promoted_exact":
                promoted_queries += 1
                expected_payload = oracle_full_payload.get(full_key)
            else:
                expected_payload = oracle_route_payload.get(route_key)
            if expected_payload is not None and read_result["predicted_payload"] == expected_payload:
                correct_queries += 1

        payload = task_payload(trace, j)
        write_result = write_memory(
            layer,
            state,
            unresolved_fraction=unresolved_fraction,
            observed_next_bit=int(trace.spins[j][0]) if trace.spins[j] else int(trace.gaps[j] == 0),
            observed_full_key=full_key,
            spin_H=trace.spins[j],
            observed_payload=payload,
        )
        promoted_writes += int(bool(write_result["promoted"]))
        route_decisions[route_key] = bool(write_result["promoted"])

        oracle_route_payload[route_key] = payload
        oracle_full_payload[full_key] = payload

        next_j = (j + 1) % trace.trace_length
        state = update_memory_state(
            state,
            fiber_moduli=trace.layer_primes,
            next_return_gap=trace.gaps[next_j],
        )
        state = initialize_memory_state(
            b=next_j % 35,
            phi=trace.phases[next_j],
            r=trace.r_depth,
            next_return_gap=trace.gaps[next_j],
        )

    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "memory_slots": len(layer.slots),
        "route_reuse_fraction": route_reuse_events / trace.trace_length,
        "query_hit_fraction_on_reuse": query_hits / query_attempts if query_attempts else 0.0,
        "task_retrieval_accuracy": correct_queries / query_attempts if query_attempts else 0.0,
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "promotion_step_fraction": promoted_writes / trace.trace_length,
        "effective_task_resolution_fraction": mean(
            [
                correct_queries / query_attempts if query_attempts else 0.0,
                1.0 - (promoted_writes / trace.trace_length),
            ]
        ),
        "route_decision_instability": 0.0,
    }


def summarize_rows(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in rows),
        "memory_slots": mean([float(row["memory_slots"]) for row in rows]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows]),
        "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in rows]),
        "task_retrieval_accuracy": mean([float(row["task_retrieval_accuracy"]) for row in rows]),
        "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_task_resolution_fraction": mean([float(row["effective_task_resolution_fraction"]) for row in rows]),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces()
    rows = [run_trace(trace) for trace in traces]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_router_memory_task_loop_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
