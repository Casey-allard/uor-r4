"""Deterministic one-step gateway driver for multi-agent execution."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from agents.gateway.action_pipeline import ActionPipelineResult, run_action_pipeline
from agents.gateway.observation_builder import build_observation_for_actor
from agents.local_runner.process_bridge import LocalProcessRunner
from agents.local_runner.session_manager import (
    DeterministicLocalRunnerSessionManager,
    LocalRunnerSessionRequest,
    LocalRunnerSessionResult,
)
from agents.protocol.observation import Observation
from core.action_processor import ActionRequest


@dataclass(frozen=True, slots=True)
class StepDriverAgentConfig:
    """Per-agent runner configuration for one gateway step."""

    actor_id: str
    runner: LocalProcessRunner
    timeout_seconds: float = 1.0
    messages: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        if self.timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be greater than zero")

        if isinstance(self.messages, (str, bytes)) or not isinstance(self.messages, Sequence):
            raise ValueError("messages must be a sequence of strings")
        normalized_messages: list[str] = []
        for message in self.messages:
            if not isinstance(message, str):
                raise ValueError("messages must contain only strings")
            normalized_messages.append(message)
        object.__setattr__(self, "messages", tuple(normalized_messages))


@dataclass(frozen=True, slots=True)
class StepDriverFailure:
    """Explicit per-agent failure record for step-driver execution."""

    actor_id: str
    stage: str
    reason: str
    detail: str | None = None


@dataclass(frozen=True, slots=True)
class GatewayStepResult:
    """Deterministic gateway step output bundle."""

    observations: tuple[tuple[str, Observation], ...]
    runner_results: tuple[LocalRunnerSessionResult, ...]
    pipeline_results: tuple[ActionPipelineResult, ...]
    accepted_action_requests: tuple[ActionRequest, ...]
    failures: tuple[StepDriverFailure, ...]


def drive_gateway_step(
    *,
    snapshot: Mapping[str, Any],
    run_id: str,
    step: int,
    max_steps: int,
    agent_configs: Sequence[StepDriverAgentConfig],
    session_manager: DeterministicLocalRunnerSessionManager | None = None,
) -> GatewayStepResult:
    """Execute one deterministic gateway step across N agents."""
    if not isinstance(snapshot, Mapping):
        raise ValueError("snapshot must be a mapping")
    if not isinstance(run_id, str) or not run_id:
        raise ValueError("run_id must be a non-empty string")
    if not isinstance(step, int) or step < 0:
        raise ValueError("step must be a non-negative integer")
    if not isinstance(max_steps, int) or max_steps < 0:
        raise ValueError("max_steps must be a non-negative integer")

    ordered_configs = tuple(sorted(agent_configs, key=_agent_sort_key))
    _validate_unique_actor_ids(ordered_configs)

    observations: list[tuple[str, Observation]] = []
    runner_requests: list[LocalRunnerSessionRequest] = []
    for config in ordered_configs:
        observation = build_observation_for_actor(
            snapshot,
            actor_id=config.actor_id,
            run_id=run_id,
            step=step,
            max_steps=max_steps,
            messages=config.messages,
        )
        observations.append((config.actor_id, observation))
        runner_requests.append(
            LocalRunnerSessionRequest(
                actor_id=config.actor_id,
                runner=config.runner,
                observation=observation,
                timeout_seconds=config.timeout_seconds,
            )
        )

    manager = session_manager or DeterministicLocalRunnerSessionManager()
    runner_results = manager.request_actions(tuple(runner_requests))
    observations_by_actor = {actor_id: observation for actor_id, observation in observations}

    pipeline_results: list[ActionPipelineResult] = []
    accepted_action_requests: list[ActionRequest] = []
    failures: list[StepDriverFailure] = []
    for runner_result in runner_results:
        if not runner_result.success:
            failures.append(_runner_failure(runner_result))
            continue

        if runner_result.action_submission is None:
            failures.append(
                StepDriverFailure(
                    actor_id=runner_result.actor_id,
                    stage="runner",
                    reason="missing_action_submission",
                    detail="runner reported success without action submission",
                )
            )
            continue

        pipeline_result = run_action_pipeline(
            actor_id=runner_result.actor_id,
            submission=runner_result.action_submission,
            observation=observations_by_actor[runner_result.actor_id],
        )
        pipeline_results.append(pipeline_result)
        if pipeline_result.accepted:
            if pipeline_result.action_request is None:
                raise RuntimeError("accepted pipeline result must include action_request")
            accepted_action_requests.append(pipeline_result.action_request)
            continue

        failures.append(
            StepDriverFailure(
                actor_id=runner_result.actor_id,
                stage="pipeline",
                reason=pipeline_result.reason or "pipeline_rejected",
                detail=pipeline_result.rejected_stage,
            )
        )

    return GatewayStepResult(
        observations=tuple(observations),
        runner_results=tuple(runner_results),
        pipeline_results=tuple(pipeline_results),
        accepted_action_requests=tuple(accepted_action_requests),
        failures=tuple(failures),
    )


def _agent_sort_key(config: StepDriverAgentConfig) -> tuple[str]:
    return (config.actor_id,)


def _validate_unique_actor_ids(agent_configs: Sequence[StepDriverAgentConfig]) -> None:
    seen: set[str] = set()
    for config in agent_configs:
        if config.actor_id in seen:
            raise ValueError(f"duplicate actor_id in agent_configs: {config.actor_id}")
        seen.add(config.actor_id)


def _runner_failure(result: LocalRunnerSessionResult) -> StepDriverFailure:
    return StepDriverFailure(
        actor_id=result.actor_id,
        stage="runner",
        reason=result.error_type or "runner_error",
        detail=result.error_message,
    )

