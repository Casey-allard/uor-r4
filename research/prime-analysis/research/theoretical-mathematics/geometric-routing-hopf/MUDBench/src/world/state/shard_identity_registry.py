"""In-process shard identity registry skeleton for persistent-world mode."""

from __future__ import annotations

import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any


_UNSET = object()


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _require_non_negative_int(value: Any, *, field_name: str) -> int:
    if not isinstance(value, int) or value < 0:
        raise ValueError(f"{field_name} must be a non-negative integer")
    return value


def _sort_records_by_id(records: tuple[Any, ...], *, field_name: str) -> tuple[Any, ...]:
    return tuple(sorted(records, key=lambda item: str(getattr(item, field_name))))


def _replace_record_by_id(
    records: tuple[Any, ...],
    updated_record: Any,
    *,
    field_name: str,
) -> tuple[Any, ...]:
    updated_id = str(getattr(updated_record, field_name))
    replaced = []
    found = False
    for record in records:
        if str(getattr(record, field_name)) == updated_id:
            replaced.append(updated_record)
            found = True
        else:
            replaced.append(record)
    if not found:
        raise ValueError(f"{field_name} '{updated_id}' was not found")
    return _sort_records_by_id(tuple(replaced), field_name=field_name)


def _mutation_generation_from_id(mutation_id: str | None) -> int | None:
    if mutation_id is None:
        return None
    prefix = "mut_"
    if not mutation_id.startswith(prefix):
        return None
    suffix = mutation_id[len(prefix) :]
    if not suffix.isdigit():
        return None
    return int(suffix)


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


@dataclass(frozen=True, slots=True)
class ShardAccountRecord:
    """Minimal durable account identity record."""

    account_id: str
    account_type: str = "human_player"
    display_name: str | None = None
    status: str = "active"
    record_version: int = 0
    created_generation: int = 0
    updated_generation: int = 0
    last_mutation_id: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.account_id, field_name="account_id")
        _require_non_empty_string(self.account_type, field_name="account_type")
        _require_non_empty_string(self.status, field_name="status")
        _require_non_negative_int(self.record_version, field_name="record_version")
        _require_non_negative_int(self.created_generation, field_name="created_generation")
        _require_non_negative_int(self.updated_generation, field_name="updated_generation")

    def to_dict(self) -> dict[str, Any]:
        return {
            "account_id": self.account_id,
            "account_type": self.account_type,
            "display_name": self.display_name,
            "status": self.status,
            "record_version": self.record_version,
            "created_generation": self.created_generation,
            "updated_generation": self.updated_generation,
            "last_mutation_id": self.last_mutation_id,
        }


@dataclass(frozen=True, slots=True)
class ShardAgentProfileRecord:
    """Minimal durable agent profile identity record."""

    agent_profile_id: str
    controller_type: str = "external_agent"
    display_name: str | None = None
    benchmark_bridge_actor_id: str | None = None
    status: str = "active"
    record_version: int = 0
    created_generation: int = 0
    updated_generation: int = 0
    last_mutation_id: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.agent_profile_id, field_name="agent_profile_id")
        _require_non_empty_string(self.controller_type, field_name="controller_type")
        _require_non_empty_string(self.status, field_name="status")
        _require_non_negative_int(self.record_version, field_name="record_version")
        _require_non_negative_int(self.created_generation, field_name="created_generation")
        _require_non_negative_int(self.updated_generation, field_name="updated_generation")

    def to_dict(self) -> dict[str, Any]:
        return {
            "agent_profile_id": self.agent_profile_id,
            "controller_type": self.controller_type,
            "display_name": self.display_name,
            "benchmark_bridge_actor_id": self.benchmark_bridge_actor_id,
            "status": self.status,
            "record_version": self.record_version,
            "created_generation": self.created_generation,
            "updated_generation": self.updated_generation,
            "last_mutation_id": self.last_mutation_id,
        }


@dataclass(frozen=True, slots=True)
class ShardCharacterRecord:
    """Minimal durable shard character record."""

    character_id: str
    identity_class: str
    owner_account_id: str | None = None
    controller_agent_profile_id: str | None = None
    system_identity_id: str | None = None
    status: str = "active"
    record_version: int = 0
    created_generation: int = 0
    updated_generation: int = 0
    last_mutation_id: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.character_id, field_name="character_id")
        _require_non_empty_string(self.identity_class, field_name="identity_class")
        _require_non_empty_string(self.status, field_name="status")
        _require_non_negative_int(self.record_version, field_name="record_version")
        _require_non_negative_int(self.created_generation, field_name="created_generation")
        _require_non_negative_int(self.updated_generation, field_name="updated_generation")

    def to_dict(self) -> dict[str, Any]:
        return {
            "character_id": self.character_id,
            "identity_class": self.identity_class,
            "owner_account_id": self.owner_account_id,
            "controller_agent_profile_id": self.controller_agent_profile_id,
            "system_identity_id": self.system_identity_id,
            "status": self.status,
            "record_version": self.record_version,
            "created_generation": self.created_generation,
            "updated_generation": self.updated_generation,
            "last_mutation_id": self.last_mutation_id,
        }


@dataclass(frozen=True, slots=True)
class ShardSessionRecord:
    """Minimal shard-local session or presence record."""

    session_id: str
    character_id: str
    status: str = "inactive"
    account_id: str | None = None
    agent_profile_id: str | None = None
    presence_id: str | None = None
    record_version: int = 0
    attached_generation: int = 0
    detached_generation: int | None = None
    last_mutation_id: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.session_id, field_name="session_id")
        _require_non_empty_string(self.character_id, field_name="character_id")
        _require_non_empty_string(self.status, field_name="status")
        _require_non_negative_int(self.record_version, field_name="record_version")
        _require_non_negative_int(self.attached_generation, field_name="attached_generation")
        if self.detached_generation is not None:
            _require_non_negative_int(self.detached_generation, field_name="detached_generation")

    def to_dict(self) -> dict[str, Any]:
        return {
            "session_id": self.session_id,
            "character_id": self.character_id,
            "status": self.status,
            "account_id": self.account_id,
            "agent_profile_id": self.agent_profile_id,
            "presence_id": self.presence_id,
            "record_version": self.record_version,
            "attached_generation": self.attached_generation,
            "detached_generation": self.detached_generation,
            "last_mutation_id": self.last_mutation_id,
        }


@dataclass(frozen=True, slots=True)
class ShardSystemIdentityRecord:
    """Minimal built-in or system-controlled identity record."""

    system_identity_id: str
    identity_class: str
    display_name: str | None = None
    status: str = "active"
    record_version: int = 0
    created_generation: int = 0
    updated_generation: int = 0
    last_mutation_id: str | None = None

    def __post_init__(self) -> None:
        _require_non_empty_string(self.system_identity_id, field_name="system_identity_id")
        _require_non_empty_string(self.identity_class, field_name="identity_class")
        _require_non_empty_string(self.status, field_name="status")
        _require_non_negative_int(self.record_version, field_name="record_version")
        _require_non_negative_int(self.created_generation, field_name="created_generation")
        _require_non_negative_int(self.updated_generation, field_name="updated_generation")

    def to_dict(self) -> dict[str, Any]:
        return {
            "system_identity_id": self.system_identity_id,
            "identity_class": self.identity_class,
            "display_name": self.display_name,
            "status": self.status,
            "record_version": self.record_version,
            "created_generation": self.created_generation,
            "updated_generation": self.updated_generation,
            "last_mutation_id": self.last_mutation_id,
        }


@dataclass(frozen=True, slots=True)
class InProcessShardIdentityRegistry:
    """Deterministic in-process shell for shard identity ownership."""

    shard_id: str
    mode_marker: str = "in_process_identity_registry_shell"
    last_committed_mutation_generation: int = 0
    last_committed_mutation_id: str | None = None
    supports_checkpoint_persistence: bool = False
    supports_mutation_journal_persistence: bool = False
    accounts: tuple[ShardAccountRecord, ...] = ()
    agent_profiles: tuple[ShardAgentProfileRecord, ...] = ()
    characters: tuple[ShardCharacterRecord, ...] = ()
    sessions: tuple[ShardSessionRecord, ...] = ()
    system_identities: tuple[ShardSystemIdentityRecord, ...] = ()

    def __post_init__(self) -> None:
        _require_non_empty_string(self.shard_id, field_name="shard_id")
        _require_non_empty_string(self.mode_marker, field_name="mode_marker")
        _require_non_negative_int(
            self.last_committed_mutation_generation,
            field_name="last_committed_mutation_generation",
        )
        object.__setattr__(self, "accounts", _sort_records_by_id(self.accounts, field_name="account_id"))
        object.__setattr__(
            self,
            "agent_profiles",
            _sort_records_by_id(self.agent_profiles, field_name="agent_profile_id"),
        )
        object.__setattr__(
            self,
            "characters",
            _sort_records_by_id(self.characters, field_name="character_id"),
        )
        object.__setattr__(self, "sessions", _sort_records_by_id(self.sessions, field_name="session_id"))
        object.__setattr__(
            self,
            "system_identities",
            _sort_records_by_id(self.system_identities, field_name="system_identity_id"),
        )

    @property
    def expected_next_mutation_generation(self) -> int:
        return self.last_committed_mutation_generation + 1

    @property
    def record_family_counts(self) -> dict[str, int]:
        return {
            "accounts": len(self.accounts),
            "agent_profiles": len(self.agent_profiles),
            "characters": len(self.characters),
            "sessions": len(self.sessions),
            "system_identities": len(self.system_identities),
        }

    @classmethod
    def create_empty(cls, shard_id: str) -> "InProcessShardIdentityRegistry":
        return cls(shard_id=shard_id)

    def get_account(self, account_id: str) -> ShardAccountRecord:
        normalized_id = _require_non_empty_string(account_id, field_name="account_id")
        for record in self.accounts:
            if record.account_id == normalized_id:
                return record
        raise ValueError(f"account_id '{normalized_id}' was not found")

    def get_agent_profile(self, agent_profile_id: str) -> ShardAgentProfileRecord:
        normalized_id = _require_non_empty_string(agent_profile_id, field_name="agent_profile_id")
        for record in self.agent_profiles:
            if record.agent_profile_id == normalized_id:
                return record
        raise ValueError(f"agent_profile_id '{normalized_id}' was not found")

    def get_character(self, character_id: str) -> ShardCharacterRecord:
        normalized_id = _require_non_empty_string(character_id, field_name="character_id")
        for record in self.characters:
            if record.character_id == normalized_id:
                return record
        raise ValueError(f"character_id '{normalized_id}' was not found")

    def get_session(self, session_id: str) -> ShardSessionRecord:
        normalized_id = _require_non_empty_string(session_id, field_name="session_id")
        for record in self.sessions:
            if record.session_id == normalized_id:
                return record
        raise ValueError(f"session_id '{normalized_id}' was not found")

    def get_system_identity(self, system_identity_id: str) -> ShardSystemIdentityRecord:
        normalized_id = _require_non_empty_string(
            system_identity_id,
            field_name="system_identity_id",
        )
        for record in self.system_identities:
            if record.system_identity_id == normalized_id:
                return record
        raise ValueError(f"system_identity_id '{normalized_id}' was not found")

    def find_sessions_by_account(self, account_id: str) -> tuple[ShardSessionRecord, ...]:
        normalized_id = _require_non_empty_string(account_id, field_name="account_id")
        return tuple(record for record in self.sessions if record.account_id == normalized_id)

    def find_sessions_by_profile(
        self,
        agent_profile_id: str,
    ) -> tuple[ShardSessionRecord, ...]:
        normalized_id = _require_non_empty_string(agent_profile_id, field_name="agent_profile_id")
        return tuple(record for record in self.sessions if record.agent_profile_id == normalized_id)

    def find_accounts_by_status(self, status: str) -> tuple[ShardAccountRecord, ...]:
        normalized_status = _require_non_empty_string(status, field_name="status")
        return tuple(record for record in self.accounts if record.status == normalized_status)

    def find_agent_profiles_by_status(self, status: str) -> tuple[ShardAgentProfileRecord, ...]:
        normalized_status = _require_non_empty_string(status, field_name="status")
        return tuple(record for record in self.agent_profiles if record.status == normalized_status)

    def find_system_identities_by_status(
        self,
        status: str,
    ) -> tuple[ShardSystemIdentityRecord, ...]:
        normalized_status = _require_non_empty_string(status, field_name="status")
        return tuple(record for record in self.system_identities if record.status == normalized_status)

    def find_characters_by_account(self, account_id: str) -> tuple[ShardCharacterRecord, ...]:
        normalized_id = _require_non_empty_string(account_id, field_name="account_id")
        return tuple(
            record
            for record in self.characters
            if record.owner_account_id == normalized_id
        )

    def find_characters_by_profile(
        self,
        agent_profile_id: str,
    ) -> tuple[ShardCharacterRecord, ...]:
        normalized_id = _require_non_empty_string(agent_profile_id, field_name="agent_profile_id")
        return tuple(
            record
            for record in self.characters
            if record.controller_agent_profile_id == normalized_id
        )

    def find_characters_by_system_identity(
        self,
        system_identity_id: str,
    ) -> tuple[ShardCharacterRecord, ...]:
        normalized_id = _require_non_empty_string(
            system_identity_id,
            field_name="system_identity_id",
        )
        return tuple(
            record
            for record in self.characters
            if record.system_identity_id == normalized_id
        )

    def find_characters_by_status(self, status: str) -> tuple[ShardCharacterRecord, ...]:
        normalized_status = _require_non_empty_string(status, field_name="status")
        return tuple(record for record in self.characters if record.status == normalized_status)

    def find_active_session_for_character(self, character_id: str) -> ShardSessionRecord | None:
        normalized_id = _require_non_empty_string(character_id, field_name="character_id")
        for record in self.sessions:
            if record.character_id == normalized_id and record.status == "active":
                return record
        return None

    def classify_session_attachment(
        self,
        *,
        character_id: str,
        account_id: str | None = None,
        agent_profile_id: str | None = None,
    ) -> dict[str, Any]:
        normalized_character_id = _require_non_empty_string(character_id, field_name="character_id")
        normalized_account_id = (
            None
            if account_id is None
            else _require_non_empty_string(account_id, field_name="account_id")
        )
        normalized_agent_profile_id = (
            None
            if agent_profile_id is None
            else _require_non_empty_string(agent_profile_id, field_name="agent_profile_id")
        )
        return self._classify_session_linkage(
            character_id=normalized_character_id,
            account_id=normalized_account_id,
            agent_profile_id=normalized_agent_profile_id,
            require_active_lifecycle=True,
            competing_active_session_id=None,
        )

    def classify_session_reconciliation(self, session_id: str) -> dict[str, Any]:
        session = self.get_session(session_id)
        reconciliation = self._classify_session_linkage(
            character_id=session.character_id,
            account_id=session.account_id,
            agent_profile_id=session.agent_profile_id,
            require_active_lifecycle=session.status == "active",
            competing_active_session_id=session.session_id if session.status == "active" else None,
        )
        issue_codes = list(reconciliation["issue_codes"])
        active_session = self.find_active_session_for_character(session.character_id)
        if (
            session.status == "active"
            and active_session is not None
            and active_session.session_id != session.session_id
        ):
            issue_codes.append("character_has_competing_active_session")
        return {
            "session_id": session.session_id,
            "session_status": session.status,
            "detached_generation": session.detached_generation,
            "record_version": session.record_version,
            "character_id": reconciliation["character_id"],
            "character_status": reconciliation["character_status"],
            "account_id": reconciliation["account_id"],
            "account_status": reconciliation["account_status"],
            "agent_profile_id": reconciliation["agent_profile_id"],
            "agent_profile_status": reconciliation["agent_profile_status"],
            "system_identity_id": reconciliation["system_identity_id"],
            "system_identity_status": reconciliation["system_identity_status"],
            "issue_codes": tuple(issue_codes),
            "is_consistent": len(issue_codes) == 0,
        }

    def classify_session_principal_reconciliation(self) -> dict[str, Any]:
        session_results = tuple(
            self.classify_session_reconciliation(session.session_id)
            for session in self.sessions
        )
        consistent_session_ids = tuple(
            result["session_id"]
            for result in session_results
            if result["is_consistent"]
        )
        inconsistent_session_ids = tuple(
            result["session_id"]
            for result in session_results
            if not result["is_consistent"]
        )
        return {
            "is_consistent": len(inconsistent_session_ids) == 0,
            "session_count": len(session_results),
            "consistent_session_ids": consistent_session_ids,
            "inconsistent_session_ids": inconsistent_session_ids,
            "session_results": session_results,
        }

    def register_account(
        self,
        *,
        account_id: str,
        account_type: str = "human_player",
        display_name: str | None = None,
        status: str = "active",
    ) -> "InProcessShardIdentityRegistry":
        normalized_id = _require_non_empty_string(account_id, field_name="account_id")
        for record in self.accounts:
            if record.account_id == normalized_id:
                raise ValueError(f"account_id already exists: {normalized_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        record = ShardAccountRecord(
            account_id=normalized_id,
            account_type=account_type,
            display_name=display_name,
            status=status,
            record_version=1,
            created_generation=mutation_generation,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            accounts=self.accounts + (record,),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def register_agent_profile(
        self,
        *,
        agent_profile_id: str,
        controller_type: str = "external_agent",
        display_name: str | None = None,
        benchmark_bridge_actor_id: str | None = None,
        status: str = "active",
    ) -> "InProcessShardIdentityRegistry":
        normalized_id = _require_non_empty_string(agent_profile_id, field_name="agent_profile_id")
        for record in self.agent_profiles:
            if record.agent_profile_id == normalized_id:
                raise ValueError(f"agent_profile_id already exists: {normalized_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        record = ShardAgentProfileRecord(
            agent_profile_id=normalized_id,
            controller_type=controller_type,
            display_name=display_name,
            benchmark_bridge_actor_id=benchmark_bridge_actor_id,
            status=status,
            record_version=1,
            created_generation=mutation_generation,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            agent_profiles=self.agent_profiles + (record,),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def register_character(
        self,
        *,
        character_id: str,
        identity_class: str,
        owner_account_id: str | None = None,
        controller_agent_profile_id: str | None = None,
        system_identity_id: str | None = None,
        status: str = "active",
    ) -> "InProcessShardIdentityRegistry":
        normalized_id = _require_non_empty_string(character_id, field_name="character_id")
        for record in self.characters:
            if record.character_id == normalized_id:
                raise ValueError(f"character_id already exists: {normalized_id}")
        if owner_account_id is not None:
            self.get_account(owner_account_id)
        if controller_agent_profile_id is not None:
            self.get_agent_profile(controller_agent_profile_id)
        mutation_generation, mutation_id = self._next_mutation_marker()
        record = ShardCharacterRecord(
            character_id=normalized_id,
            identity_class=identity_class,
            owner_account_id=owner_account_id,
            controller_agent_profile_id=controller_agent_profile_id,
            system_identity_id=system_identity_id,
            status=status,
            record_version=1,
            created_generation=mutation_generation,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            characters=self.characters + (record,),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def register_system_identity(
        self,
        *,
        system_identity_id: str,
        identity_class: str,
        display_name: str | None = None,
        status: str = "active",
    ) -> "InProcessShardIdentityRegistry":
        normalized_id = _require_non_empty_string(
            system_identity_id,
            field_name="system_identity_id",
        )
        for record in self.system_identities:
            if record.system_identity_id == normalized_id:
                raise ValueError(f"system_identity_id already exists: {normalized_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        record = ShardSystemIdentityRecord(
            system_identity_id=normalized_id,
            identity_class=identity_class,
            display_name=display_name,
            status=status,
            record_version=1,
            created_generation=mutation_generation,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            system_identities=self.system_identities + (record,),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def activate_account(self, account_id: str) -> "InProcessShardIdentityRegistry":
        account = self.get_account(account_id)
        if account.status == "active":
            raise ValueError(f"account_id is already active: {account_id}")
        return self._update_account_lifecycle(account, status="active")

    def deactivate_account(self, account_id: str) -> "InProcessShardIdentityRegistry":
        account = self.get_account(account_id)
        if account.status != "active":
            raise ValueError(f"account_id is not active: {account_id}")
        return self._update_account_lifecycle(account, status="inactive")

    def activate_agent_profile(self, agent_profile_id: str) -> "InProcessShardIdentityRegistry":
        agent_profile = self.get_agent_profile(agent_profile_id)
        if agent_profile.status == "active":
            raise ValueError(f"agent_profile_id is already active: {agent_profile_id}")
        return self._update_agent_profile_lifecycle(agent_profile, status="active")

    def deactivate_agent_profile(self, agent_profile_id: str) -> "InProcessShardIdentityRegistry":
        agent_profile = self.get_agent_profile(agent_profile_id)
        if agent_profile.status != "active":
            raise ValueError(f"agent_profile_id is not active: {agent_profile_id}")
        return self._update_agent_profile_lifecycle(agent_profile, status="inactive")

    def activate_system_identity(self, system_identity_id: str) -> "InProcessShardIdentityRegistry":
        system_identity = self.get_system_identity(system_identity_id)
        if system_identity.status == "active":
            raise ValueError(f"system_identity_id is already active: {system_identity_id}")
        return self._update_system_identity_lifecycle(system_identity, status="active")

    def deactivate_system_identity(self, system_identity_id: str) -> "InProcessShardIdentityRegistry":
        system_identity = self.get_system_identity(system_identity_id)
        if system_identity.status != "active":
            raise ValueError(f"system_identity_id is not active: {system_identity_id}")
        return self._update_system_identity_lifecycle(system_identity, status="inactive")

    def bind_character_owner(
        self,
        *,
        character_id: str,
        account_id: str,
    ) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        normalized_account_id = _require_non_empty_string(account_id, field_name="account_id")
        self.get_account(normalized_account_id)
        if character.owner_account_id == normalized_account_id:
            raise ValueError(f"character_id is already bound to owner account_id: {normalized_account_id}")
        if character.owner_account_id is not None:
            raise ValueError(
                f"character_id is already bound to a different owner account_id: {character.owner_account_id}"
            )
        return self._update_character_binding(
            character,
            owner_account_id=normalized_account_id,
        )

    def unbind_character_owner(self, character_id: str) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        if character.owner_account_id is None:
            raise ValueError(f"character_id does not have an owner account binding: {character_id}")
        return self._update_character_binding(
            character,
            owner_account_id=None,
        )

    def bind_character_profile(
        self,
        *,
        character_id: str,
        agent_profile_id: str,
    ) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        normalized_profile_id = _require_non_empty_string(
            agent_profile_id,
            field_name="agent_profile_id",
        )
        self.get_agent_profile(normalized_profile_id)
        if character.controller_agent_profile_id == normalized_profile_id:
            raise ValueError(
                f"character_id is already bound to agent_profile_id: {normalized_profile_id}"
            )
        if character.controller_agent_profile_id is not None:
            raise ValueError(
                "character_id is already bound to a different controller agent_profile_id: "
                f"{character.controller_agent_profile_id}"
            )
        return self._update_character_binding(
            character,
            controller_agent_profile_id=normalized_profile_id,
        )

    def unbind_character_profile(self, character_id: str) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        if character.controller_agent_profile_id is None:
            raise ValueError(f"character_id does not have an agent profile binding: {character_id}")
        return self._update_character_binding(
            character,
            controller_agent_profile_id=None,
        )

    def bind_character_system_identity(
        self,
        *,
        character_id: str,
        system_identity_id: str,
    ) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        normalized_system_identity_id = _require_non_empty_string(
            system_identity_id,
            field_name="system_identity_id",
        )
        self.get_system_identity(normalized_system_identity_id)
        if character.system_identity_id == normalized_system_identity_id:
            raise ValueError(
                "character_id is already bound to system_identity_id: "
                f"{normalized_system_identity_id}"
            )
        if character.system_identity_id is not None:
            raise ValueError(
                "character_id is already bound to a different system_identity_id: "
                f"{character.system_identity_id}"
            )
        return self._update_character_binding(
            character,
            system_identity_id=normalized_system_identity_id,
        )

    def unbind_character_system_identity(self, character_id: str) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        if character.system_identity_id is None:
            raise ValueError(f"character_id does not have a system identity binding: {character_id}")
        return self._update_character_binding(
            character,
            system_identity_id=None,
        )

    def activate_character(self, character_id: str) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        if character.status == "active":
            raise ValueError(f"character_id is already active: {character_id}")
        return self._update_character_lifecycle(character, status="active")

    def deactivate_character(self, character_id: str) -> "InProcessShardIdentityRegistry":
        character = self.get_character(character_id)
        if character.status != "active":
            raise ValueError(f"character_id is not active: {character_id}")
        return self._update_character_lifecycle(character, status="inactive")

    def attach_session(
        self,
        *,
        session_id: str,
        character_id: str,
        account_id: str | None = None,
        agent_profile_id: str | None = None,
        presence_id: str | None = None,
        enforce_reconciliation: bool = False,
    ) -> "InProcessShardIdentityRegistry":
        normalized_session_id = _require_non_empty_string(session_id, field_name="session_id")
        normalized_character_id = _require_non_empty_string(character_id, field_name="character_id")
        for record in self.sessions:
            if record.session_id == normalized_session_id:
                raise ValueError(f"session_id already exists: {normalized_session_id}")
        character = self.get_character(normalized_character_id)
        if account_id is not None:
            self.get_account(account_id)
            if character.owner_account_id is not None and character.owner_account_id != account_id:
                raise ValueError("session account_id does not match character owner")
        if agent_profile_id is not None:
            self.get_agent_profile(agent_profile_id)
            if (
                character.controller_agent_profile_id is not None
                and character.controller_agent_profile_id != agent_profile_id
            ):
                raise ValueError("session agent_profile_id does not match character controller")
        if enforce_reconciliation:
            self._raise_for_reconciliation_failure(
                self.classify_session_attachment(
                    character_id=normalized_character_id,
                    account_id=account_id,
                    agent_profile_id=agent_profile_id,
                ),
                operation="attach_session",
                target_id=normalized_session_id,
            )
        if self.find_active_session_for_character(normalized_character_id) is not None:
            raise ValueError(f"character_id already has an active session: {normalized_character_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        record = ShardSessionRecord(
            session_id=normalized_session_id,
            character_id=normalized_character_id,
            status="active",
            account_id=account_id,
            agent_profile_id=agent_profile_id,
            presence_id=presence_id,
            record_version=1,
            attached_generation=mutation_generation,
            detached_generation=None,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            sessions=self.sessions + (record,),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def detach_session(self, session_id: str) -> "InProcessShardIdentityRegistry":
        session = self.get_session(session_id)
        if session.detached_generation is not None:
            raise ValueError(f"session_id is already detached: {session_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated = replace(
            session,
            status="inactive",
            record_version=session.record_version + 1,
            detached_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            sessions=_replace_record_by_id(self.sessions, updated, field_name="session_id"),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def activate_session(
        self,
        session_id: str,
        *,
        enforce_reconciliation: bool = False,
    ) -> "InProcessShardIdentityRegistry":
        session = self.get_session(session_id)
        if session.status == "active":
            raise ValueError(f"session_id is already active: {session_id}")
        if enforce_reconciliation:
            self._raise_for_reconciliation_failure(
                self.classify_session_attachment(
                    character_id=session.character_id,
                    account_id=session.account_id,
                    agent_profile_id=session.agent_profile_id,
                ),
                operation="activate_session",
                target_id=session_id,
            )
        active_for_character = self.find_active_session_for_character(session.character_id)
        if active_for_character is not None and active_for_character.session_id != session_id:
            raise ValueError(f"character_id already has an active session: {session.character_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated = replace(
            session,
            status="active",
            record_version=session.record_version + 1,
            detached_generation=None,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            sessions=_replace_record_by_id(self.sessions, updated, field_name="session_id"),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def deactivate_session(self, session_id: str) -> "InProcessShardIdentityRegistry":
        session = self.get_session(session_id)
        if session.status != "active":
            raise ValueError(f"session_id is not active: {session_id}")
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated = replace(
            session,
            status="inactive",
            record_version=session.record_version + 1,
            detached_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            sessions=_replace_record_by_id(self.sessions, updated, field_name="session_id"),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def _next_mutation_marker(self) -> tuple[int, str]:
        mutation_generation = self.expected_next_mutation_generation
        return mutation_generation, f"mut_{mutation_generation:08d}"

    def build_registry_checkpoint_snapshot_placeholder(self) -> dict[str, Any]:
        checkpoint_generation = self.last_committed_mutation_generation
        checkpoint_id = (
            None
            if checkpoint_generation == 0
            else f"registry_checkpoint_placeholder_{checkpoint_generation:08d}"
        )
        return {
            "payload_kind": "registry_checkpoint_snapshot_placeholder",
            "payload_version": 1,
            "shard_id": self.shard_id,
            "mode_marker": self.mode_marker,
            "checkpoint_generation": checkpoint_generation,
            "checkpoint_id": checkpoint_id,
            "checkpoint_commit_marker": "in_process_placeholder",
            "supports_checkpoint_persistence": self.supports_checkpoint_persistence,
            "supports_mutation_journal_persistence": self.supports_mutation_journal_persistence,
            "last_committed_mutation_generation": self.last_committed_mutation_generation,
            "last_committed_mutation_id": self.last_committed_mutation_id,
            "expected_next_mutation_generation": self.expected_next_mutation_generation,
            "record_family_counts": self.record_family_counts,
            "accounts": [record.to_dict() for record in self.accounts],
            "agent_profiles": [record.to_dict() for record in self.agent_profiles],
            "characters": [record.to_dict() for record in self.characters],
            "sessions": [record.to_dict() for record in self.sessions],
            "system_identities": [record.to_dict() for record in self.system_identities],
        }

    def build_mutation_journal_placeholder(self) -> dict[str, Any]:
        placeholder_entries = []
        for record_family, field_name, records in (
            ("accounts", "account_id", self.accounts),
            ("agent_profiles", "agent_profile_id", self.agent_profiles),
            ("characters", "character_id", self.characters),
            ("sessions", "session_id", self.sessions),
            ("system_identities", "system_identity_id", self.system_identities),
        ):
            for record in records:
                mutation_id = getattr(record, "last_mutation_id", None)
                mutation_generation = _mutation_generation_from_id(mutation_id)
                if mutation_id is None or mutation_generation is None:
                    continue
                placeholder_entries.append(
                    {
                        "mutation_generation": mutation_generation,
                        "mutation_id": mutation_id,
                        "record_family": record_family,
                        "record_id": str(getattr(record, field_name)),
                        "record_version": int(getattr(record, "record_version")),
                    }
                )

        placeholder_entries = sorted(
            placeholder_entries,
            key=lambda entry: (
                int(entry["mutation_generation"]),
                str(entry["record_family"]),
                str(entry["record_id"]),
            ),
        )
        return {
            "payload_kind": "registry_mutation_journal_placeholder",
            "payload_version": 1,
            "shard_id": self.shard_id,
            "mode_marker": self.mode_marker,
            "mutation_journal_mode": "in_process_placeholder",
            "supports_mutation_journal_persistence": self.supports_mutation_journal_persistence,
            "last_committed_mutation_generation": self.last_committed_mutation_generation,
            "last_committed_mutation_id": self.last_committed_mutation_id,
            "expected_next_mutation_generation": self.expected_next_mutation_generation,
            "placeholder_entry_count": len(placeholder_entries),
            "placeholder_entries": placeholder_entries,
        }

    def write_registry_checkpoint_placeholder(self, path: Any) -> str:
        return _write_json_payload(
            path,
            self.build_registry_checkpoint_snapshot_placeholder(),
            artifact_label="registry checkpoint placeholder",
        )

    def write_mutation_journal_placeholder(self, path: Any) -> str:
        return _write_json_payload(
            path,
            self.build_mutation_journal_placeholder(),
            artifact_label="mutation journal placeholder",
        )

    def _raise_for_reconciliation_failure(
        self,
        reconciliation_result: dict[str, Any],
        *,
        operation: str,
        target_id: str,
    ) -> None:
        issue_codes = tuple(reconciliation_result["issue_codes"])
        if issue_codes:
            joined_codes = ",".join(issue_codes)
            raise ValueError(
                f"{operation} reconciliation failed for {target_id}: {joined_codes}"
            )

    def _classify_session_linkage(
        self,
        *,
        character_id: str,
        account_id: str | None,
        agent_profile_id: str | None,
        require_active_lifecycle: bool,
        competing_active_session_id: str | None,
    ) -> dict[str, Any]:
        issue_codes: list[str] = []
        character: ShardCharacterRecord | None = None
        account: ShardAccountRecord | None = None
        agent_profile: ShardAgentProfileRecord | None = None
        system_identity: ShardSystemIdentityRecord | None = None

        try:
            character = self.get_character(character_id)
        except ValueError:
            issue_codes.append("missing_character")
        else:
            if require_active_lifecycle and character.status != "active":
                issue_codes.append("inactive_character")

        if account_id is not None:
            try:
                account = self.get_account(account_id)
            except ValueError:
                issue_codes.append("missing_account")
            else:
                if require_active_lifecycle and account.status != "active":
                    issue_codes.append("inactive_account")
            if character is not None and character.owner_account_id not in (None, account_id):
                issue_codes.append("account_character_owner_mismatch")

        if agent_profile_id is not None:
            try:
                agent_profile = self.get_agent_profile(agent_profile_id)
            except ValueError:
                issue_codes.append("missing_agent_profile")
            else:
                if require_active_lifecycle and agent_profile.status != "active":
                    issue_codes.append("inactive_agent_profile")
            if (
                character is not None
                and character.controller_agent_profile_id not in (None, agent_profile_id)
            ):
                issue_codes.append("profile_character_controller_mismatch")

        if character is not None and character.system_identity_id is not None:
            try:
                system_identity = self.get_system_identity(character.system_identity_id)
            except ValueError:
                issue_codes.append("missing_system_identity")
            else:
                if require_active_lifecycle and system_identity.status != "active":
                    issue_codes.append("inactive_system_identity")

        if character is not None and require_active_lifecycle:
            active_session = self.find_active_session_for_character(character.character_id)
            if active_session is not None and active_session.session_id != competing_active_session_id:
                issue_codes.append("character_already_has_active_session")

        return {
            "character_id": character_id,
            "character_status": None if character is None else character.status,
            "account_id": account_id,
            "account_status": None if account is None else account.status,
            "agent_profile_id": agent_profile_id,
            "agent_profile_status": None if agent_profile is None else agent_profile.status,
            "system_identity_id": None if character is None else character.system_identity_id,
            "system_identity_status": None if system_identity is None else system_identity.status,
            "issue_codes": tuple(issue_codes),
            "is_consistent": len(issue_codes) == 0,
        }

    def _update_character_binding(
        self,
        character: ShardCharacterRecord,
        *,
        owner_account_id: str | None | object = _UNSET,
        controller_agent_profile_id: str | None | object = _UNSET,
        system_identity_id: str | None | object = _UNSET,
    ) -> "InProcessShardIdentityRegistry":
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated_character = replace(
            character,
            owner_account_id=(
                character.owner_account_id if owner_account_id is _UNSET else owner_account_id
            ),
            controller_agent_profile_id=(
                character.controller_agent_profile_id
                if controller_agent_profile_id is _UNSET
                else controller_agent_profile_id
            ),
            system_identity_id=(
                character.system_identity_id
                if system_identity_id is _UNSET
                else system_identity_id
            ),
            record_version=character.record_version + 1,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            characters=_replace_record_by_id(
                self.characters,
                updated_character,
                field_name="character_id",
            ),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def _update_character_lifecycle(
        self,
        character: ShardCharacterRecord,
        *,
        status: str,
    ) -> "InProcessShardIdentityRegistry":
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated_character = replace(
            character,
            status=_require_non_empty_string(status, field_name="status"),
            record_version=character.record_version + 1,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            characters=_replace_record_by_id(
                self.characters,
                updated_character,
                field_name="character_id",
            ),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def _update_account_lifecycle(
        self,
        account: ShardAccountRecord,
        *,
        status: str,
    ) -> "InProcessShardIdentityRegistry":
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated_account = replace(
            account,
            status=_require_non_empty_string(status, field_name="status"),
            record_version=account.record_version + 1,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            accounts=_replace_record_by_id(
                self.accounts,
                updated_account,
                field_name="account_id",
            ),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def _update_agent_profile_lifecycle(
        self,
        agent_profile: ShardAgentProfileRecord,
        *,
        status: str,
    ) -> "InProcessShardIdentityRegistry":
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated_agent_profile = replace(
            agent_profile,
            status=_require_non_empty_string(status, field_name="status"),
            record_version=agent_profile.record_version + 1,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            agent_profiles=_replace_record_by_id(
                self.agent_profiles,
                updated_agent_profile,
                field_name="agent_profile_id",
            ),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def _update_system_identity_lifecycle(
        self,
        system_identity: ShardSystemIdentityRecord,
        *,
        status: str,
    ) -> "InProcessShardIdentityRegistry":
        mutation_generation, mutation_id = self._next_mutation_marker()
        updated_system_identity = replace(
            system_identity,
            status=_require_non_empty_string(status, field_name="status"),
            record_version=system_identity.record_version + 1,
            updated_generation=mutation_generation,
            last_mutation_id=mutation_id,
        )
        return replace(
            self,
            system_identities=_replace_record_by_id(
                self.system_identities,
                updated_system_identity,
                field_name="system_identity_id",
            ),
            last_committed_mutation_generation=mutation_generation,
            last_committed_mutation_id=mutation_id,
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "shard_id": self.shard_id,
            "mode_marker": self.mode_marker,
            "last_committed_mutation_generation": self.last_committed_mutation_generation,
            "last_committed_mutation_id": self.last_committed_mutation_id,
            "expected_next_mutation_generation": self.expected_next_mutation_generation,
            "supports_checkpoint_persistence": self.supports_checkpoint_persistence,
            "supports_mutation_journal_persistence": self.supports_mutation_journal_persistence,
            "record_family_counts": self.record_family_counts,
            "accounts": [record.to_dict() for record in self.accounts],
            "agent_profiles": [record.to_dict() for record in self.agent_profiles],
            "characters": [record.to_dict() for record in self.characters],
            "sessions": [record.to_dict() for record in self.sessions],
            "system_identities": [record.to_dict() for record in self.system_identities],
        }
