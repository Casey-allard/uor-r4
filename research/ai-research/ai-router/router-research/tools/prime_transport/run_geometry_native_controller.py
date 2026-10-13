#!/usr/bin/env python3
"""Compare frame and geometry-native coordination controllers."""

from __future__ import annotations

from pathlib import Path

from run_coordination_frame_experiment import make_coordination_frame_policy
from run_coordination_policy_improvement import (
    build_coordination_bundles,
    improved_packet_policy,
    run_policy_bundle,
    summarize_rows,
)
from run_router_memory_workspace_experiment import write_rows


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_geometry_native_controller.csv"


def parse_route_key(route_key: str) -> tuple[int, int, int]:
    parts = {}
    for field in route_key.split(":"):
        if "=" in field:
            key, value = field.split("=", 1)
            parts[key] = int(value)
    return parts.get("r", 0), parts.get("b", 0), parts.get("gap", 0)


def visibility_zone(route_key: str) -> str:
    """Small math-native signal from the exact routing geometry."""
    r_depth, _, next_gap = parse_route_key(route_key)
    r_band = "deep" if r_depth >= 6 else "shallow"
    gap_band = "immediate" if next_gap <= 1 else "delayed"
    return f"visibility_zone:{r_band}:{gap_band}"


def make_geometry_native_policy():
    """Frame controller informed by delayed-visibility / branch-depth zone."""
    base_frame_policy = make_coordination_frame_policy()

    def policy(packet) -> str:
        zone = visibility_zone(packet.route_key)
        record = packet.payload
        task_fields = packet.context_summary["task_fields"]
        world_fields = packet.context_summary["world_fields"]
        dependencies = set(packet.context_summary["dependency_summary"]["dependencies"])

        goal = bool(record.get("goal"))
        world = bool(record.get("world"))
        combat = bool(record.get("combat"))
        pressure = bool(record.get("pressure"))

        # In deep delayed zones, keep pushing direct resolution before any softer
        # hold behavior because visible split is delayed and branch identity stays
        # unresolved longer.
        if zone == "visibility_zone:deep:delayed":
            if goal and not world and "world_ledger" in dependencies:
                return "sync_world"
            if world and not combat and (world_fields["world_active"] or True):
                return "engage_combat"
            if combat and not pressure and "task_ledger" in dependencies:
                return "stabilize_pressure"
            if pressure and world_fields["pressure_hot"] and not task_fields["is_completed"]:
                return "mark_complete"

        # In shallow immediate zones, prefer the frame policy as-is.
        action = base_frame_policy(packet)

        # In shallow delayed zones, avoid early idle/hold if direct completion is
        # only one step away.
        if zone == "visibility_zone:shallow:delayed":
            if action in {"hold_claim", "hold_combat", "idle"}:
                if goal and not world and "world_ledger" in dependencies:
                    return "sync_world"
                if combat and not pressure and "task_ledger" in dependencies:
                    return "stabilize_pressure"

        return action

    return policy


def main() -> None:
    bundles = build_coordination_bundles()
    frame_rows = [
        run_policy_bundle(
            bundle,
            policy_name="coordination_frame_policy",
            chooser=make_coordination_frame_policy(),
        )
        for bundle in bundles
    ]
    geometry_rows = [
        run_policy_bundle(
            bundle,
            policy_name="geometry_native_coordination_policy",
            chooser=make_geometry_native_policy(),
        )
        for bundle in bundles
    ]

    rows = (
        frame_rows
        + [summarize_rows("coordination_frame_policy", frame_rows)]
        + geometry_rows
        + [summarize_rows("geometry_native_coordination_policy", geometry_rows)]
    )
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_geometry_native_controller_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
