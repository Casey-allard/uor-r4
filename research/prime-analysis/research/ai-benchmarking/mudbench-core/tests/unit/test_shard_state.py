from __future__ import annotations

import hashlib
import json

import pytest

from world.state import (
    InProcessShardIdentityRegistry,
    ShardAccountRecord,
    ShardAgentProfileRecord,
    ShardCharacterRecord,
    ShardMutationPolicy,
    ShardSessionRecord,
    ShardState,
    ShardSystemIdentityRecord,
)


def test_shard_state_initialization_is_deterministic() -> None:
    first = ShardState.create_empty("shard-dev")
    second = ShardState.create_empty("shard-dev")

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.to_dict()["shard_id"] == "shard-dev"
    assert first.to_dict()["metadata"]["mode_marker"] == "persistent_world_shell"


def test_identity_registry_shell_initialization_is_deterministic() -> None:
    first = InProcessShardIdentityRegistry.create_empty("shard-dev")
    second = InProcessShardIdentityRegistry.create_empty("shard-dev")

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.record_family_counts == {
        "accounts": 0,
        "agent_profiles": 0,
        "characters": 0,
        "sessions": 0,
        "system_identities": 0,
    }


def test_shard_state_exposes_clear_metadata_and_generation_placeholders() -> None:
    state = ShardState.create_empty(
        "shard-alpha",
        world_ruleset_version="world-v1",
        benchmark_engine_version="engine-v1",
        scheduler_policy_version="scheduler-v1",
    )

    payload = state.to_dict()

    assert payload["metadata"] == {
        "shard_id": "shard-alpha",
        "mode_marker": "persistent_world_shell",
        "shard_status": "initializing",
        "world_ruleset_version": "world-v1",
        "benchmark_engine_version": "engine-v1",
        "scheduler_policy_version": "scheduler-v1",
        "world_tick_count": 0,
        "last_world_tick_heartbeat": "not_started",
        "npc_stance_phase": "dormant",
        "timing_mode": None,
        "action_cadence_interval": None,
        "actor_action_cadence_overrides": [],
        "actor_next_action_eligible_at": [],
        "season_id": None,
        "created_from_snapshot_ref": None,
    }
    assert payload["checkpoint"] == {
        "registry_checkpoint_generation": 0,
        "registry_checkpoint_id": None,
        "checkpoint_commit_marker": "not_persisted",
        "snapshot_payload_hash": None,
    }
    assert payload["journal"] == {
        "last_committed_mutation_generation": 0,
        "last_committed_mutation_id": None,
        "expected_next_mutation_generation": 1,
        "mutation_journal_mode": "not_persisted",
    }
    assert payload["mutation_policy"] == {
        "enforce_session_principal_reconciliation": False,
    }


def test_shard_state_world_tick_hook_is_deterministic_without_mutating_identity_journal() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_character(
            character_id="char-alice",
            identity_class="human_player",
            owner_account_id="acct-alice",
        )
    )

    advanced_once = state.advance_world_tick()
    advanced_twice = advanced_once.advance_world_tick()

    assert state.metadata.world_tick_count == 0
    assert state.metadata.last_world_tick_heartbeat == "not_started"
    assert state.metadata.npc_stance_phase == "dormant"
    assert advanced_once.metadata.world_tick_count == 1
    assert advanced_once.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0001"
    assert advanced_once.metadata.npc_stance_phase == "watchful"
    assert advanced_twice.metadata.world_tick_count == 2
    assert advanced_twice.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert advanced_twice.metadata.npc_stance_phase == "patrolling"
    assert advanced_twice.journal.last_committed_mutation_generation == state.journal.last_committed_mutation_generation
    assert advanced_twice.journal.expected_next_mutation_generation == state.journal.expected_next_mutation_generation
    assert advanced_twice.identity_registry.to_dict() == state.identity_registry.to_dict()


def test_shard_state_action_cadence_is_disabled_by_default() -> None:
    state = ShardState.create_empty("shard-dev")

    assert state.action_cadence_enabled is False
    assert state.action_cadence_interval is None
    assert state.actor_action_cadence_overrides == ()
    assert state.actor_next_action_eligible_at == ()
    assert state.is_actor_action_eligible("player-a") is True
    assert state.get_actor_next_action_eligible_at("player-a") == 0


def test_shard_state_action_cadence_tracks_next_eligible_tick_and_overrides_deterministically() -> None:
    state = ShardState.create_empty("shard-dev").with_action_cadence(
        timing_mode="equal-cadence",
        action_cadence_interval=2,
        actor_action_cadence_overrides={"player-b": 3},
    )

    accepted_a = state.record_actor_action_acceptance("player-a")
    accepted_b = accepted_a.record_actor_action_acceptance("player-b")
    ticked_once = accepted_b.advance_world_tick()
    ticked_twice = ticked_once.advance_world_tick()
    ticked_thrice = ticked_twice.advance_world_tick()

    assert state.action_cadence_enabled is True
    assert state.timing_mode == "equal-cadence"
    assert state.action_cadence_interval == 2
    assert state.actor_action_cadence_overrides == (("player-b", 3),)
    assert accepted_b.actor_next_action_eligible_at == (("player-a", 2), ("player-b", 3))
    assert accepted_b.is_actor_action_eligible("player-a") is False
    assert accepted_b.is_actor_action_eligible("player-b") is False
    assert ticked_once.is_actor_action_eligible("player-a") is False
    assert ticked_twice.is_actor_action_eligible("player-a") is True
    assert ticked_twice.is_actor_action_eligible("player-b") is False
    assert ticked_thrice.is_actor_action_eligible("player-b") is True


def test_shard_state_with_action_cadence_tracks_optional_timing_mode_deterministically() -> None:
    state = ShardState.create_empty("shard-dev").with_action_cadence(
        timing_mode="human-parity",
        action_cadence_interval=2,
    )

    assert state.timing_mode == "human-parity"
    assert state.to_dict()["metadata"]["timing_mode"] == "human-parity"


def test_shard_state_world_tick_scene_message_tracks_npc_stance_phase_deterministically() -> None:
    state = ShardState.create_empty("shard-dev")

    first = state.advance_world_tick()
    second = first.advance_world_tick()
    third = second.advance_world_tick()

    assert state.get_world_tick_scene_message() == (
        "The shard feels still, as if the watch has not yet begun."
    )
    assert first.get_world_tick_scene_message() == (
        "A distant sentinel grows watchful in the quiet corridors."
    )
    assert second.get_world_tick_scene_message() == (
        "You catch the measured rhythm of a distant patrol."
    )
    assert third.get_world_tick_scene_message() == (
        "The far-off watch settles back into guarded stillness."
    )


def test_shard_state_world_phase_interaction_hint_tracks_stance_phase_deterministically() -> None:
    state = ShardState.create_empty("shard-dev")

    first = state.advance_world_tick()
    second = first.advance_world_tick()
    third = second.advance_world_tick()

    assert state.get_world_phase_interaction_hint() == (
        "Hint: the route feels open while the watch remains dormant."
    )
    assert first.get_world_phase_interaction_hint() == (
        "Hint: the sharp watch makes a careful look feel safer than rushing."
    )
    assert second.get_world_phase_interaction_hint() == (
        "Hint: the moving patrol leaves brief windows for repositioning."
    )
    assert third.get_world_phase_interaction_hint() == (
        "Hint: the easing watch makes nearby movement feel less exposed."
    )
    assert second.get_world_tick_observation_messages() == (
        "You catch the measured rhythm of a distant patrol.",
        "Hint: the moving patrol leaves brief windows for repositioning.",
    )


def test_shard_state_world_phase_outcome_effect_is_deterministic_for_corridor_route() -> None:
    state = ShardState.create_empty("shard-dev")

    watchful = state.advance_world_tick()
    patrolling = watchful.advance_world_tick()
    settling = patrolling.advance_world_tick()

    action_space = ("wait", "look", "move east", "move west")

    assert state.get_world_phase_outcome_message(location="corridor") == (
        "Consequence: the exposed west passage is open before the watch begins."
    )
    assert watchful.get_world_phase_outcome_message(location="corridor") == (
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
    )
    assert patrolling.get_world_phase_outcome_message(location="corridor") == (
        "Consequence: the west passage opens again between patrol sweeps."
    )
    assert settling.get_world_phase_outcome_message(location="corridor") == (
        "Consequence: the west passage remains open as the watch settles."
    )
    assert watchful.get_world_phase_filtered_action_space(
        location="corridor",
        action_space=action_space,
    ) == ("wait", "look", "move east")
    assert patrolling.get_world_phase_filtered_action_space(
        location="corridor",
        action_space=action_space,
    ) == action_space
    assert watchful.get_world_tick_observation_messages(location="corridor") == (
        "A distant sentinel grows watchful in the quiet corridors.",
        "Hint: the sharp watch makes a careful look feel safer than rushing.",
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable.",
    )


def test_registry_shell_uses_sorted_record_categories_without_persistence_side_effects() -> None:
    registry = InProcessShardIdentityRegistry(
        shard_id="shard-dev",
        accounts=(
            ShardAccountRecord(account_id="acct-bob"),
            ShardAccountRecord(account_id="acct-alice"),
        ),
        agent_profiles=(
            ShardAgentProfileRecord(agent_profile_id="profile-z"),
            ShardAgentProfileRecord(agent_profile_id="profile-a"),
        ),
        characters=(
            ShardCharacterRecord(character_id="char-z", identity_class="external_agent"),
            ShardCharacterRecord(character_id="char-a", identity_class="human_player"),
        ),
        sessions=(
            ShardSessionRecord(session_id="sess-z", character_id="char-z"),
            ShardSessionRecord(session_id="sess-a", character_id="char-a"),
        ),
        system_identities=(
            ShardSystemIdentityRecord(
                system_identity_id="sys-z",
                identity_class="npc_controller",
            ),
            ShardSystemIdentityRecord(
                system_identity_id="sys-a",
                identity_class="built_in_actor",
            ),
        ),
    )

    payload = registry.to_dict()

    assert payload["supports_checkpoint_persistence"] is False
    assert payload["supports_mutation_journal_persistence"] is False
    assert payload["expected_next_mutation_generation"] == 1
    assert [entry["account_id"] for entry in payload["accounts"]] == ["acct-alice", "acct-bob"]
    assert [entry["agent_profile_id"] for entry in payload["agent_profiles"]] == [
        "profile-a",
        "profile-z",
    ]
    assert [entry["character_id"] for entry in payload["characters"]] == ["char-a", "char-z"]
    assert [entry["session_id"] for entry in payload["sessions"]] == ["sess-a", "sess-z"]
    assert [entry["system_identity_id"] for entry in payload["system_identities"]] == [
        "sys-a",
        "sys-z",
    ]


def test_registry_mutations_are_deterministic_for_creation_and_lookup() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-mock",
            display_name="Mock Profile",
        )
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-mock",
        )
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-mock",
            display_name="Mock Profile",
        )
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-mock",
        )
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.last_committed_mutation_generation == 3
    assert first.last_committed_mutation_id == "mut_00000003"
    assert first.expected_next_mutation_generation == 4
    assert first.get_account("acct-alice").record_version == 1
    assert first.get_agent_profile("profile-mock").record_version == 1
    assert first.get_character("char-alice").record_version == 1
    assert [record.character_id for record in first.find_characters_by_account("acct-alice")] == [
        "char-alice"
    ]
    assert [record.character_id for record in first.find_characters_by_profile("profile-mock")] == [
        "char-alice"
    ]


def test_registry_session_attach_detach_and_presence_state_are_deterministic() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-mock")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-mock",
        )
    )

    attached = registry.attach_session(
        session_id="sess-1",
        character_id="char-alice",
        account_id="acct-alice",
        agent_profile_id="profile-mock",
        presence_id="pres-1",
    )
    active_session = attached.get_session("sess-1")
    assert active_session.status == "active"
    assert active_session.record_version == 1
    assert active_session.attached_generation == 4
    assert attached.find_active_session_for_character("char-alice") == active_session

    deactivated = attached.deactivate_session("sess-1")
    inactive_session = deactivated.get_session("sess-1")
    assert inactive_session.status == "inactive"
    assert inactive_session.record_version == 2
    assert inactive_session.detached_generation == 5
    assert deactivated.find_active_session_for_character("char-alice") is None

    reactivated = deactivated.activate_session("sess-1")
    reactivated_session = reactivated.get_session("sess-1")
    assert reactivated_session.status == "active"
    assert reactivated_session.record_version == 3
    assert reactivated_session.detached_generation is None

    detached = reactivated.detach_session("sess-1")
    detached_session = detached.get_session("sess-1")
    assert detached_session.status == "inactive"
    assert detached_session.record_version == 4
    assert detached_session.detached_generation == 7
    assert detached.last_committed_mutation_generation == 7
    assert detached.last_committed_mutation_id == "mut_00000007"


def test_session_principal_reconciliation_is_deterministic_for_valid_active_state() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )

    first_attachment = first.classify_session_attachment(
        character_id="char-alice",
        account_id="acct-alice",
        agent_profile_id="profile-a",
    )
    second_attachment = second.classify_session_attachment(
        character_id="char-alice",
        account_id="acct-alice",
        agent_profile_id="profile-a",
    )
    first_session = first.classify_session_reconciliation("sess-1")
    second_session = second.classify_session_reconciliation("sess-1")
    first_summary = first.classify_session_principal_reconciliation()
    second_summary = second.classify_session_principal_reconciliation()

    assert first_attachment == second_attachment
    assert first_session == second_session
    assert first_summary == second_summary
    assert first_attachment["issue_codes"] == ("character_already_has_active_session",)
    assert first_attachment["is_consistent"] is False
    assert first_session["issue_codes"] == ()
    assert first_session["is_consistent"] is True
    assert first_session["character_status"] == "active"
    assert first_session["account_status"] == "active"
    assert first_session["agent_profile_status"] == "active"
    assert first_summary["is_consistent"] is True
    assert first_summary["session_count"] == 1
    assert first_summary["consistent_session_ids"] == ("sess-1",)
    assert first_summary["inconsistent_session_ids"] == ()
    assert first.supports_checkpoint_persistence is False
    assert first.supports_mutation_journal_persistence is False
    assert first.last_committed_mutation_generation == 4
    assert first.expected_next_mutation_generation == 5


def test_session_principal_reconciliation_detects_invalid_lifecycle_and_binding_state() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
    )

    invalid_attachment = registry.classify_session_attachment(
        character_id="char-guard",
        account_id="acct-bob",
        agent_profile_id="profile-b",
    )
    session_reconciliation = registry.classify_session_reconciliation("sess-guard")
    summary = registry.classify_session_principal_reconciliation()

    assert invalid_attachment["issue_codes"] == (
        "inactive_character",
        "account_character_owner_mismatch",
        "profile_character_controller_mismatch",
        "inactive_system_identity",
        "character_already_has_active_session",
    )
    assert invalid_attachment["is_consistent"] is False
    assert session_reconciliation["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
    )
    assert session_reconciliation["is_consistent"] is False
    assert session_reconciliation["system_identity_id"] == "sys-guard"
    assert summary["is_consistent"] is False
    assert summary["consistent_session_ids"] == ()
    assert summary["inconsistent_session_ids"] == ("sess-guard",)
    assert summary["session_results"] == (session_reconciliation,)
    assert registry.supports_checkpoint_persistence is False
    assert registry.supports_mutation_journal_persistence is False


def test_session_reconciliation_enforcement_preserves_permissive_mode_by_default() -> None:
    attached = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )

    reconciliation = attached.classify_session_reconciliation("sess-guard")

    assert attached.get_session("sess-guard").status == "active"
    assert reconciliation["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
    )
    assert reconciliation["is_consistent"] is False
    assert attached.supports_checkpoint_persistence is False
    assert attached.supports_mutation_journal_persistence is False


def test_session_reconciliation_enforcement_rejects_invalid_attach_deterministically() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
    )

    with pytest.raises(
        ValueError,
        match=(
            "attach_session reconciliation failed for sess-guard: "
            "inactive_character,inactive_account,inactive_agent_profile,inactive_system_identity"
        ),
    ):
        registry.attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            enforce_reconciliation=True,
        )


def test_session_reconciliation_enforcement_allows_valid_attach_deterministically() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            enforce_reconciliation=True,
        )
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            enforce_reconciliation=True,
        )
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.classify_session_reconciliation("sess-guard")["is_consistent"] is True
    assert first.get_session("sess-guard").status == "active"


def test_session_reconciliation_enforcement_rejects_invalid_activate_and_allows_permissive_default() -> None:
    base = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            enforce_reconciliation=True,
        )
        .detach_session("sess-guard")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
    )

    with pytest.raises(
        ValueError,
        match=(
            "activate_session reconciliation failed for sess-guard: "
            "inactive_character,inactive_account,inactive_agent_profile,inactive_system_identity"
        ),
    ):
        base.activate_session("sess-guard", enforce_reconciliation=True)

    permissive = base.activate_session("sess-guard")

    assert permissive.get_session("sess-guard").status == "active"
    assert permissive.classify_session_reconciliation("sess-guard")["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
    )


def test_shard_state_policy_default_remains_permissive_for_session_mutations() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
            .deactivate_account("acct-alice")
            .deactivate_agent_profile("profile-a")
            .deactivate_system_identity("sys-guard")
            .deactivate_character("char-guard")
        )
    )

    attached_state = state.attach_session(
        session_id="sess-guard",
        character_id="char-guard",
        account_id="acct-alice",
        agent_profile_id="profile-a",
    )

    assert attached_state.mutation_policy == ShardMutationPolicy()
    assert attached_state.identity_registry.get_session("sess-guard").status == "active"
    assert attached_state.identity_registry.classify_session_reconciliation("sess-guard")["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
    )
    assert attached_state.identity_registry.supports_checkpoint_persistence is False
    assert attached_state.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_policy_rejects_invalid_attach_when_enforced() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
            .deactivate_account("acct-alice")
            .deactivate_agent_profile("profile-a")
            .deactivate_system_identity("sys-guard")
            .deactivate_character("char-guard")
        )
        .with_session_principal_reconciliation_enforced()
    )

    with pytest.raises(
        ValueError,
        match=(
            "attach_session reconciliation failed for sess-guard: "
            "inactive_character,inactive_account,inactive_agent_profile,inactive_system_identity"
        ),
    ):
        state.attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )


def test_shard_state_policy_allows_valid_attach_deterministically_when_enforced() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
        )
        .with_session_principal_reconciliation_enforced()
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    second = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
        )
        .with_session_principal_reconciliation_enforced()
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.mutation_policy.enforce_session_principal_reconciliation is True
    assert first.identity_registry.classify_session_reconciliation("sess-guard")["is_consistent"] is True


def test_shard_state_policy_rejects_invalid_activate_when_enforced() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
        )
        .with_session_principal_reconciliation_enforced()
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
            .bind_character_owner(character_id="char-guard", account_id="acct-alice")
            .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
            .bind_character_system_identity(
                character_id="char-guard",
                system_identity_id="sys-guard",
            )
            .attach_session(
                session_id="sess-guard",
                character_id="char-guard",
                account_id="acct-alice",
                agent_profile_id="profile-a",
                enforce_reconciliation=True,
            )
            .detach_session("sess-guard")
            .deactivate_account("acct-alice")
            .deactivate_agent_profile("profile-a")
            .deactivate_system_identity("sys-guard")
            .deactivate_character("char-guard")
        )
    )

    with pytest.raises(
        ValueError,
        match=(
            "activate_session reconciliation failed for sess-guard: "
            "inactive_character,inactive_account,inactive_agent_profile,inactive_system_identity"
        ),
    ):
        state.activate_session("sess-guard")


def test_shard_state_session_exit_helpers_are_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_character(
                character_id="char-alice",
                identity_class="external_agent",
                owner_account_id="acct-alice",
                controller_agent_profile_id="profile-a",
            )
        )
        .attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .deactivate_session("sess-1")
        .activate_session("sess-1")
        .detach_session("sess-1")
    )
    second = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_character(
                character_id="char-alice",
                identity_class="external_agent",
                owner_account_id="acct-alice",
                controller_agent_profile_id="profile-a",
            )
        )
        .attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .deactivate_session("sess-1")
        .activate_session("sess-1")
        .detach_session("sess-1")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.mutation_policy == ShardMutationPolicy()
    session = first.identity_registry.get_session("sess-1")
    assert session.status == "inactive"
    assert session.record_version == 4
    assert session.detached_generation == 7
    assert first.journal.last_committed_mutation_generation == 7
    assert first.journal.last_committed_mutation_id == "mut_00000007"
    assert first.journal.expected_next_mutation_generation == 8
    assert first.checkpoint.registry_checkpoint_generation == 0
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_session_exit_helpers_preserve_default_permissive_policy() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_character(
                character_id="char-alice",
                identity_class="external_agent",
                owner_account_id="acct-alice",
                controller_agent_profile_id="profile-a",
            )
        )
        .attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .with_session_principal_reconciliation_enforced(False)
    )

    deactivated = state.deactivate_session("sess-1")
    detached = state.detach_session("sess-1")

    assert deactivated.mutation_policy.enforce_session_principal_reconciliation is False
    assert detached.mutation_policy.enforce_session_principal_reconciliation is False
    assert deactivated.identity_registry.get_session("sess-1").status == "inactive"
    assert detached.identity_registry.get_session("sess-1").status == "inactive"
    assert deactivated.identity_registry.get_session("sess-1").detached_generation == 5
    assert detached.identity_registry.get_session("sess-1").detached_generation == 5


def test_shard_state_lifecycle_and_binding_wrappers_are_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
        )
        .deactivate_account("acct-alice")
        .activate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .activate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .activate_system_identity("sys-guard")
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .unbind_character_owner("char-guard")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .unbind_character_profile("char-guard")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .unbind_character_system_identity("char-guard")
        .deactivate_character("char-guard")
        .activate_character("char-guard")
    )
    second = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
        )
        .deactivate_account("acct-alice")
        .activate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .activate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .activate_system_identity("sys-guard")
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .unbind_character_owner("char-guard")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .unbind_character_profile("char-guard")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .unbind_character_system_identity("char-guard")
        .deactivate_character("char-guard")
        .activate_character("char-guard")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.identity_registry.get_account("acct-alice").status == "active"
    assert first.identity_registry.get_agent_profile("profile-a").status == "active"
    assert first.identity_registry.get_system_identity("sys-guard").status == "active"
    assert first.identity_registry.get_character("char-guard").status == "active"
    assert first.identity_registry.get_character("char-guard").owner_account_id is None
    assert first.identity_registry.get_character("char-guard").controller_agent_profile_id is None
    assert first.identity_registry.get_character("char-guard").system_identity_id is None
    assert first.journal.last_committed_mutation_generation == 18
    assert first.journal.last_committed_mutation_id == "mut_00000018"
    assert first.journal.expected_next_mutation_generation == 19
    assert first.checkpoint.registry_checkpoint_generation == 0
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_lifecycle_and_binding_wrappers_preserve_invalid_rejections() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .with_identity_registry(
            ShardState.create_empty("shard-dev")
            .identity_registry
            .register_account(account_id="acct-alice")
            .register_account(account_id="acct-bob")
            .register_agent_profile(agent_profile_id="profile-a")
            .register_agent_profile(agent_profile_id="profile-b")
            .register_system_identity(
                system_identity_id="sys-guard",
                identity_class="npc_controller",
            )
            .register_system_identity(
                system_identity_id="sys-market",
                identity_class="system_agent",
            )
            .register_character(
                character_id="char-guard",
                identity_class="npc_controller",
            )
        )
    )

    with pytest.raises(ValueError, match="account_id is already active"):
        state.activate_account("acct-alice")

    owner_bound = state.bind_character_owner(character_id="char-guard", account_id="acct-alice")
    with pytest.raises(ValueError, match="already bound to a different owner"):
        owner_bound.bind_character_owner(character_id="char-guard", account_id="acct-bob")

    profile_bound = owner_bound.bind_character_profile(
        character_id="char-guard",
        agent_profile_id="profile-a",
    )
    with pytest.raises(ValueError, match="already bound to a different controller"):
        profile_bound.bind_character_profile(
            character_id="char-guard",
            agent_profile_id="profile-b",
        )

    system_bound = profile_bound.bind_character_system_identity(
        character_id="char-guard",
        system_identity_id="sys-guard",
    )
    with pytest.raises(ValueError, match="already bound to a different system_identity_id"):
        system_bound.bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-market",
        )

    inactive_character = system_bound.deactivate_character("char-guard")
    with pytest.raises(ValueError, match="character_id is not active"):
        inactive_character.deactivate_character("char-guard")


def test_shard_state_registration_wrappers_are_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
            benchmark_bridge_actor_id="agent-a",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
            benchmark_bridge_actor_id="agent-a",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.identity_registry.get_account("acct-alice").display_name == "Alice"
    assert first.identity_registry.get_agent_profile("profile-a").benchmark_bridge_actor_id == "agent-a"
    assert first.identity_registry.get_system_identity("sys-guard").display_name == "Guard System"
    character = first.identity_registry.get_character("char-guard")
    assert character.owner_account_id == "acct-alice"
    assert character.controller_agent_profile_id == "profile-a"
    assert character.system_identity_id == "sys-guard"
    assert first.journal.last_committed_mutation_generation == 4
    assert first.journal.last_committed_mutation_id == "mut_00000004"
    assert first.journal.expected_next_mutation_generation == 5
    assert first.checkpoint.registry_checkpoint_generation == 0
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_registration_wrappers_preserve_duplicate_rejections() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
    )

    with pytest.raises(ValueError, match="account_id already exists"):
        state.register_account(account_id="acct-alice")

    with pytest.raises(ValueError, match="agent_profile_id already exists"):
        state.register_agent_profile(agent_profile_id="profile-a")

    with pytest.raises(ValueError, match="system_identity_id already exists"):
        state.register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )

    with pytest.raises(ValueError, match="character_id already exists"):
        state.register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )


def test_shard_state_query_helpers_are_deterministic_for_common_reads() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )

    assert first.get_account("acct-alice") == second.get_account("acct-alice")
    assert first.get_agent_profile("profile-a") == second.get_agent_profile("profile-a")
    assert first.get_system_identity("sys-guard") == second.get_system_identity("sys-guard")
    assert first.get_character("char-guard") == second.get_character("char-guard")
    assert first.get_session("sess-guard") == second.get_session("sess-guard")
    assert first.find_sessions_by_account("acct-alice") == second.find_sessions_by_account("acct-alice")
    assert first.find_sessions_by_profile("profile-a") == second.find_sessions_by_profile("profile-a")
    assert first.find_characters_by_status("active") == second.find_characters_by_status("active")
    assert first.find_characters_by_system_identity("sys-guard") == second.find_characters_by_system_identity("sys-guard")
    assert first.get_account("acct-alice").display_name == "Alice"
    assert [record.session_id for record in first.find_sessions_by_account("acct-alice")] == ["sess-guard"]
    assert [record.session_id for record in first.find_sessions_by_profile("profile-a")] == ["sess-guard"]
    assert [record.character_id for record in first.find_characters_by_status("active")] == ["char-guard"]
    assert [record.character_id for record in first.find_characters_by_system_identity("sys-guard")] == ["char-guard"]
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_query_helpers_are_deterministic_for_reconciliation_summaries() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .deactivate_character("char-guard")
    )

    first_attachment = first.classify_session_attachment(
        character_id="char-guard",
        account_id="acct-alice",
        agent_profile_id="profile-a",
    )
    second_attachment = second.classify_session_attachment(
        character_id="char-guard",
        account_id="acct-alice",
        agent_profile_id="profile-a",
    )
    first_session = first.classify_session_reconciliation("sess-guard")
    second_session = second.classify_session_reconciliation("sess-guard")
    first_summary = first.classify_session_principal_reconciliation()
    second_summary = second.classify_session_principal_reconciliation()

    assert first_attachment == second_attachment
    assert first_session == second_session
    assert first_summary == second_summary
    assert first_attachment["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
        "character_already_has_active_session",
    )
    assert first_session["issue_codes"] == (
        "inactive_character",
        "inactive_account",
        "inactive_agent_profile",
        "inactive_system_identity",
    )
    assert first_summary["is_consistent"] is False
    assert first_summary["inconsistent_session_ids"] == ("sess-guard",)


def test_inprocess_checkpoint_snapshot_placeholders_are_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
            benchmark_bridge_actor_id="agent-a",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice", display_name="Alice")
        .register_agent_profile(
            agent_profile_id="profile-a",
            display_name="Profile A",
            benchmark_bridge_actor_id="agent-a",
        )
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
    )

    first_registry_snapshot = first.build_registry_checkpoint_snapshot_placeholder()
    second_registry_snapshot = second.build_registry_checkpoint_snapshot_placeholder()
    first_shard_snapshot = first.build_shard_checkpoint_snapshot_placeholder()
    second_shard_snapshot = second.build_shard_checkpoint_snapshot_placeholder()

    assert first_registry_snapshot == second_registry_snapshot
    assert first_shard_snapshot == second_shard_snapshot
    assert first_registry_snapshot["payload_kind"] == "registry_checkpoint_snapshot_placeholder"
    assert first_registry_snapshot["checkpoint_generation"] == 4
    assert first_registry_snapshot["checkpoint_id"] == "registry_checkpoint_placeholder_00000004"
    assert first_registry_snapshot["record_family_counts"] == {
        "accounts": 1,
        "agent_profiles": 1,
        "characters": 1,
        "sessions": 0,
        "system_identities": 1,
    }
    assert first_shard_snapshot["payload_kind"] == "shard_checkpoint_snapshot_placeholder"
    assert first_shard_snapshot["checkpoint_generation"] == 4
    assert first_shard_snapshot["checkpoint_id"] == "shard_checkpoint_placeholder_00000004"
    assert first_shard_snapshot["identity_registry_snapshot"] == first_registry_snapshot
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False
    assert first.checkpoint.checkpoint_commit_marker == "not_persisted"


def test_inprocess_mutation_journal_placeholder_is_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
            system_identity_id="sys-guard",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )

    first_payload = first.build_mutation_journal_placeholder()
    second_payload = second.build_mutation_journal_placeholder()

    assert first_payload == second_payload
    assert first_payload["payload_kind"] == "registry_mutation_journal_placeholder"
    assert first_payload["mutation_journal_mode"] == "in_process_placeholder"
    assert first_payload["last_committed_mutation_generation"] == 5
    assert first_payload["last_committed_mutation_id"] == "mut_00000005"
    assert first_payload["expected_next_mutation_generation"] == 6
    assert first_payload["placeholder_entry_count"] == 5
    assert first_payload["placeholder_entries"] == [
        {
            "mutation_generation": 1,
            "mutation_id": "mut_00000001",
            "record_family": "accounts",
            "record_id": "acct-alice",
            "record_version": 1,
        },
        {
            "mutation_generation": 2,
            "mutation_id": "mut_00000002",
            "record_family": "agent_profiles",
            "record_id": "profile-a",
            "record_version": 1,
        },
        {
            "mutation_generation": 3,
            "mutation_id": "mut_00000003",
            "record_family": "system_identities",
            "record_id": "sys-guard",
            "record_version": 1,
        },
        {
            "mutation_generation": 4,
            "mutation_id": "mut_00000004",
            "record_family": "characters",
            "record_id": "char-guard",
            "record_version": 1,
        },
        {
            "mutation_generation": 5,
            "mutation_id": "mut_00000005",
            "record_family": "sessions",
            "record_id": "sess-guard",
            "record_version": 1,
        },
    ]
    assert first.identity_registry.supports_mutation_journal_persistence is False
    assert first.checkpoint.registry_checkpoint_generation == 0


def test_character_binding_mutations_are_deterministic_in_memory_only() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
        )
        .bind_character_owner(character_id="char-alice", account_id="acct-alice")
        .unbind_character_owner("char-alice")
        .bind_character_profile(character_id="char-alice", agent_profile_id="profile-a")
        .unbind_character_profile("char-alice")
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
        )
        .bind_character_owner(character_id="char-alice", account_id="acct-alice")
        .unbind_character_owner("char-alice")
        .bind_character_profile(character_id="char-alice", agent_profile_id="profile-a")
        .unbind_character_profile("char-alice")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    character = first.get_character("char-alice")
    assert character.owner_account_id is None
    assert character.controller_agent_profile_id is None
    assert character.record_version == 5
    assert character.updated_generation == 9
    assert character.last_mutation_id == "mut_00000009"
    assert first.last_committed_mutation_generation == 9
    assert first.last_committed_mutation_id == "mut_00000009"
    assert first.expected_next_mutation_generation == 10
    assert first.find_characters_by_account("acct-alice") == ()
    assert first.find_characters_by_profile("profile-a") == ()
    assert first.supports_checkpoint_persistence is False
    assert first.supports_mutation_journal_persistence is False


def test_system_identity_binding_mutations_are_deterministic_in_memory_only() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard Controller",
        )
        .register_system_identity(
            system_identity_id="sys-market",
            identity_class="system_agent",
            display_name="Market System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
        .unbind_character_system_identity("char-guard")
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
            display_name="Guard Controller",
        )
        .register_system_identity(
            system_identity_id="sys-market",
            identity_class="system_agent",
            display_name="Market System",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
        .unbind_character_system_identity("char-guard")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    character = first.get_character("char-guard")
    assert character.system_identity_id is None
    assert character.record_version == 3
    assert character.updated_generation == 5
    assert character.last_mutation_id == "mut_00000005"
    assert first.last_committed_mutation_generation == 5
    assert first.last_committed_mutation_id == "mut_00000005"
    assert first.expected_next_mutation_generation == 6
    assert first.find_characters_by_system_identity("sys-guard") == ()
    assert first.supports_checkpoint_persistence is False
    assert first.supports_mutation_journal_persistence is False


def test_character_activation_mutations_are_deterministic_in_memory_only() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .deactivate_character("char-guard")
        .activate_character("char-guard")
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
        .deactivate_character("char-guard")
        .activate_character("char-guard")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    character = first.get_character("char-guard")
    assert character.status == "active"
    assert character.record_version == 3
    assert character.updated_generation == 3
    assert character.last_mutation_id == "mut_00000003"
    assert first.last_committed_mutation_generation == 3
    assert first.last_committed_mutation_id == "mut_00000003"
    assert first.expected_next_mutation_generation == 4
    assert [record.character_id for record in first.find_characters_by_status("active")] == [
        "char-guard"
    ]
    assert first.find_characters_by_status("inactive") == ()
    assert first.supports_checkpoint_persistence is False
    assert first.supports_mutation_journal_persistence is False


def test_principal_activation_mutations_are_deterministic_in_memory_only() -> None:
    first = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .activate_account("acct-alice")
        .activate_agent_profile("profile-a")
        .activate_system_identity("sys-guard")
    )
    second = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .activate_account("acct-alice")
        .activate_agent_profile("profile-a")
        .activate_system_identity("sys-guard")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    assert first.get_account("acct-alice").status == "active"
    assert first.get_account("acct-alice").record_version == 3
    assert first.get_agent_profile("profile-a").status == "active"
    assert first.get_agent_profile("profile-a").record_version == 3
    assert first.get_system_identity("sys-guard").status == "active"
    assert first.get_system_identity("sys-guard").record_version == 3
    assert [record.account_id for record in first.find_accounts_by_status("active")] == ["acct-alice"]
    assert [record.agent_profile_id for record in first.find_agent_profiles_by_status("active")] == [
        "profile-a"
    ]
    assert [
        record.system_identity_id for record in first.find_system_identities_by_status("active")
    ] == ["sys-guard"]
    assert first.last_committed_mutation_generation == 9
    assert first.last_committed_mutation_id == "mut_00000009"
    assert first.expected_next_mutation_generation == 10
    assert first.supports_checkpoint_persistence is False
    assert first.supports_mutation_journal_persistence is False


def test_registry_rejects_duplicate_and_invalid_mutations() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-mock")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-mock",
        )
    )

    with pytest.raises(ValueError, match="account_id already exists"):
        registry.register_account(account_id="acct-alice")

    with pytest.raises(ValueError, match="agent_profile_id already exists"):
        registry.register_agent_profile(agent_profile_id="profile-mock")

    with pytest.raises(ValueError, match="character_id already exists"):
        registry.register_character(character_id="char-alice", identity_class="external_agent")

    attached = registry.attach_session(
        session_id="sess-1",
        character_id="char-alice",
        account_id="acct-alice",
        agent_profile_id="profile-mock",
    )

    with pytest.raises(ValueError, match="session_id already exists"):
        attached.attach_session(
            session_id="sess-1",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-mock",
        )

    with pytest.raises(ValueError, match="already has an active session"):
        attached.attach_session(
            session_id="sess-2",
            character_id="char-alice",
            account_id="acct-alice",
            agent_profile_id="profile-mock",
        )

    detached = attached.detach_session("sess-1")
    with pytest.raises(ValueError, match="already detached"):
        detached.detach_session("sess-1")

    with pytest.raises(ValueError, match="is not active"):
        detached.deactivate_session("sess-1")


def test_character_binding_rejects_conflicting_or_duplicate_mutations() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_character(
            character_id="char-alice",
            identity_class="external_agent",
        )
    )

    owner_bound = registry.bind_character_owner(character_id="char-alice", account_id="acct-alice")
    with pytest.raises(ValueError, match="already bound to owner"):
        owner_bound.bind_character_owner(character_id="char-alice", account_id="acct-alice")
    with pytest.raises(ValueError, match="already bound to a different owner"):
        owner_bound.bind_character_owner(character_id="char-alice", account_id="acct-bob")

    owner_unbound = owner_bound.unbind_character_owner("char-alice")
    with pytest.raises(ValueError, match="does not have an owner account binding"):
        owner_unbound.unbind_character_owner("char-alice")

    profile_bound = owner_unbound.bind_character_profile(
        character_id="char-alice",
        agent_profile_id="profile-a",
    )
    with pytest.raises(ValueError, match="already bound to agent_profile_id"):
        profile_bound.bind_character_profile(character_id="char-alice", agent_profile_id="profile-a")
    with pytest.raises(ValueError, match="already bound to a different controller"):
        profile_bound.bind_character_profile(character_id="char-alice", agent_profile_id="profile-b")

    profile_unbound = profile_bound.unbind_character_profile("char-alice")
    with pytest.raises(ValueError, match="does not have an agent profile binding"):
        profile_unbound.unbind_character_profile("char-alice")


def test_system_identity_binding_rejects_conflicting_or_duplicate_mutations() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_system_identity(
            system_identity_id="sys-market",
            identity_class="system_agent",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
        )
    )

    bound = registry.bind_character_system_identity(
        character_id="char-guard",
        system_identity_id="sys-guard",
    )
    with pytest.raises(ValueError, match="already bound to system_identity_id"):
        bound.bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
    with pytest.raises(ValueError, match="already bound to a different system_identity_id"):
        bound.bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-market",
        )

    unbound = bound.unbind_character_system_identity("char-guard")
    with pytest.raises(ValueError, match="does not have a system identity binding"):
        unbound.unbind_character_system_identity("char-guard")

    with pytest.raises(ValueError, match="system_identity_id already exists"):
        registry.register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )


def test_character_activation_rejects_duplicate_or_invalid_transitions() -> None:
    registry = InProcessShardIdentityRegistry.create_empty("shard-dev").register_character(
        character_id="char-guard",
        identity_class="npc_controller",
    )

    with pytest.raises(ValueError, match="already active"):
        registry.activate_character("char-guard")

    deactivated = registry.deactivate_character("char-guard")
    with pytest.raises(ValueError, match="is not active"):
        deactivated.deactivate_character("char-guard")

    reactivated = deactivated.activate_character("char-guard")
    assert reactivated.get_character("char-guard").status == "active"


def test_principal_activation_rejects_duplicate_or_invalid_transitions() -> None:
    registry = (
        InProcessShardIdentityRegistry.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
    )

    with pytest.raises(ValueError, match="account_id is already active"):
        registry.activate_account("acct-alice")
    with pytest.raises(ValueError, match="agent_profile_id is already active"):
        registry.activate_agent_profile("profile-a")
    with pytest.raises(ValueError, match="system_identity_id is already active"):
        registry.activate_system_identity("sys-guard")

    registry = registry.deactivate_account("acct-alice")
    registry = registry.deactivate_agent_profile("profile-a")
    registry = registry.deactivate_system_identity("sys-guard")

    with pytest.raises(ValueError, match="account_id is not active"):
        registry.deactivate_account("acct-alice")
    with pytest.raises(ValueError, match="agent_profile_id is not active"):
        registry.deactivate_agent_profile("profile-a")
    with pytest.raises(ValueError, match="system_identity_id is not active"):
        registry.deactivate_system_identity("sys-guard")


def test_shard_state_syncs_journal_placeholders_from_updated_registry_in_memory_only() -> None:
    state = ShardState.create_empty("shard-dev")
    updated_registry = (
        state.identity_registry
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-mock")
    )

    updated_state = state.with_identity_registry(updated_registry)

    assert state.journal.last_committed_mutation_generation == 0
    assert state.identity_registry.record_family_counts == {
        "accounts": 0,
        "agent_profiles": 0,
        "characters": 0,
        "sessions": 0,
        "system_identities": 0,
    }
    assert updated_state.journal.last_committed_mutation_generation == 2
    assert updated_state.journal.last_committed_mutation_id == "mut_00000002"
    assert updated_state.journal.expected_next_mutation_generation == 3
    assert updated_state.checkpoint.registry_checkpoint_generation == 0
    assert updated_state.checkpoint.checkpoint_commit_marker == "not_persisted"
    assert updated_state.identity_registry.supports_checkpoint_persistence is False
    assert updated_state.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_syncs_binding_mutations_to_journal_placeholders_in_memory_only() -> None:
    state = ShardState.create_empty("shard-dev")
    updated_registry = (
        state.identity_registry
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(character_id="char-alice", identity_class="external_agent")
        .bind_character_owner(character_id="char-alice", account_id="acct-alice")
        .bind_character_profile(character_id="char-alice", agent_profile_id="profile-a")
    )

    updated_state = state.with_identity_registry(updated_registry)

    assert updated_state.journal.last_committed_mutation_generation == 5
    assert updated_state.journal.last_committed_mutation_id == "mut_00000005"
    assert updated_state.journal.expected_next_mutation_generation == 6
    assert updated_state.checkpoint.registry_checkpoint_generation == 0
    assert updated_state.checkpoint.snapshot_payload_hash is None


def test_shard_state_syncs_system_identity_binding_mutations_to_journal_placeholders_in_memory_only() -> None:
    state = ShardState.create_empty("shard-dev")
    updated_registry = (
        state.identity_registry
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .register_character(character_id="char-guard", identity_class="npc_controller")
        .bind_character_system_identity(character_id="char-guard", system_identity_id="sys-guard")
    )

    updated_state = state.with_identity_registry(updated_registry)

    assert updated_state.journal.last_committed_mutation_generation == 3
    assert updated_state.journal.last_committed_mutation_id == "mut_00000003"
    assert updated_state.journal.expected_next_mutation_generation == 4
    assert updated_state.checkpoint.registry_checkpoint_generation == 0
    assert updated_state.identity_registry.find_characters_by_system_identity("sys-guard")[0].character_id == "char-guard"


def test_shard_state_syncs_character_activation_mutations_to_journal_placeholders_in_memory_only() -> None:
    state = ShardState.create_empty("shard-dev")
    updated_registry = (
        state.identity_registry
        .register_character(character_id="char-guard", identity_class="npc_controller")
        .deactivate_character("char-guard")
    )

    updated_state = state.with_identity_registry(updated_registry)

    assert updated_state.journal.last_committed_mutation_generation == 2
    assert updated_state.journal.last_committed_mutation_id == "mut_00000002"
    assert updated_state.journal.expected_next_mutation_generation == 3
    assert [record.character_id for record in updated_state.identity_registry.find_characters_by_status("inactive")] == [
        "char-guard"
    ]
    assert updated_state.checkpoint.registry_checkpoint_generation == 0


def test_shard_state_syncs_principal_activation_mutations_to_journal_placeholders_in_memory_only() -> None:
    state = ShardState.create_empty("shard-dev")
    updated_registry = (
        state.identity_registry
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(system_identity_id="sys-guard", identity_class="npc_controller")
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
    )

    updated_state = state.with_identity_registry(updated_registry)

    assert updated_state.journal.last_committed_mutation_generation == 5
    assert updated_state.journal.last_committed_mutation_id == "mut_00000005"
    assert updated_state.journal.expected_next_mutation_generation == 6
    assert [record.account_id for record in updated_state.identity_registry.find_accounts_by_status("inactive")] == [
        "acct-alice"
    ]
    assert [
        record.agent_profile_id
        for record in updated_state.identity_registry.find_agent_profiles_by_status("inactive")
    ] == ["profile-a"]
    assert updated_state.checkpoint.registry_checkpoint_generation == 0


def test_shard_state_open_and_close_character_session_are_deterministic() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            status="inactive",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
        .open_character_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            presence_id="presence-guard",
        )
        .close_character_session("sess-guard")
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            status="inactive",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
        .open_character_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
            presence_id="presence-guard",
        )
        .close_character_session("sess-guard")
    )

    assert first == second
    assert first.to_dict() == second.to_dict()
    session = first.get_session("sess-guard")
    character = first.get_character("char-guard")
    assert session.status == "inactive"
    assert session.record_version == 2
    assert session.detached_generation == 10
    assert character.status == "inactive"
    assert character.record_version == 6
    assert first.journal.last_committed_mutation_generation == 11
    assert first.journal.last_committed_mutation_id == "mut_00000011"
    assert first.journal.expected_next_mutation_generation == 12
    assert first.checkpoint.registry_checkpoint_generation == 0
    assert first.identity_registry.supports_checkpoint_persistence is False
    assert first.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_open_character_session_respects_enforced_reconciliation_policy() -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-guard",
            identity_class="npc_controller",
            status="inactive",
        )
        .bind_character_owner(character_id="char-guard", account_id="acct-alice")
        .bind_character_profile(character_id="char-guard", agent_profile_id="profile-a")
        .bind_character_system_identity(
            character_id="char-guard",
            system_identity_id="sys-guard",
        )
        .deactivate_account("acct-alice")
        .deactivate_agent_profile("profile-a")
        .deactivate_system_identity("sys-guard")
        .with_session_principal_reconciliation_enforced()
    )

    with pytest.raises(
        ValueError,
        match=(
            "attach_session reconciliation failed for sess-guard: "
            "inactive_account,inactive_agent_profile,inactive_system_identity"
        ),
    ):
        state.open_character_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )


def test_shard_state_export_payload_is_deterministic_and_observer_friendly() -> None:
    first = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-live",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .register_character(
            character_id="char-closed",
            identity_class="external_agent",
            owner_account_id="acct-bob",
            controller_agent_profile_id="profile-b",
        )
        .register_character(
            character_id="char-npc",
            identity_class="npc_controller",
            system_identity_id="sys-guard",
            status="inactive",
        )
        .attach_session(
            session_id="sess-live",
            character_id="char-live",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .open_character_session(
            session_id="sess-closed",
            character_id="char-closed",
            account_id="acct-bob",
            agent_profile_id="profile-b",
        )
        .close_character_session("sess-closed")
    )
    second = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_account(account_id="acct-bob")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_agent_profile(agent_profile_id="profile-b")
        .register_system_identity(
            system_identity_id="sys-guard",
            identity_class="npc_controller",
        )
        .register_character(
            character_id="char-live",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .register_character(
            character_id="char-closed",
            identity_class="external_agent",
            owner_account_id="acct-bob",
            controller_agent_profile_id="profile-b",
        )
        .register_character(
            character_id="char-npc",
            identity_class="npc_controller",
            system_identity_id="sys-guard",
            status="inactive",
        )
        .attach_session(
            session_id="sess-live",
            character_id="char-live",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
        .open_character_session(
            session_id="sess-closed",
            character_id="char-closed",
            account_id="acct-bob",
            agent_profile_id="profile-b",
        )
        .close_character_session("sess-closed")
    )

    first_payload = first.build_shard_export_payload()
    second_payload = second.build_shard_export_payload()

    assert first_payload == second_payload
    assert first_payload["payload_kind"] == "shard_export_placeholder"
    assert first_payload["payload_version"] == 1
    assert first_payload["shard_id"] == "shard-dev"
    assert first_payload["record_family_counts"] == {
        "accounts": 2,
        "agent_profiles": 2,
        "characters": 3,
        "sessions": 2,
        "system_identities": 1,
    }
    assert first_payload["lifecycle_counts"] == {
        "characters": {
            "active": 1,
            "inactive": 2,
            "total": 3,
        },
        "sessions": {
            "active": 1,
            "inactive": 1,
            "total": 2,
        },
    }
    assert first_payload["identity_category_counts"] == {
        "accounts_by_type": {"human_player": 2},
        "agent_profiles_by_controller_type": {"external_agent": 2},
        "characters_by_identity_class": {
            "external_agent": 2,
            "npc_controller": 1,
        },
        "system_identities_by_class": {"npc_controller": 1},
    }
    assert first_payload["checkpoint"]["registry_checkpoint_generation"] == 0
    assert (
        first_payload["registry_checkpoint_snapshot_placeholder"]["checkpoint_generation"]
        == 12
    )
    assert (
        first_payload["journal"]["last_committed_mutation_generation"]
        == first_payload["mutation_journal_placeholder"]["last_committed_mutation_generation"]
        == 12
    )
    assert (
        first_payload["journal"]["expected_next_mutation_generation"]
        == first_payload["mutation_journal_placeholder"]["expected_next_mutation_generation"]
        == 13
    )
    assert (
        first_payload["shard_checkpoint_snapshot_placeholder"]["checkpoint_generation"]
        == 12
    )
    assert first_payload["supports_checkpoint_persistence"] is False
    assert first_payload["supports_mutation_journal_persistence"] is False
    assert "bundle_verification_summary" not in first_payload
    assert first.checkpoint.registry_checkpoint_generation == 0
    assert first.journal.last_committed_mutation_generation == 12


def test_shard_state_export_payload_empty_counts_are_stable_without_persistence_side_effects() -> None:
    state = ShardState.create_empty("shard-dev")

    payload = state.build_shard_export_payload()

    assert payload["record_family_counts"] == {
        "accounts": 0,
        "agent_profiles": 0,
        "characters": 0,
        "sessions": 0,
        "system_identities": 0,
    }
    assert payload["lifecycle_counts"] == {
        "characters": {"active": 0, "inactive": 0, "total": 0},
        "sessions": {"active": 0, "inactive": 0, "total": 0},
    }
    assert payload["identity_category_counts"] == {
        "accounts_by_type": {},
        "agent_profiles_by_controller_type": {},
        "characters_by_identity_class": {},
        "system_identities_by_class": {},
    }
    assert payload["checkpoint"] == state.checkpoint.to_dict()
    assert payload["journal"] == state.journal.to_dict()
    assert payload["mutation_policy"] == state.mutation_policy.to_dict()
    assert payload["supports_checkpoint_persistence"] is False
    assert payload["supports_mutation_journal_persistence"] is False
    assert "bundle_verification_summary" not in payload


def test_shard_state_checkpoint_writer_is_deterministic_and_side_effect_free(tmp_path) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-guard",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    before = state.to_dict()
    first_path = tmp_path / "checkpoint-a.json"
    second_path = tmp_path / "checkpoint-b.json"

    written_first = state.write_shard_checkpoint_placeholder(first_path)
    written_second = state.write_shard_checkpoint_placeholder(second_path)

    expected_payload = state.build_shard_checkpoint_snapshot_placeholder()
    expected_text = json.dumps(expected_payload, indent=2, sort_keys=True) + "\n"

    assert written_first == str(first_path)
    assert written_second == str(second_path)
    assert first_path.read_text(encoding="utf-8") == expected_text
    assert second_path.read_text(encoding="utf-8") == expected_text
    assert state.to_dict() == before
    assert state.checkpoint.registry_checkpoint_generation == 0
    assert state.identity_registry.supports_checkpoint_persistence is False
    assert state.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_checkpoint_writer_rejects_invalid_paths_deterministically(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev")
    missing_parent_path = tmp_path / "missing" / "checkpoint.json"
    directory_path = tmp_path / "checkpoint-dir"
    directory_path.mkdir()
    invalid_parent = tmp_path / "not-a-dir"
    invalid_parent.write_text("blocked", encoding="utf-8")
    child_of_file = invalid_parent / "checkpoint.json"

    with pytest.raises(
        ValueError,
        match="shard checkpoint placeholder parent directory does not exist",
    ):
        state.write_shard_checkpoint_placeholder(missing_parent_path)

    with pytest.raises(
        ValueError,
        match="shard checkpoint placeholder path must be a file path, not directory",
    ):
        state.write_shard_checkpoint_placeholder(directory_path)

    with pytest.raises(
        ValueError,
        match="shard checkpoint placeholder parent path is not a directory",
    ):
        state.write_shard_checkpoint_placeholder(child_of_file)


def test_shard_state_companion_writers_are_deterministic_and_side_effect_free(tmp_path) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-guard",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    before = state.to_dict()
    registry_first_path = tmp_path / "registry-a.json"
    registry_second_path = tmp_path / "registry-b.json"
    journal_first_path = tmp_path / "journal-a.json"
    journal_second_path = tmp_path / "journal-b.json"
    export_first_path = tmp_path / "export-a.json"
    export_second_path = tmp_path / "export-b.json"

    written_registry_first = state.identity_registry.write_registry_checkpoint_placeholder(
        registry_first_path
    )
    written_registry_second = state.identity_registry.write_registry_checkpoint_placeholder(
        registry_second_path
    )
    written_journal_first = state.write_mutation_journal_placeholder(journal_first_path)
    written_journal_second = state.write_mutation_journal_placeholder(journal_second_path)
    written_export_first = state.write_shard_export_payload(export_first_path)
    written_export_second = state.write_shard_export_payload(export_second_path)

    expected_registry_text = json.dumps(
        state.build_registry_checkpoint_snapshot_placeholder(),
        indent=2,
        sort_keys=True,
    ) + "\n"
    expected_journal_text = json.dumps(
        state.build_mutation_journal_placeholder(),
        indent=2,
        sort_keys=True,
    ) + "\n"
    expected_export_text = json.dumps(
        state.build_shard_export_payload(),
        indent=2,
        sort_keys=True,
    ) + "\n"

    assert written_registry_first == str(registry_first_path)
    assert written_registry_second == str(registry_second_path)
    assert written_journal_first == str(journal_first_path)
    assert written_journal_second == str(journal_second_path)
    assert written_export_first == str(export_first_path)
    assert written_export_second == str(export_second_path)
    assert registry_first_path.read_text(encoding="utf-8") == expected_registry_text
    assert registry_second_path.read_text(encoding="utf-8") == expected_registry_text
    assert journal_first_path.read_text(encoding="utf-8") == expected_journal_text
    assert journal_second_path.read_text(encoding="utf-8") == expected_journal_text
    assert export_first_path.read_text(encoding="utf-8") == expected_export_text
    assert export_second_path.read_text(encoding="utf-8") == expected_export_text
    assert state.to_dict() == before
    assert state.identity_registry.to_dict() == before["identity_registry"]
    assert state.checkpoint.registry_checkpoint_generation == 0
    assert state.identity_registry.supports_checkpoint_persistence is False
    assert state.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_companion_writers_reject_invalid_paths_deterministically(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev")
    missing_parent_registry = tmp_path / "missing-registry" / "registry.json"
    missing_parent_journal = tmp_path / "missing-journal" / "journal.json"
    missing_parent_export = tmp_path / "missing-export" / "export.json"
    directory_registry = tmp_path / "registry-dir"
    directory_journal = tmp_path / "journal-dir"
    directory_export = tmp_path / "export-dir"
    directory_registry.mkdir()
    directory_journal.mkdir()
    directory_export.mkdir()
    blocked_parent = tmp_path / "blocked-parent"
    blocked_parent.write_text("blocked", encoding="utf-8")

    with pytest.raises(
        ValueError,
        match="registry checkpoint placeholder parent directory does not exist",
    ):
        state.identity_registry.write_registry_checkpoint_placeholder(missing_parent_registry)

    with pytest.raises(
        ValueError,
        match="registry checkpoint placeholder path must be a file path, not directory",
    ):
        state.identity_registry.write_registry_checkpoint_placeholder(directory_registry)

    with pytest.raises(
        ValueError,
        match="registry checkpoint placeholder parent path is not a directory",
    ):
        state.identity_registry.write_registry_checkpoint_placeholder(
            blocked_parent / "registry.json"
        )

    with pytest.raises(
        ValueError,
        match="mutation journal placeholder parent directory does not exist",
    ):
        state.write_mutation_journal_placeholder(missing_parent_journal)

    with pytest.raises(
        ValueError,
        match="mutation journal placeholder path must be a file path, not directory",
    ):
        state.write_mutation_journal_placeholder(directory_journal)

    with pytest.raises(
        ValueError,
        match="mutation journal placeholder parent path is not a directory",
    ):
        state.write_mutation_journal_placeholder(blocked_parent / "journal.json")

    with pytest.raises(
        ValueError,
        match="shard export payload parent directory does not exist",
    ):
        state.write_shard_export_payload(missing_parent_export)

    with pytest.raises(
        ValueError,
        match="shard export payload path must be a file path, not directory",
    ):
        state.write_shard_export_payload(directory_export)

    with pytest.raises(
        ValueError,
        match="shard export payload parent path is not a directory",
    ):
        state.write_shard_export_payload(blocked_parent / "export.json")


def test_shard_state_artifact_bundle_writer_is_deterministic_and_side_effect_free(tmp_path) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-guard",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    before = state.to_dict()
    first_dir = tmp_path / "bundle-a"
    second_dir = tmp_path / "bundle-b"
    first_dir.mkdir()
    second_dir.mkdir()

    first_bundle = state.write_shard_artifact_bundle(first_dir)
    second_bundle = state.write_shard_artifact_bundle(second_dir)

    assert first_bundle == {
        "bundle_directory": str(first_dir),
        "shard_checkpoint_placeholder": str(first_dir / "shard-dev.shard_checkpoint_placeholder.json"),
        "registry_checkpoint_placeholder": str(first_dir / "shard-dev.registry_checkpoint_placeholder.json"),
        "mutation_journal_placeholder": str(first_dir / "shard-dev.mutation_journal_placeholder.json"),
        "shard_export_payload": str(first_dir / "shard-dev.shard_export_placeholder.json"),
        "artifact_index": str(first_dir / "shard-dev.artifact_index.json"),
        "artifact_manifest": str(first_dir / "shard-dev.artifact_manifest.json"),
    }
    assert second_bundle == {
        "bundle_directory": str(second_dir),
        "shard_checkpoint_placeholder": str(second_dir / "shard-dev.shard_checkpoint_placeholder.json"),
        "registry_checkpoint_placeholder": str(second_dir / "shard-dev.registry_checkpoint_placeholder.json"),
        "mutation_journal_placeholder": str(second_dir / "shard-dev.mutation_journal_placeholder.json"),
        "shard_export_payload": str(second_dir / "shard-dev.shard_export_placeholder.json"),
        "artifact_index": str(second_dir / "shard-dev.artifact_index.json"),
        "artifact_manifest": str(second_dir / "shard-dev.artifact_manifest.json"),
    }
    assert sorted(path.name for path in first_dir.iterdir()) == [
        "shard-dev.artifact_index.json",
        "shard-dev.artifact_manifest.json",
        "shard-dev.mutation_journal_placeholder.json",
        "shard-dev.registry_checkpoint_placeholder.json",
        "shard-dev.shard_checkpoint_placeholder.json",
        "shard-dev.shard_export_placeholder.json",
    ]
    assert sorted(path.name for path in second_dir.iterdir()) == [
        "shard-dev.artifact_index.json",
        "shard-dev.artifact_manifest.json",
        "shard-dev.mutation_journal_placeholder.json",
        "shard-dev.registry_checkpoint_placeholder.json",
        "shard-dev.shard_checkpoint_placeholder.json",
        "shard-dev.shard_export_placeholder.json",
    ]
    assert (
        (first_dir / "shard-dev.shard_checkpoint_placeholder.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.shard_checkpoint_placeholder.json").read_text(encoding="utf-8")
    )
    assert (
        (first_dir / "shard-dev.registry_checkpoint_placeholder.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.registry_checkpoint_placeholder.json").read_text(encoding="utf-8")
    )
    assert (
        (first_dir / "shard-dev.mutation_journal_placeholder.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.mutation_journal_placeholder.json").read_text(encoding="utf-8")
    )
    assert (
        (first_dir / "shard-dev.shard_export_placeholder.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.shard_export_placeholder.json").read_text(encoding="utf-8")
    )
    assert (
        (first_dir / "shard-dev.artifact_index.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.artifact_index.json").read_text(encoding="utf-8")
    )
    assert (
        (first_dir / "shard-dev.artifact_manifest.json").read_text(encoding="utf-8")
        == (second_dir / "shard-dev.artifact_manifest.json").read_text(encoding="utf-8")
    )
    index_payload = json.loads(
        (first_dir / "shard-dev.artifact_index.json").read_text(encoding="utf-8")
    )
    manifest_payload = json.loads(
        (first_dir / "shard-dev.artifact_manifest.json").read_text(encoding="utf-8")
    )
    shard_checkpoint_hash = hashlib.sha256(
        (first_dir / "shard-dev.shard_checkpoint_placeholder.json").read_bytes()
    ).hexdigest()
    registry_checkpoint_hash = hashlib.sha256(
        (first_dir / "shard-dev.registry_checkpoint_placeholder.json").read_bytes()
    ).hexdigest()
    mutation_journal_hash = hashlib.sha256(
        (first_dir / "shard-dev.mutation_journal_placeholder.json").read_bytes()
    ).hexdigest()
    shard_export_hash = hashlib.sha256(
        (first_dir / "shard-dev.shard_export_placeholder.json").read_bytes()
    ).hexdigest()
    assert index_payload == {
        "payload_kind": "shard_artifact_bundle_index_placeholder",
        "payload_version": 1,
        "shard_id": "shard-dev",
        "mode_marker": "persistent_world_shell",
        "bundle_generation": 4,
        "artifacts": [
            {
                "artifact_type": "shard_checkpoint_placeholder",
                "filename": "shard-dev.shard_checkpoint_placeholder.json",
                "payload_kind": "shard_checkpoint_snapshot_placeholder",
                "payload_version": 1,
                "generation": 4,
                "content_hash_sha256": shard_checkpoint_hash,
            },
            {
                "artifact_type": "registry_checkpoint_placeholder",
                "filename": "shard-dev.registry_checkpoint_placeholder.json",
                "payload_kind": "registry_checkpoint_snapshot_placeholder",
                "payload_version": 1,
                "generation": 4,
                "content_hash_sha256": registry_checkpoint_hash,
            },
            {
                "artifact_type": "mutation_journal_placeholder",
                "filename": "shard-dev.mutation_journal_placeholder.json",
                "payload_kind": "registry_mutation_journal_placeholder",
                "payload_version": 1,
                "generation": 4,
                "expected_next_generation": 5,
                "content_hash_sha256": mutation_journal_hash,
            },
            {
                "artifact_type": "shard_export_payload",
                "filename": "shard-dev.shard_export_placeholder.json",
                "payload_kind": "shard_export_placeholder",
                "payload_version": 1,
                "generation": 4,
                "content_hash_sha256": shard_export_hash,
            },
        ],
    }
    assert manifest_payload == {
        "payload_kind": "shard_artifact_bundle_manifest_placeholder",
        "payload_version": 1,
        "manifest_family": "shard_artifact_bundle_manifest_v1",
        "shard_id": "shard-dev",
        "mode_marker": "persistent_world_shell",
        "bundle_generation": 4,
        "expected_next_generation": 5,
        "index_filename": "shard-dev.artifact_index.json",
        "artifact_filenames": {
            "shard_checkpoint_placeholder": "shard-dev.shard_checkpoint_placeholder.json",
            "registry_checkpoint_placeholder": "shard-dev.registry_checkpoint_placeholder.json",
            "mutation_journal_placeholder": "shard-dev.mutation_journal_placeholder.json",
            "shard_export_payload": "shard-dev.shard_export_placeholder.json",
        },
        "linkage": {
            "shard_checkpoint_generation": 4,
            "registry_checkpoint_generation": 4,
            "mutation_journal_generation": 4,
            "mutation_journal_expected_next_generation": 5,
            "shard_export_generation": 4,
        },
    }
    assert state.to_dict() == before
    assert state.checkpoint.registry_checkpoint_generation == 0
    assert state.identity_registry.supports_checkpoint_persistence is False
    assert state.identity_registry.supports_mutation_journal_persistence is False


def test_shard_state_artifact_bundle_writer_rejects_invalid_targets_deterministically(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev")
    missing_directory = tmp_path / "missing-bundle"
    file_target = tmp_path / "bundle-file"
    file_target.write_text("blocked", encoding="utf-8")

    with pytest.raises(
        ValueError,
        match="artifact bundle directory does not exist",
    ):
        state.write_shard_artifact_bundle(missing_directory)

    with pytest.raises(
        ValueError,
        match="artifact bundle path is not a directory",
    ):
        state.write_shard_artifact_bundle(file_target)


def test_shard_state_bundle_verification_helper_accepts_valid_bundle(tmp_path) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-guard",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    result = state.verify_shard_artifact_bundle(bundle_dir)

    assert result == {
        "payload_kind": "shard_artifact_bundle_verification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "index_filename": "shard-dev.artifact_index.json",
        "manifest_filename": "shard-dev.artifact_manifest.json",
        "shard_id": "shard-dev",
        "is_valid": True,
        "issue_codes": [],
        "artifacts": [
            {
                "artifact_type": "shard_checkpoint_placeholder",
                "expected_filename": "shard-dev.shard_checkpoint_placeholder.json",
                "recorded_filename": "shard-dev.shard_checkpoint_placeholder.json",
                "exists": True,
                "hash_matches": True,
                "recorded_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.shard_checkpoint_placeholder.json").read_bytes()
                ).hexdigest(),
                "actual_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.shard_checkpoint_placeholder.json").read_bytes()
                ).hexdigest(),
                "issue_codes": [],
            },
            {
                "artifact_type": "registry_checkpoint_placeholder",
                "expected_filename": "shard-dev.registry_checkpoint_placeholder.json",
                "recorded_filename": "shard-dev.registry_checkpoint_placeholder.json",
                "exists": True,
                "hash_matches": True,
                "recorded_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.registry_checkpoint_placeholder.json").read_bytes()
                ).hexdigest(),
                "actual_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.registry_checkpoint_placeholder.json").read_bytes()
                ).hexdigest(),
                "issue_codes": [],
            },
            {
                "artifact_type": "mutation_journal_placeholder",
                "expected_filename": "shard-dev.mutation_journal_placeholder.json",
                "recorded_filename": "shard-dev.mutation_journal_placeholder.json",
                "exists": True,
                "hash_matches": True,
                "recorded_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.mutation_journal_placeholder.json").read_bytes()
                ).hexdigest(),
                "actual_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.mutation_journal_placeholder.json").read_bytes()
                ).hexdigest(),
                "issue_codes": [],
            },
            {
                "artifact_type": "shard_export_payload",
                "expected_filename": "shard-dev.shard_export_placeholder.json",
                "recorded_filename": "shard-dev.shard_export_placeholder.json",
                "exists": True,
                "hash_matches": True,
                "recorded_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.shard_export_placeholder.json").read_bytes()
                ).hexdigest(),
                "actual_hash_sha256": hashlib.sha256(
                    (bundle_dir / "shard-dev.shard_export_placeholder.json").read_bytes()
                ).hexdigest(),
                "issue_codes": [],
            },
        ],
    }
    assert state.to_dict() == before


def test_shard_state_bundle_verification_helper_detects_missing_artifact_file(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev")
    bundle_dir = tmp_path / "bundle-missing"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)
    (bundle_dir / "shard-dev.shard_export_placeholder.json").unlink()

    result = state.verify_shard_artifact_bundle(bundle_dir)

    assert result["is_valid"] is False
    assert result["issue_codes"] == ["missing_artifact_file"]
    shard_export_result = result["artifacts"][3]
    assert shard_export_result["artifact_type"] == "shard_export_payload"
    assert shard_export_result["exists"] is False
    assert shard_export_result["issue_codes"] == ["missing_artifact_file"]


def test_shard_state_bundle_verification_helper_detects_hash_mismatch(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev")
    bundle_dir = tmp_path / "bundle-mismatch"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)
    shard_export_path = bundle_dir / "shard-dev.shard_export_placeholder.json"
    shard_export_path.write_text("{\"tampered\":true}\n", encoding="utf-8")

    result = state.verify_shard_artifact_bundle(bundle_dir)

    assert result["is_valid"] is False
    assert result["issue_codes"] == ["content_hash_mismatch"]
    shard_export_result = result["artifacts"][3]
    assert shard_export_result["artifact_type"] == "shard_export_payload"
    assert shard_export_result["exists"] is True
    assert shard_export_result["hash_matches"] is False
    assert shard_export_result["issue_codes"] == ["content_hash_mismatch"]


def test_shard_state_bundle_verification_helper_detects_manifest_linkage_mismatch(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-manifest-mismatch"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    manifest_path = bundle_dir / "shard-dev.artifact_manifest.json"
    manifest_payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_payload["linkage"]["shard_export_generation"] = 99
    manifest_path.write_text(
        json.dumps(manifest_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    result = state.verify_shard_artifact_bundle(bundle_dir)

    assert result["is_valid"] is False
    assert result["issue_codes"] == ["manifest_linkage_mismatch"]
    assert result["manifest_filename"] == "shard-dev.artifact_manifest.json"
    assert state.to_dict() == before


def test_shard_state_bundle_verification_helper_detects_generation_linkage_mismatch(
    tmp_path,
) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
    )
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-linkage-mismatch"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    shard_export_path = bundle_dir / "shard-dev.shard_export_placeholder.json"
    shard_export_payload = json.loads(shard_export_path.read_text(encoding="utf-8"))
    shard_export_payload["journal"]["last_committed_mutation_generation"] = 99
    shard_export_payload["journal"]["expected_next_mutation_generation"] = 100
    shard_export_path.write_text(
        json.dumps(shard_export_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    shard_export_hash = hashlib.sha256(shard_export_path.read_bytes()).hexdigest()

    index_path = bundle_dir / "shard-dev.artifact_index.json"
    index_payload = json.loads(index_path.read_text(encoding="utf-8"))
    for artifact in index_payload["artifacts"]:
        if artifact["artifact_type"] == "shard_export_payload":
            artifact["generation"] = 99
            artifact["content_hash_sha256"] = shard_export_hash
            break
    index_path.write_text(
        json.dumps(index_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    result = state.verify_shard_artifact_bundle(bundle_dir)

    assert result["is_valid"] is False
    assert result["issue_codes"] == ["shard_export_generation_mismatch"]
    shard_export_result = result["artifacts"][3]
    assert shard_export_result["artifact_type"] == "shard_export_payload"
    assert shard_export_result["exists"] is True
    assert shard_export_result["hash_matches"] is True
    assert shard_export_result["issue_codes"] == ["shard_export_generation_mismatch"]
    assert state.to_dict() == before


def test_shard_state_bundle_recovery_helper_classifies_valid_bundle_as_resumable(tmp_path) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-valid"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    result = state.classify_shard_bundle_recovery_state(bundle_dir)

    assert result == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "resumable",
        "issue_codes": [],
        "fatal_issue_codes": [],
        "degradable_issue_codes": [],
        "verification_is_valid": True,
    }
    assert state.to_dict() == before


def test_shard_state_bundle_recovery_helper_classifies_filename_mismatch_as_degraded(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-degraded"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    index_path = bundle_dir / "shard-dev.artifact_index.json"
    index_payload = json.loads(index_path.read_text(encoding="utf-8"))
    for artifact in index_payload["artifacts"]:
        if artifact["artifact_type"] == "shard_export_payload":
            artifact["filename"] = "unexpected-shard-export.json"
            break
    index_path.write_text(
        json.dumps(index_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    result = state.classify_shard_bundle_recovery_state(bundle_dir)

    assert result == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "degraded",
        "issue_codes": ["filename_mismatch"],
        "fatal_issue_codes": [],
        "degradable_issue_codes": ["filename_mismatch"],
        "verification_is_valid": False,
    }
    assert state.to_dict() == before


def test_shard_state_bundle_recovery_helper_classifies_manifest_mismatch_as_non_resumable(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-nonresumable"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    manifest_path = bundle_dir / "shard-dev.artifact_manifest.json"
    manifest_payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_payload["linkage"]["shard_export_generation"] = 99
    manifest_path.write_text(
        json.dumps(manifest_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    result = state.classify_shard_bundle_recovery_state(bundle_dir)

    assert result == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "non_resumable",
        "issue_codes": ["manifest_linkage_mismatch"],
        "fatal_issue_codes": ["manifest_linkage_mismatch"],
        "degradable_issue_codes": [],
        "verification_is_valid": False,
    }
    assert state.to_dict() == before


def test_shard_state_export_payload_can_include_valid_bundle_verification_summary(tmp_path) -> None:
    state = (
        ShardState.create_empty("shard-dev")
        .register_account(account_id="acct-alice")
        .register_agent_profile(agent_profile_id="profile-a")
        .register_character(
            character_id="char-guard",
            identity_class="external_agent",
            owner_account_id="acct-alice",
            controller_agent_profile_id="profile-a",
        )
        .attach_session(
            session_id="sess-guard",
            character_id="char-guard",
            account_id="acct-alice",
            agent_profile_id="profile-a",
        )
    )
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-valid-summary"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    summary = state.build_bundle_verification_summary(bundle_dir)
    payload = state.build_shard_export_payload(bundle_verification_summary=summary)

    assert summary == {
        "is_valid": True,
        "issue_codes": [],
        "artifact_count": 4,
        "verified_artifact_count": 4,
        "hash_issue_codes": [],
        "linkage_issue_codes": [],
        "manifest_issue_codes": [],
        "issue_category_counts": {
            "hash": 0,
            "linkage": 0,
            "manifest": 0,
            "other": 0,
        },
    }
    assert payload["bundle_verification_summary"] == summary
    assert state.to_dict() == before


def test_shard_state_export_payload_can_include_resumable_bundle_recovery_summary(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-summary-valid"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    summary = state.classify_shard_bundle_recovery_state(bundle_dir)
    payload = state.build_shard_export_payload(bundle_recovery_summary=summary)

    assert summary == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "resumable",
        "issue_codes": [],
        "fatal_issue_codes": [],
        "degradable_issue_codes": [],
        "verification_is_valid": True,
    }
    assert payload["bundle_recovery_summary"] == {
        "recovery_state": "resumable",
        "issue_codes": [],
        "fatal_issue_codes": [],
        "degradable_issue_codes": [],
        "verification_is_valid": True,
    }
    assert state.to_dict() == before


def test_shard_state_export_payload_can_include_degraded_bundle_recovery_summary(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-summary-degraded"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    index_path = bundle_dir / "shard-dev.artifact_index.json"
    index_payload = json.loads(index_path.read_text(encoding="utf-8"))
    for artifact in index_payload["artifacts"]:
        if artifact["artifact_type"] == "shard_export_payload":
            artifact["filename"] = "unexpected-shard-export.json"
            break
    index_path.write_text(
        json.dumps(index_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    summary = state.classify_shard_bundle_recovery_state(bundle_dir)
    payload = state.build_shard_export_payload(bundle_recovery_summary=summary)

    assert summary == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "degraded",
        "issue_codes": ["filename_mismatch"],
        "fatal_issue_codes": [],
        "degradable_issue_codes": ["filename_mismatch"],
        "verification_is_valid": False,
    }
    assert payload["bundle_recovery_summary"] == {
        "recovery_state": "degraded",
        "issue_codes": ["filename_mismatch"],
        "fatal_issue_codes": [],
        "degradable_issue_codes": ["filename_mismatch"],
        "verification_is_valid": False,
    }
    assert state.to_dict() == before


def test_shard_state_export_payload_can_include_non_resumable_bundle_recovery_summary(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-recovery-summary-nonresumable"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    manifest_path = bundle_dir / "shard-dev.artifact_manifest.json"
    manifest_payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_payload["linkage"]["shard_export_generation"] = 99
    manifest_path.write_text(
        json.dumps(manifest_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    summary = state.classify_shard_bundle_recovery_state(bundle_dir)
    payload = state.build_shard_export_payload(bundle_recovery_summary=summary)

    assert summary == {
        "payload_kind": "shard_bundle_recovery_classification_placeholder",
        "payload_version": 1,
        "bundle_directory": str(bundle_dir),
        "shard_id": "shard-dev",
        "recovery_state": "non_resumable",
        "issue_codes": ["manifest_linkage_mismatch"],
        "fatal_issue_codes": ["manifest_linkage_mismatch"],
        "degradable_issue_codes": [],
        "verification_is_valid": False,
    }
    assert payload["bundle_recovery_summary"] == {
        "recovery_state": "non_resumable",
        "issue_codes": ["manifest_linkage_mismatch"],
        "fatal_issue_codes": ["manifest_linkage_mismatch"],
        "degradable_issue_codes": [],
        "verification_is_valid": False,
    }
    assert state.to_dict() == before


def test_shard_state_export_payload_can_include_manifest_issue_bundle_verification_summary(
    tmp_path,
) -> None:
    state = ShardState.create_empty("shard-dev").register_account(account_id="acct-alice")
    before = state.to_dict()
    bundle_dir = tmp_path / "bundle-invalid-summary"
    bundle_dir.mkdir()
    state.write_shard_artifact_bundle(bundle_dir)

    manifest_path = bundle_dir / "shard-dev.artifact_manifest.json"
    manifest_payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest_payload["linkage"]["shard_export_generation"] = 99
    manifest_path.write_text(
        json.dumps(manifest_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    summary = state.build_bundle_verification_summary(bundle_dir)
    payload = state.build_shard_export_payload(bundle_verification_summary=summary)

    assert summary == {
        "is_valid": False,
        "issue_codes": ["manifest_linkage_mismatch"],
        "artifact_count": 4,
        "verified_artifact_count": 4,
        "hash_issue_codes": [],
        "linkage_issue_codes": [],
        "manifest_issue_codes": ["manifest_linkage_mismatch"],
        "issue_category_counts": {
            "hash": 0,
            "linkage": 0,
            "manifest": 1,
            "other": 0,
        },
    }
    assert payload["bundle_verification_summary"] == summary
    assert state.to_dict() == before


# ---------------------------------------------------------------------------
# Timing consequence helper
# ---------------------------------------------------------------------------

def test_has_cadence_efficiency_consequence_detects_flag() -> None:
    """_has_cadence_efficiency_consequence returns True iff timing_consequence matches."""
    from evaluation.benchmark_runner.runner import _has_cadence_efficiency_consequence

    assert _has_cadence_efficiency_consequence({"timing_consequence": "cadence_efficiency"}) is True
    assert _has_cadence_efficiency_consequence({}) is False
    assert _has_cadence_efficiency_consequence({"timing_consequence": "other"}) is False
    assert _has_cadence_efficiency_consequence({"timing_consequence": ""}) is False
    assert _has_cadence_efficiency_consequence({"timing_consequence": None}) is False
