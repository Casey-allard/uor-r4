#!/usr/bin/env python3
"""Bounded router-memory workspace systems prototype with unchanged read path."""

from __future__ import annotations

import csv
import json
from collections import defaultdict
from pathlib import Path

from router_memory_layer import (
    initialize_memory_layer,
    initialize_memory_state,
    query_memory,
    update_memory_state,
    write_memory,
)
from run_multi_entity_router_memory_experiment import (
    ENTITY_NAMES,
    FIELDS,
    build_entity_bundles,
    empty_record,
    full_key_for_step,
    partial_update_value,
    serialize_record,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_workspace_experiment.csv"
SHARED_LEDGERS = ("task_ledger", "world_ledger", "dependency_ledger")


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


def empty_ledger(ledger_name: str) -> dict[str, object]:
    return {
        "ledger": ledger_name,
        "open": [],
        "claimed": [],
        "completed": [],
        "shared_revision": 0,
    }


def empty_dependency_summary() -> dict[str, object]:
    return {
        "ledger": "dependency_ledger",
        "dependencies": {},
        "shared_revision": 0,
    }


def parse_payload(payload: str | None, default_factory) -> dict[str, object]:
    if not payload:
        return default_factory()
    return json.loads(payload)


def build_entity_update(trace, step: int, entity_name: str, base_record: dict[str, object]) -> dict[str, object]:
    field = FIELDS[(step + ENTITY_NAMES.index(entity_name)) % len(FIELDS)]
    base_record[field] = partial_update_value(trace, step, field, entity_name)
    base_record["revision"] = int(base_record.get("revision", 0)) + 1
    return base_record


def build_task_ledger(entity_records: dict[str, dict[str, object]], revision: int) -> dict[str, object]:
    open_tasks = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if not record.get("goal")
    )
    claimed_tasks = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if record.get("goal") and not record.get("world")
    )
    completed_tasks = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if record.get("goal") and record.get("world") and record.get("pressure")
    )
    return {
        "ledger": "task_ledger",
        "open": open_tasks,
        "claimed": claimed_tasks,
        "completed": completed_tasks,
        "shared_revision": revision,
    }


def build_world_ledger(entity_records: dict[str, dict[str, object]], revision: int) -> dict[str, object]:
    active_world = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if record.get("world")
    )
    combat_hot = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if record.get("combat")
    )
    pressure_hot = sorted(
        entity_name
        for entity_name, record in entity_records.items()
        if record.get("pressure")
    )
    return {
        "ledger": "world_ledger",
        "open": active_world,
        "claimed": combat_hot,
        "completed": pressure_hot,
        "shared_revision": revision,
    }


def build_dependency_ledger(entity_records: dict[str, dict[str, object]], revision: int) -> dict[str, object]:
    dependencies = {}
    for entity_name, record in entity_records.items():
        deps = []
        if record.get("goal") and not record.get("world"):
            deps.append("world_ledger")
        if record.get("combat") and not record.get("pressure"):
            deps.append("task_ledger")
        if deps:
            dependencies[entity_name] = deps
    return {
        "ledger": "dependency_ledger",
        "dependencies": dependencies,
        "shared_revision": revision,
    }


def run_bundle(bundle: dict[str, object]) -> dict[str, object]:
    local_layers = {}
    local_states = {}
    local_oracle_route: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)
    local_oracle_full: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)

    shared_layers = {}
    shared_states = {}
    shared_oracle_route: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)
    shared_oracle_full: dict[str, dict[str, dict[str, object]]] = defaultdict(dict)

    for entity in bundle["entities"]:
        trace = entity["trace"]
        entity_name = entity["entity_name"]
        local_layers[entity_name] = initialize_memory_layer()
        local_states[entity_name] = initialize_memory_state(
            b=0,
            phi=trace.phases[0],
            r=trace.r_depth,
            next_return_gap=trace.gaps[0],
        )

    shared_trace_map = {
        "task_ledger": bundle["entities"][0]["trace"],
        "world_ledger": bundle["entities"][1]["trace"],
        "dependency_ledger": bundle["entities"][2]["trace"],
    }
    for ledger_name, trace in shared_trace_map.items():
        shared_layers[ledger_name] = initialize_memory_layer()
        shared_states[ledger_name] = initialize_memory_state(
            b=0,
            phi=trace.phases[0],
            r=trace.r_depth,
            next_return_gap=trace.gaps[0],
        )

    max_steps = max(entity["trace"].trace_length for entity in bundle["entities"])

    total_object_steps = 0
    route_reuse_events = 0
    query_attempts = 0
    local_query_attempts = 0
    shared_query_attempts = 0
    query_hits = 0
    correct_local_queries = 0
    correct_shared_queries = 0
    promoted_queries = 0
    promoted_writes = 0
    joint_attempts = 0
    correct_joint = 0

    for step in range(max_steps):
        step_all_correct = True
        current_entity_records: dict[str, dict[str, object]] = {}

        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

            total_object_steps += 1
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]
            read_result = query_memory(
                local_layers[entity_name],
                local_states[entity_name],
                unresolved_fraction=unresolved_fraction,
                full_key=full_key,
            )

            if route_key in local_oracle_route[entity_name]:
                query_attempts += 1
                local_query_attempts += 1
                route_reuse_events += 1
                query_hits += int(bool(read_result["read_hit"]))
                if read_result["read_mode"] == "promoted_exact":
                    promoted_queries += 1
                    expected_record = dict(local_oracle_full[entity_name].get(full_key, local_oracle_route[entity_name][route_key]))
                else:
                    expected_record = dict(local_oracle_route[entity_name][route_key])
                expected_payload = serialize_record(expected_record)
                if read_result.get("predicted_payload") == expected_payload:
                    correct_local_queries += 1
                else:
                    step_all_correct = False
                current_entity_records[entity_name] = expected_record
            else:
                current_entity_records[entity_name] = empty_record(entity_name)

        shared_revision = step + 1
        expected_shared = {
            "task_ledger": build_task_ledger(current_entity_records, shared_revision),
            "world_ledger": build_world_ledger(current_entity_records, shared_revision),
            "dependency_ledger": build_dependency_ledger(current_entity_records, shared_revision),
        }

        active_shared_reads = 0
        for ledger_name, trace in shared_trace_map.items():
            if step >= trace.trace_length:
                continue

            total_object_steps += 1
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]
            read_result = query_memory(
                shared_layers[ledger_name],
                shared_states[ledger_name],
                unresolved_fraction=unresolved_fraction,
                full_key=full_key,
            )

            if route_key in shared_oracle_route[ledger_name]:
                query_attempts += 1
                shared_query_attempts += 1
                route_reuse_events += 1
                active_shared_reads += 1
                query_hits += int(bool(read_result["read_hit"]))
                if read_result["read_mode"] == "promoted_exact":
                    promoted_queries += 1
                    expected_record = dict(
                        shared_oracle_full[ledger_name].get(full_key, shared_oracle_route[ledger_name][route_key])
                    )
                else:
                    expected_record = dict(shared_oracle_route[ledger_name][route_key])
                expected_payload = serialize_record(expected_record)
                if read_result.get("predicted_payload") == expected_payload:
                    correct_shared_queries += 1
                else:
                    step_all_correct = False

        if len(current_entity_records) >= 2 and active_shared_reads >= 2:
            joint_attempts += 1
            if step_all_correct:
                correct_joint += 1

        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]

            base_record = dict(
                local_oracle_full[entity_name].get(
                    full_key,
                    local_oracle_route[entity_name].get(route_key, empty_record(entity_name)),
                )
            )
            base_record = build_entity_update(trace, step, entity_name, base_record)
            payload = serialize_record(base_record)

            write_result = write_memory(
                local_layers[entity_name],
                local_states[entity_name],
                unresolved_fraction=unresolved_fraction,
                observed_next_bit=int(trace.spins[step][0]) if trace.spins[step] else int(trace.gaps[step] == 0),
                observed_full_key=full_key,
                spin_H=trace.spins[step],
                observed_payload=payload,
            )
            promoted_writes += int(bool(write_result["promoted"]))

            local_oracle_route[entity_name][route_key] = dict(base_record)
            local_oracle_full[entity_name][full_key] = dict(base_record)

            next_j = (step + 1) % trace.trace_length
            local_states[entity_name] = update_memory_state(
                local_states[entity_name],
                fiber_moduli=trace.layer_primes,
                next_return_gap=trace.gaps[next_j],
            )
            local_states[entity_name] = initialize_memory_state(
                b=next_j % 35,
                phi=trace.phases[next_j],
                r=trace.r_depth,
                next_return_gap=trace.gaps[next_j],
            )

        updated_entity_records = {
            entity["entity_name"]: dict(
                local_oracle_route[entity["entity_name"]].get(
                    f"base_gap:r={entity['trace'].r_depth}:b={step % 35}:gap={entity['trace'].gaps[step]}",
                    empty_record(entity["entity_name"]),
                )
            )
            for entity in bundle["entities"]
            if step < entity["trace"].trace_length
        }
        updated_shared = {
            "task_ledger": build_task_ledger(updated_entity_records, shared_revision),
            "world_ledger": build_world_ledger(updated_entity_records, shared_revision),
            "dependency_ledger": build_dependency_ledger(updated_entity_records, shared_revision),
        }

        for ledger_name, trace in shared_trace_map.items():
            if step >= trace.trace_length:
                continue

            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]
            payload = serialize_record(updated_shared[ledger_name])

            write_result = write_memory(
                shared_layers[ledger_name],
                shared_states[ledger_name],
                unresolved_fraction=unresolved_fraction,
                observed_next_bit=int(trace.spins[step][0]) if trace.spins[step] else int(trace.gaps[step] == 0),
                observed_full_key=full_key,
                spin_H=trace.spins[step],
                observed_payload=payload,
            )
            promoted_writes += int(bool(write_result["promoted"]))

            shared_oracle_route[ledger_name][route_key] = dict(updated_shared[ledger_name])
            shared_oracle_full[ledger_name][full_key] = dict(updated_shared[ledger_name])

            next_j = (step + 1) % trace.trace_length
            shared_states[ledger_name] = update_memory_state(
                shared_states[ledger_name],
                fiber_moduli=trace.layer_primes,
                next_return_gap=trace.gaps[next_j],
            )
            shared_states[ledger_name] = initialize_memory_state(
                b=next_j % 35,
                phi=trace.phases[next_j],
                r=trace.r_depth,
                next_return_gap=trace.gaps[next_j],
            )

    return {
        "bundle_id": bundle["bundle_id"],
        "trace_source": bundle["trace_source"],
        "entity_count": len(bundle["entities"]),
        "route_reuse_fraction": route_reuse_events / total_object_steps if total_object_steps else 0.0,
        "query_hit_fraction_on_reuse": query_hits / query_attempts if query_attempts else 0.0,
        "per_entity_retrieval_accuracy": correct_local_queries / local_query_attempts if local_query_attempts else 0.0,
        "shared_ledger_retrieval_accuracy": correct_shared_queries / shared_query_attempts if shared_query_attempts else 0.0,
        "joint_snapshot_accuracy": correct_joint / joint_attempts if joint_attempts else 0.0,
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "promotion_step_fraction": promoted_writes / total_object_steps if total_object_steps else 0.0,
        "effective_joint_state_resolution_fraction": mean(
            [
                correct_local_queries / local_query_attempts if local_query_attempts else 0.0,
                correct_shared_queries / shared_query_attempts if shared_query_attempts else 0.0,
                correct_joint / joint_attempts if joint_attempts else 0.0,
                1.0 - (promoted_writes / total_object_steps if total_object_steps else 0.0),
            ]
        ),
        "route_decision_instability": 0.0,
    }


def summarize_rows(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "bundle_id": "ALL",
        "trace_source": "ALL",
        "entity_count": mean([float(row["entity_count"]) for row in rows]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows]),
        "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in rows]),
        "per_entity_retrieval_accuracy": mean([float(row["per_entity_retrieval_accuracy"]) for row in rows]),
        "shared_ledger_retrieval_accuracy": mean([float(row["shared_ledger_retrieval_accuracy"]) for row in rows]),
        "joint_snapshot_accuracy": mean([float(row["joint_snapshot_accuracy"]) for row in rows]),
        "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_joint_state_resolution_fraction": mean(
            [float(row["effective_joint_state_resolution_fraction"]) for row in rows]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    bundles = build_entity_bundles()
    rows = [run_bundle(bundle) for bundle in bundles]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_router_memory_workspace_experiment_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
