"""Deterministic replay reconstruction engine."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from replay.logging.replay_artifact import (
    ReplayArtifact,
    ReplayArtifactEmitResult,
    parse_replay_artifact,
)

_STATE_SNAPSHOT_EVENT_TYPE = "state_snapshot"
_DEFAULT_RECONSTRUCTION_STATE_SCHEMA = "benchmark_runtime_state_v1"
_REQUIRED_CANONICAL_STATE_FIELDS = (
    "schema_version",
    "run_id",
    "benchmark_id",
    "scenario_id",
    "step_index",
    "agent_states",
    "item_states",
    "npc_states",
    "room_states",
    "tracker_total_signals",
)
_REQUIRED_AGENT_STATE_FIELDS = ("actor_id", "metrics")
_REQUIRED_AGENT_METRIC_FIELDS = (
    "metric_name",
    "sample_count",
    "value_sum",
    "min_value",
    "max_value",
    "last_step",
    "last_value",
)


@dataclass(frozen=True, slots=True)
class ReconstructedEntityState:
    """Deterministically reconstructed terminal entity state."""

    entity_id: str
    location: str | None
    health: int | None
    inventory: tuple[str, ...]
    last_event_type: str | None

    def __post_init__(self) -> None:
        if not isinstance(self.entity_id, str) or not self.entity_id:
            raise ValueError("entity_id must be a non-empty string")
        if self.location is not None and (not isinstance(self.location, str) or not self.location):
            raise ValueError("location must be None or a non-empty string")
        if self.health is not None and (not isinstance(self.health, int) or isinstance(self.health, bool)):
            raise ValueError("health must be None or an integer")
        if not isinstance(self.inventory, tuple):
            raise ValueError("inventory must be a tuple of strings")
        if tuple(sorted(self.inventory)) != self.inventory:
            raise ValueError("inventory must be sorted")
        for item_id in self.inventory:
            if not isinstance(item_id, str) or not item_id:
                raise ValueError("inventory must contain non-empty strings")
        if self.last_event_type is not None and (
            not isinstance(self.last_event_type, str) or not self.last_event_type
        ):
            raise ValueError("last_event_type must be None or a non-empty string")

    def to_dict(self) -> dict[str, Any]:
        return {
            "entity_id": self.entity_id,
            "location": self.location,
            "health": self.health,
            "inventory": list(self.inventory),
            "last_event_type": self.last_event_type,
        }


@dataclass(frozen=True, slots=True)
class RejectedActionTrace:
    """Deterministic trace item for one rejected action event."""

    step_index: int
    actor_id: str | None
    action_type: str
    reason: str

    def __post_init__(self) -> None:
        if not isinstance(self.step_index, int) or isinstance(self.step_index, bool) or self.step_index < 0:
            raise ValueError("step_index must be a non-negative integer")
        if self.actor_id is not None and (not isinstance(self.actor_id, str) or not self.actor_id):
            raise ValueError("actor_id must be None or a non-empty string")
        if not isinstance(self.action_type, str) or not self.action_type:
            raise ValueError("action_type must be a non-empty string")
        if not isinstance(self.reason, str) or not self.reason:
            raise ValueError("reason must be a non-empty string")

    def to_dict(self) -> dict[str, Any]:
        return {
            "step_index": self.step_index,
            "actor_id": self.actor_id,
            "action_type": self.action_type,
            "reason": self.reason,
        }


@dataclass(frozen=True, slots=True)
class ReconstructedReplayState:
    """Immutable deterministic reconstructed replay state."""

    run_id: str
    initial_seed: int
    seed_source: str
    max_steps: int
    terminal_step: int | None
    event_count: int
    applied_steps: tuple[int, ...]
    entities: tuple[ReconstructedEntityState, ...]
    event_type_counts: tuple[tuple[str, int], ...]
    rejected_actions: tuple[RejectedActionTrace, ...]
    canonical_state_json: str | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.initial_seed, int) or isinstance(self.initial_seed, bool) or self.initial_seed < 0:
            raise ValueError("initial_seed must be a non-negative integer")
        if not isinstance(self.seed_source, str) or not self.seed_source:
            raise ValueError("seed_source must be a non-empty string")
        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")
        if self.terminal_step is not None:
            if (
                not isinstance(self.terminal_step, int)
                or isinstance(self.terminal_step, bool)
                or self.terminal_step < 0
            ):
                raise ValueError("terminal_step must be None or a non-negative integer")
            if self.terminal_step > self.max_steps:
                raise ValueError("terminal_step must be less than or equal to max_steps")
        if not isinstance(self.event_count, int) or isinstance(self.event_count, bool) or self.event_count < 0:
            raise ValueError("event_count must be a non-negative integer")
        if not isinstance(self.applied_steps, tuple):
            raise ValueError("applied_steps must be a tuple of step indexes")
        for step_index in self.applied_steps:
            if not isinstance(step_index, int) or isinstance(step_index, bool) or step_index < 0:
                raise ValueError("applied_steps must contain non-negative integers")
        if self.canonical_state_json is not None:
            if not isinstance(self.canonical_state_json, str) or not self.canonical_state_json:
                raise ValueError("canonical_state_json must be None or a non-empty canonical JSON object")
            normalized = _normalize_canonical_state_json(self.canonical_state_json)
            if normalized != self.canonical_state_json:
                raise ValueError("canonical_state_json must be canonical JSON")

    def to_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "run_id": self.run_id,
            "initial_seed": self.initial_seed,
            "seed_source": self.seed_source,
            "max_steps": self.max_steps,
            "terminal_step": self.terminal_step,
            "event_count": self.event_count,
            "applied_steps": list(self.applied_steps),
            "entities": [entity.to_dict() for entity in self.entities],
            "event_type_counts": [
                {"event_type": event_type, "count": count}
                for event_type, count in self.event_type_counts
            ],
            "rejected_actions": [item.to_dict() for item in self.rejected_actions],
        }
        if self.canonical_state_json is not None:
            payload["canonical_state"] = json.loads(self.canonical_state_json)
        return payload

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class ReplayReconstructionResult:
    """Explicit accept/reject replay reconstruction result."""

    accepted: bool
    reconstructed_state: ReconstructedReplayState | None = None
    reason: str | None = None
    artifact_result: ReplayArtifactEmitResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.reconstructed_state is None:
                raise ValueError("accepted reconstruction result must include reconstructed_state")
            if self.reason is not None:
                raise ValueError("accepted reconstruction result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected reconstruction result must include reason")
        if self.reconstructed_state is not None:
            raise ValueError("rejected reconstruction result must not include reconstructed_state")


def reconstruct_replay_state(artifact: object) -> ReplayReconstructionResult:
    """Reconstruct deterministic terminal state from replay artifact."""
    resolved_artifact, artifact_result, reason = _resolve_artifact(artifact)
    if reason is not None:
        return _rejected_result(reason=reason, artifact_result=artifact_result)
    if resolved_artifact is None:
        raise RuntimeError("internal error resolving replay artifact")

    try:
        reconstructed_state = _reconstruct_from_artifact(resolved_artifact)
    except ValueError as exc:
        return _rejected_result(
            reason=f"unreconstructable_event_stream:{exc}",
            artifact_result=artifact_result,
        )
    return _accepted_result(reconstructed_state=reconstructed_state, artifact_result=artifact_result)


def _reconstruct_from_artifact(artifact: ReplayArtifact) -> ReconstructedReplayState:
    max_steps = artifact.envelope.max_steps
    envelope_metadata = {key: value for key, value in artifact.envelope.run_metadata}
    declared_state_schema, state_snapshot_event_type = _resolve_declared_state_contract(
        envelope_metadata
    )

    event_type_counts: dict[str, int] = {}
    locations: dict[str, str] = {}
    health: dict[str, int] = {}
    inventory: dict[str, set[str]] = {}
    last_event_type: dict[str, str] = {}
    rejected_actions: list[RejectedActionTrace] = []
    applied_steps: list[int] = []
    terminal_canonical_state_json: str | None = None

    expected_step = 0
    last_seen_step = -1
    for event_index, event in enumerate(artifact.events):
        step_index = event.step_index
        if step_index > max_steps:
            raise ValueError(
                f"step_overflow_at_index:{event_index}:step:{step_index}:max_steps:{max_steps}"
            )
        if step_index < last_seen_step:
            raise ValueError(
                f"step_regression_at_index:{event_index}:previous_step:{last_seen_step}:step:{step_index}"
            )
        if step_index != last_seen_step:
            # Replay contract: sparse/no-domain steps must still appear in the event stream
            # via explicit step markers (e.g. step_completed), so step indexes remain contiguous.
            if step_index != expected_step:
                raise ValueError(
                    f"step_gap_at_index:{event_index}:expected_step:{expected_step}:step:{step_index}"
                )
            applied_steps.append(step_index)
            expected_step += 1
            last_seen_step = step_index

        event_type_counts[event.event_type] = event_type_counts.get(event.event_type, 0) + 1
        payload = {key: value for key, value in event.payload}
        if event.event_type == state_snapshot_event_type:
            terminal_canonical_state_json = _parse_state_snapshot_payload(
                payload=payload,
                expected_schema=declared_state_schema,
                expected_run_id=artifact.envelope.run_id,
                expected_step_index=step_index,
                event_index=event_index,
            )

        if event.actor_id is not None:
            inventory.setdefault(event.actor_id, set())
            last_event_type[event.actor_id] = event.event_type

        if event.event_type == "action_move" and event.actor_id is not None:
            destination = payload.get("destination_room_id")
            if isinstance(destination, str) and destination:
                locations[event.actor_id] = destination
        elif event.event_type == "action_look" and event.actor_id is not None:
            location = payload.get("location")
            if isinstance(location, str) and location:
                locations[event.actor_id] = location
        elif event.event_type == "action_take" and event.actor_id is not None:
            item_id = payload.get("item_id")
            if isinstance(item_id, str) and item_id:
                inventory.setdefault(event.actor_id, set()).add(item_id)
        elif event.event_type in {"action_drop", "action_use"} and event.actor_id is not None:
            item_id = payload.get("item_id")
            if isinstance(item_id, str) and item_id:
                inventory.setdefault(event.actor_id, set()).discard(item_id)
        elif event.event_type == "action_attack":
            target_id = payload.get("target_id")
            resulting_health = payload.get("resulting_health")
            if (
                isinstance(target_id, str)
                and target_id
                and isinstance(resulting_health, int)
                and not isinstance(resulting_health, bool)
            ):
                health[target_id] = resulting_health
        elif event.event_type == "action_rejected":
            action_type = payload.get("action_type")
            reason = payload.get("reason")
            if isinstance(action_type, str) and action_type and isinstance(reason, str) and reason:
                rejected_actions.append(
                    RejectedActionTrace(
                        step_index=event.step_index,
                        actor_id=event.actor_id,
                        action_type=action_type,
                        reason=reason,
                    )
                )

    if declared_state_schema is not None and terminal_canonical_state_json is None:
        raise ValueError(
            f"missing_state_snapshot_for_declared_schema:{declared_state_schema}"
        )

    entity_ids = sorted(
        set(artifact.envelope.actor_ids)
        | set(locations.keys())
        | set(health.keys())
        | set(inventory.keys())
        | set(last_event_type.keys())
    )
    entities = tuple(
        ReconstructedEntityState(
            entity_id=entity_id,
            location=locations.get(entity_id),
            health=health.get(entity_id),
            inventory=tuple(sorted(inventory.get(entity_id, set()))),
            last_event_type=last_event_type.get(entity_id),
        )
        for entity_id in entity_ids
    )

    terminal_step = applied_steps[-1] if applied_steps else None
    return ReconstructedReplayState(
        run_id=artifact.envelope.run_id,
        initial_seed=artifact.envelope.initial_seed,
        seed_source=artifact.envelope.seed_source,
        max_steps=max_steps,
        terminal_step=terminal_step,
        event_count=len(artifact.events),
        applied_steps=tuple(applied_steps),
        entities=entities,
        event_type_counts=tuple(sorted(event_type_counts.items(), key=lambda item: item[0])),
        rejected_actions=tuple(rejected_actions),
        canonical_state_json=terminal_canonical_state_json,
    )


def _resolve_declared_state_contract(metadata: Mapping[str, Any]) -> tuple[str | None, str]:
    declared_schema = metadata.get("reconstruction_state_schema")
    if declared_schema is None:
        return None, _STATE_SNAPSHOT_EVENT_TYPE
    if not isinstance(declared_schema, str) or not declared_schema:
        raise ValueError("invalid_reconstruction_state_schema_in_run_metadata")

    event_type = metadata.get("reconstruction_state_event_type", _STATE_SNAPSHOT_EVENT_TYPE)
    if not isinstance(event_type, str) or not event_type:
        raise ValueError("invalid_reconstruction_state_event_type_in_run_metadata")
    return declared_schema, event_type


def _parse_state_snapshot_payload(
    *,
    payload: Mapping[str, Any],
    expected_schema: str | None,
    expected_run_id: str,
    expected_step_index: int,
    event_index: int,
) -> str:
    state_schema = payload.get("state_schema")
    if not isinstance(state_schema, str) or not state_schema:
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:state_schema_missing")
    if expected_schema is not None and state_schema != expected_schema:
        raise ValueError(
            f"invalid_state_snapshot_at_index:{event_index}:"
            f"state_schema_mismatch:expected:{expected_schema}:actual:{state_schema}"
        )
    state_json = payload.get("state_json")
    if not isinstance(state_json, str) or not state_json:
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:state_json_missing")

    canonical_state_json = _normalize_canonical_state_json(state_json)
    state_payload = json.loads(canonical_state_json)
    if not isinstance(state_payload, Mapping):
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:state_json_not_object")

    _validate_canonical_state_payload(
        payload=state_payload,
        expected_schema=state_schema,
        expected_run_id=expected_run_id,
        expected_step_index=expected_step_index,
        event_index=event_index,
    )
    return canonical_state_json


def _normalize_canonical_state_json(value: str) -> str:
    try:
        parsed = json.loads(value)
    except json.JSONDecodeError as exc:
        raise ValueError("canonical_state_json must be valid JSON object") from exc
    if not isinstance(parsed, Mapping):
        raise ValueError("canonical_state_json must decode to object")
    return json.dumps(parsed, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _validate_canonical_state_payload(
    *,
    payload: Mapping[str, Any],
    expected_schema: str,
    expected_run_id: str,
    expected_step_index: int,
    event_index: int,
) -> None:
    missing_fields = [field for field in _REQUIRED_CANONICAL_STATE_FIELDS if field not in payload]
    if missing_fields:
        raise ValueError(
            f"invalid_state_snapshot_at_index:{event_index}:"
            f"missing_required_fields:{','.join(missing_fields)}"
        )

    schema_version = payload.get("schema_version")
    if not isinstance(schema_version, str) or schema_version != expected_schema:
        raise ValueError(
            f"invalid_state_snapshot_at_index:{event_index}:schema_version_mismatch"
        )
    if (
        expected_schema == _DEFAULT_RECONSTRUCTION_STATE_SCHEMA
        and schema_version != _DEFAULT_RECONSTRUCTION_STATE_SCHEMA
    ):
        raise ValueError(
            f"invalid_state_snapshot_at_index:{event_index}:unsupported_schema_version:{schema_version}"
        )

    run_id = payload.get("run_id")
    if not isinstance(run_id, str) or run_id != expected_run_id:
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:run_id_mismatch")

    step_index = payload.get("step_index")
    if (
        not isinstance(step_index, int)
        or isinstance(step_index, bool)
        or step_index != expected_step_index
    ):
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:step_index_mismatch")

    tracker_total_signals = payload.get("tracker_total_signals")
    if (
        not isinstance(tracker_total_signals, int)
        or isinstance(tracker_total_signals, bool)
        or tracker_total_signals < 0
    ):
        raise ValueError(
            f"invalid_state_snapshot_at_index:{event_index}:tracker_total_signals_invalid"
        )

    agent_states = payload.get("agent_states")
    if not _is_non_string_sequence(agent_states):
        raise ValueError(f"invalid_state_snapshot_at_index:{event_index}:agent_states_invalid")
    for agent_index, agent_state in enumerate(agent_states):
        if not isinstance(agent_state, Mapping):
            raise ValueError(
                f"invalid_state_snapshot_at_index:{event_index}:agent_state_not_mapping:{agent_index}"
            )
        missing_agent_fields = [
            field for field in _REQUIRED_AGENT_STATE_FIELDS if field not in agent_state
        ]
        if missing_agent_fields:
            raise ValueError(
                f"invalid_state_snapshot_at_index:{event_index}:"
                f"agent_state_missing_fields:{agent_index}:{','.join(missing_agent_fields)}"
            )
        actor_id = agent_state.get("actor_id")
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError(
                f"invalid_state_snapshot_at_index:{event_index}:agent_state_actor_id_invalid:{agent_index}"
            )
        metrics = agent_state.get("metrics")
        if not _is_non_string_sequence(metrics):
            raise ValueError(
                f"invalid_state_snapshot_at_index:{event_index}:agent_state_metrics_invalid:{agent_index}"
            )
        for metric_index, metric in enumerate(metrics):
            if not isinstance(metric, Mapping):
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:"
                    f"agent_metric_not_mapping:{agent_index}:{metric_index}"
                )
            missing_metric_fields = [
                field for field in _REQUIRED_AGENT_METRIC_FIELDS if field not in metric
            ]
            if missing_metric_fields:
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:agent_metric_missing_fields:"
                    f"{agent_index}:{metric_index}:{','.join(missing_metric_fields)}"
                )
            metric_name = metric.get("metric_name")
            if not isinstance(metric_name, str) or not metric_name:
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:agent_metric_name_invalid:"
                    f"{agent_index}:{metric_index}"
                )
            sample_count = metric.get("sample_count")
            if (
                not isinstance(sample_count, int)
                or isinstance(sample_count, bool)
                or sample_count <= 0
            ):
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:agent_metric_sample_count_invalid:"
                    f"{agent_index}:{metric_index}"
                )
            last_step = metric.get("last_step")
            if not isinstance(last_step, int) or isinstance(last_step, bool) or last_step < 0:
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:agent_metric_last_step_invalid:"
                    f"{agent_index}:{metric_index}"
                )
            for numeric_field in ("value_sum", "min_value", "max_value", "last_value"):
                numeric_value = metric.get(numeric_field)
                if not _is_finite_number(numeric_value):
                    raise ValueError(
                        f"invalid_state_snapshot_at_index:{event_index}:"
                        f"agent_metric_{numeric_field}_invalid:{agent_index}:{metric_index}"
                    )

    for state_field in ("item_states", "npc_states", "room_states"):
        state_entries = payload.get(state_field)
        if not _is_non_string_sequence(state_entries):
            raise ValueError(
                f"invalid_state_snapshot_at_index:{event_index}:{state_field}_invalid"
            )
        for entry_index, entry in enumerate(state_entries):
            if not isinstance(entry, Mapping):
                raise ValueError(
                    f"invalid_state_snapshot_at_index:{event_index}:{state_field}_entry_not_mapping:{entry_index}"
                )


def _is_non_string_sequence(value: object) -> bool:
    return isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray))


def _is_finite_number(value: object) -> bool:
    if isinstance(value, bool):
        return False
    if isinstance(value, int):
        return True
    if isinstance(value, float):
        return value == value and value not in (float("inf"), float("-inf"))
    return False


def _resolve_artifact(
    artifact: object,
) -> tuple[ReplayArtifact | None, ReplayArtifactEmitResult | None, str | None]:
    if isinstance(artifact, ReplayArtifact):
        return artifact, None, None
    if isinstance(artifact, Mapping):
        artifact_result = parse_replay_artifact(artifact)
        if not artifact_result.accepted or artifact_result.artifact is None:
            parse_reason = artifact_result.reason or "unknown_parse_rejection"
            return None, artifact_result, f"invalid_artifact:{parse_reason}"
        return artifact_result.artifact, artifact_result, None
    return None, None, "artifact_must_be_replay_artifact_or_mapping"


def _accepted_result(
    *,
    reconstructed_state: ReconstructedReplayState,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayReconstructionResult:
    return ReplayReconstructionResult(
        accepted=True,
        reconstructed_state=reconstructed_state,
        artifact_result=artifact_result,
    )


def _rejected_result(
    *,
    reason: str,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayReconstructionResult:
    return ReplayReconstructionResult(
        accepted=False,
        reason=reason,
        artifact_result=artifact_result,
    )
