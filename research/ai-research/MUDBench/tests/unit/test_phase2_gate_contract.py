from __future__ import annotations

import json
import sys
from typing import Any

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner
from core.action_processor import ActionRequest

WAIT_AGENT_SCRIPT = """
import json

print(json.dumps({"action":"wait"}))
"""


def _sample_snapshot() -> dict[str, object]:
    return {
        "tick": 0,
        "entities": {
            "agent-a": {"entity_type": "agent", "location": "room-a"},
            "agent-b": {"entity_type": "agent", "location": "room-a"},
            "npc-1": {"entity_type": "npc", "location": "room-a"},
        },
        "rooms": {
            "room-a": {
                "description": "Start room.",
                "exits": {"north": "room-b"},
                "entities_present": ["agent-a", "agent-b", "npc-1"],
            },
            "room-b": {
                "description": "North room.",
                "exits": {"south": "room-a"},
                "entities_present": [],
            },
        },
        "scenario_vars": {},
    }


def _write_agent_script(tmp_path, name: str, content: str):
    script_path = tmp_path / name
    script_path.write_text(content)
    return script_path


def _action_request_to_dict(action_request: ActionRequest) -> dict[str, object]:
    return {
        "actor_id": action_request.actor_id,
        "action_type": action_request.action_type,
        "arguments": [[key, value] for key, value in action_request.arguments],
    }


def _build_phase2_gate_contract_artifact(tmp_path) -> tuple[str, dict[str, object]]:
    wait_script = _write_agent_script(tmp_path, "phase2_gate_contract_wait.py", WAIT_AGENT_SCRIPT)
    wait_runner = LocalProcessRunner((sys.executable, str(wait_script)))

    result = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="phase2-gate-contract",
        step=0,
        max_steps=4,
        agent_configs=(
            StepDriverAgentConfig(actor_id="agent-b", runner=wait_runner),
            StepDriverAgentConfig(actor_id="agent-a", runner=wait_runner),
        ),
    )

    payload: dict[str, Any] = {
        "observation_actor_order": [actor_id for actor_id, _ in result.observations],
        "observations": [
            {
                "actor_id": actor_id,
                "location": observation.location,
                "remaining_steps": observation.remaining_steps,
                "action_space": list(observation.action_space),
            }
            for actor_id, observation in result.observations
        ],
        "runner_results": [
            {
                "actor_id": runner_result.actor_id,
                "success": runner_result.success,
                "error_type": runner_result.error_type,
                "error_message": runner_result.error_message,
            }
            for runner_result in result.runner_results
        ],
        "pipeline_results": [
            {
                "actor_id": pipeline_result.actor_id,
                "accepted": pipeline_result.accepted,
                "reason": pipeline_result.reason,
                "rejected_stage": pipeline_result.rejected_stage,
            }
            for pipeline_result in result.pipeline_results
        ],
        "accepted_action_requests": [
            _action_request_to_dict(action_request)
            for action_request in result.accepted_action_requests
        ],
        "failures": [
            {
                "actor_id": failure.actor_id,
                "stage": failure.stage,
                "reason": failure.reason,
                "detail": failure.detail,
            }
            for failure in result.failures
        ],
    }
    artifact = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return artifact, payload


def test_phase2_gate_contract_happy_path_invariants(tmp_path) -> None:
    artifact, payload = _build_phase2_gate_contract_artifact(tmp_path)
    del artifact

    assert payload["observation_actor_order"] == ["agent-a", "agent-b"]
    assert payload["observations"] == [
        {
            "actor_id": "agent-a",
            "location": "room-a",
            "remaining_steps": 4,
            "action_space": ["wait", "look", "move north", "talk npc-1", "defend", "attack npc-1"],
        },
        {
            "actor_id": "agent-b",
            "location": "room-a",
            "remaining_steps": 4,
            "action_space": ["wait", "look", "move north", "talk npc-1", "defend", "attack npc-1"],
        },
    ]
    assert payload["runner_results"] == [
        {
            "actor_id": "agent-a",
            "success": True,
            "error_type": None,
            "error_message": None,
        },
        {
            "actor_id": "agent-b",
            "success": True,
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
        {"actor_id": "agent-a", "action_type": "wait", "arguments": []},
        {"actor_id": "agent-b", "action_type": "wait", "arguments": []},
    ]
    assert payload["failures"] == []


def test_phase2_gate_contract_artifact_is_stable_for_identical_inputs(tmp_path) -> None:
    first_artifact, _ = _build_phase2_gate_contract_artifact(tmp_path)
    second_artifact, _ = _build_phase2_gate_contract_artifact(tmp_path)

    assert first_artifact == second_artifact
