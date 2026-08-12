from __future__ import annotations

import sys

import pytest

from agents.gateway.timeout_policy import classify_timeout, classify_timeout_expired
from agents.local_runner.process_bridge import LocalProcessRunner, LocalRunnerTimeoutError
from agents.protocol.observation import Observation

_TIMEOUT_AGENT_SCRIPT = """
import time

time.sleep(1.0)
print('{"action":"wait"}')
"""


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-timeout",
            "step": 0,
            "location": "room-a",
            "description": "A room.",
            "exits": [],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["wait"],
            "remaining_steps": 1,
        }
    )


def _write_agent_script(tmp_path):
    script_path = tmp_path / "agent_timeout.py"
    script_path.write_text(_TIMEOUT_AGENT_SCRIPT)
    return script_path


def test_classify_timeout_within_budget() -> None:
    decision = classify_timeout(elapsed_seconds=0.02, timeout_seconds=0.05)

    assert decision.timed_out is False
    assert decision.reason == "within_timeout_budget"
    assert decision.elapsed_seconds == 0.02
    assert decision.timeout_seconds == 0.05


def test_classify_timeout_boundary_is_timeout() -> None:
    decision = classify_timeout(elapsed_seconds=0.05, timeout_seconds=0.05)

    assert decision.timed_out is True
    assert decision.reason == "timeout_at_or_after_budget"


def test_classify_timeout_expired_matches_boundary_policy() -> None:
    decision = classify_timeout_expired(timeout_seconds=0.1)

    assert decision.timed_out is True
    assert decision.reason == "timeout_at_or_after_budget"
    assert decision.elapsed_seconds == 0.1
    assert decision.timeout_seconds == 0.1


def test_timeout_classification_is_deterministic() -> None:
    first = classify_timeout(elapsed_seconds=0.03, timeout_seconds=0.04)
    second = classify_timeout(elapsed_seconds=0.03, timeout_seconds=0.04)
    third = classify_timeout(elapsed_seconds=0.03, timeout_seconds=0.04)

    assert first == second == third


@pytest.mark.parametrize("timeout_seconds", (0, -0.1))
def test_timeout_classification_rejects_invalid_timeout_budget(timeout_seconds: float) -> None:
    with pytest.raises(ValueError, match="timeout_seconds"):
        classify_timeout(elapsed_seconds=0.01, timeout_seconds=timeout_seconds)


def test_local_runner_timeout_uses_deterministic_timeout_policy(tmp_path) -> None:
    script_path = _write_agent_script(tmp_path)
    runner = LocalProcessRunner((sys.executable, str(script_path)))

    with pytest.raises(LocalRunnerTimeoutError) as exc_info:
        runner.request_action(_sample_observation(), timeout_seconds=0.05)

    message = str(exc_info.value)
    assert "timeout_at_or_after_budget" in message
    assert "timeout_seconds=0.050000" in message

