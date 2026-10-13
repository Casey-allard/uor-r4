#!/usr/bin/env python3
"""Bounded demo for the structured read/write router-memory layer."""

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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_layer_demo.csv"


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
    full_state = FullState(b=base_phase, spin_H=spin_H)
    return str(score_or_route(full_state)["route_key"])


def run_trace(trace) -> dict[str, object]:
    layer = initialize_memory_layer()

    warm_state = initialize_memory_state(
        b=0,
        phi=trace.phases[0],
        r=trace.r_depth,
        next_return_gap=trace.gaps[0],
    )
    warm_hits = 0

    for j in range(trace.trace_length):
        route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        read_result = query_memory(
            layer,
            warm_state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
        )
        warm_hits += int(bool(read_result["read_hit"]))

        observed_key = full_key_for_step(j % 35, trace.spins[j])
        observed_next_bit = int(trace.spins[j][0]) if trace.spins[j] else int(trace.gaps[j] == 0)
        _ = write_memory(
            layer,
            warm_state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
            observed_next_bit=observed_next_bit,
            observed_full_key=observed_key,
            spin_H=trace.spins[j],
        )

        next_j = (j + 1) % trace.trace_length
        warm_state = initialize_memory_state(
            b=next_j % 35,
            phi=trace.phases[next_j],
            r=trace.r_depth,
            next_return_gap=trace.gaps[next_j],
        )

    eval_state = initialize_memory_state(
        b=0,
        phi=trace.phases[0],
        r=trace.r_depth,
        next_return_gap=trace.gaps[0],
    )
    eval_hits = 0
    promoted_reads = 0
    exact_matches = 0
    weighted_resolution_scores: list[float] = []

    for j in range(trace.trace_length):
        route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        observed_key = full_key_for_step(j % 35, trace.spins[j])
        read_result = query_memory(
            layer,
            eval_state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
        )
        eval_hits += int(bool(read_result["read_hit"]))
        promoted_reads += int(read_result["read_mode"] == "promoted_exact")

        if read_result["predicted_key"] == observed_key:
            exact_matches += 1

        confidence = float(read_result["predicted_confidence"])
        weighted_resolution_scores.append(confidence)

        observed_next_bit = int(trace.spins[j][0]) if trace.spins[j] else int(trace.gaps[j] == 0)
        _ = write_memory(
            layer,
            eval_state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
            observed_next_bit=observed_next_bit,
            observed_full_key=observed_key,
            spin_H=trace.spins[j],
        )

        next_j = (j + 1) % trace.trace_length
        eval_state = update_memory_state(
            eval_state,
            fiber_moduli=trace.layer_primes,
            next_return_gap=trace.gaps[next_j],
        )
        eval_state = initialize_memory_state(
            b=next_j % 35,
            phi=trace.phases[next_j],
            r=trace.r_depth,
            next_return_gap=trace.gaps[next_j],
        )

    promoted_slots = sum(1 for slot in layer.slots.values() if slot.promoted)
    total_reads = sum(slot.read_count for slot in layer.slots.values())
    total_writes = sum(slot.write_count for slot in layer.slots.values())

    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "memory_slots": len(layer.slots),
        "promoted_slot_fraction": promoted_slots / len(layer.slots) if layer.slots else 0.0,
        "warm_read_hit_fraction": warm_hits / trace.trace_length,
        "eval_read_hit_fraction": eval_hits / trace.trace_length,
        "eval_promoted_read_fraction": promoted_reads / trace.trace_length,
        "eval_exact_retrieval_fraction": exact_matches / trace.trace_length,
        "eval_effective_resolution_fraction": mean(weighted_resolution_scores),
        "memory_total_reads": total_reads,
        "memory_total_writes": total_writes,
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
        "promoted_slot_fraction": mean([float(row["promoted_slot_fraction"]) for row in rows]),
        "warm_read_hit_fraction": mean([float(row["warm_read_hit_fraction"]) for row in rows]),
        "eval_read_hit_fraction": mean([float(row["eval_read_hit_fraction"]) for row in rows]),
        "eval_promoted_read_fraction": mean([float(row["eval_promoted_read_fraction"]) for row in rows]),
        "eval_exact_retrieval_fraction": mean([float(row["eval_exact_retrieval_fraction"]) for row in rows]),
        "eval_effective_resolution_fraction": mean([float(row["eval_effective_resolution_fraction"]) for row in rows]),
        "memory_total_reads": mean([float(row["memory_total_reads"]) for row in rows]),
        "memory_total_writes": mean([float(row["memory_total_writes"]) for row in rows]),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces()
    rows = [run_trace(trace) for trace in traces]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_router_memory_layer_demo_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
