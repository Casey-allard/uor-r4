#!/usr/bin/env python3
"""Bounded agent-style control loop on top of the unchanged workspace memory."""

from __future__ import annotations

import csv
from pathlib import Path

from router_memory_layer import (
    initialize_memory_layer,
    initialize_memory_state,
    query_memory,
    update_memory_state,
    write_memory,
)
from run_multi_entity_router_memory_experiment import (
    build_entity_bundles,
    empty_record,
    full_key_for_step,
    serialize_record,
)
from run_router_memory_workspace_experiment import (
    build_dependency_ledger,
    build_task_ledger,
    build_world_ledger,
    mean,
    write_rows,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_agent_loop.csv"
SHARED_LEDGERS = ("task_ledger", "world_ledger", "dependency_ledger")


def default_ledger(ledger_name: str) -> dict[str, object]:
    if ledger_name == "dependency_ledger":
        return {"ledger": ledger_name, "dependencies": {}, "shared_revision": 0}
    return {
        "ledger": ledger_name,
        "open": [],
        "claimed": [],
        "completed": [],
        "shared_revision": 0,
    }


def choose_action(
    entity_name: str,
    record: dict[str, object],
    task_ledger: dict[str, object],
    world_ledger: dict[str, object],
    dependency_ledger: dict[str, object],
) -> str:
    dependencies = dependency_ledger.get("dependencies", {}).get(entity_name, [])
    open_tasks = set(task_ledger.get("open", []))
    claimed_tasks = set(task_ledger.get("claimed", []))
    completed_tasks = set(task_ledger.get("completed", []))
    active_world = set(world_ledger.get("open", []))
    combat_hot = set(world_ledger.get("claimed", []))
    pressure_hot = set(world_ledger.get("completed", []))

    if not record.get("goal") and entity_name in open_tasks:
        return "claim_goal"
    if record.get("goal") and not record.get("world") and "world_ledger" in dependencies:
        return "sync_world"
    if record.get("world") and not record.get("combat") and entity_name in active_world:
        return "engage_combat"
    if record.get("combat") and not record.get("pressure") and "task_ledger" in dependencies:
        return "stabilize_pressure"
    if record.get("pressure") and entity_name in pressure_hot and entity_name not in completed_tasks:
        return "mark_complete"
    if record.get("goal") and entity_name in claimed_tasks and entity_name not in completed_tasks:
        return "hold_claim"
    if record.get("combat") and entity_name in combat_hot:
        return "hold_combat"
    return "idle"


def apply_action(
    entity_name: str,
    trace,
    step: int,
    base_record: dict[str, object],
    action: str,
) -> dict[str, object]:
    record = dict(base_record)
    gap = trace.gaps[step]
    phase_head = trace.phases[step][0] if trace.phases[step] else 0
    spin_mass = sum(trace.spins[step][: min(3, len(trace.spins[step]))])

    if action == "claim_goal":
        record["goal"] = f"{entity_name}:goal@g{gap}:p{phase_head}"
    elif action == "sync_world":
        record["world"] = f"{entity_name}:world@g{gap}:s{spin_mass}"
    elif action == "engage_combat":
        record["combat"] = f"{entity_name}:combat@g{gap}:p{phase_head}"
    elif action == "stabilize_pressure":
        record["pressure"] = f"{entity_name}:pressure@g{gap}:s{spin_mass}"
    elif action == "mark_complete":
        record["goal"] = f"{record.get('goal', '')}:done"
    record["last_action"] = action
    record["revision"] = int(record.get("revision", 0)) + 1
    return record


def run_bundle(bundle: dict[str, object]) -> dict[str, object]:
    local_layers = {}
    local_states = {}
    local_oracle_route = {}
    local_oracle_full = {}

    shared_layers = {}
    shared_states = {}
    shared_oracle_route = {}
    shared_oracle_full = {}

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
        local_oracle_route[entity_name] = {}
        local_oracle_full[entity_name] = {}

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
        shared_oracle_route[ledger_name] = {}
        shared_oracle_full[ledger_name] = {}

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
    action_attempts = 0
    correct_actions = 0
    control_attempts = 0
    correct_control_steps = 0

    for step in range(max_steps):
        current_entity_records: dict[str, dict[str, object]] = {}
        predicted_entity_records: dict[str, dict[str, object]] = {}
        expected_shared = {}
        predicted_shared = {}
        step_all_correct = True

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
                if read_result.get("predicted_payload") == serialize_record(expected_record):
                    correct_local_queries += 1
                else:
                    step_all_correct = False
                current_entity_records[entity_name] = expected_record
                predicted_entity_records[entity_name] = expected_record
            else:
                default_record = empty_record(entity_name)
                current_entity_records[entity_name] = default_record
                predicted_entity_records[entity_name] = default_record

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
                    expected_record = dict(shared_oracle_full[ledger_name].get(full_key, shared_oracle_route[ledger_name][route_key]))
                else:
                    expected_record = dict(shared_oracle_route[ledger_name][route_key])
                if read_result.get("predicted_payload") == serialize_record(expected_record):
                    correct_shared_queries += 1
                else:
                    step_all_correct = False
                predicted_shared[ledger_name] = expected_record
            else:
                predicted_shared[ledger_name] = default_ledger(ledger_name)

        step_actions_correct = True
        updated_entity_records = {}
        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

            oracle_action = choose_action(
                entity_name,
                current_entity_records[entity_name],
                expected_shared["task_ledger"],
                expected_shared["world_ledger"],
                expected_shared["dependency_ledger"],
            )
            chosen_action = choose_action(
                entity_name,
                predicted_entity_records[entity_name],
                predicted_shared["task_ledger"],
                predicted_shared["world_ledger"],
                predicted_shared["dependency_ledger"],
            )
            action_attempts += 1
            if chosen_action == oracle_action:
                correct_actions += 1
            else:
                step_actions_correct = False
                step_all_correct = False

            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            full_key = full_key_for_step(step % 35, trace.spins[step])
            unresolved_fraction = trace.unresolved_by_route[route_key]

            base_record = dict(
                local_oracle_full[entity_name].get(
                    full_key,
                    local_oracle_route[entity_name].get(route_key, empty_record(entity_name)),
                )
            )
            updated_record = apply_action(entity_name, trace, step, base_record, oracle_action)
            updated_entity_records[entity_name] = updated_record
            payload = serialize_record(updated_record)

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

            local_oracle_route[entity_name][route_key] = dict(updated_record)
            local_oracle_full[entity_name][full_key] = dict(updated_record)

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

        updated_shared = {
            "task_ledger": build_task_ledger(updated_entity_records, shared_revision),
            "world_ledger": build_world_ledger(updated_entity_records, shared_revision),
            "dependency_ledger": build_dependency_ledger(updated_entity_records, shared_revision),
        }

        if len(updated_entity_records) >= 2 and active_shared_reads >= 2:
            control_attempts += 1
            if step_all_correct and step_actions_correct:
                correct_control_steps += 1

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
        "action_correctness": correct_actions / action_attempts if action_attempts else 0.0,
        "joint_control_loop_correctness": correct_control_steps / control_attempts if control_attempts else 0.0,
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "promotion_step_fraction": promoted_writes / total_object_steps if total_object_steps else 0.0,
        "effective_joint_state_resolution_fraction": mean(
            [
                correct_local_queries / local_query_attempts if local_query_attempts else 0.0,
                correct_shared_queries / shared_query_attempts if shared_query_attempts else 0.0,
                correct_actions / action_attempts if action_attempts else 0.0,
                correct_control_steps / control_attempts if control_attempts else 0.0,
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
        "action_correctness": mean([float(row["action_correctness"]) for row in rows]),
        "joint_control_loop_correctness": mean([float(row["joint_control_loop_correctness"]) for row in rows]),
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
    print(f"prime_transport_router_memory_agent_loop_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
