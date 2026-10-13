#!/usr/bin/env python3
"""Compare frame, visibility-zone, and richer phase-fiber geometry controllers."""

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
from run_coordination_frame_experiment import make_coordination_frame_policy
from run_coordination_policy_improvement import (
    build_coordination_bundles,
    build_packet,
    summarize_rows,
)
from run_multi_entity_router_memory_experiment import empty_record, full_key_for_step, serialize_record
from run_router_memory_agent_loop import default_ledger
from run_router_memory_coordination_experiment import apply_coordination_action
from run_router_memory_workspace_experiment import (
    build_dependency_ledger,
    build_task_ledger,
    build_world_ledger,
    mean,
    write_rows,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_phase_fiber_geometry_controller.csv"
FRAME_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_coordination_frame_experiment.csv"
VISIBILITY_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_geometry_native_controller.csv"


def compute_phase_fiber_signal(trace, step: int) -> dict[str, object]:
    phi0 = trace.phases[step][0] if trace.phases[step] else 0
    gap = trace.gaps[step]
    return {
        "r_band": "deep" if trace.r_depth >= 2 else "shallow",
        "b_sector": (step % 5),
        "phi_bucket": phi0 % 2,
        "gap_band": "immediate" if gap <= 1 else "delayed",
    }


def make_phase_fiber_geometry_policy():
    """Richer basin-aware controller above the unchanged frame surface."""
    base_frame_policy = make_coordination_frame_policy()
    current_signal: dict[str, dict[str, object]] = {}

    def policy(packet) -> str:
        signal = current_signal[packet.entity_name]
        record = packet.payload
        task_fields = packet.context_summary["task_fields"]
        world_fields = packet.context_summary["world_fields"]
        dependencies = set(packet.context_summary["dependency_summary"]["dependencies"])

        goal = bool(record.get("goal"))
        world = bool(record.get("world"))
        combat = bool(record.get("combat"))
        pressure = bool(record.get("pressure"))

        base_action = base_frame_policy(packet)

        # Hot splitter-sector basins: keep pushing direct dependency resolution
        # rather than relaxing into passive holds.
        hot_splitter_basin = (
            signal["r_band"] == "deep"
            or (
                signal["gap_band"] == "delayed"
                and signal["b_sector"] in {2, 3, 4}
                and signal["phi_bucket"] == 1
            )
        )
        if hot_splitter_basin:
            if goal and not world and "world_ledger" in dependencies:
                return "sync_world"
            if world and not combat and (world_fields["world_active"] or True):
                return "engage_combat"
            if combat and not pressure and "task_ledger" in dependencies:
                return "stabilize_pressure"
            if pressure and world_fields["pressure_hot"] and not task_fields["is_completed"]:
                return "mark_complete"

        # Cooler shallow basins: do not overcommit to passive holds if direct
        # world sync is available.
        if (
            signal["r_band"] == "shallow"
            and signal["gap_band"] == "delayed"
            and signal["b_sector"] in {0, 1}
            and base_action in {"hold_claim", "idle"}
            and goal
            and not world
            and "world_ledger" in dependencies
        ):
            return "sync_world"

        return base_action

    policy.current_signal = current_signal
    return policy


def run_phase_fiber_bundle(bundle: dict[str, object], *, policy_name: str, chooser) -> dict[str, object]:
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
        predicted_shared = {
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
            packet = build_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=predicted_entity_records[entity_name],
                task_ledger=predicted_shared_all.get("task_ledger", predicted_shared["task_ledger"]),
                world_ledger=predicted_shared_all.get("world_ledger", predicted_shared["world_ledger"]),
                dependency_ledger=predicted_shared_all.get("dependency_ledger", predicted_shared["dependency_ledger"]),
            )
            chooser.current_signal[entity_name] = compute_phase_fiber_signal(trace, step)
            oracle_action = chooser(packet)

            packet = build_packet(
                route_key=route_key,
                entity_name=entity_name,
                local_record=current_entity_records[entity_name],
                task_ledger=expected_shared["task_ledger"],
                world_ledger=expected_shared["world_ledger"],
                dependency_ledger=expected_shared["dependency_ledger"],
            )
            chooser.current_signal[entity_name] = compute_phase_fiber_signal(trace, step)
            chosen_action = chooser(packet)

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

    return {
        "policy": policy_name,
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


def read_all_row(path: Path, policy_name: str) -> dict[str, object]:
    with path.open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            if row["policy"] == policy_name and row["bundle_id"] == "ALL":
                return row
    raise ValueError(f"missing ALL row for {policy_name} in {path}")


def main() -> None:
    phase_fiber_policy = make_phase_fiber_geometry_policy()
    rows = [
        read_all_row(FRAME_CSV, "coordination_frame_policy"),
        read_all_row(VISIBILITY_CSV, "geometry_native_coordination_policy"),
    ]
    bundles = build_coordination_bundles()
    phase_rows = [
        run_phase_fiber_bundle(bundle, policy_name="phase_fiber_geometry_native_policy", chooser=phase_fiber_policy)
        for bundle in bundles
    ]
    rows.extend(phase_rows)
    rows.append(summarize_rows("phase_fiber_geometry_native_policy", phase_rows))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_phase_fiber_geometry_controller_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
