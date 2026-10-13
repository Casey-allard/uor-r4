"""Minimal text-only human console client for deterministic MUDBench scenarios."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Mapping, Sequence

from agents.gateway.action_pipeline import run_action_pipeline
from agents.gateway.observation_builder import build_observation_for_actor
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from evaluation.benchmark_runner.runner import (
    build_playable_benchmark_session,
    build_shared_shard_loop_session,
)
from world.state.world_persistence import (
    WORLD_SAVE_DIR_DEFAULT,
    load_world_slot,
    save_world_slot,
    save_world_snapshot,
)


def _extract_scenario_id_from_payload(scenario: Mapping[str, Any] | str) -> str | None:
    """Extract scenario_id from a scenario payload (dict or JSON string) without full loading."""
    if isinstance(scenario, Mapping):
        val = scenario.get("scenario_id")
        return str(val) if val else None
    if isinstance(scenario, str):
        try:
            import json as _json
            parsed = _json.loads(scenario)
            if isinstance(parsed, dict):
                val = parsed.get("scenario_id")
                return str(val) if val else None
        except (ValueError, TypeError):
            pass
    return None

_DEFAULT_RUN_ID = "human-console"
_DEFAULT_ACTOR_ID = "human-player"
_HELP_COMMANDS = frozenset({"?", "help"})
_EXIT_COMMANDS = frozenset({"exit", "quit"})
_DIRECTION_ALIASES = {
    "n": "north",
    "north": "north",
    "s": "south",
    "south": "south",
    "e": "east",
    "east": "east",
    "w": "west",
    "west": "west",
}
_VERB_ALIASES = {
    "go": "move",
    "move": "move",
    "walk": "move",
    "take": "take",
    "get": "take",
    "grab": "take",
    "use": "use",
    "attack": "attack",
    "look": "look",
    "wait": "wait",
}


@dataclass(frozen=True, slots=True)
class HumanConsoleSessionResult:
    """Deterministic result summary for a bounded human console session."""

    run_id: str
    scenario_id: str
    actor_id: str
    step_count: int
    max_steps: int
    objective_completed: bool
    quit_requested: bool
    final_location: str
    final_inventory: tuple[str, ...]
    action_history: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class HumanSharedShardSessionResult:
    """Deterministic summary for a minimal multi-participant shared shard session."""

    run_id: str
    shard_id: str
    scenario_id: str
    actor_ids: tuple[str, ...]
    step_count: int
    max_steps: int
    completed_actor_ids: tuple[str, ...]
    quit_requested: bool
    shard_mutation_generation: int
    world_tick_count: int
    last_world_tick_heartbeat: str
    world_npc_stance_phase: str
    timing_mode: str | None = None
    action_cadence_interval: int | None = None
    actor_action_cadence_overrides: tuple[tuple[str, int], ...] = ()
    actor_next_action_eligible_at: tuple[tuple[str, int], ...] = ()


def render_human_observation(observation: Observation) -> str:
    """Render one observation deterministically for terminal play.

    Entities are grouped by type (NPCs / Items / Other) so hostile NPCs,
    interactable objects, and other actors are immediately distinguishable.
    Messages are rendered one per line instead of comma-joined so multi-line
    status blocks (history, objectives, world events) remain readable.
    """
    targets = _build_allowed_target_summary(observation.action_space)
    inventory = ", ".join(observation.inventory) or "empty"
    exits = ", ".join(observation.exits) or "none"

    # Split entities by type for clarity.
    npc_names = [e.name for e in observation.entities if e.type == "npc"]
    item_names = [e.name for e in observation.entities if e.type in {"item", "consumable"}]
    other_names = [
        e.name for e in observation.entities if e.type not in {"npc", "item", "consumable"}
    ]

    action_lines = [
        f"  {index}. {action}"
        for index, action in enumerate(observation.action_space, start=1)
    ]
    target_lines = [
        f"  {verb}: {', '.join(target_values)}"
        for verb, target_values in targets
    ] or ["  none"]

    # Messages as one-per-line list — stays readable for both humans and agents.
    message_lines: list[str] = (
        [f"  {msg}" for msg in observation.messages] if observation.messages else ["  (none)"]
    )

    lines: list[str] = [
        f"Run: {observation.run_id}",
        f"Step: {observation.step + 1}",
        f"Remaining Steps: {observation.remaining_steps}",
        f"Location: {observation.location}",
        f"Health: {observation.health}",
        f"Description: {observation.description}",
        f"Exits: {exits}",
    ]
    if npc_names:
        lines.append(f"NPCs: {', '.join(npc_names)}")
    if item_names:
        lines.append(f"Items: {', '.join(item_names)}")
    if other_names:
        lines.append(f"Other: {', '.join(other_names)}")
    if not npc_names and not item_names and not other_names:
        lines.append("Entities: none")
    lines += [
        f"Inventory: {inventory}",
        "Messages:",
        *message_lines,
        "Available Actions:",
        *action_lines,
        "Allowed Targets:",
        *target_lines,
        "Enter an exact action, a 1-based action number, or a small alias like 'east', 'take key', or 'use key'.",
    ]
    return "\n".join(lines)


def render_human_turn_result(
    *,
    action: str,
    event_types: Sequence[str],
    location: str,
    inventory: Sequence[str],
) -> str:
    """Render the outcome of one accepted human action."""
    rendered_inventory = ", ".join(inventory) or "empty"
    rendered_events = ", ".join(event_types) or "none"
    return "\n".join(
        (
            f"Accepted Action: {action}",
            f"Event Types: {rendered_events}",
            f"Current Location: {location}",
            f"Current Inventory: {rendered_inventory}",
        )
    )


def run_human_console_session(
    *,
    scenario: Mapping[str, Any] | str,
    actor_id: str = _DEFAULT_ACTOR_ID,
    run_id: str = _DEFAULT_RUN_ID,
    run_seed: int | None = None,
    max_steps_override: int | None = None,
    input_reader: Callable[[str], str] | None = None,
    output_writer: Callable[[str], None] = print,
) -> HumanConsoleSessionResult:
    """Run a minimal deterministic human-play console loop."""
    resolved_input_reader = input if input_reader is None else input_reader
    session = build_playable_benchmark_session(
        scenario=scenario,
        actor_ids=(actor_id,),
        run_id=run_id,
        run_seed=run_seed,
        max_steps_override=max_steps_override,
    )
    action_history: list[str] = []
    quit_requested = False

    while True:
        if _objectives_completed(
            snapshot=session.world_state.get_snapshot(),
            actor_id=actor_id,
            objectives=session.scenario_initialization.objectives,
        ):
            session.controller.terminate()
            output_writer("Objective complete.")
            break
        if session.controller.run_state.status.name != "RUNNING":
            break

        step_index = session.controller.run_state.step_index
        observation = build_observation_for_actor(
            session.world_state.get_snapshot(),
            actor_id=actor_id,
            run_id=run_id,
            step=step_index,
            max_steps=session.controller.run_state.max_steps,
            messages=(),
        )
        output_writer(render_human_observation(observation))

        while True:
            raw_command = resolved_input_reader("action> ")
            try:
                selected_action = _resolve_console_action(raw_command, observation)
            except _HelpRequested:
                output_writer(render_human_observation(observation))
                continue
            except ValueError as exc:
                output_writer(f"Rejected Input: {exc}")
                continue

            if selected_action is None:
                quit_requested = True
                session.controller.terminate()
                output_writer("Session ended by player.")
                break

            pipeline_result = run_action_pipeline(
                actor_id=actor_id,
                submission=ActionSubmission(action=selected_action),
                observation=observation,
            )
            if not pipeline_result.accepted or pipeline_result.action_request is None:
                output_writer(
                    "Rejected Input: "
                    f"{pipeline_result.reason or 'action_rejected'}"
                )
                continue

            step_outcome = session.controller.step((pipeline_result.action_request,))
            action_history.append(selected_action)
            snapshot = session.world_state.get_snapshot()
            actor_payload = _require_actor_payload(snapshot, actor_id)
            output_writer(
                render_human_turn_result(
                    action=selected_action,
                    event_types=tuple(event.event_type for event in step_outcome.emitted_events),
                    location=str(actor_payload.get("location", "")),
                    inventory=tuple(_string_sequence(actor_payload.get("inventory", ()))),
                )
            )
            break

        if quit_requested:
            break

    final_snapshot = session.world_state.get_snapshot()
    final_actor_payload = _require_actor_payload(final_snapshot, actor_id)
    return HumanConsoleSessionResult(
        run_id=run_id,
        scenario_id=session.scenario_initialization.scenario_id,
        actor_id=actor_id,
        step_count=session.controller.run_state.step_index,
        max_steps=session.controller.run_state.max_steps,
        objective_completed=_objectives_completed(
            snapshot=final_snapshot,
            actor_id=actor_id,
            objectives=session.scenario_initialization.objectives,
        ),
        quit_requested=quit_requested,
        final_location=str(final_actor_payload.get("location", "")),
        final_inventory=tuple(_string_sequence(final_actor_payload.get("inventory", ()))),
        action_history=tuple(action_history),
    )


def run_human_shared_shard_session(
    *,
    scenario: Mapping[str, Any] | str,
    actor_ids: Sequence[str],
    mock_agent_actor_ids: Sequence[str] = (),
    external_agent_commands_by_actor: Mapping[str, Sequence[str]] | None = None,
    persistent_agent_session: bool = False,
    run_id: str = "human-shared-shard",
    shard_id: str = "shared-shard-local",
    run_seed: int | None = None,
    max_steps_override: int | None = None,
    timing_mode: str | None = None,
    action_cadence_interval: int | None = None,
    actor_action_cadence_overrides: Mapping[str, int] | None = None,
    external_agent_timeout_seconds: float | None = None,
    input_reader: Callable[[str], str] | None = None,
    output_writer: Callable[[str], None] = print,
    world_load_path: str | None = None,
    world_save_path: str | None = None,
    world_load_slot: str | None = None,
    world_save_slot: str | None = None,
    save_dir: str | None = None,
) -> HumanSharedShardSessionResult:
    """Run a minimal deterministic multi-participant shared shard console loop."""
    resolved_input_reader = input if input_reader is None else input_reader
    _effective_save_dir = save_dir or WORLD_SAVE_DIR_DEFAULT

    # Resolve named save slot into a pre-loaded world state if requested.
    _preloaded_world_state = None
    if world_load_slot is not None:
        _scenario_id_for_guard = _extract_scenario_id_from_payload(scenario)
        _slot_result = load_world_slot(
            world_load_slot,
            _effective_save_dir,
            required_scenario_id=_scenario_id_for_guard,
        )
        if not _slot_result.accepted or _slot_result.world_state is None:
            raise ValueError(
                f"world_load_slot_rejected:{world_load_slot}:{_slot_result.reason}"
            )
        _preloaded_world_state = _slot_result.world_state
        output_writer(
            f"World: loaded save slot '{world_load_slot}' "
            f"(scenario={_slot_result.scenario_id}, tick={_slot_result.world_tick})"
        )

    session = build_shared_shard_loop_session(
        scenario=scenario,
        actor_ids=tuple(actor_ids),
        agent_actor_ids=tuple(mock_agent_actor_ids),
        external_agent_commands_by_actor=external_agent_commands_by_actor,
        persistent_agent_session=persistent_agent_session,
        run_id=run_id,
        shard_id=shard_id,
        run_seed=run_seed,
        max_steps_override=max_steps_override,
        timing_mode=timing_mode,
        action_cadence_interval=action_cadence_interval,
        actor_action_cadence_overrides=actor_action_cadence_overrides,
        external_agent_timeout_seconds=external_agent_timeout_seconds,
        world_load_path=world_load_path,
        world_state=_preloaded_world_state,
    )
    quit_requested = False

    try:
        while True:
            completed_actor_ids = tuple(
                actor_id
                for actor_id in session.active_actor_ids()
                if _objectives_completed(
                    snapshot=session.world_state.get_snapshot(),
                    actor_id=actor_id,
                    objectives=session.scenario_initialization.objectives,
                )
            )
            if len(completed_actor_ids) > 0:
                session.controller.terminate()
                output_writer(f"Objective complete for: {', '.join(completed_actor_ids)}")
                break
            if session.controller.run_state.status.name != "RUNNING":
                break

            submitted_actions: dict[str, str] = {}
            for actor_id in session.active_actor_ids():
                if not session.is_actor_action_eligible(actor_id):
                    output_writer(
                        "Actor Timing: "
                        f"{actor_id} next eligible at world tick "
                        f"{session.get_actor_next_action_eligible_at(actor_id)}"
                    )
                    continue
                observation = session.get_observation(actor_id)
                output_writer(f"Actor Turn: {actor_id}")
                output_writer(render_human_observation(observation))

                if session.is_external_agent_participant(actor_id):
                    action_submission = session.request_external_agent_action(actor_id)
                    selected_action = action_submission.action
                    submitted_actions[actor_id] = selected_action
                    output_writer(f"External Agent Selected Action: {selected_action}")
                    continue

                if session.is_agent_participant(actor_id):
                    turn_result = session.build_mock_agent_turn(actor_id)
                    selected_action = turn_result.action_submission.action
                    submitted_actions[actor_id] = selected_action
                    output_writer(f"Agent Selected Action: {selected_action}")
                    continue

                while True:
                    raw_command = resolved_input_reader(f"{actor_id}> ")
                    try:
                        selected_action = _resolve_console_action(raw_command, observation)
                    except _HelpRequested:
                        output_writer(render_human_observation(observation))
                        continue
                    except ValueError as exc:
                        output_writer(f"Rejected Input: {exc}")
                        continue

                    if selected_action is None:
                        quit_requested = True
                        session.controller.terminate()
                        output_writer("Shared shard session ended by player.")
                        break

                    submitted_actions[actor_id] = selected_action
                    break

                if quit_requested:
                    break

            if quit_requested:
                break

            step_result = session.advance_tick(submitted_actions)
            snapshot = session.world_state.get_snapshot()
            for actor_id, accepted_action in step_result.accepted_actions:
                actor_payload = _require_actor_payload(snapshot, actor_id)
                output_writer(
                    "\n".join(
                        (
                            f"Actor Result: {actor_id}",
                            render_human_turn_result(
                                action=accepted_action,
                                event_types=step_result.emitted_event_types,
                                location=str(actor_payload.get("location", "")),
                                inventory=tuple(_string_sequence(actor_payload.get("inventory", ()))),
                            ),
                        )
                    )
                )
            output_writer(
                "World Tick Effect: "
                f"npc_stance_phase={step_result.world_npc_stance_phase}"
            )
            if step_result.action_cadence_interval is not None:
                rendered_next_eligible = ", ".join(
                    f"{actor_id}={next_action_eligible_at}"
                    for actor_id, next_action_eligible_at in step_result.actor_next_action_eligible_at
                )
                if rendered_next_eligible == "":
                    rendered_next_eligible = "none"
                timing_prefix = ""
                if step_result.timing_mode is not None:
                    timing_prefix = f"timing_mode={step_result.timing_mode}; "
                output_writer(
                    "World Timing: "
                    f"{timing_prefix}"
                    f"action_cadence_interval={step_result.action_cadence_interval}; "
                    f"next_action_eligible_at={rendered_next_eligible}"
                )

        final_snapshot = session.world_state.get_snapshot()
        completed_actor_ids = tuple(
            actor_id
            for actor_id in session.active_actor_ids()
            if _objectives_completed(
                snapshot=final_snapshot,
                actor_id=actor_id,
                objectives=session.scenario_initialization.objectives,
            )
        )
        return HumanSharedShardSessionResult(
            run_id=run_id,
            shard_id=session.shard_id,
            scenario_id=session.scenario_initialization.scenario_id,
            actor_ids=tuple(actor_ids),
            step_count=session.current_tick,
            max_steps=session.max_steps,
            completed_actor_ids=completed_actor_ids,
            quit_requested=quit_requested,
            shard_mutation_generation=session.shard_state.journal.last_committed_mutation_generation,
            world_tick_count=session.shard_state.metadata.world_tick_count,
            last_world_tick_heartbeat=session.shard_state.metadata.last_world_tick_heartbeat,
            world_npc_stance_phase=session.shard_state.metadata.npc_stance_phase,
            timing_mode=session.timing_mode,
            action_cadence_interval=session.action_cadence_interval,
            actor_action_cadence_overrides=session.actor_action_cadence_overrides,
            actor_next_action_eligible_at=session.shard_state.actor_next_action_eligible_at,
        )
    finally:
        if world_save_slot is not None:
            _save_result = save_world_slot(
                world_save_slot,
                _effective_save_dir,
                session.world_state,
                run_id=run_id,
                scenario_id=session.scenario_initialization.scenario_id,
                scenario_version=session.scenario_initialization.version,
                session_id=session.shard_id,
                actor_ids=list(actor_ids),
            )
            if _save_result.accepted:
                output_writer(f"World: saved to slot '{world_save_slot}' ({_save_result.path})")
        elif world_save_path is not None:
            save_world_snapshot(
                world_save_path,
                session.world_state,
                run_id=run_id,
                scenario_id=session.scenario_initialization.scenario_id,
                scenario_version=session.scenario_initialization.version,
                actor_ids=list(actor_ids),
            )
        session.close_external_agent_participants()


class _HelpRequested(Exception):
    """Internal control-flow marker for help re-rendering."""


def _resolve_console_action(raw_command: str, observation: Observation) -> str | None:
    normalized = raw_command.strip()
    if normalized == "":
        raise ValueError("empty_action_input")
    lower = normalized.lower()
    if lower in _HELP_COMMANDS:
        raise _HelpRequested
    if lower in _EXIT_COMMANDS:
        return None
    if normalized in observation.action_space:
        return normalized
    if normalized.isdigit():
        index = int(normalized)
        if index < 1 or index > len(observation.action_space):
            raise ValueError("action_index_out_of_range")
        return observation.action_space[index - 1]
    aliased_action = _resolve_action_alias(lower, observation.action_space)
    if aliased_action is not None:
        return aliased_action
    raise ValueError("action_not_in_action_space")


def _resolve_action_alias(raw_command: str, action_space: Sequence[str]) -> str | None:
    if raw_command in _DIRECTION_ALIASES:
        resolved_direction = _DIRECTION_ALIASES[raw_command]
        move_action = f"move {resolved_direction}"
        if move_action in action_space:
            return move_action
        raise ValueError("direction_not_available")

    command_parts = tuple(part for part in raw_command.split() if part)
    if len(command_parts) == 0:
        return None
    verb = _VERB_ALIASES.get(command_parts[0])
    if verb is None:
        return None
    if len(command_parts) == 1:
        if verb in {"look", "wait"} and verb in action_space:
            return verb
        return None

    target_parts = command_parts[1:]
    candidate_actions = _match_alias_candidates(
        action_space=action_space,
        verb=verb,
        target_parts=target_parts,
    )
    if len(candidate_actions) == 1:
        return candidate_actions[0]
    if len(candidate_actions) > 1:
        raise ValueError("ambiguous_action_alias")
    raise ValueError("action_alias_not_resolvable")


def _match_alias_candidates(
    *,
    action_space: Sequence[str],
    verb: str,
    target_parts: Sequence[str],
) -> tuple[str, ...]:
    normalized_target = _tokenize_alias_target(" ".join(target_parts))
    candidates: list[str] = []
    for action in action_space:
        action_verb, separator, action_target = action.partition(" ")
        if separator == "" or action_verb != verb:
            continue
        action_target_parts = _tokenize_alias_target(action_target)
        if action_target_parts == normalized_target or action_target_parts[-len(normalized_target) :] == normalized_target:
            candidates.append(action)
    return tuple(sorted(candidates))


def _tokenize_alias_target(value: str) -> tuple[str, ...]:
    normalized = value.lower().replace("-", " ")
    return tuple(part for part in normalized.split() if part)


def _build_allowed_target_summary(
    action_space: Sequence[str],
) -> tuple[tuple[str, tuple[str, ...]], ...]:
    target_map: dict[str, list[str]] = {}
    for action in action_space:
        verb, separator, remainder = action.partition(" ")
        if separator == "":
            continue
        target_map.setdefault(verb, []).append(remainder)
    summary: list[tuple[str, tuple[str, ...]]] = []
    for verb in sorted(target_map):
        summary.append((verb, tuple(sorted(target_map[verb]))))
    return tuple(summary)


def _objectives_completed(
    *,
    snapshot: Mapping[str, Any],
    actor_id: str,
    objectives: Sequence[tuple[str, tuple[tuple[str, Any], ...]]],
) -> bool:
    actor_payload = _require_actor_payload(snapshot, actor_id)
    inventory = tuple(_string_sequence(actor_payload.get("inventory", ())))
    for _, objective_fields in objectives:
        field_map = {key: value for key, value in objective_fields}
        if field_map.get("objective_type") != "collect_item":
            return False
        target_id = field_map.get("target_id")
        required_count = field_map.get("required_count", 1)
        if not isinstance(target_id, str) or not isinstance(required_count, int):
            return False
        if inventory.count(target_id) < required_count:
            return False
    return True


def _require_actor_payload(snapshot: Mapping[str, Any], actor_id: str) -> Mapping[str, Any]:
    entities = snapshot.get("entities", {})
    if not isinstance(entities, Mapping):
        raise ValueError("world_snapshot_missing_entities")
    actor_payload = entities.get(actor_id)
    if not isinstance(actor_payload, Mapping):
        raise ValueError(f"actor_not_found:{actor_id}")
    return actor_payload


def _string_sequence(value: object) -> tuple[str, ...]:
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        return ()
    items: list[str] = []
    for item in value:
        if isinstance(item, str):
            items.append(item)
    return tuple(items)
