#!/usr/bin/env python3
"""Compare coordination controllers with a bounded coordination-frame layer."""

from __future__ import annotations

from pathlib import Path

from run_coordination_conflict_arbitration import arbitration_policy
from run_coordination_lookahead import lookahead_policy
from run_coordination_policy_improvement import (
    build_coordination_bundles,
    improved_packet_policy,
    run_policy_bundle,
    summarize_rows,
)
from run_router_memory_workspace_experiment import write_rows


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_coordination_frame_experiment.csv"


def make_coordination_frame_policy():
    """Small transaction-like controller over the unchanged packet surface."""
    active_frames: dict[str, dict[str, object]] = {}
    episode_counter = 0

    def clear_if_complete(entity_name: str, frame: dict[str, object], packet) -> bool:
        record = packet.payload
        if frame["kind"] == "goal_resolution":
            return bool(record.get("pressure"))
        if frame["kind"] == "pressure_resolution":
            return bool(record.get("pressure"))
        if frame["kind"] == "completion_push":
            return packet.context_summary["task_fields"]["is_completed"] or bool(record.get("pressure"))
        return False

    def packet_stage(packet) -> tuple[bool, bool, bool, bool]:
        record = packet.payload
        return (
            bool(record.get("goal")),
            bool(record.get("world")),
            bool(record.get("combat")),
            bool(record.get("pressure")),
        )

    def next_frame_action(frame: dict[str, object], packet) -> str:
        goal, world, combat, pressure = packet_stage(packet)
        task_fields = packet.context_summary["task_fields"]
        world_fields = packet.context_summary["world_fields"]
        dependencies = set(packet.context_summary["dependency_summary"]["dependencies"])

        if frame["kind"] in {"goal_resolution", "completion_push"}:
            if pressure and world_fields["pressure_hot"] and not task_fields["is_completed"]:
                return "mark_complete"
            if combat and not pressure and "task_ledger" in dependencies:
                return "stabilize_pressure"
            if world and not combat and (world_fields["world_active"] or frame["kind"] == "goal_resolution"):
                return "engage_combat"
            if goal and not world and "world_ledger" in dependencies:
                return "sync_world"
        if frame["kind"] == "pressure_resolution":
            if pressure and world_fields["pressure_hot"] and not task_fields["is_completed"]:
                return "mark_complete"
            if combat and not pressure and "task_ledger" in dependencies:
                return "stabilize_pressure"
        return "idle"

    def policy(packet) -> str:
        nonlocal episode_counter

        entity_name = packet.entity_name
        goal, world, combat, pressure = packet_stage(packet)
        task_fields = packet.context_summary["task_fields"]
        world_fields = packet.context_summary["world_fields"]
        dependency_summary = packet.context_summary["dependency_summary"]
        dependencies = set(dependency_summary["dependencies"])

        frame = active_frames.get(entity_name)
        if frame is not None:
            if clear_if_complete(entity_name, frame, packet):
                active_frames.pop(entity_name, None)
            else:
                frame_action = next_frame_action(frame, packet)
                if frame_action != "idle":
                    return frame_action
                active_frames.pop(entity_name, None)

        if goal and not world and "world_ledger" in dependencies:
            episode_counter += 1
            active_frames[entity_name] = {
                "episode_id": f"goal_resolution:{entity_name}:{episode_counter}",
                "kind": "goal_resolution",
                "entity_name": entity_name,
                "blocked_target": "world_ledger",
                "release_condition": "pressure_or_completion",
            }
            return "sync_world"

        if combat and not pressure and "task_ledger" in dependencies:
            episode_counter += 1
            active_frames[entity_name] = {
                "episode_id": f"pressure_resolution:{entity_name}:{episode_counter}",
                "kind": "pressure_resolution",
                "entity_name": entity_name,
                "blocked_target": "task_ledger",
                "release_condition": "pressure_or_completion",
            }
            return "stabilize_pressure"

        if world and not pressure and world_fields["world_active"]:
            episode_counter += 1
            active_frames[entity_name] = {
                "episode_id": f"completion_push:{entity_name}:{episode_counter}",
                "kind": "completion_push",
                "entity_name": entity_name,
                "blocked_target": "task_ledger",
                "release_condition": "completion",
            }
            if not combat:
                return "engage_combat"
            if not pressure and "task_ledger" in dependencies:
                return "stabilize_pressure"

        return improved_packet_policy(packet)

    return policy


def main() -> None:
    bundles = build_coordination_bundles()
    improved_rows = [
        run_policy_bundle(bundle, policy_name="improved_coordination_policy", chooser=improved_packet_policy)
        for bundle in bundles
    ]
    lookahead_rows = [
        run_policy_bundle(bundle, policy_name="lookahead_coordination_policy", chooser=lookahead_policy)
        for bundle in bundles
    ]
    arbitration_rows = [
        run_policy_bundle(bundle, policy_name="conflict_arbitration_policy", chooser=arbitration_policy)
        for bundle in bundles
    ]
    frame_rows = [
        run_policy_bundle(
            bundle,
            policy_name="coordination_frame_policy",
            chooser=make_coordination_frame_policy(),
        )
        for bundle in bundles
    ]

    rows = (
        improved_rows
        + [summarize_rows("improved_coordination_policy", improved_rows)]
        + lookahead_rows
        + [summarize_rows("lookahead_coordination_policy", lookahead_rows)]
        + arbitration_rows
        + [summarize_rows("conflict_arbitration_policy", arbitration_rows)]
        + frame_rows
        + [summarize_rows("coordination_frame_policy", frame_rows)]
    )
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_coordination_frame_experiment_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
