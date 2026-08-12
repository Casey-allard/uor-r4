#!/usr/bin/env python3
"""Bounded multi-entity router-memory experiment with unchanged read path."""

from __future__ import annotations

import csv
import json
from collections import defaultdict
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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_multi_entity_router_memory_experiment.csv"
FIELDS = ("goal", "world", "combat", "pressure")
ENTITY_NAMES = ("entity_a", "entity_b", "entity_c")


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


def empty_record(entity_name: str) -> dict[str, object]:
    return {
        "entity": entity_name,
        "goal": "",
        "world": "",
        "combat": "",
        "pressure": "",
        "revision": 0,
    }


def serialize_record(record: dict[str, object]) -> str:
    return json.dumps(record, sort_keys=True, separators=(",", ":"))


def partial_update_value(trace, step_index: int, field: str, entity_name: str) -> str:
    phi_head = trace.phases[step_index][0] if trace.phases[step_index] else 0
    spin_mass = sum(trace.spins[step_index][: min(3, len(trace.spins[step_index]))])
    return f"{entity_name}:{field}@g{trace.gaps[step_index]}:p{phi_head}:s{spin_mass}:t{step_index % 13}"


def build_entity_bundles() -> list[dict[str, object]]:
    traces = build_bounded_traces()
    grouped: dict[str, list[object]] = defaultdict(list)
    for trace in traces:
        grouped[trace.trace_source].append(trace)

    bundles: list[dict[str, object]] = []
    for trace_source, source_traces in grouped.items():
        selected = source_traces[: len(ENTITY_NAMES)]
        if len(selected) < 2:
            continue
        bundles.append(
            {
                "bundle_id": f"{trace_source}:bundle0",
                "trace_source": trace_source,
                "entities": [
                    {"entity_name": ENTITY_NAMES[index], "trace": trace}
                    for index, trace in enumerate(selected)
                ],
            }
        )
    return bundles


def coordination_summary(records: list[dict[str, object]]) -> str:
    if not records:
        return "coord:none"
    total_revision = sum(int(record.get("revision", 0)) for record in records)
    filled_fields = 0
    pressure_entities = 0
    for record in records:
        for field in FIELDS:
            filled_fields += int(bool(record.get(field, "")))
        pressure_entities += int(bool(record.get("pressure", "")))
    return f"coord:n={len(records)}:rev={total_revision}:filled={filled_fields}:pressure={pressure_entities}"


def parse_payload(payload: str | None, entity_name: str) -> dict[str, object]:
    if not payload:
        return empty_record(entity_name)
    return json.loads(payload)


def run_bundle(bundle: dict[str, object]) -> dict[str, object]:
    layers = {}
    states = {}
    oracle_route_record: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)
    oracle_full_record: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)

    for entity in bundle["entities"]:
        trace = entity["trace"]
        entity_name = entity["entity_name"]
        layers[entity_name] = initialize_memory_layer()
        states[entity_name] = initialize_memory_state(
            b=0,
            phi=trace.phases[0],
            r=trace.r_depth,
            next_return_gap=trace.gaps[0],
        )

    max_steps = max(entity["trace"].trace_length for entity in bundle["entities"])
    query_attempts = 0
    query_hits = 0
    correct_entity_queries = 0
    promoted_queries = 0
    route_reuse_events = 0
    promoted_writes = 0
    coordination_attempts = 0
    correct_coordination = 0
    total_entity_steps = 0

    for step in range(max_steps):
        active_reads: list[dict[str, object]] = []
        step_all_correct = True

        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

            total_entity_steps += 1
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]

            read_result = query_memory(
                layers[entity_name],
                states[entity_name],
                unresolved_fraction=unresolved_fraction,
                full_key=full_key,
            )

            if route_key in oracle_route_record[entity_name]:
                query_attempts += 1
                route_reuse_events += 1
                query_hits += int(bool(read_result["read_hit"]))
                if read_result["read_mode"] == "promoted_exact":
                    promoted_queries += 1
                    expected_record = dict(
                        oracle_full_record[entity_name].get(full_key, oracle_route_record[entity_name][route_key])
                    )
                    expected_payload = serialize_record(expected_record)
                else:
                    expected_record = dict(oracle_route_record[entity_name][route_key])
                    expected_payload = serialize_record(expected_record)
                read_correct = read_result["predicted_payload"] == expected_payload
                if read_correct:
                    correct_entity_queries += 1
                else:
                    step_all_correct = False
                active_reads.append(
                    {
                        "entity_name": entity_name,
                        "predicted_record": parse_payload(read_result["predicted_payload"], entity_name),
                    }
                )

        if len(active_reads) >= 2:
            coordination_attempts += 1
            if step_all_correct:
                correct_coordination += 1

        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]

            base_record = dict(
                oracle_full_record[entity_name].get(
                    full_key,
                    oracle_route_record[entity_name].get(route_key, empty_record(entity_name)),
                )
            )
            field = FIELDS[(step + bundle["entities"].index(entity)) % len(FIELDS)]
            base_record[field] = partial_update_value(trace, step, field, entity_name)
            base_record["revision"] = int(base_record.get("revision", 0)) + 1
            payload = serialize_record(base_record)

            write_result = write_memory(
                layers[entity_name],
                states[entity_name],
                unresolved_fraction=unresolved_fraction,
                observed_next_bit=int(trace.spins[step][0]) if trace.spins[step] else int(trace.gaps[step] == 0),
                observed_full_key=full_key,
                spin_H=trace.spins[step],
                observed_payload=payload,
            )
            promoted_writes += int(bool(write_result["promoted"]))

            oracle_route_record[entity_name][route_key] = dict(base_record)
            oracle_full_record[entity_name][full_key] = dict(base_record)

            next_j = (step + 1) % trace.trace_length
            states[entity_name] = update_memory_state(
                states[entity_name],
                fiber_moduli=trace.layer_primes,
                next_return_gap=trace.gaps[next_j],
            )
            states[entity_name] = initialize_memory_state(
                b=next_j % 35,
                phi=trace.phases[next_j],
                r=trace.r_depth,
                next_return_gap=trace.gaps[next_j],
            )

    return {
        "bundle_id": bundle["bundle_id"],
        "trace_source": bundle["trace_source"],
        "entity_count": len(bundle["entities"]),
        "trace_length": max_steps,
        "route_reuse_fraction": route_reuse_events / total_entity_steps if total_entity_steps else 0.0,
        "query_hit_fraction_on_reuse": query_hits / query_attempts if query_attempts else 0.0,
        "multi_record_retrieval_accuracy": correct_entity_queries / query_attempts if query_attempts else 0.0,
        "coordination_snapshot_accuracy": correct_coordination / coordination_attempts if coordination_attempts else 0.0,
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "promotion_step_fraction": promoted_writes / total_entity_steps if total_entity_steps else 0.0,
        "effective_multi_entity_resolution_fraction": mean(
            [
                correct_entity_queries / query_attempts if query_attempts else 0.0,
                correct_coordination / coordination_attempts if coordination_attempts else 0.0,
                1.0 - (promoted_writes / total_entity_steps if total_entity_steps else 0.0),
            ]
        ),
        "route_decision_instability": 0.0,
    }


def summarize_rows(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "bundle_id": "ALL",
        "trace_source": "ALL",
        "entity_count": mean([float(row["entity_count"]) for row in rows]),
        "trace_length": sum(int(row["trace_length"]) for row in rows),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows]),
        "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in rows]),
        "multi_record_retrieval_accuracy": mean([float(row["multi_record_retrieval_accuracy"]) for row in rows]),
        "coordination_snapshot_accuracy": mean([float(row["coordination_snapshot_accuracy"]) for row in rows]),
        "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_multi_entity_resolution_fraction": mean(
            [float(row["effective_multi_entity_resolution_fraction"]) for row in rows]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    bundles = build_entity_bundles()
    rows = [run_bundle(bundle) for bundle in bundles]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_multi_entity_router_memory_experiment_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
