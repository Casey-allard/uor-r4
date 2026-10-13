from __future__ import annotations

import json

from core.action_processor import ActionRequest, normalize_arguments
from core.event_logger import EventLogger, EventRecord
from core.run_state import RunStatus
from core.simulation_controller import SimulationController, StepOutcome
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


class InMemoryEventLogger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-a": {
                    "title": "Room A",
                    "description": "Start room.",
                    "exits": {"east": "room-b"},
                    "entities": [],
                },
                "room-b": {
                    "title": "Room B",
                    "description": "Combat room.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
            }
        }
    )


def _build_initialized_world(seed: int) -> DeterministicWorldStateManager:
    world = bootstrap_world_state_manager(_room_graph(), seed=seed)
    DeterministicSpawnManager(seed=0).place_actors(
        world,
        (
            SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),
            SpawnRequest(
                actor_id="npc-1",
                actor_type="npc",
                preferred_room_id="room-b",
                health=30,
            ),
        ),
    )
    # Freeze current snapshot as reset baseline so controller.initialize() preserves setup.
    return DeterministicWorldStateManager.from_json(world.to_json())


def _run_gate_flow(seed: int) -> tuple[str, dict[str, object]]:
    world = _build_initialized_world(seed=seed)
    logger = InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=logger,
        seed=seed,
        max_steps=2,
    )

    run_state = controller.initialize()
    move_outcome = controller.step(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "east"}),
            ),
        )
    )
    attack_outcome = controller.step(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="attack",
                arguments=normalize_arguments({"target_id": "npc-1"}),
            ),
        )
    )

    artifact_payload = {
        "seed": seed,
        "run_state": {
            "status": run_state.status.value,
            "step_index": run_state.step_index,
            "max_steps": run_state.max_steps,
        },
        "outcomes": [_step_outcome_to_dict(move_outcome), _step_outcome_to_dict(attack_outcome)],
        "event_log": [_event_record_to_dict(record) for record in logger.records()],
        "world_json": world.to_json(),
    }
    artifact = json.dumps(artifact_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return artifact, artifact_payload


def _step_outcome_to_dict(outcome: StepOutcome) -> dict[str, object]:
    return {
        "step_index": outcome.step_index,
        "processed_actions": outcome.processed_actions,
        "status": outcome.status.value,
        "emitted_events": [_event_record_to_dict(record) for record in outcome.emitted_events],
    }


def _event_record_to_dict(record: EventRecord) -> dict[str, object]:
    return {
        "step_index": record.step_index,
        "event_type": record.event_type,
        "actor_id": record.actor_id,
        "payload": [[key, value] for key, value in record.payload],
    }


def test_phase1_gate_flow_init_move_combat_updates_state() -> None:
    artifact, parsed = _run_gate_flow(seed=31)
    del artifact  # artifact is validated in deterministic byte-identical test.

    world = json.loads(parsed["world_json"])
    assert parsed["run_state"]["status"] == RunStatus.TERMINATED.value
    assert parsed["run_state"]["step_index"] == 2
    assert parsed["outcomes"][0]["status"] == RunStatus.RUNNING.value
    assert parsed["outcomes"][1]["status"] == RunStatus.TERMINATED.value

    assert world["entities"]["agent-1"]["location"] == "room-b"
    assert world["entities"]["npc-1"]["location"] == "room-b"
    assert world["entities"]["npc-1"]["health"] == 20


def test_phase1_gate_repeated_runs_are_byte_identical() -> None:
    first_artifact, _ = _run_gate_flow(seed=31)
    second_artifact, _ = _run_gate_flow(seed=31)

    assert first_artifact == second_artifact
