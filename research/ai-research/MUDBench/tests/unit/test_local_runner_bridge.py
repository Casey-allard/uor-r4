from __future__ import annotations

import sys

import pytest

from agents.local_runner.process_bridge import (
    LocalProcessRunner,
    LocalRunnerProtocolError,
    LocalRunnerTimeoutError,
)
from agents.protocol.observation import Observation


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

UNSUPPORTED_PROTOCOL_AGENT_SCRIPT = """
import json
print(json.dumps({"protocol_version":"9.9","action":"wait"}))
"""

PERSISTENT_AGENT_SCRIPT = """
import json
import sys

turn = 0
for line in sys.stdin:
    obs = json.loads(line)
    action_space = tuple(obs.get("action_space", ()))
    if turn == 0:
        action = "move north" if "move north" in action_space else (action_space[0] if action_space else "wait")
    else:
        action = "wait" if "wait" in action_space else (action_space[0] if action_space else "wait")
    print(json.dumps({"action": action}))
    sys.stdout.flush()
    turn += 1
"""


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "abc123",
            "step": 1,
            "location": "forest_path",
            "description": "A path.",
            "exits": ["north"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["move north", "wait"],
            "remaining_steps": 100,
        }
    )


def _write_agent_script(tmp_path, content: str):
    script_path = tmp_path / "agent.py"
    script_path.write_text(content)
    return script_path


def test_local_process_runner_returns_action_from_agent(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)))

    action = runner.request_action(_sample_observation(), timeout_seconds=0.5)

    assert action.action == "move north"


def test_local_process_runner_timeout_raises(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, TIMEOUT_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)))

    with pytest.raises(LocalRunnerTimeoutError):
        runner.request_action(_sample_observation(), timeout_seconds=0.05)


def test_local_process_runner_invalid_json_raises(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, INVALID_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)))

    with pytest.raises(LocalRunnerProtocolError):
        runner.request_action(_sample_observation(), timeout_seconds=0.5)


def test_local_process_runner_rejects_incompatible_observation_protocol_version(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)))
    observation = _sample_observation()
    object.__setattr__(observation, "protocol_version", "9.9")

    with pytest.raises(
        LocalRunnerProtocolError,
        match="incompatible_protocol_version:unsupported_protocol_version:9.9:supported=1.0",
    ):
        runner.request_action(observation, timeout_seconds=0.5)


def test_local_process_runner_rejects_unsupported_action_envelope_protocol_version(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, UNSUPPORTED_PROTOCOL_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)))

    with pytest.raises(
        LocalRunnerProtocolError,
        match="unsupported_protocol_version:9.9:supported=1.0",
    ):
        runner.request_action(_sample_observation(), timeout_seconds=0.5)


def test_local_process_runner_persistent_session_reuses_process_across_requests(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, PERSISTENT_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)), persistent_session=True)

    first_action = runner.request_action(_sample_observation(), timeout_seconds=0.5)
    second_action = runner.request_action(_sample_observation(), timeout_seconds=0.5)
    runner.close()

    assert first_action.action == "move north"
    assert second_action.action == "wait"


def test_local_process_runner_rejects_broken_persistent_session_lifecycle(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path, SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(script_path)), persistent_session=True)

    runner.request_action(_sample_observation(), timeout_seconds=0.5)
    with pytest.raises(LocalRunnerProtocolError, match="persistent_session_"):
        runner.request_action(_sample_observation(), timeout_seconds=0.5)
    runner.close()
