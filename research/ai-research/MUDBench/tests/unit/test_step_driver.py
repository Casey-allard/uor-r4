from __future__ import annotations

import sys

import pytest

from agents.gateway.step_driver import StepDriverAgentConfig, drive_gateway_step
from agents.local_runner.process_bridge import LocalProcessRunner

SUCCESS_AGENT_SCRIPT = """
import json
import sys

line = sys.stdin.readline()
obs = json.loads(line)
action_space = obs.get('action_space', [])
action = action_space[0] if action_space else 'wait'
print(json.dumps({'action': action}))
"""

TIMEOUT_AGENT_SCRIPT = """
import time

time.sleep(1.0)
print('{"action":"wait"}')
"""

INVALID_AGENT_SCRIPT = """
print('not-json')
"""

BAD_ACTION_AGENT_SCRIPT = """
import json

print(json.dumps({'action': 'dance'}))
"""


def _sample_snapshot() -> dict[str, object]:
    return {
        "tick": 0,
        "entities": {
            "agent-a": {"entity_type": "agent", "location": "room-a"},
            "agent-b": {"entity_type": "agent", "location": "room-a"},
            "agent-c": {"entity_type": "agent", "location": "room-a"},
            "npc-1": {"entity_type": "npc", "location": "room-a"},
        },
        "rooms": {
            "room-a": {
                "description": "Start room.",
                "exits": {"north": "room-b"},
                "entities_present": ["agent-a", "agent-b", "agent-c", "npc-1"],
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


def test_step_driver_returns_ordered_observations_and_actions(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(success_script)))

    result = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="run-1",
        step=1,
        max_steps=5,
        agent_configs=(
            StepDriverAgentConfig(actor_id="agent-b", runner=runner),
            StepDriverAgentConfig(actor_id="agent-a", runner=runner),
        ),
    )

    assert tuple(actor_id for actor_id, _ in result.observations) == ("agent-a", "agent-b")
    assert tuple(action.actor_id for action in result.accepted_action_requests) == ("agent-a", "agent-b")
    assert all(action.action_type == "wait" for action in result.accepted_action_requests)
    assert result.failures == ()


def test_step_driver_handles_timeout_and_malformed_runner_results_explicitly(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    timeout_script = _write_agent_script(tmp_path, "timeout.py", TIMEOUT_AGENT_SCRIPT)
    invalid_script = _write_agent_script(tmp_path, "invalid.py", INVALID_AGENT_SCRIPT)

    result = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="run-2",
        step=1,
        max_steps=5,
        agent_configs=(
            StepDriverAgentConfig(
                actor_id="agent-c",
                runner=LocalProcessRunner((sys.executable, str(success_script))),
            ),
            StepDriverAgentConfig(
                actor_id="agent-a",
                runner=LocalProcessRunner((sys.executable, str(timeout_script))),
                timeout_seconds=0.05,
            ),
            StepDriverAgentConfig(
                actor_id="agent-b",
                runner=LocalProcessRunner((sys.executable, str(invalid_script))),
            ),
        ),
    )

    assert tuple(action.actor_id for action in result.accepted_action_requests) == ("agent-c",)
    failures_by_actor = {failure.actor_id: failure for failure in result.failures}
    assert failures_by_actor["agent-a"].stage == "runner"
    assert failures_by_actor["agent-a"].reason == "timeout"
    assert failures_by_actor["agent-b"].stage == "runner"
    assert failures_by_actor["agent-b"].reason == "protocol_error"


def test_step_driver_surfaces_pipeline_rejection_without_emitting_action(tmp_path) -> None:
    bad_action_script = _write_agent_script(tmp_path, "bad_action.py", BAD_ACTION_AGENT_SCRIPT)
    result = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="run-3",
        step=1,
        max_steps=5,
        agent_configs=(
            StepDriverAgentConfig(
                actor_id="agent-a",
                runner=LocalProcessRunner((sys.executable, str(bad_action_script))),
            ),
        ),
    )

    assert result.accepted_action_requests == ()
    assert len(result.failures) == 1
    failure = result.failures[0]
    assert failure.actor_id == "agent-a"
    assert failure.stage == "pipeline"
    assert failure.reason == "action_not_in_action_space"
    assert failure.detail == "validation"


def test_step_driver_is_deterministic_for_identical_inputs(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    bad_action_script = _write_agent_script(tmp_path, "bad_action.py", BAD_ACTION_AGENT_SCRIPT)
    configs = (
        StepDriverAgentConfig(
            actor_id="agent-b",
            runner=LocalProcessRunner((sys.executable, str(success_script))),
        ),
        StepDriverAgentConfig(
            actor_id="agent-a",
            runner=LocalProcessRunner((sys.executable, str(bad_action_script))),
        ),
    )

    first = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="run-4",
        step=2,
        max_steps=7,
        agent_configs=configs,
    )
    second = drive_gateway_step(
        snapshot=_sample_snapshot(),
        run_id="run-4",
        step=2,
        max_steps=7,
        agent_configs=configs,
    )

    assert first == second


def test_step_driver_rejects_duplicate_actor_configs(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(success_script)))
    with pytest.raises(ValueError, match="duplicate actor_id"):
        drive_gateway_step(
            snapshot=_sample_snapshot(),
            run_id="run-5",
            step=0,
            max_steps=3,
            agent_configs=(
                StepDriverAgentConfig(actor_id="agent-a", runner=runner),
                StepDriverAgentConfig(actor_id="agent-a", runner=runner),
            ),
        )

