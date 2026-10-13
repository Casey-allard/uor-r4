#!/usr/bin/env python3
"""First larger bounded router-native memory experiment on structured records."""

from __future__ import annotations

import csv
import json
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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_larger_router_memory_experiment.csv"
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


def empty_record() -> dict[str, object]:
    return {
        "goal": "",
        "world": "",
        "combat": "",
        "pressure": "",
        "revision": 0,
    }


def serialize_record(record: dict[str, object]) -> str:
    return json.dumps(record, sort_keys=True, separators=(",", ":"))


def partial_update_value(trace, step_index: int, field: str) -> str:
    phi_head = trace.phases[step_index][0] if trace.phases[step_index] else 0
    spin_mass = sum(trace.spins[step_index][: min(3, len(trace.spins[step_index]))])
    return f"{field}@g{trace.gaps[step_index]}:p{phi_head}:s{spin_mass}:t{step_index % 11}"


def run_trace(trace) -> dict[str, object]:
    layer = initialize_memory_layer()
    state = initialize_memory_state(
        b=0,
        phi=trace.phases[0],
        r=trace.r_depth,
        next_return_gap=trace.gaps[0],
    )

    oracle_route_record: dict[str, dict[str, object]] = {}
    oracle_full_record: dict[str, dict[str, object]] = {}

    query_attempts = 0
    query_hits = 0
    correct_queries = 0
    promoted_queries = 0
    route_reuse_events = 0
    promoted_writes = 0

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

        if route_key in oracle_route_record:
            query_attempts += 1
            route_reuse_events += 1
            query_hits += int(bool(read_result["read_hit"]))
            if read_result["read_mode"] == "promoted_exact":
                promoted_queries += 1
                expected_payload = serialize_record(oracle_full_record.get(full_key, oracle_route_record[route_key]))
            else:
                expected_payload = serialize_record(oracle_route_record[route_key])
            if read_result["predicted_payload"] == expected_payload:
                correct_queries += 1

        base_record = dict(oracle_full_record.get(full_key, oracle_route_record.get(route_key, empty_record())))
        field = FIELDS[j % len(FIELDS)]
        base_record[field] = partial_update_value(trace, j, field)
        base_record["revision"] = int(base_record.get("revision", 0)) + 1
        payload = serialize_record(base_record)

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

        oracle_route_record[route_key] = dict(base_record)
        oracle_full_record[full_key] = dict(base_record)

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
        "structured_record_accuracy": correct_queries / query_attempts if query_attempts else 0.0,
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "promotion_step_fraction": promoted_writes / trace.trace_length,
        "effective_structured_resolution_fraction": mean(
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
        "structured_record_accuracy": mean([float(row["structured_record_accuracy"]) for row in rows]),
        "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_structured_resolution_fraction": mean(
            [float(row["effective_structured_resolution_fraction"]) for row in rows]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces()
    rows = [run_trace(trace) for trace in traces]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_larger_router_memory_experiment_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
