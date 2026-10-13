#!/usr/bin/env python3
"""Compare improved local and bounded lookahead coordination policies."""

from __future__ import annotations

from pathlib import Path

from run_coordination_policy_improvement import (
    build_packet,
    build_coordination_bundles,
    improved_packet_policy,
    run_policy_bundle,
    summarize_rows,
)
from run_router_memory_context_packet import attractor_identity
from run_router_memory_workspace_experiment import write_rows


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_coordination_lookahead.csv"


ACTION_VALUE = {
    "idle": 0.0,
    "hold_claim": 0.2,
    "hold_combat": 0.2,
    "claim_goal": 1.2,
    "sync_world": 1.5,
    "engage_combat": 1.7,
    "stabilize_pressure": 2.0,
    "mark_complete": 2.3,
    "reassign_claim": 0.7,
    "handoff_dependency": 0.8,
}


def abstract_state(packet) -> dict[str, object]:
    record = packet.payload
    task_fields = packet.context_summary["task_fields"]
    world_fields = packet.context_summary["world_fields"]
    dependency_summary = packet.context_summary["dependency_summary"]

    local_goal = bool(record.get("goal"))
    local_world = bool(record.get("world"))
    local_combat = bool(record.get("combat"))
    local_pressure = bool(record.get("pressure"))

    return {
        "goal": local_goal,
        "world": local_world,
        "combat": local_combat,
        "pressure": local_pressure,
        "is_open": bool(task_fields["is_open"]) and not local_goal,
        "is_claimed": bool(task_fields["is_claimed"]) or (local_goal and not local_world),
        "is_completed": bool(task_fields["is_completed"]) or (local_goal and local_world and local_pressure),
        "world_active": bool(world_fields["world_active"]) or local_world,
        "combat_hot": bool(world_fields["combat_hot"]) or local_combat,
        "pressure_hot": bool(world_fields["pressure_hot"]) or local_pressure,
        "dependencies": set(dependency_summary["dependencies"]),
        "dependency_count": int(dependency_summary["dependency_count"]),
        "anchor": attractor_identity(packet),
    }


def candidate_actions(state: dict[str, object]) -> list[str]:
    actions: list[str] = []
    if state["pressure"] and state["pressure_hot"] and not state["is_completed"]:
        actions.append("mark_complete")
    if state["goal"] and not state["world"] and "world_ledger" in state["dependencies"]:
        actions.append("sync_world")
    if state["world"] and not state["combat"] and state["world_active"]:
        actions.append("engage_combat")
    if state["combat"] and not state["pressure"] and "task_ledger" in state["dependencies"]:
        actions.append("stabilize_pressure")
    if not state["goal"] and state["is_open"]:
        actions.append("claim_goal")
    if (
        state["goal"]
        and not state["world"]
        and state["is_claimed"]
        and state["dependency_count"] >= 2
        and "world_ledger" not in state["dependencies"]
        and not state["world_active"]
        and "stage=(1, 0, 0, 0)" in state["anchor"]
    ):
        actions.append("reassign_claim")
    if (
        state["combat"]
        and not state["pressure"]
        and state["dependency_count"] >= 2
        and "task_ledger" not in state["dependencies"]
        and not state["pressure_hot"]
        and "stage=(1, 1, 1, 0)" in state["anchor"]
    ):
        actions.append("handoff_dependency")
    if state["is_claimed"] and not state["is_completed"]:
        actions.append("hold_claim")
    if state["combat_hot"]:
        actions.append("hold_combat")
    return actions or ["idle"]


def apply_abstract_action(state: dict[str, object], action: str) -> dict[str, object]:
    next_state = dict(state)
    next_state["dependencies"] = set(state["dependencies"])
    if action == "claim_goal":
        next_state["goal"] = True
        next_state["is_open"] = False
        next_state["is_claimed"] = True
    elif action == "sync_world":
        next_state["world"] = True
        next_state["world_active"] = True
        next_state["is_claimed"] = False
    elif action == "engage_combat":
        next_state["combat"] = True
        next_state["combat_hot"] = True
    elif action == "stabilize_pressure":
        next_state["pressure"] = True
        next_state["pressure_hot"] = True
        if next_state["goal"] and next_state["world"]:
            next_state["is_completed"] = True
    elif action == "mark_complete":
        next_state["is_completed"] = True
    elif action == "reassign_claim":
        next_state["goal"] = False
        next_state["is_open"] = True
        next_state["is_claimed"] = False
    elif action == "handoff_dependency":
        next_state["combat"] = False
        next_state["combat_hot"] = False
    return next_state


def packet_from_state(packet, state: dict[str, object]):
    record = {}
    if state["goal"]:
        record["goal"] = "goal"
    if state["world"]:
        record["world"] = "world"
    if state["combat"]:
        record["combat"] = "combat"
    if state["pressure"]:
        record["pressure"] = "pressure"

    task_ledger = {
        "open": [packet.entity_name] if state["is_open"] else [],
        "claimed": [packet.entity_name] if state["is_claimed"] else [],
        "completed": [packet.entity_name] if state["is_completed"] else [],
        "shared_revision": 0,
    }
    world_ledger = {
        "open": [packet.entity_name] if state["world_active"] else [],
        "claimed": [packet.entity_name] if state["combat_hot"] else [],
        "completed": [packet.entity_name] if state["pressure_hot"] else [],
        "shared_revision": 0,
    }
    dependency_ledger = {
        "dependencies": {packet.entity_name: sorted(state["dependencies"])},
        "shared_revision": 0,
    }
    return build_packet(
        route_key=packet.route_key,
        entity_name=packet.entity_name,
        local_record=record,
        task_ledger=task_ledger,
        world_ledger=world_ledger,
        dependency_ledger=dependency_ledger,
    )


def state_progress(state: dict[str, object]) -> float:
    return (
        0.8 * float(state["goal"])
        + 1.0 * float(state["world"])
        + 1.2 * float(state["combat"])
        + 1.6 * float(state["pressure"])
        + 1.8 * float(state["is_completed"])
    )


def lookahead_policy(packet) -> str:
    base_state = abstract_state(packet)
    best_action = "idle"
    best_score = float("-inf")

    for action in candidate_actions(base_state):
        one_step = apply_abstract_action(base_state, action)
        replanned_packet = packet_from_state(packet, one_step)
        next_action = improved_packet_policy(replanned_packet)
        two_step = apply_abstract_action(one_step, next_action)

        transfer_penalty = 0.35 if action in {"reassign_claim", "handoff_dependency"} else 0.0
        score = (
            ACTION_VALUE[action]
            + 0.55 * ACTION_VALUE.get(next_action, 0.0)
            + 0.30 * state_progress(one_step)
            + 0.15 * state_progress(two_step)
            - transfer_penalty
        )

        if score > best_score:
            best_score = score
            best_action = action
    return best_action


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

    rows = (
        improved_rows
        + [summarize_rows("improved_coordination_policy", improved_rows)]
        + lookahead_rows
        + [summarize_rows("lookahead_coordination_policy", lookahead_rows)]
    )
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_coordination_lookahead_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
