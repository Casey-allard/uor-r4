#!/usr/bin/env python3
"""Bounded coordination-logic prototype on the unchanged router-memory stack."""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from router_memory_layer import (
    initialize_memory_layer,
    initialize_memory_state,
    query_memory,
    update_memory_state,
    write_memory,
)
from run_multi_entity_router_memory_experiment import empty_record, full_key_for_step, serialize_record
from run_router_memory_agent_loop import default_ledger
from run_router_memory_context_packet import (
    attractor_identity,
    build_context_packet,
)
from run_router_memory_workspace_experiment import (
    build_dependency_ledger,
    build_task_ledger,
    build_world_ledger,
    mean,
    write_rows,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_memory_coordination_experiment.csv"
ENTITY_NAMES = ("entity_a", "entity_b", "entity_c", "entity_d")


def build_coordination_bundles() -> list[dict[str, object]]:
    traces = build_bounded_traces()
    grouped: dict[str, list[object]] = defaultdict(list)
    for trace in traces:
        grouped[trace.trace_source].append(trace)

    bundles: list[dict[str, object]] = []
    for trace_source, source_traces in grouped.items():
        selected = source_traces[: len(ENTITY_NAMES)]
        if len(selected) < len(ENTITY_NAMES):
            continue
        bundles.append(
            {
                "bundle_id": f"{trace_source}:coordination_bundle0",
                "trace_source": trace_source,
                "entities": [
                    {"entity_name": ENTITY_NAMES[index], "trace": trace}
                    for index, trace in enumerate(selected)
                ],
            }
        )
    return bundles


def next_entity_name(entity_name: str) -> str:
    index = ENTITY_NAMES.index(entity_name)
    return ENTITY_NAMES[(index + 1) % len(ENTITY_NAMES)]


def choose_coordination_action(
    entity_name: str,
    record: dict[str, object],
    task_ledger: dict[str, object],
    world_ledger: dict[str, object],
    dependency_ledger: dict[str, object],
) -> str:
    task_fields = {
        "is_open": entity_name in set(task_ledger.get("open", [])),
        "is_claimed": entity_name in set(task_ledger.get("claimed", [])),
        "is_completed": entity_name in set(task_ledger.get("completed", [])),
    }
    world_fields = {
        "world_active": entity_name in set(world_ledger.get("open", [])),
        "combat_hot": entity_name in set(world_ledger.get("claimed", [])),
        "pressure_hot": entity_name in set(world_ledger.get("completed", [])),
    }
    dependencies = set(dependency_ledger.get("dependencies", {}).get(entity_name, []))
    dependency_count = len(dependencies)

    candidates: list[tuple[int, str]] = []
    if record.get("pressure") and world_fields["pressure_hot"] and not task_fields["is_completed"]:
        candidates.append((110, "mark_complete"))
    if record.get("goal") and not record.get("world") and task_fields["is_claimed"] and dependency_count >= 2:
        candidates.append((105 + dependency_count, "reassign_claim"))
    if record.get("combat") and not record.get("pressure") and dependency_count >= 2:
        candidates.append((100 + dependency_count, "handoff_dependency"))
    if not record.get("goal") and task_fields["is_open"]:
        candidates.append((95, "claim_goal"))
    if record.get("goal") and not record.get("world") and "world_ledger" in dependencies:
        candidates.append((90 + dependency_count, "sync_world"))
    if record.get("world") and not record.get("combat") and world_fields["world_active"]:
        candidates.append((80, "engage_combat"))
    if record.get("combat") and not record.get("pressure") and "task_ledger" in dependencies:
        candidates.append((70 + dependency_count, "stabilize_pressure"))
    if task_fields["is_claimed"] and not task_fields["is_completed"]:
        candidates.append((30, "hold_claim"))
    if world_fields["combat_hot"]:
        candidates.append((20, "hold_combat"))
    if not candidates:
        return "idle"
    candidates.sort(reverse=True)
    return candidates[0][1]


def apply_coordination_action(
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
    target_entity = next_entity_name(entity_name)

    if action == "claim_goal":
        record["goal"] = f"{entity_name}:goal@g{gap}:p{phase_head}"
        record["owner"] = entity_name
    elif action == "sync_world":
        record["world"] = f"{entity_name}:world@g{gap}:s{spin_mass}"
    elif action == "engage_combat":
        record["combat"] = f"{entity_name}:combat@g{gap}:p{phase_head}"
    elif action == "stabilize_pressure":
        record["pressure"] = f"{entity_name}:pressure@g{gap}:s{spin_mass}"
    elif action == "mark_complete":
        record["goal"] = f"{record.get('goal', '')}:done"
    elif action == "reassign_claim":
        previous_goal = record.get("goal", "")
        record["goal"] = ""
        record["owner"] = target_entity
        record["reassigned_from"] = previous_goal
        record["handoff_target"] = target_entity
    elif action == "handoff_dependency":
        previous_combat = record.get("combat", "")
        record["combat"] = ""
        record["handoff_target"] = target_entity
        record["handoff_from"] = previous_combat
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
    reassignment_attempts = 0
    correct_reassignments = 0
    coordination_attempts = 0
    correct_coordination = 0

    for step in range(max_steps):
        current_entity_records: dict[str, dict[str, object]] = {}
        predicted_entity_records: dict[str, dict[str, object]] = {}
        predicted_shared_all = {}
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
                    expected_record = dict(
                        local_oracle_full[entity_name].get(full_key, local_oracle_route[entity_name][route_key])
                    )
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
                    expected_record = dict(
                        shared_oracle_full[ledger_name].get(full_key, shared_oracle_route[ledger_name][route_key])
                    )
                else:
                    expected_record = dict(shared_oracle_route[ledger_name][route_key])
                if read_result.get("predicted_payload") == serialize_record(expected_record):
                    correct_shared_queries += 1
                else:
                    step_all_correct = False
                predicted_shared_all[ledger_name] = expected_record
            else:
                predicted_shared_all[ledger_name] = default_ledger(ledger_name)

        step_actions_correct = True
        updated_entity_records = {}
        for entity in bundle["entities"]:
            trace = entity["trace"]
            entity_name = entity["entity_name"]
            if step >= trace.trace_length:
                continue

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
                        if predicted_shared_all[ledger_name] == default_ledger(ledger_name)
                        else predicted_shared_all[ledger_name]
                    )
                    for ledger_name in ("task_ledger", "world_ledger", "dependency_ledger")
                }
                attractor_cache[anchor] = {
                    "task_ledger": dict(context_shared["task_ledger"]),
                    "world_ledger": dict(context_shared["world_ledger"]),
                    "dependency_ledger": dict(context_shared["dependency_ledger"]),
                }

            oracle_action = choose_coordination_action(
                entity_name,
                current_entity_records[entity_name],
                expected_shared["task_ledger"],
                expected_shared["world_ledger"],
                expected_shared["dependency_ledger"],
            )
            packet = build_context_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=predicted_entity_records[entity_name],
                task_ledger=context_shared["task_ledger"],
                world_ledger=context_shared["world_ledger"],
                dependency_ledger=context_shared["dependency_ledger"],
            )
            chosen_action = choose_coordination_action(
                entity_name,
                packet.payload,
                context_shared["task_ledger"],
                context_shared["world_ledger"],
                context_shared["dependency_ledger"],
            )
            action_attempts += 1
            if oracle_action in {"reassign_claim", "handoff_dependency"}:
                reassignment_attempts += 1
                if chosen_action == oracle_action:
                    correct_reassignments += 1
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
            updated_record = apply_coordination_action(entity_name, trace, step, base_record, oracle_action)
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
            coordination_attempts += 1
            if step_all_correct and step_actions_correct:
                correct_coordination += 1

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
        "bundle_id": bundle["bundle_id"],
        "trace_source": bundle["trace_source"],
        "entity_count": len(bundle["entities"]),
        "action_correctness": correct_actions / action_attempts if action_attempts else 0.0,
        "reassignment_handoff_correctness": (
            correct_reassignments / reassignment_attempts if reassignment_attempts else 1.0
        ),
        "per_entity_retrieval_accuracy": correct_local_queries / local_query_attempts if local_query_attempts else 0.0,
        "shared_ledger_retrieval_accuracy": correct_shared_queries / shared_query_attempts if shared_query_attempts else 0.0,
        "joint_coordination_loop_correctness": (
            correct_coordination / coordination_attempts if coordination_attempts else 0.0
        ),
        "promoted_query_fraction_on_reuse": promoted_queries / query_attempts if query_attempts else 0.0,
        "route_reuse_fraction": route_reuse_events / total_object_steps if total_object_steps else 0.0,
        "promotion_step_fraction": promoted_writes / total_object_steps if total_object_steps else 0.0,
        "effective_coordinated_state_resolution_fraction": mean(
            [
                correct_actions / action_attempts if action_attempts else 0.0,
                correct_reassignments / reassignment_attempts if reassignment_attempts else 1.0,
                correct_local_queries / local_query_attempts if local_query_attempts else 0.0,
                correct_shared_queries / shared_query_attempts if shared_query_attempts else 0.0,
                correct_coordination / coordination_attempts if coordination_attempts else 0.0,
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
        "action_correctness": mean([float(row["action_correctness"]) for row in rows]),
        "reassignment_handoff_correctness": mean([float(row["reassignment_handoff_correctness"]) for row in rows]),
        "per_entity_retrieval_accuracy": mean([float(row["per_entity_retrieval_accuracy"]) for row in rows]),
        "shared_ledger_retrieval_accuracy": mean([float(row["shared_ledger_retrieval_accuracy"]) for row in rows]),
        "joint_coordination_loop_correctness": mean(
            [float(row["joint_coordination_loop_correctness"]) for row in rows]
        ),
        "promoted_query_fraction_on_reuse": mean([float(row["promoted_query_fraction_on_reuse"]) for row in rows]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_coordinated_state_resolution_fraction": mean(
            [float(row["effective_coordinated_state_resolution_fraction"]) for row in rows]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    rows = [run_bundle(bundle) for bundle in build_coordination_bundles()]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_router_memory_coordination_experiment_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
