"""Deterministic replay reconstruction verifier."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from evaluation.scorecards.scorecard import Scorecard
from replay.reconstruction.reconstructor import (
    ReconstructedEntityState,
    ReconstructedReplayState,
    RejectedActionTrace,
    ReplayReconstructionResult,
    reconstruct_replay_state,
)

REQUIRED_EXPECTED_STATE_FIELDS = (
    "run_id",
    "initial_seed",
    "seed_source",
    "max_steps",
    "terminal_step",
    "event_count",
    "applied_steps",
    "entities",
    "event_type_counts",
    "rejected_actions",
)
REQUIRED_EXPECTED_ENTITY_FIELDS = (
    "entity_id",
    "location",
    "health",
    "inventory",
    "last_event_type",
)
REQUIRED_EXPECTED_REJECTED_ACTION_FIELDS = (
    "step_index",
    "actor_id",
    "action_type",
    "reason",
)
REQUIRED_PARITY_ARTIFACT_FIELDS = (
    "run_id",
    "terminal_state_hash",
    "replay_event_count",
    "terminal_step",
    "step_count",
    "applied_steps_hash",
    "scorecard_step_count",
    "aggregate_score",
    "score_summary_hash",
    "scorer_version",
)


@dataclass(frozen=True, slots=True)
class ReplayReconstructionVerificationResult:
    """Explicit deterministic verification result for reconstructed replay state."""

    accepted: bool
    matched: bool | None = None
    actual_state: ReconstructedReplayState | None = None
    expected_state: ReconstructedReplayState | None = None
    mismatches: tuple[str, ...] = ()
    reason: str | None = None
    reconstruction_result: ReplayReconstructionResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.matched is None:
                raise ValueError("accepted verification result must include matched")
            if self.actual_state is None:
                raise ValueError("accepted verification result must include actual_state")
            if self.expected_state is None:
                raise ValueError("accepted verification result must include expected_state")
            if self.reason is not None:
                raise ValueError("accepted verification result must not include reason")
            if self.matched and self.mismatches:
                raise ValueError("matched verification result must not include mismatches")
            if not self.matched and not self.mismatches:
                raise ValueError("mismatched verification result must include mismatches")
            return
        if self.reason is None:
            raise ValueError("rejected verification result must include reason")
        if self.matched is not None:
            raise ValueError("rejected verification result must not include matched")
        if self.actual_state is not None or self.expected_state is not None:
            raise ValueError("rejected verification result must not include compared states")
        if self.mismatches:
            raise ValueError("rejected verification result must not include mismatches")


@dataclass(frozen=True, slots=True)
class ReplayParityArtifact:
    """Deterministic parity artifact derived from replay + benchmark outputs."""

    run_id: str
    terminal_state_hash: str
    replay_event_count: int
    terminal_step: int | None
    step_count: int
    applied_steps_hash: str
    scorecard_step_count: int
    aggregate_score: float
    score_summary_hash: str
    scorer_version: str

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        for field_name in (
            "terminal_state_hash",
            "applied_steps_hash",
            "score_summary_hash",
        ):
            value = getattr(self, field_name)
            if not _is_sha256_hex(value):
                raise ValueError(f"{field_name} must be a 64-character lowercase hex digest")
        if (
            not isinstance(self.replay_event_count, int)
            or isinstance(self.replay_event_count, bool)
            or self.replay_event_count < 0
        ):
            raise ValueError("replay_event_count must be a non-negative integer")
        if self.terminal_step is not None:
            if (
                not isinstance(self.terminal_step, int)
                or isinstance(self.terminal_step, bool)
                or self.terminal_step < 0
            ):
                raise ValueError("terminal_step must be None or a non-negative integer")
        if not isinstance(self.step_count, int) or isinstance(self.step_count, bool) or self.step_count < 0:
            raise ValueError("step_count must be a non-negative integer")
        if (
            self.terminal_step is None
            and self.step_count != 0
        ) or (
            self.terminal_step is not None
            and self.step_count != self.terminal_step + 1
        ):
            raise ValueError("step_count must align with terminal_step")
        if (
            not isinstance(self.scorecard_step_count, int)
            or isinstance(self.scorecard_step_count, bool)
            or self.scorecard_step_count < 0
        ):
            raise ValueError("scorecard_step_count must be a non-negative integer")
        _require_unit_interval(self.aggregate_score, field_name="aggregate_score")
        if not isinstance(self.scorer_version, str) or not self.scorer_version:
            raise ValueError("scorer_version must be a non-empty string")

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "terminal_state_hash": self.terminal_state_hash,
            "replay_event_count": self.replay_event_count,
            "terminal_step": self.terminal_step,
            "step_count": self.step_count,
            "applied_steps_hash": self.applied_steps_hash,
            "scorecard_step_count": self.scorecard_step_count,
            "aggregate_score": self.aggregate_score,
            "score_summary_hash": self.score_summary_hash,
            "scorer_version": self.scorer_version,
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class ReplayParityComputationResult:
    """Explicit accept/reject result for parity artifact computation."""

    accepted: bool
    parity_artifact: ReplayParityArtifact | None = None
    reason: str | None = None
    reconstruction_result: ReplayReconstructionResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.parity_artifact is None:
                raise ValueError("accepted parity computation must include parity_artifact")
            if self.reason is not None:
                raise ValueError("accepted parity computation must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected parity computation must include reason")
        if self.parity_artifact is not None:
            raise ValueError("rejected parity computation must not include parity_artifact")


@dataclass(frozen=True, slots=True)
class ReplayParityVerificationResult:
    """Explicit deterministic parity verification result."""

    accepted: bool
    matched: bool | None = None
    actual_parity: ReplayParityArtifact | None = None
    expected_parity: ReplayParityArtifact | None = None
    mismatches: tuple[str, ...] = ()
    reason: str | None = None
    parity_computation_result: ReplayParityComputationResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.matched is None:
                raise ValueError("accepted parity verification must include matched")
            if self.actual_parity is None or self.expected_parity is None:
                raise ValueError("accepted parity verification must include compared artifacts")
            if self.reason is not None:
                raise ValueError("accepted parity verification must not include reason")
            if self.matched and self.mismatches:
                raise ValueError("matched parity verification must not include mismatches")
            if not self.matched and not self.mismatches:
                raise ValueError("mismatched parity verification must include mismatches")
            return
        if self.reason is None:
            raise ValueError("rejected parity verification must include reason")
        if self.matched is not None:
            raise ValueError("rejected parity verification must not include matched")
        if self.actual_parity is not None or self.expected_parity is not None:
            raise ValueError("rejected parity verification must not include compared artifacts")
        if self.mismatches:
            raise ValueError("rejected parity verification must not include mismatches")


def compute_replay_parity_artifact(
    *,
    replay_artifact: object,
    scorecard: object,
) -> ReplayParityComputationResult:
    """Compute deterministic parity artifact from replay and scorecard outputs."""
    resolved_scorecard, scorecard_reason = _resolve_scorecard(scorecard)
    if scorecard_reason is not None:
        return _rejected_parity_computation(reason=f"invalid_scorecard:{scorecard_reason}")
    if resolved_scorecard is None:
        raise RuntimeError("internal error resolving scorecard")

    reconstruction_result = reconstruct_replay_state(replay_artifact)
    if not reconstruction_result.accepted or reconstruction_result.reconstructed_state is None:
        reconstruction_reason = reconstruction_result.reason or "unknown_reconstruction_failure"
        return _rejected_parity_computation(
            reason=f"cannot_reconstruct:{reconstruction_reason}",
            reconstruction_result=reconstruction_result,
        )
    reconstructed = reconstruction_result.reconstructed_state
    if resolved_scorecard.metadata.run_id != reconstructed.run_id:
        return _rejected_parity_computation(
            reason=(
                "run_id_mismatch:scorecard_vs_replay:"
                f"{resolved_scorecard.metadata.run_id}!={reconstructed.run_id}"
            ),
            reconstruction_result=reconstruction_result,
        )

    state_json = reconstructed.canonical_state_json
    if state_json is None:
        state_json = reconstructed.to_canonical_json()
    state_hash = hashlib.sha256(state_json.encode("utf-8")).hexdigest()
    applied_steps_json = json.dumps(
        list(reconstructed.applied_steps),
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    )
    applied_steps_hash = hashlib.sha256(applied_steps_json.encode("utf-8")).hexdigest()

    score_summary = _score_summary_payload(resolved_scorecard)
    score_summary_json = json.dumps(score_summary, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    score_summary_hash = hashlib.sha256(score_summary_json.encode("utf-8")).hexdigest()

    step_count = (reconstructed.terminal_step + 1) if reconstructed.terminal_step is not None else 0
    parity_artifact = ReplayParityArtifact(
        run_id=reconstructed.run_id,
        terminal_state_hash=state_hash,
        replay_event_count=reconstructed.event_count,
        terminal_step=reconstructed.terminal_step,
        step_count=step_count,
        applied_steps_hash=applied_steps_hash,
        scorecard_step_count=resolved_scorecard.metadata.step_count,
        aggregate_score=resolved_scorecard.aggregate_score,
        score_summary_hash=score_summary_hash,
        scorer_version=resolved_scorecard.metadata.scorer_version,
    )
    return ReplayParityComputationResult(
        accepted=True,
        parity_artifact=parity_artifact,
        reconstruction_result=reconstruction_result,
    )


def verify_replay_reexecution_parity(
    *,
    replay_artifact: object,
    scorecard: object,
    expected_parity_artifact: object,
) -> ReplayParityVerificationResult:
    """Verify deterministic replay re-execution parity against expected parity artifact."""
    expected_parity, expected_reason = _resolve_parity_artifact(expected_parity_artifact)
    if expected_reason is not None:
        return _rejected_parity_verification(reason=f"invalid_expected_parity:{expected_reason}")
    if expected_parity is None:
        raise RuntimeError("internal error resolving expected parity artifact")

    computation_result = compute_replay_parity_artifact(
        replay_artifact=replay_artifact,
        scorecard=scorecard,
    )
    if not computation_result.accepted or computation_result.parity_artifact is None:
        reason = computation_result.reason or "unknown_parity_computation_failure"
        return _rejected_parity_verification(
            reason=f"cannot_compute_parity:{reason}",
            parity_computation_result=computation_result,
        )

    actual_parity = computation_result.parity_artifact
    if actual_parity.to_canonical_json() == expected_parity.to_canonical_json():
        return ReplayParityVerificationResult(
            accepted=True,
            matched=True,
            actual_parity=actual_parity,
            expected_parity=expected_parity,
            parity_computation_result=computation_result,
        )

    mismatch_paths = tuple(_compute_mismatch_paths(actual_parity.to_dict(), expected_parity.to_dict()))
    if not mismatch_paths:
        mismatch_paths = ("$.parity_difference_detected",)
    return ReplayParityVerificationResult(
        accepted=True,
        matched=False,
        actual_parity=actual_parity,
        expected_parity=expected_parity,
        mismatches=mismatch_paths,
        parity_computation_result=computation_result,
    )


def verify_replay_reconstruction(
    *,
    artifact: object,
    expected_terminal_state: object,
) -> ReplayReconstructionVerificationResult:
    """Verify reconstructed terminal state against expected terminal state."""
    expected_state, expected_reason = _resolve_expected_state(expected_terminal_state)
    if expected_reason is not None:
        return _rejected_result(reason=f"invalid_expected_state:{expected_reason}")
    if expected_state is None:
        raise RuntimeError("internal error resolving expected state")

    reconstruction_result = reconstruct_replay_state(artifact)
    if not reconstruction_result.accepted or reconstruction_result.reconstructed_state is None:
        reconstruction_reason = reconstruction_result.reason or "unknown_reconstruction_failure"
        return _rejected_result(
            reason=f"cannot_reconstruct:{reconstruction_reason}",
            reconstruction_result=reconstruction_result,
        )

    actual_state = reconstruction_result.reconstructed_state
    if actual_state.to_canonical_json() == expected_state.to_canonical_json():
        return _accepted_result(
            matched=True,
            actual_state=actual_state,
            expected_state=expected_state,
            reconstruction_result=reconstruction_result,
        )

    mismatch_paths = tuple(_compute_mismatch_paths(actual_state.to_dict(), expected_state.to_dict()))
    if not mismatch_paths:
        mismatch_paths = ("$.state_difference_detected",)
    return _accepted_result(
        matched=False,
        actual_state=actual_state,
        expected_state=expected_state,
        mismatches=mismatch_paths,
        reconstruction_result=reconstruction_result,
    )


def _resolve_expected_state(value: object) -> tuple[ReconstructedReplayState | None, str | None]:
    if isinstance(value, ReconstructedReplayState):
        return value, None
    if isinstance(value, Mapping):
        return _parse_expected_state_mapping(value)
    return None, "expected_state_must_be_reconstructed_state_or_mapping"


def _parse_expected_state_mapping(
    payload: Mapping[str, Any],
) -> tuple[ReconstructedReplayState | None, str | None]:
    missing_fields = [field for field in REQUIRED_EXPECTED_STATE_FIELDS if field not in payload]
    if missing_fields:
        return None, f"missing_required_fields:{','.join(missing_fields)}"

    run_id, reason = _read_required_non_empty_string(payload, "run_id")
    if reason is not None:
        return None, reason
    initial_seed, reason = _read_required_non_negative_int(payload, "initial_seed")
    if reason is not None:
        return None, reason
    seed_source, reason = _read_required_non_empty_string(payload, "seed_source")
    if reason is not None:
        return None, reason
    max_steps, reason = _read_required_positive_int(payload, "max_steps")
    if reason is not None:
        return None, reason
    terminal_step, reason = _read_optional_non_negative_int(payload, "terminal_step")
    if reason is not None:
        return None, reason
    event_count, reason = _read_required_non_negative_int(payload, "event_count")
    if reason is not None:
        return None, reason

    applied_steps, reason = _parse_applied_steps(payload.get("applied_steps"))
    if reason is not None:
        return None, reason
    entities, reason = _parse_entities(payload.get("entities"))
    if reason is not None:
        return None, reason
    event_type_counts, reason = _parse_event_type_counts(payload.get("event_type_counts"))
    if reason is not None:
        return None, reason
    rejected_actions, reason = _parse_rejected_actions(payload.get("rejected_actions"))
    if reason is not None:
        return None, reason
    canonical_state_json, reason = _parse_optional_canonical_state(payload.get("canonical_state"))
    if reason is not None:
        return None, reason

    try:
        return (
            ReconstructedReplayState(
                run_id=run_id,
                initial_seed=initial_seed,
                seed_source=seed_source,
                max_steps=max_steps,
                terminal_step=terminal_step,
                event_count=event_count,
                applied_steps=applied_steps,
                entities=entities,
                event_type_counts=event_type_counts,
                rejected_actions=rejected_actions,
                canonical_state_json=canonical_state_json,
            ),
            None,
        )
    except ValueError as exc:
        return None, str(exc)


def _parse_applied_steps(value: object) -> tuple[tuple[int, ...], str | None]:
    if _is_non_string_sequence(value):
        parsed: list[int] = []
        for index, item in enumerate(value):
            if not isinstance(item, int) or isinstance(item, bool) or item < 0:
                return (), f"applied_steps_item_invalid_at_index:{index}"
            parsed.append(item)
        return tuple(parsed), None
    return (), "applied_steps_must_be_sequence_of_non_negative_integers"


def _parse_entities(
    value: object,
) -> tuple[tuple[ReconstructedEntityState, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "entities_must_be_sequence"

    parsed_entities: list[ReconstructedEntityState] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"entity_must_be_mapping_at_index:{index}"
        missing_fields = [field for field in REQUIRED_EXPECTED_ENTITY_FIELDS if field not in item]
        if missing_fields:
            return (), f"entity_missing_required_fields_at_index:{index}:{','.join(missing_fields)}"

        entity_id, reason = _read_required_non_empty_string(item, "entity_id")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"

        location, reason = _read_optional_non_empty_string(item, "location")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"

        health, reason = _read_optional_int(item, "health")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"

        inventory, reason = _parse_inventory(item.get("inventory"))
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"

        last_event_type, reason = _read_optional_non_empty_string(item, "last_event_type")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"

        try:
            parsed_entities.append(
                ReconstructedEntityState(
                    entity_id=entity_id,
                    location=location,
                    health=health,
                    inventory=inventory,
                    last_event_type=last_event_type,
                )
            )
        except ValueError as exc:
            return (), f"entity_invalid_at_index:{index}:{exc}"
    return tuple(parsed_entities), None


def _parse_event_type_counts(
    value: object,
) -> tuple[tuple[tuple[str, int], ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "event_type_counts_must_be_sequence"

    parsed_counts: list[tuple[str, int]] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"event_type_count_must_be_mapping_at_index:{index}"
        event_type, reason = _read_required_non_empty_string(item, "event_type")
        if reason is not None:
            return (), f"event_type_count_invalid_at_index:{index}:{reason}"
        count, reason = _read_required_non_negative_int(item, "count")
        if reason is not None:
            return (), f"event_type_count_invalid_at_index:{index}:{reason}"
        parsed_counts.append((event_type, count))
    return tuple(parsed_counts), None


def _parse_rejected_actions(
    value: object,
) -> tuple[tuple[RejectedActionTrace, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "rejected_actions_must_be_sequence"

    parsed_actions: list[RejectedActionTrace] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"rejected_action_must_be_mapping_at_index:{index}"
        missing_fields = [field for field in REQUIRED_EXPECTED_REJECTED_ACTION_FIELDS if field not in item]
        if missing_fields:
            return (), f"rejected_action_missing_required_fields_at_index:{index}:{','.join(missing_fields)}"

        step_index, reason = _read_required_non_negative_int(item, "step_index")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        actor_id, reason = _read_optional_non_empty_string(item, "actor_id")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        action_type, reason = _read_required_non_empty_string(item, "action_type")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        reason_value, reason = _read_required_non_empty_string(item, "reason")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        try:
            parsed_actions.append(
                RejectedActionTrace(
                    step_index=step_index,
                    actor_id=actor_id,
                    action_type=action_type,
                    reason=reason_value,
                )
            )
        except ValueError as exc:
            return (), f"rejected_action_invalid_at_index:{index}:{exc}"
    return tuple(parsed_actions), None


def _parse_inventory(value: object) -> tuple[tuple[str, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "inventory_must_be_sequence_of_non_empty_strings"
    inventory: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item:
            return (), f"inventory_item_invalid_at_index:{index}"
        inventory.append(item)
    return tuple(inventory), None


def _parse_optional_canonical_state(value: object) -> tuple[str | None, str | None]:
    if value is None:
        return None, None
    if not isinstance(value, Mapping):
        return None, "canonical_state_must_be_mapping"
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True), None


def _read_required_non_empty_string(payload: Mapping[str, Any], field_name: str) -> tuple[str, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        return "", f"{field_name}_must_be_non_empty_string"
    return value, None


def _read_optional_non_empty_string(payload: Mapping[str, Any], field_name: str) -> tuple[str | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, str) or not value:
        return None, f"{field_name}_must_be_none_or_non_empty_string"
    return value, None


def _read_required_positive_int(payload: Mapping[str, Any], field_name: str) -> tuple[int, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        return 0, f"{field_name}_must_be_positive_integer"
    return value, None


def _read_required_non_negative_int(payload: Mapping[str, Any], field_name: str) -> tuple[int, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        return 0, f"{field_name}_must_be_non_negative_integer"
    return value, None


def _read_optional_non_negative_int(
    payload: Mapping[str, Any], field_name: str
) -> tuple[int | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        return None, f"{field_name}_must_be_none_or_non_negative_integer"
    return value, None


def _read_optional_int(payload: Mapping[str, Any], field_name: str) -> tuple[int | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, int) or isinstance(value, bool):
        return None, f"{field_name}_must_be_none_or_integer"
    return value, None


def _compute_mismatch_paths(
    actual: object,
    expected: object,
    *,
    path: str = "$",
) -> list[str]:
    if isinstance(actual, Mapping) and isinstance(expected, Mapping):
        mismatches: list[str] = []
        all_keys = sorted(set(actual.keys()) | set(expected.keys()), key=lambda key: str(key))
        for key in all_keys:
            key_path = f"{path}.{key}"
            if key not in actual:
                mismatches.append(f"{key_path}:missing_in_actual")
                continue
            if key not in expected:
                mismatches.append(f"{key_path}:missing_in_expected")
                continue
            mismatches.extend(_compute_mismatch_paths(actual[key], expected[key], path=key_path))
        return mismatches

    if _is_non_string_sequence(actual) and _is_non_string_sequence(expected):
        mismatches = []
        if len(actual) != len(expected):
            mismatches.append(f"{path}:length_mismatch:actual={len(actual)}:expected={len(expected)}")
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected)):
            mismatches.extend(
                _compute_mismatch_paths(
                    actual_item,
                    expected_item,
                    path=f"{path}[{index}]",
                )
            )
        return mismatches

    if type(actual) is not type(expected):
        return [f"{path}:type_mismatch:actual={type(actual).__name__}:expected={type(expected).__name__}"]

    if actual != expected:
        return [f"{path}:actual={_render_value(actual)}:expected={_render_value(expected)}"]
    return []


def _render_value(value: object) -> str:
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    except TypeError:
        return repr(value)


def _is_non_string_sequence(value: object) -> bool:
    return isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray))


def _accepted_result(
    *,
    matched: bool,
    actual_state: ReconstructedReplayState,
    expected_state: ReconstructedReplayState,
    mismatches: tuple[str, ...] = (),
    reconstruction_result: ReplayReconstructionResult | None = None,
) -> ReplayReconstructionVerificationResult:
    return ReplayReconstructionVerificationResult(
        accepted=True,
        matched=matched,
        actual_state=actual_state,
        expected_state=expected_state,
        mismatches=mismatches,
        reconstruction_result=reconstruction_result,
    )


def _rejected_result(
    *,
    reason: str,
    reconstruction_result: ReplayReconstructionResult | None = None,
) -> ReplayReconstructionVerificationResult:
    return ReplayReconstructionVerificationResult(
        accepted=False,
        reason=reason,
        reconstruction_result=reconstruction_result,
    )


def _resolve_scorecard(value: object) -> tuple[Scorecard | None, str | None]:
    if isinstance(value, Scorecard):
        return value, None
    if isinstance(value, Mapping):
        try:
            return Scorecard.from_dict(value), None
        except ValueError as exc:
            return None, str(exc)
    return None, "scorecard_must_be_scorecard_or_mapping"


def _resolve_parity_artifact(value: object) -> tuple[ReplayParityArtifact | None, str | None]:
    if isinstance(value, ReplayParityArtifact):
        return value, None
    if isinstance(value, Mapping):
        return _parse_parity_artifact_mapping(value)
    return None, "expected_parity_must_be_replay_parity_artifact_or_mapping"


def _parse_parity_artifact_mapping(
    payload: Mapping[str, Any],
) -> tuple[ReplayParityArtifact | None, str | None]:
    missing_fields = [field for field in REQUIRED_PARITY_ARTIFACT_FIELDS if field not in payload]
    if missing_fields:
        return None, f"missing_required_fields:{','.join(missing_fields)}"

    run_id, reason = _read_required_non_empty_string(payload, "run_id")
    if reason is not None:
        return None, reason
    terminal_state_hash, reason = _read_required_sha256_hex(payload, "terminal_state_hash")
    if reason is not None:
        return None, reason
    replay_event_count, reason = _read_required_non_negative_int(payload, "replay_event_count")
    if reason is not None:
        return None, reason
    terminal_step, reason = _read_optional_non_negative_int(payload, "terminal_step")
    if reason is not None:
        return None, reason
    step_count, reason = _read_required_non_negative_int(payload, "step_count")
    if reason is not None:
        return None, reason
    applied_steps_hash, reason = _read_required_sha256_hex(payload, "applied_steps_hash")
    if reason is not None:
        return None, reason
    scorecard_step_count, reason = _read_required_non_negative_int(payload, "scorecard_step_count")
    if reason is not None:
        return None, reason
    aggregate_score, reason = _read_required_unit_interval_float(payload, "aggregate_score")
    if reason is not None:
        return None, reason
    score_summary_hash, reason = _read_required_sha256_hex(payload, "score_summary_hash")
    if reason is not None:
        return None, reason
    scorer_version, reason = _read_required_non_empty_string(payload, "scorer_version")
    if reason is not None:
        return None, reason

    try:
        return (
            ReplayParityArtifact(
                run_id=run_id,
                terminal_state_hash=terminal_state_hash,
                replay_event_count=replay_event_count,
                terminal_step=terminal_step,
                step_count=step_count,
                applied_steps_hash=applied_steps_hash,
                scorecard_step_count=scorecard_step_count,
                aggregate_score=aggregate_score,
                score_summary_hash=score_summary_hash,
                scorer_version=scorer_version,
            ),
            None,
        )
    except ValueError as exc:
        return None, str(exc)


def _score_summary_payload(scorecard: Scorecard) -> dict[str, Any]:
    actor_summaries = [
        {
            "actor_id": actor.actor_id,
            "composite_score": actor.composite_score,
            "normalized_metrics": {key: value for key, value in actor.normalized_metrics},
            "contributions": {key: value for key, value in actor.contributions},
        }
        for actor in sorted(scorecard.actors, key=lambda candidate: candidate.actor_id)
    ]
    return {
        "run_id": scorecard.metadata.run_id,
        "benchmark_id": scorecard.metadata.benchmark_id,
        "scenario_id": scorecard.metadata.scenario_id,
        "seed": scorecard.metadata.seed,
        "step_count": scorecard.metadata.step_count,
        "scorer_version": scorecard.metadata.scorer_version,
        "aggregate_score": scorecard.aggregate_score,
        "aggregate_metrics": {key: value for key, value in scorecard.aggregate_metrics},
        "aggregate_contributions": {key: value for key, value in scorecard.aggregate_contributions},
        "actors": actor_summaries,
    }


def _read_required_sha256_hex(payload: Mapping[str, Any], field_name: str) -> tuple[str, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, str) or not _is_sha256_hex(value):
        return "", f"{field_name}_must_be_sha256_hex_digest"
    return value, None


def _read_required_unit_interval_float(
    payload: Mapping[str, Any], field_name: str
) -> tuple[float, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return 0.0, f"{field_name}_must_be_number_in_unit_interval"
    normalized = float(value)
    if normalized < 0.0 or normalized > 1.0:
        return 0.0, f"{field_name}_must_be_number_in_unit_interval"
    return normalized, None


def _require_unit_interval(value: object, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric and within [0.0, 1.0]")
    normalized = float(value)
    if normalized < 0.0 or normalized > 1.0:
        raise ValueError(f"{field_name} must be numeric and within [0.0, 1.0]")


def _is_sha256_hex(value: object) -> bool:
    if not isinstance(value, str) or len(value) != 64:
        return False
    return all(character in "0123456789abcdef" for character in value)


def _rejected_parity_computation(
    *,
    reason: str,
    reconstruction_result: ReplayReconstructionResult | None = None,
) -> ReplayParityComputationResult:
    return ReplayParityComputationResult(
        accepted=False,
        reason=reason,
        reconstruction_result=reconstruction_result,
    )


def _rejected_parity_verification(
    *,
    reason: str,
    parity_computation_result: ReplayParityComputationResult | None = None,
) -> ReplayParityVerificationResult:
    return ReplayParityVerificationResult(
        accepted=False,
        reason=reason,
        parity_computation_result=parity_computation_result,
    )
