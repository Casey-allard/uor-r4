from __future__ import annotations

import sys

import pytest

from agents.local_runner.process_bridge import LocalProcessRunner
from agents.local_runner.session_manager import (
    DeterministicLocalRunnerSessionManager,
    LocalRunnerSessionRequest,
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


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-1",
            "step": 0,
            "location": "room-a",
            "description": "A room.",
            "exits": ["north"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["wait", "move north"],
            "remaining_steps": 5,
        }
    )


def _write_agent_script(tmp_path, name: str, content: str):
    script_path = tmp_path / name
    script_path.write_text(content)
    return script_path


def test_session_manager_orders_requests_by_actor_id(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(success_script)))
    manager = DeterministicLocalRunnerSessionManager()

    requests = (
        LocalRunnerSessionRequest(actor_id="agent-c", runner=runner, observation=_sample_observation()),
        LocalRunnerSessionRequest(actor_id="agent-a", runner=runner, observation=_sample_observation()),
        LocalRunnerSessionRequest(actor_id="agent-b", runner=runner, observation=_sample_observation()),
    )
    results = manager.request_actions(requests)

    assert tuple(result.actor_id for result in results) == ("agent-a", "agent-b", "agent-c")
    assert all(result.success for result in results)
    assert all(result.action_submission is not None for result in results)
    assert all(result.action_submission.action == "wait" for result in results if result.action_submission)


def test_session_manager_returns_explicit_per_agent_errors_and_continues(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    timeout_script = _write_agent_script(tmp_path, "timeout.py", TIMEOUT_AGENT_SCRIPT)
    invalid_script = _write_agent_script(tmp_path, "invalid.py", INVALID_AGENT_SCRIPT)

    manager = DeterministicLocalRunnerSessionManager()
    requests = (
        LocalRunnerSessionRequest(
            actor_id="agent-c",
            runner=LocalProcessRunner((sys.executable, str(success_script))),
            observation=_sample_observation(),
        ),
        LocalRunnerSessionRequest(
            actor_id="agent-a",
            runner=LocalProcessRunner((sys.executable, str(timeout_script))),
            observation=_sample_observation(),
            timeout_seconds=0.05,
        ),
        LocalRunnerSessionRequest(
            actor_id="agent-b",
            runner=LocalProcessRunner((sys.executable, str(invalid_script))),
            observation=_sample_observation(),
        ),
    )

    results = manager.request_actions(requests)
    by_actor = {result.actor_id: result for result in results}

    assert tuple(result.actor_id for result in results) == ("agent-a", "agent-b", "agent-c")
    assert by_actor["agent-a"].success is False
    assert by_actor["agent-a"].error_type == "timeout"
    assert "timeout_at_or_after_budget" in (by_actor["agent-a"].error_message or "")
    assert by_actor["agent-b"].success is False
    assert by_actor["agent-b"].error_type == "protocol_error"
    assert by_actor["agent-c"].success is True
    assert by_actor["agent-c"].action_submission is not None
    assert by_actor["agent-c"].action_submission.action == "wait"


def test_session_manager_is_deterministic_for_identical_inputs(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    invalid_script = _write_agent_script(tmp_path, "invalid.py", INVALID_AGENT_SCRIPT)
    manager = DeterministicLocalRunnerSessionManager()
    requests = (
        LocalRunnerSessionRequest(
            actor_id="agent-2",
            runner=LocalProcessRunner((sys.executable, str(invalid_script))),
            observation=_sample_observation(),
        ),
        LocalRunnerSessionRequest(
            actor_id="agent-1",
            runner=LocalProcessRunner((sys.executable, str(success_script))),
            observation=_sample_observation(),
        ),
    )

    first = manager.request_actions(requests)
    second = manager.request_actions(requests)

    assert first == second


def test_session_manager_rejects_duplicate_actor_ids(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(success_script)))
    manager = DeterministicLocalRunnerSessionManager()

    requests = (
        LocalRunnerSessionRequest(actor_id="agent-1", runner=runner, observation=_sample_observation()),
        LocalRunnerSessionRequest(actor_id="agent-1", runner=runner, observation=_sample_observation()),
    )

    with pytest.raises(ValueError, match="duplicate actor_id"):
        manager.request_actions(requests)


def test_session_request_rejects_non_positive_timeout(tmp_path) -> None:
    success_script = _write_agent_script(tmp_path, "success.py", SUCCESS_AGENT_SCRIPT)
    runner = LocalProcessRunner((sys.executable, str(success_script)))

    with pytest.raises(ValueError, match="timeout_seconds"):
        LocalRunnerSessionRequest(
            actor_id="agent-1",
            runner=runner,
            observation=_sample_observation(),
            timeout_seconds=0,
        )

