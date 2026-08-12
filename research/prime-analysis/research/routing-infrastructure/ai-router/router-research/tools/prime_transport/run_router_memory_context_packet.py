#!/usr/bin/env python3
"""Compare baseline and context-packet control policies on router-memory workspace."""

from __future__ import annotations

import csv
from dataclasses import dataclass
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
from run_router_memory_agent_loop import (
    apply_action,
    choose_action,
    default_ledger,
    run_bundle as run_baseline_bundle,
)
from run_router_memory_workspace_experiment import (
    build_dependency_ledger,
    build_task_ledger,
    build_world_ledger,
    mean,
    write_rows,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_context_packet.csv"


@dataclass
class ContextPacket:
    route_key: str
    entity_name: str
    intent: str
    payload: dict[str, object]
    context_summary: dict[str, object]


def derive_intent(
    entity_name: str,
    record: dict[str, object],
    task_ledger: dict[str, object],
    world_ledger: dict[str, object],
    dependency_ledger: dict[str, object],
) -> str:
    dependencies = dependency_ledger.get("dependencies", {}).get(entity_name, [])
    open_tasks = set(task_ledger.get("open", []))
    active_world = set(world_ledger.get("open", []))

    if not record.get("goal") and entity_name in open_tasks:
        return "update"
    if dependencies:
        return "resolve"
    if record.get("world") or entity_name in active_world:
        return "update"
    return "read"


def build_context_packet(
    *,
    route_key: str,
    entity_name: str,
    local_record: dict[str, object],
    task_ledger: dict[str, object],
    world_ledger: dict[str, object],
    dependency_ledger: dict[str, object],
) -> ContextPacket:
    dependencies = dependency_ledger.get("dependencies", {}).get(entity_name, [])
    context_summary = {
        "local_record_snapshot": dict(local_record),
        "task_fields": {
            "is_open": entity_name in set(task_ledger.get("open", [])),
            "is_claimed": entity_name in set(task_ledger.get("claimed", [])),
            "is_completed": entity_name in set(task_ledger.get("completed", [])),
            "shared_revision": task_ledger.get("shared_revision", 0),
        },
        "world_fields": {
            "world_active": entity_name in set(world_ledger.get("open", [])),
            "combat_hot": entity_name in set(world_ledger.get("claimed", [])),
            "pressure_hot": entity_name in set(world_ledger.get("completed", [])),
            "shared_revision": world_ledger.get("shared_revision", 0),
        },
        "dependency_summary": {
            "dependencies": list(dependencies),
            "dependency_count": len(dependencies),
            "shared_revision": dependency_ledger.get("shared_revision", 0),
        },
    }
    return ContextPacket(
        route_key=route_key,
        entity_name=entity_name,
        intent=derive_intent(entity_name, local_record, task_ledger, world_ledger, dependency_ledger),
        payload=dict(local_record),
        context_summary=context_summary,
    )


def choose_action_from_packet(packet: ContextPacket) -> str:
    record = packet.payload
    task_fields = packet.context_summary["task_fields"]
    world_fields = packet.context_summary["world_fields"]
    dependency_summary = packet.context_summary["dependency_summary"]
    dependencies = dependency_summary["dependencies"]

    if packet.intent == "update" and not record.get("goal") and task_fields["is_open"]:
        return "claim_goal"
    if packet.intent == "resolve" and record.get("goal") and not record.get("world") and "world_ledger" in dependencies:
        return "sync_world"
    if packet.intent == "update" and record.get("world") and not record.get("combat") and world_fields["world_active"]:
        return "engage_combat"
    if packet.intent == "resolve" and record.get("combat") and not record.get("pressure") and "task_ledger" in dependencies:
        return "stabilize_pressure"
    if packet.intent == "update" and record.get("pressure") and world_fields["pressure_hot"] and not task_fields["is_completed"]:
        return "mark_complete"
    if task_fields["is_claimed"] and not task_fields["is_completed"]:
        return "hold_claim"
    if world_fields["combat_hot"]:
        return "hold_combat"
    return "idle"


def choose_action_from_packet_ranked(packet: ContextPacket) -> str:
    """Priority-ranked controller over the same unchanged context packet."""
    record = packet.payload
    task_fields = packet.context_summary["task_fields"]
    world_fields = packet.context_summary["world_fields"]
    dependency_summary = packet.context_summary["dependency_summary"]
    dependencies = set(dependency_summary["dependencies"])
    dependency_count = int(dependency_summary["dependency_count"])

    candidates: list[tuple[int, str]] = []
    if not record.get("goal") and task_fields["is_open"]:
        candidates.append((100, "claim_goal"))
    if record.get("pressure") and world_fields["pressure_hot"] and not task_fields["is_completed"]:
        candidates.append((95, "mark_complete"))
    if record.get("combat") and not record.get("pressure") and "task_ledger" in dependencies:
        candidates.append((90 + dependency_count, "stabilize_pressure"))
    if record.get("goal") and not record.get("world") and "world_ledger" in dependencies:
        candidates.append((80 + dependency_count, "sync_world"))
    if record.get("world") and not record.get("combat") and world_fields["world_active"]:
        candidates.append((70, "engage_combat"))
    if task_fields["is_claimed"] and not task_fields["is_completed"]:
        candidates.append((30, "hold_claim"))
    if world_fields["combat_hot"]:
        candidates.append((20, "hold_combat"))
    if not candidates:
        return "idle"
    candidates.sort(reverse=True)
    return candidates[0][1]


def attractor_identity(packet: ContextPacket) -> str:
    """Small converged-control neighborhood label derived from packet summary."""
    record = packet.payload
    task_fields = packet.context_summary["task_fields"]
    world_fields = packet.context_summary["world_fields"]
    dependency_summary = packet.context_summary["dependency_summary"]
    stage = (
        int(bool(record.get("goal"))),
        int(bool(record.get("world"))),
        int(bool(record.get("combat"))),
        int(bool(record.get("pressure"))),
    )
    task_shape = (
        int(bool(task_fields["is_open"])),
        int(bool(task_fields["is_claimed"])),
        int(bool(task_fields["is_completed"])),
    )
    world_shape = (
        int(bool(world_fields["world_active"])),
        int(bool(world_fields["combat_hot"])),
        int(bool(world_fields["pressure_hot"])),
    )
    return (
        f"anchor:intent={packet.intent}:stage={stage}:deps={dependency_summary['dependency_count']}:"
        f"task={task_shape}:world={world_shape}"
    )


def run_context_packet_bundle(
    bundle: dict[str, object],
    *,
    policy_name: str,
    chooser,
) -> dict[str, object]:
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

        expected_shared = {
            "task_ledger": build_task_ledger(current_entity_records, step + 1),
            "world_ledger": build_world_ledger(current_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(current_entity_records, step + 1),
        }
        derived_predicted_shared = {
            "task_ledger": build_task_ledger(predicted_entity_records, step + 1),
            "world_ledger": build_world_ledger(predicted_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(predicted_entity_records, step + 1),
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

        context_shared = {}
        for ledger_name in ("task_ledger", "world_ledger", "dependency_ledger"):
            if predicted_shared[ledger_name] == default_ledger(ledger_name):
                context_shared[ledger_name] = derived_predicted_shared[ledger_name]
            else:
                context_shared[ledger_name] = predicted_shared[ledger_name]

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
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            packet = build_context_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=predicted_entity_records[entity_name],
                task_ledger=context_shared["task_ledger"],
                world_ledger=context_shared["world_ledger"],
                dependency_ledger=context_shared["dependency_ledger"],
            )
            chosen_action = chooser(packet)
            action_attempts += 1
            if chosen_action == oracle_action:
                correct_actions += 1
            else:
                step_actions_correct = False
                step_all_correct = False

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
            "task_ledger": build_task_ledger(updated_entity_records, step + 1),
            "world_ledger": build_world_ledger(updated_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(updated_entity_records, step + 1),
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
        "policy": policy_name,
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


def run_attractor_packet_bundle(bundle: dict[str, object]) -> dict[str, object]:
    local_layers = {}
    local_states = {}
    local_oracle_route = {}
    local_oracle_full = {}

    shared_layers = {}
    shared_states = {}
    shared_oracle_route = {}
    shared_oracle_full = {}
    attractor_cache: dict[str, dict[str, dict[str, object]]] = {}

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
        shared_predictions_by_entity: dict[str, dict[str, dict[str, object]]] = {}
        shared_predictions_all = {}
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

        expected_shared = {
            "task_ledger": build_task_ledger(current_entity_records, step + 1),
            "world_ledger": build_world_ledger(current_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(current_entity_records, step + 1),
        }
        derived_predicted_shared = {
            "task_ledger": build_task_ledger(predicted_entity_records, step + 1),
            "world_ledger": build_world_ledger(predicted_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(predicted_entity_records, step + 1),
        }

        # Shared queries stay unchanged at the memory layer; attractor identity only
        # decides whether the controller can reuse a converged shared neighborhood.
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
                shared_predictions_all[ledger_name] = expected_record
            else:
                shared_predictions_all[ledger_name] = default_ledger(ledger_name)

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
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            seed_packet = build_context_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=predicted_entity_records[entity_name],
                task_ledger=derived_predicted_shared["task_ledger"],
                world_ledger=derived_predicted_shared["world_ledger"],
                dependency_ledger=derived_predicted_shared["dependency_ledger"],
            )
            anchor = attractor_identity(seed_packet)
            cached_shared = attractor_cache.get(anchor)
            if cached_shared is not None:
                context_shared = cached_shared
            else:
                context_shared = {
                    ledger_name: (
                        derived_predicted_shared[ledger_name]
                        if shared_predictions_all[ledger_name] == default_ledger(ledger_name)
                        else shared_predictions_all[ledger_name]
                    )
                    for ledger_name in ("task_ledger", "world_ledger", "dependency_ledger")
                }
                attractor_cache[anchor] = {
                    "task_ledger": dict(context_shared["task_ledger"]),
                    "world_ledger": dict(context_shared["world_ledger"]),
                    "dependency_ledger": dict(context_shared["dependency_ledger"]),
                }
            shared_predictions_by_entity[entity_name] = context_shared

            packet = build_context_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=predicted_entity_records[entity_name],
                task_ledger=context_shared["task_ledger"],
                world_ledger=context_shared["world_ledger"],
                dependency_ledger=context_shared["dependency_ledger"],
            )
            chosen_action = choose_action_from_packet_ranked(packet)
            action_attempts += 1
            if chosen_action == oracle_action:
                correct_actions += 1
            else:
                step_actions_correct = False
                step_all_correct = False

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
            "task_ledger": build_task_ledger(updated_entity_records, step + 1),
            "world_ledger": build_world_ledger(updated_entity_records, step + 1),
            "dependency_ledger": build_dependency_ledger(updated_entity_records, step + 1),
        }

        if len(updated_entity_records) >= 2:
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

        # Refresh attractor cache from exact shared state after the step.
        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue
            route_key = f"base_gap:r={trace.r_depth}:b={step % 35}:gap={trace.gaps[step]}"
            packet = build_context_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=updated_entity_records[entity_name],
                task_ledger=updated_shared["task_ledger"],
                world_ledger=updated_shared["world_ledger"],
                dependency_ledger=updated_shared["dependency_ledger"],
            )
            attractor_cache[attractor_identity(packet)] = {
                "task_ledger": dict(updated_shared["task_ledger"]),
                "world_ledger": dict(updated_shared["world_ledger"]),
                "dependency_ledger": dict(updated_shared["dependency_ledger"]),
            }

    return {
        "policy": "attractor_identity",
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


def summarize_baseline(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "policy": "baseline",
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
    baseline_rows = [run_baseline_bundle(bundle) for bundle in bundles]
    context_rows = [
        run_context_packet_bundle(bundle, policy_name="context_packet", chooser=choose_action_from_packet)
        for bundle in bundles
    ]
    improved_rows = [
        run_context_packet_bundle(bundle, policy_name="improved_packet", chooser=choose_action_from_packet_ranked)
        for bundle in bundles
    ]
    attractor_rows = [run_attractor_packet_bundle(bundle) for bundle in bundles]
    rows = [
        summarize_baseline(baseline_rows),
        {
            "policy": "context_packet",
            "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in context_rows]),
            "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in context_rows]),
            "per_entity_retrieval_accuracy": mean([float(row["per_entity_retrieval_accuracy"]) for row in context_rows]),
            "shared_ledger_retrieval_accuracy": mean([float(row["shared_ledger_retrieval_accuracy"]) for row in context_rows]),
            "action_correctness": mean([float(row["action_correctness"]) for row in context_rows]),
            "joint_control_loop_correctness": mean([float(row["joint_control_loop_correctness"]) for row in context_rows]),
            "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in context_rows]),
            "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in context_rows]),
            "effective_joint_state_resolution_fraction": mean(
                [float(row["effective_joint_state_resolution_fraction"]) for row in context_rows]
            ),
            "route_decision_instability": 0.0,
        },
        {
            "policy": "improved_packet",
            "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in improved_rows]),
            "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in improved_rows]),
            "per_entity_retrieval_accuracy": mean([float(row["per_entity_retrieval_accuracy"]) for row in improved_rows]),
            "shared_ledger_retrieval_accuracy": mean([float(row["shared_ledger_retrieval_accuracy"]) for row in improved_rows]),
            "action_correctness": mean([float(row["action_correctness"]) for row in improved_rows]),
            "joint_control_loop_correctness": mean([float(row["joint_control_loop_correctness"]) for row in improved_rows]),
            "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in improved_rows]),
            "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in improved_rows]),
            "effective_joint_state_resolution_fraction": mean(
                [float(row["effective_joint_state_resolution_fraction"]) for row in improved_rows]
            ),
            "route_decision_instability": 0.0,
        },
        {
            "policy": "attractor_identity",
            "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in attractor_rows]),
            "query_hit_fraction_on_reuse": mean([float(row["query_hit_fraction_on_reuse"]) for row in attractor_rows]),
            "per_entity_retrieval_accuracy": mean([float(row["per_entity_retrieval_accuracy"]) for row in attractor_rows]),
            "shared_ledger_retrieval_accuracy": mean([float(row["shared_ledger_retrieval_accuracy"]) for row in attractor_rows]),
            "action_correctness": mean([float(row["action_correctness"]) for row in attractor_rows]),
            "joint_control_loop_correctness": mean([float(row["joint_control_loop_correctness"]) for row in attractor_rows]),
            "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in attractor_rows]),
            "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in attractor_rows]),
            "effective_joint_state_resolution_fraction": mean(
                [float(row["effective_joint_state_resolution_fraction"]) for row in attractor_rows]
            ),
            "route_decision_instability": 0.0,
        },
    ]
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_router_memory_context_packet_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
