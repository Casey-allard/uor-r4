"""Minimal shard-state skeleton for future persistent-world mode."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any, Mapping

from .shard_identity_registry import InProcessShardIdentityRegistry


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _require_non_negative_int(value: Any, *, field_name: str) -> int:
    if not isinstance(value, int) or value < 0:
        raise ValueError(f"{field_name} must be a non-negative integer")
    return value


def _require_positive_int(value: Any, *, field_name: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ValueError(f"{field_name} must be a positive integer")
    return value


def _normalize_optional_timing_mode(value: Any) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value:
        raise ValueError("timing_mode must be None or a non-empty string")
    return value


def _normalize_actor_tick_mapping(
    value: Mapping[str, int] | tuple[tuple[str, int], ...],
    *,
    field_name: str,
    require_positive_values: bool,
) -> tuple[tuple[str, int], ...]:
    if isinstance(value, tuple):
        items = value
    elif isinstance(value, Mapping):
        items = tuple(value.items())
    else:
        raise ValueError(f"{field_name} must be a mapping or tuple of pairs")
    normalized: dict[str, int] = {}
    for actor_id, tick_value in items:
        _require_non_empty_string(actor_id, field_name=f"{field_name}.actor_id")
        if require_positive_values:
            normalized_value = _require_positive_int(
                tick_value,
                field_name=f"{field_name}.{actor_id}",
            )
        else:
            normalized_value = _require_non_negative_int(
                tick_value,
                field_name=f"{field_name}.{actor_id}",
            )
        normalized[actor_id] = normalized_value
    return tuple((actor_id, normalized[actor_id]) for actor_id in sorted(normalized))


def _count_records_by_attr(records: tuple[Any, ...], *, field_name: str) -> dict[str, int]:
    counts: dict[str, int] = {}
    for record in records:
        key = str(getattr(record, field_name))
        counts[key] = counts.get(key, 0) + 1
    return {key: counts[key] for key in sorted(counts)}


def _coerce_output_path(path: Any) -> Path:
    try:
        return Path(path)
    except TypeError as exc:
        raise ValueError("checkpoint path must be a string or path-like value") from exc


def _write_json_payload(path: Any, payload: dict[str, Any], *, artifact_label: str) -> str:
    output_path = _coerce_output_path(path)
    if output_path.exists() and output_path.is_dir():
        raise ValueError(f"{artifact_label} path must be a file path, not directory: {output_path}")

    parent = output_path.parent
    if not parent.exists():
        raise ValueError(f"{artifact_label} parent directory does not exist: {parent}")
    if not parent.is_dir():
        raise ValueError(f"{artifact_label} parent path is not a directory: {parent}")

    serialized = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    try:
        output_path.write_text(serialized, encoding="utf-8")
    except OSError as exc:
        raise ValueError(
            f"failed to write {artifact_label} to {output_path}: {exc}"
        ) from exc
    return str(output_path)


def _coerce_output_directory(path: Any) -> Path:
    try:
        output_directory = Path(path)
    except TypeError as exc:
        raise ValueError("artifact bundle directory must be a string or path-like value") from exc
    if not output_directory.exists():
        raise ValueError(f"artifact bundle directory does not exist: {output_directory}")
    if not output_directory.is_dir():
        raise ValueError(f"artifact bundle path is not a directory: {output_directory}")
    return output_directory


def _sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _read_json_file(path: Path) -> dict[str, Any] | None:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if not isinstance(payload, dict):
        return None
    return payload


def _expected_bundle_filenames(shard_id: str) -> dict[str, str]:
    file_prefix = shard_id + "."
    return {
        "artifact_index": file_prefix + "artifact_index.json",
        "artifact_manifest": file_prefix + "artifact_manifest.json",
        "shard_checkpoint_placeholder": file_prefix + "shard_checkpoint_placeholder.json",
        "registry_checkpoint_placeholder": file_prefix + "registry_checkpoint_placeholder.json",
        "mutation_journal_placeholder": file_prefix + "mutation_journal_placeholder.json",
        "shard_export_payload": file_prefix + "shard_export_placeholder.json",
    }


def _world_npc_stance_phase_for_tick_count(world_tick_count: int) -> str:
    _require_non_negative_int(world_tick_count, field_name="world_tick_count")
    if world_tick_count == 0:
        return "dormant"
    phase_cycle = ("watchful", "patrolling", "settling")
    return phase_cycle[(world_tick_count - 1) % len(phase_cycle)]


def _world_tick_scene_message_for_stance_phase(npc_stance_phase: str) -> str:
    _require_non_empty_string(npc_stance_phase, field_name="npc_stance_phase")
    scene_messages = {
        "dormant": "The shard feels still, as if the watch has not yet begun.",
        "watchful": "A distant sentinel grows watchful in the quiet corridors.",
        "patrolling": "You catch the measured rhythm of a distant patrol.",
        "settling": "The far-off watch settles back into guarded stillness.",
    }
    return scene_messages.get(
        npc_stance_phase,
        f"The shard shifts into a {npc_stance_phase} phase.",
    )


def _world_phase_interaction_hint_for_stance_phase(npc_stance_phase: str) -> str:
    _require_non_empty_string(npc_stance_phase, field_name="npc_stance_phase")
    interaction_hints = {
        "dormant": "Hint: the route feels open while the watch remains dormant.",
        "watchful": "Hint: the sharp watch makes a careful look feel safer than rushing.",
        "patrolling": "Hint: the moving patrol leaves brief windows for repositioning.",
        "settling": "Hint: the easing watch makes nearby movement feel less exposed.",
    }
    return interaction_hints.get(
        npc_stance_phase,
        f"Hint: the shard's {npc_stance_phase} phase may change how the route feels.",
    )


def _world_phase_outcome_message_for_location_and_stance_phase(
    *,
    location: str,
    npc_stance_phase: str,
) -> str | None:
    _require_non_empty_string(location, field_name="location")
    _require_non_empty_string(npc_stance_phase, field_name="npc_stance_phase")
    if location != "corridor":
        return None

    phase_messages = {
        "dormant": "Consequence: the exposed west passage is open before the watch begins.",
        "watchful": (
            "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
        ),
        "patrolling": "Consequence: the west passage opens again between patrol sweeps.",
        "settling": "Consequence: the west passage remains open as the watch settles.",
    }
    return phase_messages.get(
        npc_stance_phase,
        f"Consequence: the corridor route shifts under the shard's {npc_stance_phase} phase.",
    )


def _world_phase_filtered_action_space_for_location_and_stance_phase(
    *,
    location: str,
    npc_stance_phase: str,
    action_space: tuple[str, ...],
) -> tuple[str, ...]:
    _require_non_empty_string(location, field_name="location")
    _require_non_empty_string(npc_stance_phase, field_name="npc_stance_phase")
    if location == "corridor" and npc_stance_phase == "watchful":
        return tuple(action for action in action_space if action != "move west")
    return action_space


_HASH_BUNDLE_ISSUE_CODES = frozenset({
    "content_hash_mismatch",
})

_LINKAGE_BUNDLE_ISSUE_CODES = frozenset({
    "shard_checkpoint_generation_mismatch",
    "registry_checkpoint_generation_mismatch",
    "mutation_journal_generation_mismatch",
    "shard_export_generation_mismatch",
})

_MANIFEST_BUNDLE_ISSUE_CODES = frozenset({
    "missing_artifact_manifest",
    "invalid_artifact_manifest",
    "manifest_shard_id_mismatch",
    "manifest_index_filename_mismatch",
    "manifest_bundle_generation_mismatch",
    "manifest_expected_next_generation_mismatch",
    "manifest_artifact_filename_mismatch",
    "manifest_linkage_mismatch",
})

_DEGRADED_BUNDLE_ISSUE_CODES = frozenset({
    "filename_mismatch",
    "artifact_index_filename_mismatch",
})

_NON_RESUMABLE_BUNDLE_ISSUE_CODES = frozenset({
    "missing_artifact_index",
    "multiple_artifact_indexes",
    "invalid_artifact_index",
    "invalid_shard_id",
    "invalid_artifact_entries",
    "missing_artifact_entry",
    "missing_artifact_file",
    "invalid_artifact_payload",
    "invalid_bundle_generation",
}) | _HASH_BUNDLE_ISSUE_CODES | _LINKAGE_BUNDLE_ISSUE_CODES | _MANIFEST_BUNDLE_ISSUE_CODES


def _summarize_bundle_verification_result(verification_result: dict[str, Any]) -> dict[str, Any]:
    artifacts = verification_result.get("artifacts", ())
    if not isinstance(artifacts, list):
        artifacts = []
    verified_artifact_count = sum(
        1
        for artifact in artifacts
        if isinstance(artifact, dict)
        and artifact.get("exists") is True
        and artifact.get("hash_matches") is True
    )
    issue_codes = verification_result.get("issue_codes", ())
    if not isinstance(issue_codes, list):
        issue_codes = []
    normalized_issue_codes = sorted({str(issue_code) for issue_code in issue_codes})
    hash_issue_codes = [
        issue_code for issue_code in normalized_issue_codes if issue_code in _HASH_BUNDLE_ISSUE_CODES
    ]
    linkage_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code in _LINKAGE_BUNDLE_ISSUE_CODES
    ]
    manifest_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code in _MANIFEST_BUNDLE_ISSUE_CODES
    ]
    other_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code not in _HASH_BUNDLE_ISSUE_CODES
        and issue_code not in _LINKAGE_BUNDLE_ISSUE_CODES
        and issue_code not in _MANIFEST_BUNDLE_ISSUE_CODES
    ]
    return {
        "is_valid": bool(verification_result.get("is_valid", False)),
        "issue_codes": normalized_issue_codes,
        "artifact_count": len(artifacts),
        "verified_artifact_count": verified_artifact_count,
        "hash_issue_codes": hash_issue_codes,
        "linkage_issue_codes": linkage_issue_codes,
        "manifest_issue_codes": manifest_issue_codes,
        "issue_category_counts": {
            "hash": len(hash_issue_codes),
            "linkage": len(linkage_issue_codes),
            "manifest": len(manifest_issue_codes),
            "other": len(other_issue_codes),
        },
    }


def _classify_bundle_recovery_result(verification_result: dict[str, Any]) -> dict[str, Any]:
    issue_codes = verification_result.get("issue_codes", ())
    if not isinstance(issue_codes, list):
        issue_codes = []
    normalized_issue_codes = sorted({str(issue_code) for issue_code in issue_codes})
    fatal_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code in _NON_RESUMABLE_BUNDLE_ISSUE_CODES
    ]
    degradable_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code in _DEGRADED_BUNDLE_ISSUE_CODES
    ]
    other_issue_codes = [
        issue_code
        for issue_code in normalized_issue_codes
        if issue_code not in _NON_RESUMABLE_BUNDLE_ISSUE_CODES
        and issue_code not in _DEGRADED_BUNDLE_ISSUE_CODES
    ]
    if len(fatal_issue_codes) > 0 or len(other_issue_codes) > 0:
        recovery_state = "non_resumable"
    elif len(degradable_issue_codes) > 0:
        recovery_state = "degraded"
    else:
        recovery_state = "resumable"
    return {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": verification_result.get("bundle_directory"),
        "shard_id": verification_result.get("shard_id"),
        "recovery_state": recovery_state,
        "issue_codes": normalized_issue_codes,
        "fatal_issue_codes": fatal_issue_codes,
        "degradable_issue_codes": degradable_issue_codes,
        "verification_is_valid": bool(verification_result.get("is_valid", False)),
    }


@dataclass(frozen=True, slots=True)
class ShardMetadata:
    """Minimal shard metadata for in-process persistent-world scaffolding."""

    shard_id: str
    mode_marker: str = "persistent_world_shell"
    shard_status: str = "initializing"
    world_ruleset_version: str = "unconfigured"
    benchmark_engine_version: str = "unconfigured"
    scheduler_policy_version: str = "unconfigured"
    world_tick_count: int = 0
    last_world_tick_heartbeat: str = "not_started"
    npc_stance_phase: str = "dormant"
    timing_mode: str | None = None
    action_cadence_interval: int | None = None
    actor_action_cadence_overrides: tuple[tuple[str, int], ...] = ()
    actor_next_action_eligible_at: tuple[tuple[str, int], ...] = ()
    season_id: str | None = None
    created_from_snapshot_ref: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.shard_id, field_name="shard_id")
        _require_non_empty_string(self.mode_marker, field_name="mode_marker")
        _require_non_empty_string(self.shard_status, field_name="shard_status")
        _require_non_empty_string(self.world_ruleset_version, field_name="world_ruleset_version")
        _require_non_empty_string(
            self.benchmark_engine_version,
            field_name="benchmark_engine_version",
        )
        _require_non_empty_string(
            self.scheduler_policy_version,
            field_name="scheduler_policy_version",
        )
        _require_non_negative_int(self.world_tick_count, field_name="world_tick_count")
        _require_non_empty_string(
            self.last_world_tick_heartbeat,
            field_name="last_world_tick_heartbeat",
        )
        _require_non_empty_string(self.npc_stance_phase, field_name="npc_stance_phase")
        object.__setattr__(self, "timing_mode", _normalize_optional_timing_mode(self.timing_mode))
        if self.action_cadence_interval is not None:
            _require_positive_int(
                self.action_cadence_interval,
                field_name="action_cadence_interval",
            )
        normalized_overrides = _normalize_actor_tick_mapping(
            self.actor_action_cadence_overrides,
            field_name="actor_action_cadence_overrides",
            require_positive_values=True,
        )
        normalized_next_eligible = _normalize_actor_tick_mapping(
            self.actor_next_action_eligible_at,
            field_name="actor_next_action_eligible_at",
            require_positive_values=False,
        )
        object.__setattr__(self, "actor_action_cadence_overrides", normalized_overrides)
        object.__setattr__(self, "actor_next_action_eligible_at", normalized_next_eligible)
        if self.action_cadence_interval is None and len(normalized_overrides) > 0:
            raise ValueError("actor_action_cadence_overrides require action_cadence_interval")

    def to_dict(self) -> dict[str, Any]:
        return {
            "shard_id": self.shard_id,
            "mode_marker": self.mode_marker,
            "shard_status": self.shard_status,
            "world_ruleset_version": self.world_ruleset_version,
            "benchmark_engine_version": self.benchmark_engine_version,
            "scheduler_policy_version": self.scheduler_policy_version,
            "world_tick_count": self.world_tick_count,
            "last_world_tick_heartbeat": self.last_world_tick_heartbeat,
            "npc_stance_phase": self.npc_stance_phase,
            "timing_mode": self.timing_mode,
            "action_cadence_interval": self.action_cadence_interval,
            "actor_action_cadence_overrides": [
                {"actor_id": actor_id, "cadence_interval": cadence_interval}
                for actor_id, cadence_interval in self.actor_action_cadence_overrides
            ],
            "actor_next_action_eligible_at": [
                {"actor_id": actor_id, "next_action_eligible_at": next_action_eligible_at}
                for actor_id, next_action_eligible_at in self.actor_next_action_eligible_at
            ],
            "season_id": self.season_id,
            "created_from_snapshot_ref": self.created_from_snapshot_ref,
        }


@dataclass(frozen=True, slots=True)
class ShardCheckpointMetadata:
    """Placeholder checkpoint metadata with no persistence backend yet."""

    registry_checkpoint_generation: int = 0
    registry_checkpoint_id: str | None = None
    checkpoint_commit_marker: str = "not_persisted"
    snapshot_payload_hash: str | None = None

    def __post_init__(self) -> None:
        _require_non_negative_int(
            self.registry_checkpoint_generation,
            field_name="registry_checkpoint_generation",
        )
        _require_non_empty_string(self.checkpoint_commit_marker, field_name="checkpoint_commit_marker")

    def to_dict(self) -> dict[str, Any]:
        return {
            "registry_checkpoint_generation": self.registry_checkpoint_generation,
            "registry_checkpoint_id": self.registry_checkpoint_id,
            "checkpoint_commit_marker": self.checkpoint_commit_marker,
            "snapshot_payload_hash": self.snapshot_payload_hash,
        }


@dataclass(frozen=True, slots=True)
class ShardJournalMetadata:
    """Placeholder journal metadata with deterministic generation markers."""

    last_committed_mutation_generation: int = 0
    last_committed_mutation_id: str | None = None
    expected_next_mutation_generation: int = 1
    mutation_journal_mode: str = "not_persisted"

    def __post_init__(self) -> None:
        _require_non_negative_int(
            self.last_committed_mutation_generation,
            field_name="last_committed_mutation_generation",
        )
        _require_non_negative_int(
            self.expected_next_mutation_generation,
            field_name="expected_next_mutation_generation",
        )
        _require_non_empty_string(self.mutation_journal_mode, field_name="mutation_journal_mode")
        if self.expected_next_mutation_generation != self.last_committed_mutation_generation + 1:
            raise ValueError(
                "expected_next_mutation_generation must equal last_committed_mutation_generation + 1"
            )

    def to_dict(self) -> dict[str, Any]:
        return {
            "last_committed_mutation_generation": self.last_committed_mutation_generation,
            "last_committed_mutation_id": self.last_committed_mutation_id,
            "expected_next_mutation_generation": self.expected_next_mutation_generation,
            "mutation_journal_mode": self.mutation_journal_mode,
        }


@dataclass(frozen=True, slots=True)
class ShardMutationPolicy:
    """Minimal in-process mutation policy surface for shard-state helpers."""

    enforce_session_principal_reconciliation: bool = False

    def to_dict(self) -> dict[str, Any]:
        return {
            "enforce_session_principal_reconciliation": self.enforce_session_principal_reconciliation,
        }


@dataclass(frozen=True, slots=True)
class ShardState:
    """Deterministic in-process shard-state shell."""

    metadata: ShardMetadata
    checkpoint: ShardCheckpointMetadata
    journal: ShardJournalMetadata
    identity_registry: InProcessShardIdentityRegistry
    mutation_policy: ShardMutationPolicy = field(default_factory=ShardMutationPolicy)

    def __post_init__(self) -> None:
        if self.metadata.shard_id != self.identity_registry.shard_id:
            raise ValueError("metadata.shard_id must match identity_registry.shard_id")
        if (
            self.journal.last_committed_mutation_generation
            != self.identity_registry.last_committed_mutation_generation
        ):
            raise ValueError(
                "journal.last_committed_mutation_generation must match identity_registry"
            )
        if (
            self.journal.expected_next_mutation_generation
            != self.identity_registry.expected_next_mutation_generation
        ):
            raise ValueError(
                "journal.expected_next_mutation_generation must match identity_registry"
            )
        if self.checkpoint.registry_checkpoint_generation > self.journal.last_committed_mutation_generation:
            raise ValueError(
                "checkpoint generation cannot exceed the latest committed mutation generation"
            )

    @property
    def shard_id(self) -> str:
        return self.metadata.shard_id

    def with_mutation_policy(self, mutation_policy: ShardMutationPolicy) -> "ShardState":
        return replace(self, mutation_policy=mutation_policy)

    @property
    def action_cadence_interval(self) -> int | None:
        return self.metadata.action_cadence_interval

    @property
    def action_cadence_enabled(self) -> bool:
        return self.metadata.action_cadence_interval is not None

    @property
    def timing_mode(self) -> str | None:
        return self.metadata.timing_mode

    @property
    def actor_action_cadence_overrides(self) -> tuple[tuple[str, int], ...]:
        return self.metadata.actor_action_cadence_overrides

    @property
    def actor_next_action_eligible_at(self) -> tuple[tuple[str, int], ...]:
        return self.metadata.actor_next_action_eligible_at

    def with_action_cadence(
        self,
        *,
        action_cadence_interval: int | None,
        actor_action_cadence_overrides: Mapping[str, int] | None = None,
        timing_mode: str | None = None,
    ) -> "ShardState":
        if action_cadence_interval is None:
            normalized_overrides: tuple[tuple[str, int], ...] = ()
        else:
            _require_positive_int(
                action_cadence_interval,
                field_name="action_cadence_interval",
            )
            normalized_overrides = _normalize_actor_tick_mapping(
                actor_action_cadence_overrides or (),
                field_name="actor_action_cadence_overrides",
                require_positive_values=True,
            )
        return replace(
            self,
            metadata=replace(
                self.metadata,
                timing_mode=_normalize_optional_timing_mode(timing_mode),
                action_cadence_interval=action_cadence_interval,
                actor_action_cadence_overrides=normalized_overrides,
                actor_next_action_eligible_at=(),
            ),
        )

    def with_session_principal_reconciliation_enforced(
        self,
        enforced: bool = True,
    ) -> "ShardState":
        return self.with_mutation_policy(
            ShardMutationPolicy(
                enforce_session_principal_reconciliation=bool(enforced),
            )
        )

    def with_identity_registry(self, identity_registry: InProcessShardIdentityRegistry) -> "ShardState":
        if self.metadata.shard_id != identity_registry.shard_id:
            raise ValueError("metadata.shard_id must match identity_registry.shard_id")
        return replace(
            self,
            identity_registry=identity_registry,
            journal=ShardJournalMetadata(
                last_committed_mutation_generation=identity_registry.last_committed_mutation_generation,
                last_committed_mutation_id=identity_registry.last_committed_mutation_id,
                expected_next_mutation_generation=identity_registry.expected_next_mutation_generation,
            ),
        )

    def advance_world_tick(
        self,
        *,
        heartbeat_prefix: str = "shared_shard_world_tick",
    ) -> "ShardState":
        _require_non_empty_string(heartbeat_prefix, field_name="heartbeat_prefix")
        next_tick_count = self.metadata.world_tick_count + 1
        return replace(
            self,
            metadata=replace(
                self.metadata,
                world_tick_count=next_tick_count,
                last_world_tick_heartbeat=f"{heartbeat_prefix}:{next_tick_count:04d}",
                npc_stance_phase=_world_npc_stance_phase_for_tick_count(next_tick_count),
            ),
        )

    def get_actor_action_cadence_interval(self, actor_id: str) -> int | None:
        _require_non_empty_string(actor_id, field_name="actor_id")
        if not self.action_cadence_enabled:
            return None
        override_map = dict(self.metadata.actor_action_cadence_overrides)
        return override_map.get(actor_id, self.metadata.action_cadence_interval)

    def get_actor_next_action_eligible_at(self, actor_id: str) -> int:
        _require_non_empty_string(actor_id, field_name="actor_id")
        next_eligible_map = dict(self.metadata.actor_next_action_eligible_at)
        return next_eligible_map.get(actor_id, 0)

    def is_actor_action_eligible(self, actor_id: str) -> bool:
        _require_non_empty_string(actor_id, field_name="actor_id")
        if not self.action_cadence_enabled:
            return True
        return self.metadata.world_tick_count >= self.get_actor_next_action_eligible_at(actor_id)

    def record_actor_action_acceptance(self, actor_id: str) -> "ShardState":
        _require_non_empty_string(actor_id, field_name="actor_id")
        cadence_interval = self.get_actor_action_cadence_interval(actor_id)
        if cadence_interval is None:
            return self
        next_eligible_map = dict(self.metadata.actor_next_action_eligible_at)
        next_eligible_map[actor_id] = self.metadata.world_tick_count + cadence_interval
        return replace(
            self,
            metadata=replace(
                self.metadata,
                actor_next_action_eligible_at=_normalize_actor_tick_mapping(
                    next_eligible_map,
                    field_name="actor_next_action_eligible_at",
                    require_positive_values=False,
                ),
            ),
        )

    def get_world_tick_scene_message(self) -> str:
        return _world_tick_scene_message_for_stance_phase(self.metadata.npc_stance_phase)

    def get_world_phase_interaction_hint(self) -> str:
        return _world_phase_interaction_hint_for_stance_phase(self.metadata.npc_stance_phase)

    def get_world_phase_outcome_message(self, *, location: str) -> str | None:
        return _world_phase_outcome_message_for_location_and_stance_phase(
            location=location,
            npc_stance_phase=self.metadata.npc_stance_phase,
        )

    def get_world_phase_filtered_action_space(
        self,
        *,
        location: str,
        action_space: tuple[str, ...],
    ) -> tuple[str, ...]:
        return _world_phase_filtered_action_space_for_location_and_stance_phase(
            location=location,
            npc_stance_phase=self.metadata.npc_stance_phase,
            action_space=action_space,
        )

    def get_world_tick_observation_messages(self, *, location: str | None = None) -> tuple[str, ...]:
        messages = [
            self.get_world_tick_scene_message(),
            self.get_world_phase_interaction_hint(),
        ]
        if location is not None:
            outcome_message = self.get_world_phase_outcome_message(location=location)
            if outcome_message is not None:
                messages.append(outcome_message)
        return tuple(messages)

    def get_account(self, account_id: str) -> Any:
        return self.identity_registry.get_account(account_id)

    def get_agent_profile(self, agent_profile_id: str) -> Any:
        return self.identity_registry.get_agent_profile(agent_profile_id)

    def get_system_identity(self, system_identity_id: str) -> Any:
        return self.identity_registry.get_system_identity(system_identity_id)

    def get_character(self, character_id: str) -> Any:
        return self.identity_registry.get_character(character_id)

    def get_session(self, session_id: str) -> Any:
        return self.identity_registry.get_session(session_id)

    def find_sessions_by_account(self, account_id: str) -> tuple[Any, ...]:
        return self.identity_registry.find_sessions_by_account(account_id)

    def find_sessions_by_profile(self, agent_profile_id: str) -> tuple[Any, ...]:
        return self.identity_registry.find_sessions_by_profile(agent_profile_id)

    def find_characters_by_status(self, status: str) -> tuple[Any, ...]:
        return self.identity_registry.find_characters_by_status(status)

    def find_characters_by_system_identity(self, system_identity_id: str) -> tuple[Any, ...]:
        return self.identity_registry.find_characters_by_system_identity(system_identity_id)

    def classify_session_attachment(
        self,
        *,
        character_id: str,
        account_id: str | None = None,
        agent_profile_id: str | None = None,
    ) -> dict[str, Any]:
        return self.identity_registry.classify_session_attachment(
            character_id=character_id,
            account_id=account_id,
            agent_profile_id=agent_profile_id,
        )

    def classify_session_reconciliation(self, session_id: str) -> dict[str, Any]:
        return self.identity_registry.classify_session_reconciliation(session_id)

    def classify_session_principal_reconciliation(self) -> dict[str, Any]:
        return self.identity_registry.classify_session_principal_reconciliation()

    def build_registry_checkpoint_snapshot_placeholder(self) -> dict[str, Any]:
        return self.identity_registry.build_registry_checkpoint_snapshot_placeholder()

    def build_mutation_journal_placeholder(self) -> dict[str, Any]:
        return self.identity_registry.build_mutation_journal_placeholder()

    def build_shard_checkpoint_snapshot_placeholder(self) -> dict[str, Any]:
        checkpoint_generation = self.journal.last_committed_mutation_generation
        checkpoint_id = (
            None
            if checkpoint_generation == 0
            else f"shard_checkpoint_placeholder_{checkpoint_generation:08d}"
        )
        return {
            "payload_kind": "shard_checkpoint_snapshot_placeholder",
            "payload_version": 1,
            "shard_id": self.shard_id,
            "mode_marker": self.metadata.mode_marker,
            "checkpoint_generation": checkpoint_generation,
            "checkpoint_id": checkpoint_id,
            "checkpoint_commit_marker": "in_process_placeholder",
            "metadata": self.metadata.to_dict(),
            "checkpoint": self.checkpoint.to_dict(),
            "journal": self.journal.to_dict(),
            "mutation_policy": self.mutation_policy.to_dict(),
            "identity_registry_snapshot": self.build_registry_checkpoint_snapshot_placeholder(),
        }

    def build_bundle_verification_summary(self, directory_path: Any) -> dict[str, Any]:
        verification_result = self.verify_shard_artifact_bundle(directory_path)
        return _summarize_bundle_verification_result(verification_result)

    def classify_shard_bundle_recovery_state(self, directory_path: Any) -> dict[str, Any]:
        verification_result = self.verify_shard_artifact_bundle(directory_path)
        return _classify_bundle_recovery_result(verification_result)

    def build_shard_export_payload(
        self,
        *,
        bundle_verification_summary: dict[str, Any] | None = None,
        bundle_recovery_summary: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        active_characters = len(self.find_characters_by_status("active"))
        inactive_characters = len(self.find_characters_by_status("inactive"))
        active_sessions = sum(
            1 for session in self.identity_registry.sessions if session.status == "active"
        )
        inactive_sessions = sum(
            1 for session in self.identity_registry.sessions if session.status == "inactive"
        )
        payload = {
            "payload_kind": "shard_export_placeholder",
            "payload_version": 1,
            "shard_id": self.shard_id,
            "metadata": self.metadata.to_dict(),
            "record_family_counts": self.identity_registry.record_family_counts,
            "lifecycle_counts": {
                "characters": {
                    "active": active_characters,
                    "inactive": inactive_characters,
                    "total": len(self.identity_registry.characters),
                },
                "sessions": {
                    "active": active_sessions,
                    "inactive": inactive_sessions,
                    "total": len(self.identity_registry.sessions),
                },
            },
            "identity_category_counts": {
                "accounts_by_type": _count_records_by_attr(
                    self.identity_registry.accounts,
                    field_name="account_type",
                ),
                "agent_profiles_by_controller_type": _count_records_by_attr(
                    self.identity_registry.agent_profiles,
                    field_name="controller_type",
                ),
                "characters_by_identity_class": _count_records_by_attr(
                    self.identity_registry.characters,
                    field_name="identity_class",
                ),
                "system_identities_by_class": _count_records_by_attr(
                    self.identity_registry.system_identities,
                    field_name="identity_class",
                ),
            },
            "checkpoint": self.checkpoint.to_dict(),
            "journal": self.journal.to_dict(),
            "mutation_policy": self.mutation_policy.to_dict(),
            "supports_checkpoint_persistence": self.identity_registry.supports_checkpoint_persistence,
            "supports_mutation_journal_persistence": (
                self.identity_registry.supports_mutation_journal_persistence
            ),
            "registry_checkpoint_snapshot_placeholder": (
                self.build_registry_checkpoint_snapshot_placeholder()
            ),
            "mutation_journal_placeholder": self.build_mutation_journal_placeholder(),
            "shard_checkpoint_snapshot_placeholder": (
                self.build_shard_checkpoint_snapshot_placeholder()
            ),
        }
        if bundle_verification_summary is not None:
            payload["bundle_verification_summary"] = {
                "is_valid": bool(bundle_verification_summary.get("is_valid", False)),
                "issue_codes": sorted(
                    {str(issue_code) for issue_code in bundle_verification_summary.get("issue_codes", ())}
                ),
                "artifact_count": int(bundle_verification_summary.get("artifact_count", 0)),
                "verified_artifact_count": int(
                    bundle_verification_summary.get("verified_artifact_count", 0)
                ),
                "hash_issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_verification_summary.get("hash_issue_codes", ())
                    }
                ),
                "linkage_issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_verification_summary.get("linkage_issue_codes", ())
                    }
                ),
                "manifest_issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_verification_summary.get("manifest_issue_codes", ())
                    }
                ),
                "issue_category_counts": {
                    "hash": int(
                        bundle_verification_summary.get("issue_category_counts", {}).get("hash", 0)
                    ),
                    "linkage": int(
                        bundle_verification_summary.get("issue_category_counts", {}).get(
                            "linkage",
                            0,
                        )
                    ),
                    "manifest": int(
                        bundle_verification_summary.get("issue_category_counts", {}).get(
                            "manifest",
                            0,
                        )
                    ),
                    "other": int(
                        bundle_verification_summary.get("issue_category_counts", {}).get("other", 0)
                    ),
                },
            }
        if bundle_recovery_summary is not None:
            payload["bundle_recovery_summary"] = {
                "recovery_state": str(
                    bundle_recovery_summary.get("recovery_state", "non_resumable")
                ),
                "issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_recovery_summary.get("issue_codes", ())
                    }
                ),
                "fatal_issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_recovery_summary.get("fatal_issue_codes", ())
                    }
                ),
                "degradable_issue_codes": sorted(
                    {
                        str(issue_code)
                        for issue_code in bundle_recovery_summary.get("degradable_issue_codes", ())
                    }
                ),
                "verification_is_valid": bool(
                    bundle_recovery_summary.get("verification_is_valid", False)
                ),
            }
        return payload

    def write_shard_checkpoint_placeholder(self, path: Any) -> str:
        return _write_json_payload(
            path,
            self.build_shard_checkpoint_snapshot_placeholder(),
            artifact_label="shard checkpoint placeholder",
        )

    def write_shard_export_payload(self, path: Any) -> str:
        return _write_json_payload(
            path,
            self.build_shard_export_payload(),
            artifact_label="shard export payload",
        )

    def write_mutation_journal_placeholder(self, path: Any) -> str:
        return self.identity_registry.write_mutation_journal_placeholder(path)

    def write_shard_artifact_bundle(self, directory_path: Any) -> dict[str, str]:
        output_directory = _coerce_output_directory(directory_path)
        expected_filenames = _expected_bundle_filenames(self.shard_id)
        shard_checkpoint_path = output_directory / expected_filenames["shard_checkpoint_placeholder"]
        registry_checkpoint_path = output_directory / (
            expected_filenames["registry_checkpoint_placeholder"]
        )
        mutation_journal_path = output_directory / (
            expected_filenames["mutation_journal_placeholder"]
        )
        shard_export_path = output_directory / expected_filenames["shard_export_payload"]
        manifest_path = output_directory / expected_filenames["artifact_manifest"]
        artifact_paths = {
            "bundle_directory": str(output_directory),
            "shard_checkpoint_placeholder": self.write_shard_checkpoint_placeholder(
                shard_checkpoint_path
            ),
            "registry_checkpoint_placeholder": self.identity_registry.write_registry_checkpoint_placeholder(
                registry_checkpoint_path
            ),
            "mutation_journal_placeholder": self.write_mutation_journal_placeholder(
                mutation_journal_path
            ),
            "shard_export_payload": self.write_shard_export_payload(shard_export_path),
        }
        shard_checkpoint_hash = _sha256_file(Path(artifact_paths["shard_checkpoint_placeholder"]))
        registry_checkpoint_hash = _sha256_file(
            Path(artifact_paths["registry_checkpoint_placeholder"])
        )
        mutation_journal_hash = _sha256_file(Path(artifact_paths["mutation_journal_placeholder"]))
        shard_export_hash = _sha256_file(Path(artifact_paths["shard_export_payload"]))
        index_path = output_directory / expected_filenames["artifact_index"]
        shard_checkpoint_payload = self.build_shard_checkpoint_snapshot_placeholder()
        registry_checkpoint_payload = self.build_registry_checkpoint_snapshot_placeholder()
        mutation_journal_payload = self.build_mutation_journal_placeholder()
        shard_export_payload = self.build_shard_export_payload()
        index_payload = {
            "payload_kind": "shard_artifact_bundle_index_placeholder",
            "payload_version": 1,
            "shard_id": self.shard_id,
            "mode_marker": self.metadata.mode_marker,
            "bundle_generation": self.journal.last_committed_mutation_generation,
            "artifacts": [
                {
                    "artifact_type": "shard_checkpoint_placeholder",
                    "filename": shard_checkpoint_path.name,
                    "payload_kind": shard_checkpoint_payload["payload_kind"],
                    "payload_version": shard_checkpoint_payload["payload_version"],
                    "generation": shard_checkpoint_payload["checkpoint_generation"],
                    "content_hash_sha256": shard_checkpoint_hash,
                },
                {
                    "artifact_type": "registry_checkpoint_placeholder",
                    "filename": registry_checkpoint_path.name,
                    "payload_kind": registry_checkpoint_payload["payload_kind"],
                    "payload_version": registry_checkpoint_payload["payload_version"],
                    "generation": registry_checkpoint_payload["checkpoint_generation"],
                    "content_hash_sha256": registry_checkpoint_hash,
                },
                {
                    "artifact_type": "mutation_journal_placeholder",
                    "filename": mutation_journal_path.name,
                    "payload_kind": mutation_journal_payload["payload_kind"],
                    "payload_version": mutation_journal_payload["payload_version"],
                    "generation": mutation_journal_payload["last_committed_mutation_generation"],
                    "expected_next_generation": mutation_journal_payload[
                        "expected_next_mutation_generation"
                    ],
                    "content_hash_sha256": mutation_journal_hash,
                },
                {
                    "artifact_type": "shard_export_payload",
                    "filename": shard_export_path.name,
                    "payload_kind": shard_export_payload["payload_kind"],
                    "payload_version": shard_export_payload["payload_version"],
                    "generation": shard_export_payload["journal"][
                        "last_committed_mutation_generation"
                    ],
                    "content_hash_sha256": shard_export_hash,
                },
            ],
        }
        artifact_paths["artifact_index"] = _write_json_payload(
            index_path,
            index_payload,
            artifact_label="artifact bundle index",
        )
        manifest_payload = {
            "payload_kind": "shard_artifact_bundle_manifest_placeholder",
            "payload_version": 1,
            "manifest_family": "shard_artifact_bundle_manifest_v1",
            "shard_id": self.shard_id,
            "mode_marker": self.metadata.mode_marker,
            "bundle_generation": self.journal.last_committed_mutation_generation,
            "expected_next_generation": self.journal.expected_next_mutation_generation,
            "index_filename": index_path.name,
            "artifact_filenames": {
                "shard_checkpoint_placeholder": shard_checkpoint_path.name,
                "registry_checkpoint_placeholder": registry_checkpoint_path.name,
                "mutation_journal_placeholder": mutation_journal_path.name,
                "shard_export_payload": shard_export_path.name,
            },
            "linkage": {
                "shard_checkpoint_generation": shard_checkpoint_payload["checkpoint_generation"],
                "registry_checkpoint_generation": registry_checkpoint_payload["checkpoint_generation"],
                "mutation_journal_generation": mutation_journal_payload[
                    "last_committed_mutation_generation"
                ],
                "mutation_journal_expected_next_generation": mutation_journal_payload[
                    "expected_next_mutation_generation"
                ],
                "shard_export_generation": shard_export_payload["journal"][
                    "last_committed_mutation_generation"
                ],
            },
        }
        artifact_paths["artifact_manifest"] = _write_json_payload(
            manifest_path,
            manifest_payload,
            artifact_label="artifact bundle manifest",
        )
        return artifact_paths

    def verify_shard_artifact_bundle(self, directory_path: Any) -> dict[str, Any]:
        output_directory = _coerce_output_directory(directory_path)
        index_candidates = sorted(output_directory.glob("*.artifact_index.json"))
        result = {
            "payload_kind": "shard_artifact_bundle_verification_placeholder",
            "payload_version": 1,
            "bundle_directory": str(output_directory),
            "index_filename": None,
            "manifest_filename": None,
            "shard_id": None,
            "is_valid": False,
            "issue_codes": [],
            "artifacts": [],
        }

        if len(index_candidates) == 0:
            result["issue_codes"].append("missing_artifact_index")
            return result
        if len(index_candidates) > 1:
            result["issue_codes"].append("multiple_artifact_indexes")
            return result

        index_path = index_candidates[0]
        result["index_filename"] = index_path.name
        try:
            index_payload = json.loads(index_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            result["issue_codes"].append("invalid_artifact_index")
            return result

        shard_id = index_payload.get("shard_id")
        if not isinstance(shard_id, str) or not shard_id:
            result["issue_codes"].append("invalid_shard_id")
            return result
        result["shard_id"] = shard_id

        expected_filenames = _expected_bundle_filenames(shard_id)
        if index_path.name != expected_filenames["artifact_index"]:
            result["issue_codes"].append("artifact_index_filename_mismatch")

        manifest_path = output_directory / expected_filenames["artifact_manifest"]
        result["manifest_filename"] = expected_filenames["artifact_manifest"]
        manifest_payload = _read_json_file(manifest_path)
        if not manifest_path.exists() or not manifest_path.is_file():
            result["issue_codes"].append("missing_artifact_manifest")
            manifest_payload = None
        elif manifest_payload is None:
            result["issue_codes"].append("invalid_artifact_manifest")

        if manifest_payload is not None:
            if manifest_payload.get("shard_id") != shard_id:
                result["issue_codes"].append("manifest_shard_id_mismatch")
            if manifest_payload.get("index_filename") != index_path.name:
                result["issue_codes"].append("manifest_index_filename_mismatch")

        artifact_entries = index_payload.get("artifacts")
        if not isinstance(artifact_entries, list):
            result["issue_codes"].append("invalid_artifact_entries")
            return result

        artifact_entry_by_type = {}
        for entry in artifact_entries:
            artifact_type = entry.get("artifact_type")
            if isinstance(artifact_type, str) and artifact_type not in artifact_entry_by_type:
                artifact_entry_by_type[artifact_type] = entry

        artifact_result_by_type: dict[str, dict[str, Any]] = {}
        artifact_payload_by_type: dict[str, dict[str, Any]] = {}

        ordered_artifact_types = [
            "shard_checkpoint_placeholder",
            "registry_checkpoint_placeholder",
            "mutation_journal_placeholder",
            "shard_export_payload",
        ]
        for artifact_type in ordered_artifact_types:
            expected_filename = expected_filenames[artifact_type]
            entry = artifact_entry_by_type.get(artifact_type)
            artifact_result = {
                "artifact_type": artifact_type,
                "expected_filename": expected_filename,
                "recorded_filename": None,
                "exists": False,
                "hash_matches": False,
                "recorded_hash_sha256": None,
                "actual_hash_sha256": None,
                "issue_codes": [],
            }
            artifact_result_by_type[artifact_type] = artifact_result
            if entry is None:
                artifact_result["issue_codes"].append("missing_artifact_entry")
                result["artifacts"].append(artifact_result)
                result["issue_codes"].append("missing_artifact_entry")
                continue

            recorded_filename = entry.get("filename")
            recorded_hash = entry.get("content_hash_sha256")
            artifact_result["recorded_filename"] = recorded_filename
            artifact_result["recorded_hash_sha256"] = recorded_hash
            if recorded_filename != expected_filename:
                artifact_result["issue_codes"].append("filename_mismatch")
                result["issue_codes"].append("filename_mismatch")

            artifact_path = output_directory / expected_filename
            if not artifact_path.exists() or not artifact_path.is_file():
                artifact_result["issue_codes"].append("missing_artifact_file")
                result["artifacts"].append(artifact_result)
                result["issue_codes"].append("missing_artifact_file")
                continue

            artifact_result["exists"] = True
            actual_hash = _sha256_file(artifact_path)
            artifact_result["actual_hash_sha256"] = actual_hash
            if recorded_hash == actual_hash:
                artifact_result["hash_matches"] = True
                payload = _read_json_file(artifact_path)
                if payload is None:
                    artifact_result["issue_codes"].append("invalid_artifact_payload")
                    result["issue_codes"].append("invalid_artifact_payload")
                else:
                    artifact_payload_by_type[artifact_type] = payload
            else:
                artifact_result["issue_codes"].append("content_hash_mismatch")
                result["issue_codes"].append("content_hash_mismatch")
            result["artifacts"].append(artifact_result)

        bundle_generation = index_payload.get("bundle_generation")
        if not isinstance(bundle_generation, int) or bundle_generation < 0:
            result["issue_codes"].append("invalid_bundle_generation")
        else:
            if manifest_payload is not None:
                manifest_artifact_filenames = manifest_payload.get("artifact_filenames", {})
                manifest_linkage = manifest_payload.get("linkage", {})
                if manifest_payload.get("bundle_generation") != bundle_generation:
                    result["issue_codes"].append("manifest_bundle_generation_mismatch")
                if manifest_payload.get("expected_next_generation") != bundle_generation + 1:
                    result["issue_codes"].append("manifest_expected_next_generation_mismatch")
                if not isinstance(manifest_artifact_filenames, dict):
                    result["issue_codes"].append("manifest_artifact_filename_mismatch")
                else:
                    ordered_artifact_types = [
                        "shard_checkpoint_placeholder",
                        "registry_checkpoint_placeholder",
                        "mutation_journal_placeholder",
                        "shard_export_payload",
                    ]
                    for artifact_type in ordered_artifact_types:
                        if manifest_artifact_filenames.get(artifact_type) != expected_filenames[artifact_type]:
                            result["issue_codes"].append("manifest_artifact_filename_mismatch")
                            break
                if (
                    not isinstance(manifest_linkage, dict)
                    or manifest_linkage.get("shard_checkpoint_generation") != bundle_generation
                    or manifest_linkage.get("registry_checkpoint_generation") != bundle_generation
                    or manifest_linkage.get("mutation_journal_generation") != bundle_generation
                    or manifest_linkage.get("mutation_journal_expected_next_generation")
                    != bundle_generation + 1
                    or manifest_linkage.get("shard_export_generation") != bundle_generation
                ):
                    result["issue_codes"].append("manifest_linkage_mismatch")

            shard_checkpoint_payload = artifact_payload_by_type.get("shard_checkpoint_placeholder")
            if shard_checkpoint_payload is not None:
                shard_checkpoint_generation = shard_checkpoint_payload.get("checkpoint_generation")
                shard_checkpoint_journal = shard_checkpoint_payload.get("journal", {})
                shard_checkpoint_registry = shard_checkpoint_payload.get(
                    "identity_registry_snapshot", {}
                )
                shard_checkpoint_entry = artifact_entry_by_type.get("shard_checkpoint_placeholder", {})
                shard_checkpoint_entry_generation = shard_checkpoint_entry.get("generation")
                if (
                    not isinstance(shard_checkpoint_generation, int)
                    or shard_checkpoint_generation != bundle_generation
                    or not isinstance(shard_checkpoint_journal, dict)
                    or shard_checkpoint_journal.get("last_committed_mutation_generation")
                    != shard_checkpoint_generation
                    or not isinstance(shard_checkpoint_registry, dict)
                    or shard_checkpoint_registry.get("checkpoint_generation")
                    != shard_checkpoint_generation
                    or shard_checkpoint_entry_generation != shard_checkpoint_generation
                ):
                    artifact_result_by_type["shard_checkpoint_placeholder"]["issue_codes"].append(
                        "shard_checkpoint_generation_mismatch"
                    )
                    result["issue_codes"].append("shard_checkpoint_generation_mismatch")

            registry_checkpoint_payload = artifact_payload_by_type.get(
                "registry_checkpoint_placeholder"
            )
            if registry_checkpoint_payload is not None:
                registry_checkpoint_generation = registry_checkpoint_payload.get("checkpoint_generation")
                registry_entry = artifact_entry_by_type.get("registry_checkpoint_placeholder", {})
                registry_entry_generation = registry_entry.get("generation")
                if (
                    not isinstance(registry_checkpoint_generation, int)
                    or registry_checkpoint_generation != bundle_generation
                    or registry_checkpoint_payload.get("last_committed_mutation_generation")
                    != registry_checkpoint_generation
                    or registry_checkpoint_payload.get("expected_next_mutation_generation")
                    != registry_checkpoint_generation + 1
                    or registry_entry_generation != registry_checkpoint_generation
                ):
                    artifact_result_by_type["registry_checkpoint_placeholder"]["issue_codes"].append(
                        "registry_checkpoint_generation_mismatch"
                    )
                    result["issue_codes"].append("registry_checkpoint_generation_mismatch")

            mutation_journal_payload = artifact_payload_by_type.get("mutation_journal_placeholder")
            if mutation_journal_payload is not None:
                mutation_journal_generation = mutation_journal_payload.get(
                    "last_committed_mutation_generation"
                )
                mutation_journal_expected_next = mutation_journal_payload.get(
                    "expected_next_mutation_generation"
                )
                mutation_journal_entry = artifact_entry_by_type.get(
                    "mutation_journal_placeholder",
                    {},
                )
                if (
                    not isinstance(mutation_journal_generation, int)
                    or mutation_journal_generation != bundle_generation
                    or mutation_journal_expected_next != bundle_generation + 1
                    or mutation_journal_entry.get("generation") != mutation_journal_generation
                    or mutation_journal_entry.get("expected_next_generation")
                    != mutation_journal_expected_next
                ):
                    artifact_result_by_type["mutation_journal_placeholder"]["issue_codes"].append(
                        "mutation_journal_generation_mismatch"
                    )
                    result["issue_codes"].append("mutation_journal_generation_mismatch")

            shard_export_payload = artifact_payload_by_type.get("shard_export_payload")
            if shard_export_payload is not None:
                export_journal = shard_export_payload.get("journal", {})
                export_registry_checkpoint = shard_export_payload.get(
                    "registry_checkpoint_snapshot_placeholder",
                    {},
                )
                export_mutation_journal = shard_export_payload.get(
                    "mutation_journal_placeholder",
                    {},
                )
                export_shard_checkpoint = shard_export_payload.get(
                    "shard_checkpoint_snapshot_placeholder",
                    {},
                )
                shard_export_entry = artifact_entry_by_type.get("shard_export_payload", {})
                export_generation = (
                    export_journal.get("last_committed_mutation_generation")
                    if isinstance(export_journal, dict)
                    else None
                )
                if (
                    not isinstance(export_generation, int)
                    or export_generation != bundle_generation
                    or export_journal.get("expected_next_mutation_generation")
                    != bundle_generation + 1
                    or not isinstance(export_registry_checkpoint, dict)
                    or export_registry_checkpoint.get("checkpoint_generation") != bundle_generation
                    or not isinstance(export_mutation_journal, dict)
                    or export_mutation_journal.get("last_committed_mutation_generation")
                    != bundle_generation
                    or export_mutation_journal.get("expected_next_mutation_generation")
                    != bundle_generation + 1
                    or not isinstance(export_shard_checkpoint, dict)
                    or export_shard_checkpoint.get("checkpoint_generation") != bundle_generation
                    or shard_export_entry.get("generation") != export_generation
                ):
                    artifact_result_by_type["shard_export_payload"]["issue_codes"].append(
                        "shard_export_generation_mismatch"
                    )
                    result["issue_codes"].append("shard_export_generation_mismatch")

        for artifact_result in result["artifacts"]:
            artifact_result["issue_codes"] = sorted(set(artifact_result["issue_codes"]))
        result["issue_codes"] = sorted(set(result["issue_codes"]))
        result["is_valid"] = len(result["issue_codes"]) == 0
        return result

    def register_account(
        self,
        *,
        account_id: str,
        account_type: str = "human_player",
        display_name: str | None = None,
        status: str = "active",
    ) -> "ShardState":
        updated_registry = self.identity_registry.register_account(
            account_id=account_id,
            account_type=account_type,
            display_name=display_name,
            status=status,
        )
        return self.with_identity_registry(updated_registry)

    def register_agent_profile(
        self,
        *,
        agent_profile_id: str,
        controller_type: str = "external_agent",
        display_name: str | None = None,
        benchmark_bridge_actor_id: str | None = None,
        status: str = "active",
    ) -> "ShardState":
        updated_registry = self.identity_registry.register_agent_profile(
            agent_profile_id=agent_profile_id,
            controller_type=controller_type,
            display_name=display_name,
            benchmark_bridge_actor_id=benchmark_bridge_actor_id,
            status=status,
        )
        return self.with_identity_registry(updated_registry)

    def register_system_identity(
        self,
        *,
        system_identity_id: str,
        identity_class: str,
        display_name: str | None = None,
        status: str = "active",
    ) -> "ShardState":
        updated_registry = self.identity_registry.register_system_identity(
            system_identity_id=system_identity_id,
            identity_class=identity_class,
            display_name=display_name,
            status=status,
        )
        return self.with_identity_registry(updated_registry)

    def register_character(
        self,
        *,
        character_id: str,
        identity_class: str,
        owner_account_id: str | None = None,
        controller_agent_profile_id: str | None = None,
        system_identity_id: str | None = None,
        status: str = "active",
    ) -> "ShardState":
        updated_registry = self.identity_registry.register_character(
            character_id=character_id,
            identity_class=identity_class,
            owner_account_id=owner_account_id,
            controller_agent_profile_id=controller_agent_profile_id,
            system_identity_id=system_identity_id,
            status=status,
        )
        return self.with_identity_registry(updated_registry)

    def attach_session(
        self,
        *,
        session_id: str,
        character_id: str,
        account_id: str | None = None,
        agent_profile_id: str | None = None,
        presence_id: str | None = None,
    ) -> "ShardState":
        updated_registry = self.identity_registry.attach_session(
            session_id=session_id,
            character_id=character_id,
            account_id=account_id,
            agent_profile_id=agent_profile_id,
            presence_id=presence_id,
            enforce_reconciliation=self.mutation_policy.enforce_session_principal_reconciliation,
        )
        return self.with_identity_registry(updated_registry)

    def activate_session(self, session_id: str) -> "ShardState":
        updated_registry = self.identity_registry.activate_session(
            session_id,
            enforce_reconciliation=self.mutation_policy.enforce_session_principal_reconciliation,
        )
        return self.with_identity_registry(updated_registry)

    def detach_session(self, session_id: str) -> "ShardState":
        updated_registry = self.identity_registry.detach_session(session_id)
        return self.with_identity_registry(updated_registry)

    def deactivate_session(self, session_id: str) -> "ShardState":
        updated_registry = self.identity_registry.deactivate_session(session_id)
        return self.with_identity_registry(updated_registry)

    def open_character_session(
        self,
        *,
        session_id: str,
        character_id: str,
        account_id: str | None = None,
        agent_profile_id: str | None = None,
        presence_id: str | None = None,
    ) -> "ShardState":
        state = self
        if state.get_character(character_id).status != "active":
            state = state.activate_character(character_id)
        return state.attach_session(
            session_id=session_id,
            character_id=character_id,
            account_id=account_id,
            agent_profile_id=agent_profile_id,
            presence_id=presence_id,
        )

    def close_character_session(self, session_id: str) -> "ShardState":
        detached_state = self.detach_session(session_id)
        character_id = detached_state.get_session(session_id).character_id
        if detached_state.get_character(character_id).status != "inactive":
            return detached_state.deactivate_character(character_id)
        return detached_state

    def activate_account(self, account_id: str) -> "ShardState":
        updated_registry = self.identity_registry.activate_account(account_id)
        return self.with_identity_registry(updated_registry)

    def deactivate_account(self, account_id: str) -> "ShardState":
        updated_registry = self.identity_registry.deactivate_account(account_id)
        return self.with_identity_registry(updated_registry)

    def activate_agent_profile(self, agent_profile_id: str) -> "ShardState":
        updated_registry = self.identity_registry.activate_agent_profile(agent_profile_id)
        return self.with_identity_registry(updated_registry)

    def deactivate_agent_profile(self, agent_profile_id: str) -> "ShardState":
        updated_registry = self.identity_registry.deactivate_agent_profile(agent_profile_id)
        return self.with_identity_registry(updated_registry)

    def activate_system_identity(self, system_identity_id: str) -> "ShardState":
        updated_registry = self.identity_registry.activate_system_identity(system_identity_id)
        return self.with_identity_registry(updated_registry)

    def deactivate_system_identity(self, system_identity_id: str) -> "ShardState":
        updated_registry = self.identity_registry.deactivate_system_identity(system_identity_id)
        return self.with_identity_registry(updated_registry)

    def activate_character(self, character_id: str) -> "ShardState":
        updated_registry = self.identity_registry.activate_character(character_id)
        return self.with_identity_registry(updated_registry)

    def deactivate_character(self, character_id: str) -> "ShardState":
        updated_registry = self.identity_registry.deactivate_character(character_id)
        return self.with_identity_registry(updated_registry)

    def bind_character_owner(self, *, character_id: str, account_id: str) -> "ShardState":
        updated_registry = self.identity_registry.bind_character_owner(
            character_id=character_id,
            account_id=account_id,
        )
        return self.with_identity_registry(updated_registry)

    def unbind_character_owner(self, character_id: str) -> "ShardState":
        updated_registry = self.identity_registry.unbind_character_owner(character_id)
        return self.with_identity_registry(updated_registry)

    def bind_character_profile(
        self,
        *,
        character_id: str,
        agent_profile_id: str,
    ) -> "ShardState":
        updated_registry = self.identity_registry.bind_character_profile(
            character_id=character_id,
            agent_profile_id=agent_profile_id,
        )
        return self.with_identity_registry(updated_registry)

    def unbind_character_profile(self, character_id: str) -> "ShardState":
        updated_registry = self.identity_registry.unbind_character_profile(character_id)
        return self.with_identity_registry(updated_registry)

    def bind_character_system_identity(
        self,
        *,
        character_id: str,
        system_identity_id: str,
    ) -> "ShardState":
        updated_registry = self.identity_registry.bind_character_system_identity(
            character_id=character_id,
            system_identity_id=system_identity_id,
        )
        return self.with_identity_registry(updated_registry)

    def unbind_character_system_identity(self, character_id: str) -> "ShardState":
        updated_registry = self.identity_registry.unbind_character_system_identity(character_id)
        return self.with_identity_registry(updated_registry)

    @classmethod
    def create_empty(
        cls,
        shard_id: str,
        *,
        world_ruleset_version: str = "unconfigured",
        benchmark_engine_version: str = "unconfigured",
        scheduler_policy_version: str = "unconfigured",
    ) -> "ShardState":
        metadata = ShardMetadata(
            shard_id=shard_id,
            world_ruleset_version=world_ruleset_version,
            benchmark_engine_version=benchmark_engine_version,
            scheduler_policy_version=scheduler_policy_version,
        )
        registry = InProcessShardIdentityRegistry.create_empty(shard_id)
        checkpoint = ShardCheckpointMetadata()
        journal = ShardJournalMetadata(
            last_committed_mutation_generation=registry.last_committed_mutation_generation,
            last_committed_mutation_id=registry.last_committed_mutation_id,
            expected_next_mutation_generation=registry.expected_next_mutation_generation,
        )
        return cls(
            metadata=metadata,
            checkpoint=checkpoint,
            journal=journal,
            identity_registry=registry,
            mutation_policy=ShardMutationPolicy(),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "shard_id": self.shard_id,
            "metadata": self.metadata.to_dict(),
            "checkpoint": self.checkpoint.to_dict(),
            "journal": self.journal.to_dict(),
            "identity_registry": self.identity_registry.to_dict(),
            "mutation_policy": self.mutation_policy.to_dict(),
        }
