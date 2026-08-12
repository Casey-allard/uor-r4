"""Deterministic multi-agent local runner orchestration."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Sequence

from agents.local_runner.process_bridge import (
    LocalProcessRunner,
    LocalRunnerError,
    LocalRunnerProtocolError,
    LocalRunnerTimeoutError,
)
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation


@dataclass(frozen=True, slots=True)
class LocalRunnerSessionRequest:
    """Single-agent execution request for a step."""

    actor_id: str
    runner: LocalProcessRunner
    observation: Observation
    timeout_seconds: float = 1.0

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        if self.timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be greater than zero")


@dataclass(frozen=True, slots=True)
class LocalRunnerSessionResult:
    """Deterministic per-agent execution outcome."""

    actor_id: str
    success: bool
    action_submission: ActionSubmission | None = None
    error_type: str | None = None
    error_message: str | None = None


class DeterministicLocalRunnerSessionManager:
    """Executes local-process calls for multiple agents in deterministic order."""

    def request_actions(
        self,
        requests: Sequence[LocalRunnerSessionRequest],
    ) -> tuple[LocalRunnerSessionResult, ...]:
        ordered_requests = tuple(sorted(requests, key=_request_sort_key))
        _validate_unique_actor_ids(ordered_requests)

        results: list[LocalRunnerSessionResult] = []
        for request in ordered_requests:
            try:
                action_submission = request.runner.request_action(
                    request.observation,
                    timeout_seconds=request.timeout_seconds,
                )
                results.append(
                    LocalRunnerSessionResult(
                        actor_id=request.actor_id,
                        success=True,
                        action_submission=action_submission,
                    )
                )
            except LocalRunnerTimeoutError as exc:
                results.append(
                    LocalRunnerSessionResult(
                        actor_id=request.actor_id,
                        success=False,
                        error_type="timeout",
                        error_message=str(exc),
                    )
                )
            except LocalRunnerProtocolError as exc:
                results.append(
                    LocalRunnerSessionResult(
                        actor_id=request.actor_id,
                        success=False,
                        error_type="protocol_error",
                        error_message=str(exc),
                    )
                )
            except LocalRunnerError as exc:
                results.append(
                    LocalRunnerSessionResult(
                        actor_id=request.actor_id,
                        success=False,
                        error_type="runner_error",
                        error_message=str(exc),
                    )
                )
        return tuple(results)

    def close(self, runners: Sequence[LocalProcessRunner]) -> None:
        ordered_runners = tuple(sorted(runners, key=lambda runner: runner.command))
        seen_runners: set[int] = set()
        for runner in ordered_runners:
            runner_id = id(runner)
            if runner_id in seen_runners:
                continue
            seen_runners.add(runner_id)
            runner.close()


def _request_sort_key(request: LocalRunnerSessionRequest) -> tuple[str]:
    return (request.actor_id,)


def _validate_unique_actor_ids(requests: Sequence[LocalRunnerSessionRequest]) -> None:
    seen: set[str] = set()
    for request in requests:
        if request.actor_id in seen:
            raise ValueError(f"duplicate actor_id in requests: {request.actor_id}")
        seen.add(request.actor_id)
