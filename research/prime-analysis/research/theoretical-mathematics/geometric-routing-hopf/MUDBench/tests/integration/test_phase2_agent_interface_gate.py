from __future__ import annotations

import json
import sys
from typing import Any

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from core.action_processor import ActionRequest
from core.event_logger import EventLogger, EventRecord
from core.run_state import RunStatus
from core.simulation_controller import SimulationController, StepOutcome
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager

MOVER_AGENT_SCRIPT = """
import json
import sys

line = sys.stdin.readline()
observation = json.loads(line)
action_space = observation.get("action_space", [])
action = "move east" if "move east" in action_space else "wait"
print(json.dumps({"action": action}))
"""

WAITER_AGENT_SCRIPT = """
import json

print(json.dumps({"action":"wait"}))
"""


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
                    "description": "Destination room.",
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
            SpawnRequest(actor_id="agent-a", actor_type="agent", preferred_room_id="room-a"),
            SpawnRequest(actor_id="agent-b", actor_type="agent", preferred_room_id="room-a"),
        ),
    )
    return DeterministicWorldStateManager.from_json(world.to_json())


def _write_agent_script(tmp_path, name: str, script: str):
    path = tmp_path / name
    path.write_text(script)
    return path


def _action_request_to_dict(action: ActionRequest) -> dict[str, object]:
    return {
        "actor_id": action.actor_id,
        "action_type": action.action_type,
        "arguments": [[key, value] for key, value in action.arguments],
    }


def _event_record_to_dict(record: EventRecord) -> dict[str, object]:
    return {
        "step_index": record.step_index,
        "event_type": record.event_type,
        "actor_id": record.actor_id,
        "payload": [[key, value] for key, value in record.payload],
    }


def _step_outcome_to_dict(outcome: StepOutcome) -> dict[str, object]:
    return {
        "step_index": outcome.step_index,
        "processed_actions": outcome.processed_actions,
        "status": outcome.status.value,
        "emitted_events": [_event_record_to_dict(record) for record in outcome.emitted_events],
    }


def _run_phase2_gate(seed: int, tmp_path) -> tuple[str, dict[str, object]]:
    world = _build_initialized_world(seed=seed)
    mover_script = _write_agent_script(tmp_path, "phase2_gate_mover.py", MOVER_AGENT_SCRIPT)
    waiter_script = _write_agent_script(tmp_path, "phase2_gate_waiter.py", WAITER_AGENT_SCRIPT)

    step_result = drive_gateway_step(
        snapshot=world.get_snapshot(),
        run_id="phase2-gate-run",
        step=0,
        max_steps=3,
        agent_configs=(
            StepDriverAgentConfig(
                actor_id="agent-b",
                runner=LocalProcessRunner((sys.executable, str(waiter_script))),
            ),
            StepDriverAgentConfig(
                actor_id="agent-a",
                runner=LocalProcessRunner((sys.executable, str(mover_script))),
            ),
        ),
    )

    logger = InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=logger,
        seed=seed,
        max_steps=3,
        run_id="phase2-gate-run",
    )
    controller.initialize()
    step_outcome = controller.step(step_result.accepted_action_requests)
    snapshot = world.get_snapshot()
    entities = snapshot["entities"]

    payload: dict[str, Any] = {
        "observed_actor_order": [actor_id for actor_id, _ in step_result.observations],
        "runner_results": [
            {
                "actor_id": result.actor_id,
                "success": result.success,
                "action": result.action_submission.action if result.action_submission else None,
                "error_type": result.error_type,
                "error_message": result.error_message,
            }
            for result in step_result.runner_results
        ],
        "pipeline_results": [
            {
                "actor_id": result.actor_id,
                "accepted": result.accepted,
                "reason": result.reason,
                "rejected_stage": result.rejected_stage,
            }
            for result in step_result.pipeline_results
        ],
        "accepted_action_requests": [
            _action_request_to_dict(action_request)
            for action_request in step_result.accepted_action_requests
        ],
        "failures": [
            {
                "actor_id": failure.actor_id,
                "stage": failure.stage,
                "reason": failure.reason,
                "detail": failure.detail,
            }
            for failure in step_result.failures
        ],
        "step_outcome": _step_outcome_to_dict(step_outcome),
        "actor_locations": {
            "agent-a": entities["agent-a"]["location"],
            "agent-b": entities["agent-b"]["location"],
        },
    }
    artifact = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return artifact, payload


def test_phase2_agent_interface_gate_reliable_interaction_contract(tmp_path) -> None:
    artifact, payload = _run_phase2_gate(seed=47, tmp_path=tmp_path)
    del artifact

    assert payload["observed_actor_order"] == ["agent-a", "agent-b"]
    assert payload["runner_results"] == [
        {
            "actor_id": "agent-a",
            "success": True,
            "action": "move east",
            "error_type": None,
            "error_message": None,
        },
        {
            "actor_id": "agent-b",
            "success": True,
            "action": "wait",
            "error_type": None,
            "error_message": None,
        },
    ]
    assert payload["pipeline_results"] == [
        {
            "actor_id": "agent-a",
            "accepted": True,
            "reason": None,
            "rejected_stage": None,
        },
        {
            "actor_id": "agent-b",
            "accepted": True,
            "reason": None,
            "rejected_stage": None,
        },
    ]
    assert payload["accepted_action_requests"] == [
        {"actor_id": "agent-a", "action_type": "move", "arguments": [["direction", "east"]]},
        {"actor_id": "agent-b", "action_type": "wait", "arguments": []},
    ]
    assert payload["failures"] == []
    assert payload["actor_locations"] == {"agent-a": "room-b", "agent-b": "room-a"}

    step_outcome = payload["step_outcome"]
    assert step_outcome["step_index"] == 0
    assert step_outcome["processed_actions"] == 2
    assert step_outcome["status"] == RunStatus.RUNNING.value
    assert step_outcome["emitted_events"] == [
        {
            "step_index": 0,
            "event_type": "action_move",
            "actor_id": "agent-a",
            "payload": [
                ["destination_room_id", "room-b"],
                ["direction", "east"],
                ["source_room_id", "room-a"],
            ],
        },
        {
            "step_index": 0,
            "event_type": "action_wait",
            "actor_id": "agent-b",
            "payload": [["result", "no_state_change"]],
        },
        {
            "step_index": 0,
            "event_type": "step_completed",
            "actor_id": None,
            "payload": [["domain_event_count", 2], ["processed_actions", 2]],
        },
    ]


def test_phase2_agent_interface_gate_repeated_runs_are_byte_identical(tmp_path) -> None:
    first_artifact, _ = _run_phase2_gate(seed=47, tmp_path=tmp_path)
    second_artifact, _ = _run_phase2_gate(seed=47, tmp_path=tmp_path)

    assert first_artifact == second_artifact
