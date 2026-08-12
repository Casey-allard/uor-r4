from __future__ import annotations

import json
import sys

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from core.event_logger import EventLogger, EventRecord
from core.simulation_controller import SimulationController
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


def _run_flow(seed: int, tmp_path) -> tuple[str, dict[str, object]]:
    world = _build_initialized_world(seed=seed)
    mover_script = _write_agent_script(tmp_path, "mover.py", MOVER_AGENT_SCRIPT)
    waiter_script = _write_agent_script(tmp_path, "waiter.py", WAITER_AGENT_SCRIPT)

    step_result = drive_gateway_step(
        snapshot=world.get_snapshot(),
        run_id="phase2-int-run",
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
        run_id="phase2-int-run",
    )
    controller.initialize()
    step_outcome = controller.step(step_result.accepted_action_requests)
    snapshot = world.get_snapshot()

    payload = {
        "accepted_actions": [
            {
                "actor_id": action.actor_id,
                "action_type": action.action_type,
                "arguments": [[key, value] for key, value in action.arguments],
            }
            for action in step_result.accepted_action_requests
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
        "observed_actor_order": [actor_id for actor_id, _ in step_result.observations],
        "processed_actions": step_outcome.processed_actions,
        "world": snapshot,
    }
    artifact = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return artifact, payload


def test_phase2_interface_flow_executes_two_agents_through_gateway_and_simulation(tmp_path) -> None:
    artifact, payload = _run_flow(seed=41, tmp_path=tmp_path)
    del artifact

    assert payload["observed_actor_order"] == ["agent-a", "agent-b"]
    assert payload["failures"] == []
    assert payload["processed_actions"] == 2

    accepted_actions = payload["accepted_actions"]
    assert accepted_actions == [
        {"actor_id": "agent-a", "action_type": "move", "arguments": [["direction", "east"]]},
        {"actor_id": "agent-b", "action_type": "wait", "arguments": []},
    ]

    world = payload["world"]
    assert world["entities"]["agent-a"]["location"] == "room-b"
    assert world["entities"]["agent-b"]["location"] == "room-a"


def test_phase2_interface_flow_is_deterministic_for_identical_seed_and_scripts(tmp_path) -> None:
    first_artifact, _ = _run_flow(seed=41, tmp_path=tmp_path)
    second_artifact, _ = _run_flow(seed=41, tmp_path=tmp_path)

    assert first_artifact == second_artifact

