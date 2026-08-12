from __future__ import annotations

import json
import sys
from typing import Any, Mapping

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from core.action_processor import ActionRequest
from world.rooms.room_graph import DeterministicRoomGraph
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
                    "description": "Second room.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
            }
        }
    )


def _build_snapshot(seed: int) -> Mapping[str, Any]:
    world = bootstrap_world_state_manager(_room_graph(), seed=seed)
    DeterministicSpawnManager(seed=seed).place_actors(
        world,
        (
            SpawnRequest(actor_id="agent-a", actor_type="agent"),
            SpawnRequest(actor_id="agent-b", actor_type="agent"),
        ),
    )
    frozen_world = DeterministicWorldStateManager.from_json(world.to_json())
    return frozen_world.get_snapshot()


def _write_agent_script(tmp_path, name: str, script: str):
    path = tmp_path / name
    path.write_text(script)
    return path


def _action_request_payload(action_request: ActionRequest | None) -> dict[str, object] | None:
    if action_request is None:
        return None
    return {
        "actor_id": action_request.actor_id,
        "action_type": action_request.action_type,
        "arguments": [[key, value] for key, value in action_request.arguments],
    }


def _interface_transcript(seed: int, tmp_path) -> str:
    snapshot = _build_snapshot(seed=seed)
    mover_script = _write_agent_script(tmp_path, "mover.py", MOVER_AGENT_SCRIPT)
    waiter_script = _write_agent_script(tmp_path, "waiter.py", WAITER_AGENT_SCRIPT)

    step_result = drive_gateway_step(
        snapshot=snapshot,
        run_id="phase2-determinism-run",
        step=0,
        max_steps=4,
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

    transcript = {
        "observations": [
            {"actor_id": actor_id, "observation": observation.to_dict()}
            for actor_id, observation in step_result.observations
        ],
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
                "action_request": _action_request_payload(result.action_request),
            }
            for result in step_result.pipeline_results
        ],
        "accepted_action_requests": [
            _action_request_payload(action_request)
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
    }
    return json.dumps(transcript, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def test_phase2_interface_transcript_is_byte_identical_for_repeated_seeded_runs(tmp_path) -> None:
    first = _interface_transcript(seed=41, tmp_path=tmp_path)
    second = _interface_transcript(seed=41, tmp_path=tmp_path)

    assert first == second


def test_phase2_interface_transcript_changes_when_seed_changes(tmp_path) -> None:
    first = _interface_transcript(seed=41, tmp_path=tmp_path)
    second = _interface_transcript(seed=42, tmp_path=tmp_path)

    assert first != second

