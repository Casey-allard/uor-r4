#!/usr/bin/env python3
"""Compare coordination controllers with explicit conflict arbitration."""

from __future__ import annotations

from pathlib import Path

from run_coordination_lookahead import lookahead_policy
from run_coordination_policy_improvement import (
    build_coordination_bundles,
    build_packet,
    improved_packet_policy,
    run_policy_bundle,
    summarize_rows,
)
from run_router_memory_context_packet import attractor_identity
from run_router_memory_workspace_experiment import write_rows


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_coordination_conflict_arbitration.csv"


def arbitration_policy(packet) -> str:
    """Shared-conflict controller on the unchanged packet + attractor surface."""
    record = packet.payload
    task_fields = packet.context_summary["task_fields"]
    world_fields = packet.context_summary["world_fields"]
    dependency_summary = packet.context_summary["dependency_summary"]
    full_task = packet.context_summary["task_fields_full"]
    full_world = packet.context_summary["world_fields_full"]

    dependencies = set(dependency_summary["dependencies"])
    dependency_count = int(dependency_summary["dependency_count"])
    anchor = attractor_identity(packet)

    local_goal = bool(record.get("goal"))
    local_world = bool(record.get("world"))
    local_combat = bool(record.get("combat"))
    local_pressure = bool(record.get("pressure"))

    is_open = bool(task_fields["is_open"]) and not local_goal
    is_claimed = bool(task_fields["is_claimed"]) or (local_goal and not local_world)
    is_completed = bool(task_fields["is_completed"]) or (local_goal and local_world and local_pressure)

    world_active = bool(world_fields["world_active"]) or local_world
    combat_hot = bool(world_fields["combat_hot"]) or local_combat
    pressure_hot = bool(world_fields["pressure_hot"]) or local_pressure

    open_count = len(full_task.get("open", []))
    claimed_count = len(full_task.get("claimed", []))
    completed_count = len(full_task.get("completed", []))
    active_world_count = len(full_world.get("open", []))
    combat_hot_count = len(full_world.get("claimed", []))
    pressure_hot_count = len(full_world.get("completed", []))

    candidates: list[tuple[float, str]] = []

    if local_pressure and pressure_hot and not is_completed:
        # Completion clears both claimed and pressure burden.
        score = 8.0 + pressure_hot_count + 0.5 * claimed_count
        candidates.append((score, "mark_complete"))

    if local_combat and not local_pressure and "task_ledger" in dependencies:
        # Pressure stabilization resolves the hottest blocked combat states.
        score = 7.0 + combat_hot_count - pressure_hot_count + 0.25 * dependency_count
        candidates.append((score, "stabilize_pressure"))

    if local_goal and not local_world and "world_ledger" in dependencies:
        # World sync is the main resolution step for a crowded claimed ledger.
        score = 6.0 + claimed_count - active_world_count + 0.25 * dependency_count
        candidates.append((score, "sync_world"))

    if local_world and not local_combat and world_active:
        score = 5.0 + active_world_count - combat_hot_count
        candidates.append((score, "engage_combat"))

    if not local_goal and is_open:
        # Only claim aggressively when open backlog materially exceeds claims.
        score = 4.0 + max(0, open_count - claimed_count)
        candidates.append((score, "claim_goal"))

    if (
        local_goal
        and not local_world
        and is_claimed
        and dependency_count >= 2
        and "world_ledger" not in dependencies
        and not world_active
        and claimed_count > max(1, active_world_count + 1)
        and "stage=(1, 0, 0, 0)" in anchor
    ):
        score = 3.0 + (claimed_count - active_world_count)
        candidates.append((score, "reassign_claim"))

    if (
        local_combat
        and not local_pressure
        and dependency_count >= 2
        and "task_ledger" not in dependencies
        and not pressure_hot
        and combat_hot_count > max(1, pressure_hot_count + 1)
        and "stage=(1, 1, 1, 0)" in anchor
    ):
        score = 3.0 + (combat_hot_count - pressure_hot_count)
        candidates.append((score, "handoff_dependency"))

    if is_claimed and not is_completed:
        candidates.append((1.0, "hold_claim"))
    if combat_hot:
        candidates.append((0.9, "hold_combat"))
    if not candidates:
        return "idle"
    candidates.sort(reverse=True)
    return candidates[0][1]


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

    rows = (
        improved_rows
        + [summarize_rows("improved_coordination_policy", improved_rows)]
        + lookahead_rows
        + [summarize_rows("lookahead_coordination_policy", lookahead_rows)]
        + arbitration_rows
        + [summarize_rows("conflict_arbitration_policy", arbitration_rows)]
    )
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_coordination_conflict_arbitration_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
