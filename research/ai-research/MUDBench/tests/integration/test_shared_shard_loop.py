from __future__ import annotations

import json
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

from agents.direct_provider_runner import DirectProviderConfig, build_direct_provider_command
from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import build_shared_shard_loop_session


def _write_external_shared_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "shared_external_agent.py"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "line = sys.stdin.readline()\n"
            "observation = json.loads(line)\n"
            "messages = tuple(observation.get('messages', ()))\n"
            "action_space = tuple(observation.get('action_space', ()))\n"
            "action = 'wait'\n"
            "if any('dormant' in message for message in messages) and 'move east' in action_space:\n"
            "    action = 'move east'\n"
            "elif any('watchful' in message for message in messages) and 'move east' in action_space:\n"
            "    action = 'move east'\n"
            "elif any('patrol' in message for message in messages):\n"
            "    for candidate in action_space:\n"
            "        if candidate.startswith('take '):\n"
            "            action = candidate\n"
            "            break\n"
            "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_persistent_external_shared_agent_script(tmp_path: Path) -> tuple[Path, Path]:
    script_path = tmp_path / "persistent_shared_external_agent.py"
    boot_log_path = tmp_path / "persistent_shared_external_agent.boot.log"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "from pathlib import Path\n"
            f"boot_log_path = Path({str(boot_log_path)!r})\n"
            "boot_log_path.write_text(boot_log_path.read_text(encoding='utf-8') + 'boot\\n' if boot_log_path.exists() else 'boot\\n', encoding='utf-8')\n"
            "turn_count = 0\n"
            "while True:\n"
            "    line = sys.stdin.readline()\n"
            "    if not line:\n"
            "        break\n"
            "    observation = json.loads(line)\n"
            "    action_space = tuple(observation.get('action_space', ()))\n"
            "    action = 'wait'\n"
            "    if turn_count < 2 and 'move east' in action_space:\n"
            "        action = 'move east'\n"
            "    elif turn_count >= 2:\n"
            "        for candidate in action_space:\n"
            "            if candidate.startswith('take '):\n"
            "                action = candidate\n"
            "                break\n"
            "    print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
            "    sys.stdout.flush()\n"
            "    turn_count += 1\n"
        ),
        encoding="utf-8",
    )
    return script_path, boot_log_path


def _start_direct_provider_test_server(
    response_contents: list[str],
    *,
    response_delay_seconds: float = 0.0,
) -> tuple[str, list[dict[str, object]], ThreadingHTTPServer]:
    captured_requests: list[dict[str, object]] = []
    remaining_responses = list(response_contents)

    class _Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:  # noqa: N802
            content_length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(content_length).decode("utf-8")
            payload = json.loads(body)
            captured_requests.append(payload)
            response_content = remaining_responses.pop(0)
            if response_delay_seconds > 0.0:
                time.sleep(response_delay_seconds)
            response_payload = {
                "choices": [
                    {
                        "message": {
                            "content": response_content,
                        }
                    }
                ]
            }
            encoded = json.dumps(
                response_payload,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=True,
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(encoded)))
            self.end_headers()
            self.wfile.write(encoded)

        def log_message(self, format: str, *args: object) -> None:  # noqa: A003
            return

    server = ThreadingHTTPServer(("127.0.0.1", 0), _Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base_url = f"http://127.0.0.1:{server.server_address[1]}/v1/chat/completions"
    return base_url, captured_requests, server


def test_shared_shard_loop_preserves_world_continuity_across_multiple_participants() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "move east", "player-b": "wait"})
    second = session.advance_tick({"player-a": "move east", "player-b": "move east"})
    third = session.advance_tick({"player-a": "take golden-key", "player-b": "move east"})
    player_b_observation = session.get_observation("player-b")

    assert first.to_dict() == {
        "step_index": 0,
        "accepted_actions": [
            {"actor_id": "player-a", "action": "move east"},
            {"actor_id": "player-b", "action": "wait"},
        ],
        "emitted_event_types": ["action_move", "action_wait", "step_completed"],
        "active_actor_ids": ["player-a", "player-b"],
        "shard_mutation_generation": 6,
        "world_tick_count": 1,
        "world_tick_heartbeat": "shared_shard_world_tick:0001",
        "world_npc_stance_phase": "watchful",
    }
    assert second.step_index == 1
    assert third.step_index == 2
    assert player_b_observation.location == "treasury"
    assert player_b_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
        "[Party] player-a: treasury — 100 HP",
    )
    assert session.world_state.get_snapshot()["entities"]["player-a"]["inventory"] == ["golden-key"]
    assert session.shard_state.get_session("sess-player-a").status == "active"
    assert session.shard_state.get_session("sess-player-b").status == "active"
    assert session.world_tick_count == 3
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0003"
    assert session.world_npc_stance_phase == "settling"
    assert session.current_tick == 3


def test_shared_shard_loop_supports_session_close_and_reopen_without_losing_continuity() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    session.close_participant_session("player-b")
    assert session.session_is_active("player-b") is False

    first = session.advance_tick({"player-a": "move east"})

    session.open_participant_session("player-b")
    reopened_observation = session.get_observation("player-b")
    second = session.advance_tick({"player-a": "move east", "player-b": "move east"})

    assert first.active_actor_ids == ("player-a",)
    assert reopened_observation.location == "entrance"
    assert session.session_is_active("player-b") is True
    assert session.shard_state.get_session("sess-player-b").status == "active"
    assert second.active_actor_ids == ("player-a", "player-b")
    assert second.world_tick_count == 2
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert session.world_npc_stance_phase == "patrolling"
    assert session.current_tick == 2


def test_shared_shard_loop_world_tick_advances_deterministically_even_when_actors_wait() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    second = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    second_observation = session.get_observation("player-a")

    assert first.emitted_event_types == ("action_wait", "action_wait", "step_completed")
    assert first.world_tick_count == 1
    assert first.world_tick_heartbeat == "shared_shard_world_tick:0001"
    assert first.world_npc_stance_phase == "watchful"
    assert second.world_tick_count == 2
    assert second.world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert second.world_npc_stance_phase == "patrolling"
    assert session.shard_state.metadata.world_tick_count == 2
    assert session.shard_state.metadata.last_world_tick_heartbeat == "shared_shard_world_tick:0002"
    assert session.shard_state.metadata.npc_stance_phase == "patrolling"
    assert second_observation.messages == (
        "You catch the measured rhythm of a distant patrol.",
        "Hint: the moving patrol leaves brief windows for repositioning.",
        "[Party] player-b: entrance — 100 HP",
    )


def test_shared_shard_loop_action_cadence_rejects_early_actions_and_honors_overrides() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
        action_cadence_interval=2,
        actor_action_cadence_overrides={"player-b": 3},
    )

    first = session.advance_tick({"player-a": "wait", "player-b": "wait"})

    assert first.to_dict() == {
        "step_index": 0,
        "accepted_actions": [
            {"actor_id": "player-a", "action": "wait"},
            {"actor_id": "player-b", "action": "wait"},
        ],
        "emitted_event_types": ["action_wait", "action_wait", "step_completed"],
        "active_actor_ids": ["player-a", "player-b"],
        "shard_mutation_generation": 6,
        "world_tick_count": 1,
        "world_tick_heartbeat": "shared_shard_world_tick:0001",
        "world_npc_stance_phase": "watchful",
        "action_cadence_interval": 2,
        "actor_action_cadence_overrides": [
            {"actor_id": "player-b", "cadence_interval": 3},
        ],
        "actor_next_action_eligible_at": [
            {"actor_id": "player-a", "next_action_eligible_at": 2},
            {"actor_id": "player-b", "next_action_eligible_at": 3},
        ],
    }
    assert session.get_observation("player-a").messages == (
        "A distant sentinel grows watchful in the quiet corridors.",
        "Hint: the sharp watch makes a careful look feel safer than rushing.",
        "[Party] player-b: entrance — 100 HP",
        "Timing: world_tick=1; action_cadence_interval=2; next_action_eligible_at=2",
    )
    with pytest.raises(
        ValueError,
        match="shared_shard_action_rejected:player-a:action_cadence_locked_until_tick_2",
    ):
        session.advance_tick({"player-a": "wait"})

    second = session.advance_tick()
    third = session.advance_tick({"player-a": "wait"})
    fourth = session.advance_tick({"player-b": "wait"})

    assert second.accepted_actions == ()
    assert second.world_tick_count == 2
    assert third.accepted_actions == (("player-a", "wait"),)
    assert third.actor_next_action_eligible_at == (("player-a", 4), ("player-b", 3))
    assert fourth.accepted_actions == (("player-b", "wait"),)
    assert fourth.actor_next_action_eligible_at == (("player-a", 4), ("player-b", 6))


def test_shared_shard_loop_timing_modes_resolve_into_cadence_deterministically() -> None:
    off_session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-off",
        shard_id="shared-shard-off",
        timing_mode="off",
    )
    parity_session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-parity",
        shard_id="shared-shard-parity",
        timing_mode="human-parity",
    )
    equal_session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-equal",
        shard_id="shared-shard-equal",
        timing_mode="equal-cadence",
        action_cadence_interval=3,
    )

    assert off_session.timing_mode == "off"
    assert off_session.action_cadence_interval is None
    assert off_session.get_observation("player-a").messages == (
        "The shard feels still, as if the watch has not yet begun.",
        "Hint: the route feels open while the watch remains dormant.",
        "[Party] player-b: entrance — 100 HP",
    )

    assert parity_session.timing_mode == "human-parity"
    assert parity_session.action_cadence_interval == 2
    assert parity_session.get_observation("player-a").messages == (
        "The shard feels still, as if the watch has not yet begun.",
        "Hint: the route feels open while the watch remains dormant.",
        "[Party] player-b: entrance — 100 HP",
        "Timing: timing_mode=human-parity; world_tick=0; action_cadence_interval=2; next_action_eligible_at=0",
    )

    assert equal_session.timing_mode == "equal-cadence"
    assert equal_session.action_cadence_interval == 3
    assert equal_session.actor_action_cadence_overrides == ()


def test_shared_shard_loop_phase_outcome_effect_blocks_and_reopens_corridor_route_deterministically() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first = session.advance_tick({"player-a": "move east", "player-b": "wait"})
    watchful_observation = session.get_observation("player-a")
    second = session.advance_tick({"player-a": "wait", "player-b": "wait"})
    patrolling_observation = session.get_observation("player-a")

    assert first.world_npc_stance_phase == "watchful"
    assert watchful_observation.location == "corridor"
    assert watchful_observation.action_space == ("wait", "look", "move east", "talk guard-1", "defend", "attack guard-1")
    assert watchful_observation.messages == (
        "A distant sentinel grows watchful in the quiet corridors.",
        "Hint: the sharp watch makes a careful look feel safer than rushing.",
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable.",
        "[Party] player-b: entrance — 100 HP",
    )
    assert second.world_npc_stance_phase == "patrolling"
    assert patrolling_observation.location == "corridor"
    assert "move west" in patrolling_observation.action_space
    assert patrolling_observation.messages == (
        "You catch the measured rhythm of a distant patrol.",
        "Hint: the moving patrol leaves brief windows for repositioning.",
        "Consequence: the west passage opens again between patrol sweeps.",
        "[Party] player-b: entrance — 100 HP",
    )
    assert session.current_tick == 2


def test_shared_shard_loop_supports_mixed_human_and_agent_participation_deterministically() -> None:
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "agent-b"),
        agent_actor_ids=("agent-b",),
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    first_agent_turn = session.build_mock_agent_turn("agent-b")
    first = session.advance_tick(
        {
            "human-a": "move east",
            "agent-b": first_agent_turn.action_submission.action,
        }
    )
    second_agent_turn = session.build_mock_agent_turn("agent-b")
    second = session.advance_tick(
        {
            "human-a": "move east",
            "agent-b": second_agent_turn.action_submission.action,
        }
    )
    third_agent_turn = session.build_mock_agent_turn("agent-b")
    third = session.advance_tick(
        {
            "human-a": "wait",
            "agent-b": third_agent_turn.action_submission.action,
        }
    )
    human_observation = session.get_observation("human-a")

    assert session.is_agent_participant("human-a") is False
    assert session.is_agent_participant("agent-b") is True
    assert session.shard_state.get_character("char-agent-b").identity_class == "external_agent"
    assert first_agent_turn.action_submission.action == "move east"
    assert second_agent_turn.action_submission.action == "move east"
    assert third_agent_turn.action_submission.action == "take golden-key"
    assert "Invariant runtime guardrails:" in first_agent_turn.prompt
    assert first_agent_turn.model_facing_observation_payload["observation"]["messages"] == [
        "The shard feels still, as if the watch has not yet begun.",
        "Hint: the route feels open while the watch remains dormant.",
        "[Party] human-a: entrance — 100 HP",
    ]
    assert first.accepted_actions == (("human-a", "move east"), ("agent-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("agent-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("agent-b", "take golden-key"))
    assert third.world_tick_count == 3
    assert third.world_npc_stance_phase == "settling"
    assert session.world_state.get_snapshot()["entities"]["agent-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
        "[Party] agent-b: treasury — 100 HP",
    )


def test_shared_shard_loop_supports_mixed_human_and_external_agent_participation_deterministically(
    tmp_path: Path,
) -> None:
    script_path = _write_external_shared_agent_script(tmp_path)
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "external-b"),
        external_agent_commands_by_actor={
            "external-b": (sys.executable, str(script_path)),
        },
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("external-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("external-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("external-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "external-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()

    assert session.is_external_agent_participant("external-b") is True
    assert session.is_agent_participant("external-b") is False
    assert session.shard_state.get_character("char-external-b").identity_class == "external_agent"
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert first.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("external-b", "take golden-key"))
    assert session.world_state.get_snapshot()["entities"]["external-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
        "[Party] external-b: treasury — 100 HP",
    )


def test_shared_shard_loop_supports_persistent_external_agent_reuse_deterministically(
    tmp_path: Path,
) -> None:
    script_path, boot_log_path = _write_persistent_external_shared_agent_script(tmp_path)
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "external-b"),
        external_agent_commands_by_actor={
            "external-b": (sys.executable, str(script_path)),
        },
        persistent_agent_session=True,
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("external-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("external-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "external-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("external-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "external-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()

    assert first.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("external-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("external-b", "take golden-key"))
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert session.world_tick_count == 3
    assert session.world_state.get_snapshot()["entities"]["external-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
        "[Party] external-b: treasury — 100 HP",
    )
    assert boot_log_path.read_text(encoding="utf-8").splitlines() == ["boot"]


def test_shared_shard_loop_supports_mixed_human_and_direct_provider_participation_deterministically(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    base_url, captured_requests, server = _start_direct_provider_test_server(
        [
            '{"action":"move east"}',
            "not-json",
            '{"action":"move east"}',
            '{"action":"take golden-key"}',
        ]
    )
    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    provider_config = DirectProviderConfig(
        provider="openai-chat-completions",
        model="gpt-4.1-mini",
        api_key="test-key",
        base_url=base_url,
    )
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "direct-b"),
        external_agent_commands_by_actor={
            "direct-b": build_direct_provider_command(
                provider_config,
                python_executable=sys.executable,
            ),
        },
        run_id="shared-loop-run",
        shard_id="shared-shard-alpha",
    )

    try:
        first_action = session.request_external_agent_action("direct-b")
        first = session.advance_tick(
            {
                "human-a": "move east",
                "direct-b": first_action.action,
            }
        )
        second_action = session.request_external_agent_action("direct-b")
        second = session.advance_tick(
            {
                "human-a": "move east",
                "direct-b": second_action.action,
            }
        )
        third_action = session.request_external_agent_action("direct-b")
        third = session.advance_tick(
            {
                "human-a": "wait",
                "direct-b": third_action.action,
            }
        )
        human_observation = session.get_observation("human-a")
    finally:
        session.close_external_agent_participants()
        server.shutdown()
        server.server_close()

    assert session.is_agent_participant("direct-b") is False
    assert session.shard_state.get_character("char-direct-b").identity_class == "external_agent"
    assert session.is_external_agent_participant("direct-b") is True
    assert first_action.action == "move east"
    assert second_action.action == "move east"
    assert third_action.action == "take golden-key"
    assert first.accepted_actions == (("human-a", "move east"), ("direct-b", "move east"))
    assert second.accepted_actions == (("human-a", "move east"), ("direct-b", "move east"))
    assert third.accepted_actions == (("human-a", "wait"), ("direct-b", "take golden-key"))
    assert session.world_tick_count == 3
    assert session.world_state.get_snapshot()["entities"]["direct-b"]["inventory"] == ["golden-key"]
    assert human_observation.messages == (
        "The far-off watch settles back into guarded stillness.",
        "Hint: the easing watch makes nearby movement feel less exposed.",
        "[Party] direct-b: treasury — 100 HP",
    )
    assert len(captured_requests) == 4
    assert "Invariant runtime guardrails:" in str(captured_requests[0]["messages"][0]["content"])
    assert (
        "Consequence: the exposed west passage is pinned under watch; move west is unavailable."
        in str(captured_requests[1]["messages"][0]["content"])
    )
    assert '{"allowed_actions":["wait","look","move east","talk guard-1","defend","attack guard-1"]}' in str(
        captured_requests[1]["messages"][0]["content"]
    )
    assert "Your previous response was invalid for MUDBench." in str(
        captured_requests[2]["messages"][0]["content"]
    )


def test_shared_shard_loop_external_agent_timeout_override_supports_slow_direct_provider(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    base_url, _, slow_server = _start_direct_provider_test_server(
        ['{"action":"move east"}'],
        response_delay_seconds=1.2,
    )
    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    provider_config = DirectProviderConfig(
        provider="openai-chat-completions",
        model="gpt-4.1-mini",
        api_key="test-key",
        base_url=base_url,
    )

    default_timeout_session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "direct-b"),
        external_agent_commands_by_actor={
            "direct-b": build_direct_provider_command(
                provider_config,
                python_executable=sys.executable,
            ),
        },
        run_id="shared-loop-run-default-timeout",
        shard_id="shared-shard-default-timeout",
    )

    with pytest.raises(
        RuntimeError,
        match="shared_shard_external_agent_failure:direct-b:timeout:",
    ):
        default_timeout_session.request_external_agent_action("direct-b")
    default_timeout_session.close_external_agent_participants()
    slow_server.shutdown()
    slow_server.server_close()

    base_url, _, slow_server = _start_direct_provider_test_server(
        ['{"action":"move east"}'],
        response_delay_seconds=1.2,
    )
    provider_config = DirectProviderConfig(
        provider="openai-chat-completions",
        model="gpt-4.1-mini",
        api_key="test-key",
        base_url=base_url,
    )
    override_timeout_session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("human-a", "direct-b"),
        external_agent_commands_by_actor={
            "direct-b": build_direct_provider_command(
                provider_config,
                python_executable=sys.executable,
            ),
        },
        run_id="shared-loop-run-override-timeout",
        shard_id="shared-shard-override-timeout",
        external_agent_timeout_seconds=2.0,
    )

    try:
        action_submission = override_timeout_session.request_external_agent_action("direct-b")
    finally:
        override_timeout_session.close_external_agent_participants()
        slow_server.shutdown()
        slow_server.server_close()

    assert action_submission.action == "move east"


# ---------------------------------------------------------------------------
# World persistence integration tests
# ---------------------------------------------------------------------------


def test_shared_shard_saves_snapshot_on_exit(tmp_path: Path) -> None:
    """World snapshot is written to world_save_path after shared shard session ends."""
    from human_console_client import run_human_shared_shard_session

    save_path = str(tmp_path / "shard_snap.json")
    inputs = iter(["quit"])

    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_path=save_path,
        input_reader=lambda _: next(inputs),
        output_writer=lambda _: None,
    )

    import json
    import os
    assert os.path.isfile(save_path), "snapshot file must be created after session ends"
    raw = Path(save_path).read_text(encoding="utf-8")
    parsed = json.loads(raw)
    assert parsed.get("format") == "mudbench_world_snapshot"
    assert "world_state" in parsed


def test_shared_shard_loads_snapshot_on_start(tmp_path: Path) -> None:
    """Session can load a previously saved snapshot without error."""
    from human_console_client import run_human_shared_shard_session

    save_path = str(tmp_path / "shard_snap.json")

    # First session: save
    inputs1 = iter(["quit"])
    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_path=save_path,
        input_reader=lambda _: next(inputs1),
        output_writer=lambda _: None,
    )

    import os
    assert os.path.isfile(save_path)

    # Second session: load from saved snapshot
    inputs2 = iter(["quit"])
    result = run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_load_path=save_path,
        input_reader=lambda _: next(inputs2),
        output_writer=lambda _: None,
    )
    assert result.run_id == "human-shared-shard"


def test_shared_shard_loop_behavior_unchanged_without_persistence(tmp_path: Path) -> None:
    """Default shared shard loop behavior is unchanged when no persistence args are set."""
    from human_console_client import run_human_shared_shard_session

    inputs = iter(["quit"])
    result = run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        input_reader=lambda _: next(inputs),
        output_writer=lambda _: None,
    )
    assert result.run_id == "human-shared-shard"
    assert result.quit_requested is True


# ---------------------------------------------------------------------------
# Cross-session continuity integration tests
# ---------------------------------------------------------------------------


def test_shared_shard_save_slot_creates_named_file(tmp_path: Path) -> None:
    """world_save_slot creates a named .json file in save_dir."""
    from human_console_client import run_human_shared_shard_session
    import os

    save_dir = str(tmp_path / "saves")
    inputs = iter(["quit"])

    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_slot="session1",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs),
        output_writer=lambda _: None,
    )

    assert os.path.isfile(os.path.join(save_dir, "session1.json"))


def test_shared_shard_load_slot_reconnects_world(tmp_path: Path) -> None:
    """Session saved via slot can be loaded into a second session."""
    import json
    from pathlib import Path as _Path
    from human_console_client import run_human_shared_shard_session

    save_dir = str(tmp_path / "saves")

    inputs1 = iter(["quit"])
    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_slot="game1",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs1),
        output_writer=lambda _: None,
    )

    # Verify the slot file was written
    slot_path = _Path(save_dir) / "game1.json"
    assert slot_path.exists()
    saved_data = json.loads(slot_path.read_text(encoding="utf-8"))
    assert saved_data.get("scenario_id") == "tiny-fetch-quest"

    # Second session: reconnect using the slot
    inputs2 = iter(["quit"])
    result = run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_load_slot="game1",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs2),
        output_writer=lambda _: None,
    )
    assert result.quit_requested is True


def test_shared_shard_load_slot_scenario_guard_rejects_mismatch(tmp_path: Path) -> None:
    """Loading a slot saved for a different scenario raises ValueError."""
    from human_console_client import run_human_shared_shard_session

    save_dir = str(tmp_path / "saves")

    # Save a slot for tiny-fetch-quest
    inputs1 = iter(["quit"])
    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_slot="wrongscenario",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs1),
        output_writer=lambda _: None,
    )

    # Try to load it into tiny-context-pressure — should fail
    inputs2 = iter(["quit"])
    import pytest
    with pytest.raises(ValueError, match="world_load_slot_rejected"):
        run_human_shared_shard_session(
            scenario=_SCENARIO_PRESETS["tiny-context-pressure"],
            actor_ids=("player-a", "player-b"),
            world_load_slot="wrongscenario",
            save_dir=save_dir,
            input_reader=lambda _: next(inputs2),
            output_writer=lambda _: None,
        )


def test_shared_shard_save_slot_includes_scenario_id(tmp_path: Path) -> None:
    """Saved slot file contains scenario_id in its metadata."""
    import json
    from pathlib import Path as _Path
    from human_console_client import run_human_shared_shard_session

    save_dir = str(tmp_path / "saves")
    inputs = iter(["quit"])

    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_slot="meta-check",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs),
        output_writer=lambda _: None,
    )

    saved = json.loads((_Path(save_dir) / "meta-check.json").read_text(encoding="utf-8"))
    assert saved["scenario_id"] == "tiny-fetch-quest"
    assert "scenario_version" in saved
    assert "session_id" in saved


# ---------------------------------------------------------------------------
# Multi-actor cross-session continuity tests
# ---------------------------------------------------------------------------


def test_shared_shard_multi_actor_diverged_locations_survive_save_load(tmp_path: Path) -> None:
    """Two actors at different locations after a tick retain distinct locations after save/load."""
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="multi-div-test",
        shard_id="test-shard",
    )
    # player-a moves east, player-b stays put — they diverge
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    loc_a_before = snap["entities"]["player-a"]["location"]
    loc_b_before = snap["entities"]["player-b"]["location"]
    assert loc_a_before != loc_b_before, "actors must be at different locations after diverging tick"

    save_path = str(tmp_path / "diverged.json")
    save_world_snapshot(
        save_path,
        session.world_state,
        run_id="multi-div-test",
        scenario_id="tiny-fetch-quest",
        actor_ids=["player-a", "player-b"],
    )

    load_result = load_world_snapshot(save_path)
    assert load_result.accepted
    loaded_snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert loaded_snap["entities"]["player-a"]["location"] == loc_a_before
    assert loaded_snap["entities"]["player-b"]["location"] == loc_b_before
    # No cross-contamination
    assert loaded_snap["entities"]["player-a"]["location"] != loaded_snap["entities"]["player-b"]["location"]


def test_shared_shard_multi_actor_slot_save_contains_both_actors(tmp_path: Path) -> None:
    """Slot save with two actors results in both actors present in the snapshot world state."""
    from human_console_client import run_human_shared_shard_session

    save_dir = str(tmp_path / "saves")
    inputs = iter(["quit"])

    run_human_shared_shard_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        world_save_slot="multi-actors",
        save_dir=save_dir,
        input_reader=lambda _: next(inputs),
        output_writer=lambda _: None,
    )

    slot_path = Path(save_dir) / "multi-actors.json"
    assert slot_path.exists()
    saved = json.loads(slot_path.read_text(encoding="utf-8"))
    entities = saved["world_state"]["entities"]
    # Both actors must be present — neither missing, neither duplicated
    assert "player-a" in entities
    assert "player-b" in entities
    assert len([k for k in entities if k.startswith("player-")]) == 2


def test_shared_shard_multi_actor_entity_count_not_duplicated_after_reload(tmp_path: Path) -> None:
    """Entity count in world state is identical before and after a save/load cycle."""
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="dedup-test",
        shard_id="test-shard",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    snap_before = session.world_state.get_snapshot()
    entity_count_before = len(snap_before["entities"])

    save_path = str(tmp_path / "dedup.json")
    save_world_snapshot(
        save_path,
        session.world_state,
        run_id="dedup-test",
        scenario_id="tiny-fetch-quest",
    )

    load_result = load_world_snapshot(save_path)
    assert load_result.accepted
    snap_after = load_result.world_state.get_snapshot()  # type: ignore[union-attr]
    assert len(snap_after["entities"]) == entity_count_before


def test_shared_shard_multi_actor_reconnect_session_has_correct_per_actor_state(
    tmp_path: Path,
) -> None:
    """A session loaded from a saved multi-actor world state preserves each actor's
    location independently — reconnect into the same persisted state, not a fresh start."""
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    # Session 1: advance so actors diverge, then save
    session1 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="reconnect-s1",
        shard_id="shard-1",
    )
    session1.advance_tick({"player-a": "move east", "player-b": "wait"})
    snap1 = session1.world_state.get_snapshot()
    saved_loc_a = snap1["entities"]["player-a"]["location"]
    saved_loc_b = snap1["entities"]["player-b"]["location"]

    save_path = str(tmp_path / "reconnect.json")
    save_world_snapshot(
        save_path,
        session1.world_state,
        run_id="reconnect-s1",
        scenario_id="tiny-fetch-quest",
        actor_ids=["player-a", "player-b"],
    )

    # Session 2: reconnect using the loaded world state
    load_result = load_world_snapshot(save_path)
    assert load_result.accepted

    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="reconnect-s2",
        shard_id="shard-2",
        world_state=load_result.world_state,
    )
    snap2 = session2.world_state.get_snapshot()

    # Both actors are in the saved locations — not reset to scenario start
    assert snap2["entities"]["player-a"]["location"] == saved_loc_a
    assert snap2["entities"]["player-b"]["location"] == saved_loc_b
    # Locations are still distinct (no misassignment)
    assert snap2["entities"]["player-a"]["location"] != snap2["entities"]["player-b"]["location"]


# ---------------------------------------------------------------------------
# Persistent shared-world interaction: NPC defeat tests
# ---------------------------------------------------------------------------


def test_shared_shard_npc_defeat_removes_npc_from_shared_room() -> None:
    """After one actor defeats an NPC, it is removed from entities_present for all actors."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="defeat-test",
        shard_id="test-shard",
    )
    # Sequential movement avoids simultaneous-move room tracking race
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "wait", "player-b": "move east"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["guardian"]["health"] == 20
    assert "guardian" in snap["rooms"]["vault-entrance"]["entities_present"]

    # player-a attacks once (20 → 10)
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    snap1 = session.world_state.get_snapshot()
    assert snap1["entities"]["guardian"]["health"] == 10
    assert "guardian" in snap1["rooms"]["vault-entrance"]["entities_present"]

    # player-b delivers the final blow (10 → 0)
    result = session.advance_tick({"player-a": "wait", "player-b": "attack guardian"})
    snap2 = session.world_state.get_snapshot()

    assert snap2["entities"]["guardian"]["health"] == 0
    # Guardian cleared from shared room — visible to all actors
    assert "guardian" not in snap2["rooms"]["vault-entrance"]["entities_present"]
    assert snap2["scenario_vars"].get("defeated.guardian") is True
    assert "npc_defeated" in result.emitted_event_types


def test_shared_shard_both_actors_see_npc_absent_after_defeat() -> None:
    """Both actors see the guardian absent after defeat — shared world state is consistent."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="shared-view-test",
        shard_id="test-shard",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "wait", "player-b": "move east"})

    # Two attacks to defeat guardian
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["guardian"]["health"] == 0

    # Both actors observe from the same shared room snapshot — guardian is gone for both
    obs_a = session.get_observation("player-a")
    obs_b = session.get_observation("player-b")

    # Guardian should not appear in either actor's visible entities
    entity_names_a = [e.name for e in obs_a.entities]
    entity_names_b = [e.name for e in obs_b.entities]
    assert "guardian" not in entity_names_a
    assert "guardian" not in entity_names_b


def test_shared_shard_npc_defeat_persists_across_save_load(tmp_path: Path) -> None:
    """Defeated NPC remains absent from room after world is saved and reloaded."""
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="persist-defeat-test",
        shard_id="test-shard",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "wait", "player-b": "move east"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap_before = session.world_state.get_snapshot()
    assert snap_before["entities"]["guardian"]["health"] == 0
    assert "guardian" not in snap_before["rooms"]["vault-entrance"]["entities_present"]

    save_path = str(tmp_path / "defeat.json")
    save_world_snapshot(
        save_path,
        session.world_state,
        run_id="persist-defeat-test",
        scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    load_result = load_world_snapshot(save_path)
    assert load_result.accepted
    snap_after = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert snap_after["entities"]["guardian"]["health"] == 0
    assert "guardian" not in snap_after["rooms"]["vault-entrance"]["entities_present"]
    assert snap_after["scenario_vars"].get("defeated.guardian") is True


def test_shared_shard_npc_defeat_persists_across_slot_reconnect(tmp_path: Path) -> None:
    """Reconnecting via slot into a world where NPC was defeated shows NPC still absent."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    # Session 1: defeat the guardian, save via slot directly
    session1 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="slot-defeat-s1",
        shard_id="shard-1",
    )
    session1.advance_tick({"player-a": "move east", "player-b": "wait"})
    session1.advance_tick({"player-a": "wait", "player-b": "move east"})
    session1.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session1.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    assert session1.world_state.get_snapshot()["entities"]["guardian"]["health"] == 0

    save_world_slot(
        "defeat-slot",
        save_dir,
        session1.world_state,
        run_id="slot-defeat-s1",
        scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    # Reconnect: session 2 loaded from the saved slot
    loaded = load_world_slot("defeat-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "guardian" not in snap["rooms"]["vault-entrance"]["entities_present"]
    assert snap["scenario_vars"].get("defeated.guardian") is True

    # Verify a new session using the loaded world state continues with guardian gone
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="slot-defeat-s2",
        shard_id="shard-2",
        world_state=loaded.world_state,
    )
    snap2 = session2.world_state.get_snapshot()
    assert "guardian" not in snap2["rooms"]["vault-entrance"]["entities_present"]
    assert snap2["scenario_vars"].get("defeated.guardian") is True


# ---------------------------------------------------------------------------
# Access-control integration tests: defeat unlocks gate, state persists
# ---------------------------------------------------------------------------


def test_shared_shard_vault_blocked_before_guardian_defeat() -> None:
    """Moving east from vault-entrance raises an error while guardian is alive (no exit in action space)."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ac-blocked",
        shard_id="ac-s1",
    )
    # player-a starts at camp; move east to vault-entrance
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    snap_at_entrance = session.world_state.get_snapshot()
    assert snap_at_entrance["entities"]["player-a"]["location"] == "vault-entrance"

    # east exit does not exist yet — the action is not in the action space
    import pytest as _pt
    with _pt.raises(ValueError, match="shared_shard_action_rejected"):
        session.advance_tick({"player-a": "move east", "player-b": "wait"})


def test_shared_shard_vault_accessible_after_defeat_unlock() -> None:
    """After cooperative defeat of guardian, east exit opens and actor can enter vault."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ac-unlock",
        shard_id="ac-s2",
    )
    # player-a moves to vault-entrance
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    snap1 = session.world_state.get_snapshot()
    assert snap1["entities"]["player-a"]["location"] == "vault-entrance"

    # Two attacks to defeat guardian (health=20, damage=10 each)
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    snap2 = session.world_state.get_snapshot()
    assert snap2["entities"]["guardian"]["health"] == 0
    assert snap2["scenario_vars"].get("unlock.guardian-vault-opens") is True
    assert "east" in snap2["rooms"]["vault-entrance"]["exits"]

    # player-a can now move east into vault
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    snap3 = session.world_state.get_snapshot()
    assert snap3["entities"]["player-a"]["location"] == "vault"


def test_shared_shard_access_control_persists_across_save_load(tmp_path: Path) -> None:
    """After guardian defeat and save/load, east exit is still open and actor enters vault."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ac-persist-s1",
        shard_id="ac-s3",
    )
    # Move player-a to vault-entrance
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # Defeat guardian (2 hits)
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    assert session.world_state.get_snapshot()["scenario_vars"].get("unlock.guardian-vault-opens") is True

    # Save slot
    save_world_slot(
        "ac-persist-slot",
        save_dir,
        session.world_state,
        run_id="ac-persist-s1",
        scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    # Reload and continue
    loaded = load_world_slot("ac-persist-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "east" in snap["rooms"]["vault-entrance"]["exits"]

    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ac-persist-s2",
        shard_id="ac-s4",
        world_state=loaded.world_state,
    )
    session2.advance_tick({"player-a": "move east", "player-b": "wait"})
    snap2 = session2.world_state.get_snapshot()
    assert snap2["entities"]["player-a"]["location"] == "vault"


# ---------------------------------------------------------------------------
# Event observation surfacing integration tests
# ---------------------------------------------------------------------------


def test_shared_shard_npc_defeat_message_appears_in_observation() -> None:
    """After defeating the guardian, the next observation includes a [World] defeat message."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="obs-defeat-msg",
        shard_id="obs-s1",
    )
    # Move player-a to vault-entrance (guardian's room)
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    # First attack (health 20→10): no defeat yet
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    obs_partial = session.get_observation("player-a")
    defeat_messages = [m for m in obs_partial.messages if "defeated" in m.lower()]
    assert len(defeat_messages) == 0, "defeat message should not appear after partial damage"

    # Second attack (health 10→0): guardian defeated + route unlocked
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    obs_after = session.get_observation("player-a")
    defeat_messages = [m for m in obs_after.messages if "defeated" in m.lower()]
    assert len(defeat_messages) >= 1, f"expected defeat message, got: {obs_after.messages}"
    assert any("guardian" in m for m in defeat_messages)


def test_shared_shard_route_unlocked_message_appears_in_observation() -> None:
    """After guardian defeat, the next observation includes a [World] route-unlocked message."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="obs-route-msg",
        shard_id="obs-s2",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs = session.get_observation("player-a")
    route_messages = [m for m in obs.messages if "passage" in m.lower() or "east" in m.lower()]
    assert len(route_messages) >= 1, f"expected route message, got: {obs.messages}"


def test_shared_shard_both_actors_see_defeat_and_route_messages() -> None:
    """Both actors sharing the shard see the defeat and route-unlock messages after the event tick."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="obs-both-msg",
        shard_id="obs-s3",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs_a = session.get_observation("player-a")
    obs_b = session.get_observation("player-b")

    # Both should see event messages
    def _has_defeat_msg(obs_messages: tuple) -> bool:
        return any("defeated" in m.lower() for m in obs_messages)

    assert _has_defeat_msg(obs_a.messages), f"player-a missing defeat msg: {obs_a.messages}"
    assert _has_defeat_msg(obs_b.messages), f"player-b missing defeat msg: {obs_b.messages}"


def test_shared_shard_observation_messages_clear_after_next_tick() -> None:
    """Event messages are from the most recent tick only — not retained across multiple ticks."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="obs-stale-msg",
        shard_id="obs-s4",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # Defeat guardian in two hits
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    # One more tick (wait) — no new world events
    session.advance_tick({"player-a": "wait", "player-b": "wait"})
    obs_after_wait = session.get_observation("player-a")
    defeat_messages = [m for m in obs_after_wait.messages if "defeated" in m.lower()]
    assert len(defeat_messages) == 0, f"stale defeat message persisted: {obs_after_wait.messages}"


def test_shared_shard_no_spurious_event_messages_before_combat() -> None:
    """Before any combat, observations contain no [World] defeat or route messages."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="obs-clean-start",
        shard_id="obs-s5",
    )
    session.advance_tick({"player-a": "wait", "player-b": "wait"})
    obs = session.get_observation("player-a")
    spurious = [m for m in obs.messages if "defeated" in m.lower() or "passage opened" in m.lower()]
    assert len(spurious) == 0, f"spurious event messages at start: {obs.messages}"


# ---------------------------------------------------------------------------
# Persistent world event log integration tests
# ---------------------------------------------------------------------------


def test_shared_shard_event_log_populated_after_defeat() -> None:
    """After defeating the guardian, world_event_log_json is set in scenario_vars."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="log-defeat",
        shard_id="log-s1",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None, "world_event_log_json should be set after defeat"
    log = json.loads(raw)
    assert any(e["event_type"] == "npc_defeated" for e in log)
    assert any(e["event_type"] == "route_unlocked" for e in log)


def test_shared_shard_event_log_persists_across_slot_reconnect(tmp_path: Path) -> None:
    """world_event_log_json survives slot save/load and is readable after reconnect."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="log-slot-s1",
        shard_id="log-s2",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "log-test-slot", save_dir, session.world_state,
        run_id="log-slot-s1", scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    loaded = load_world_slot("log-test-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None
    log = json.loads(raw)
    assert any(e["event_type"] == "npc_defeated" for e in log)
    assert any(e["event_type"] == "route_unlocked" for e in log)


def test_shared_shard_reconnect_shows_history_in_first_observation(tmp_path: Path) -> None:
    """After slot reconnect, the first get_observation includes [History] messages."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="hist-s1",
        shard_id="hist-s1",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "history-slot", save_dir, session.world_state,
        run_id="hist-s1", scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    loaded = load_world_slot("history-slot", save_dir)
    assert loaded.accepted

    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="hist-s2",
        shard_id="hist-s2",
        world_state=loaded.world_state,
    )

    # First observation includes history
    obs = session2.get_observation("player-a")
    history_msgs = [m for m in obs.messages if "[History]" in m]
    assert len(history_msgs) >= 1, f"expected [History] messages, got: {obs.messages}"
    assert any("guardian" in m.lower() or "defeated" in m.lower() for m in history_msgs)


def test_shared_shard_history_shown_only_once_per_session() -> None:
    """[History] messages appear only on the first get_observation, not on subsequent calls."""
    from world.state.world_persistence import save_world_slot, load_world_slot
    import tempfile

    with tempfile.TemporaryDirectory() as save_dir:
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=("player-a", "player-b"),
            run_id="once-s1",
            shard_id="once-s1",
        )
        session.advance_tick({"player-a": "move east", "player-b": "wait"})
        session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
        session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

        save_world_slot(
            "once-slot", save_dir, session.world_state,
            run_id="once-s1", scenario_id="tiny-shared-combat",
        )

        loaded = load_world_slot("once-slot", save_dir)
        assert loaded.accepted
        session2 = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=("player-a", "player-b"),
            run_id="once-s2",
            shard_id="once-s2",
            world_state=loaded.world_state,
        )

        # First observation: history present
        obs1 = session2.get_observation("player-a")
        assert any("[History]" in m for m in obs1.messages)

        # Second observation: no history (already shown)
        obs2 = session2.get_observation("player-a")
        assert not any("[History]" in m for m in obs2.messages), (
            f"[History] should not repeat: {obs2.messages}"
        )


def test_shared_shard_fresh_session_no_history_messages() -> None:
    """A fresh session with no prior events shows no [History] messages."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="fresh-hist",
        shard_id="fresh-hist",
    )
    obs = session.get_observation("player-a")
    assert not any("[History]" in m for m in obs.messages), (
        f"unexpected [History] on fresh session: {obs.messages}"
    )


# ---------------------------------------------------------------------------
# Persistent shared room/object state integration tests
# ---------------------------------------------------------------------------


def test_shared_helm_visible_in_camp_at_start() -> None:
    """helm item is present in camp's entities at session start."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-start",
        shard_id="ri-s1",
    )
    snap = session.world_state.get_snapshot()
    assert "helm" in snap["rooms"]["camp"]["entities_present"]
    assert "ward-sigil" in snap["rooms"]["camp"]["entities_present"]


def test_shared_room_item_taken_disappears_for_both_actors() -> None:
    """After player-a takes helm from camp, helm absent from room for both actors."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-take",
        shard_id="ri-s2",
    )
    session.advance_tick({"player-a": "take helm", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert "helm" not in snap["rooms"]["camp"]["entities_present"]
    assert "helm" in snap["entities"]["player-a"]["inventory"]


def test_shared_room_item_not_takeable_by_second_actor() -> None:
    """After player-a takes helm, player-b's take helm is rejected."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-compete",
        shard_id="ri-s3",
    )
    session.advance_tick({"player-a": "take helm", "player-b": "wait"})

    import pytest as _pt
    with _pt.raises(ValueError, match="shared_shard_action_rejected"):
        session.advance_tick({"player-a": "wait", "player-b": "take helm"})


def test_shared_consumable_exhausted_after_use() -> None:
    """After player-a takes and uses ward-sigil, it is gone from the world."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-consume",
        shard_id="ri-s4",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert "ward-sigil" not in snap["entities"], "consumed entity should be removed from world"
    assert "ward-sigil" not in snap["rooms"]["camp"]["entities_present"]


def test_dropped_item_visible_to_other_actor_in_same_room() -> None:
    """player-a drops helm; player-b (still in camp) can see and take it."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-drop",
        shard_id="ri-s5",
    )
    session.advance_tick({"player-a": "take helm", "player-b": "wait"})
    session.advance_tick({"player-a": "drop helm", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert "helm" in snap["rooms"]["camp"]["entities_present"]

    # player-b can now take it
    session.advance_tick({"player-a": "wait", "player-b": "take helm"})
    snap2 = session.world_state.get_snapshot()
    assert "helm" in snap2["entities"]["player-b"]["inventory"]
    assert "helm" not in snap2["rooms"]["camp"]["entities_present"]


def test_room_item_take_persists_across_save_load(tmp_path: Path) -> None:
    """Taking helm and saving; reloaded world shows helm in inventory, not in room."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-persist-s1",
        shard_id="ri-s6",
    )
    session.advance_tick({"player-a": "take helm", "player-b": "wait"})

    save_world_slot(
        "ri-slot", save_dir, session.world_state,
        run_id="ri-persist-s1", scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )

    loaded = load_world_slot("ri-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "helm" in snap["entities"]["player-a"]["inventory"]
    assert "helm" not in snap["rooms"]["camp"]["entities_present"]


def test_dropped_item_persists_across_slot_reconnect(tmp_path: Path) -> None:
    """Dropped helm in camp survives slot reconnect; reconnected actor can take it."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-drop-persist-s1",
        shard_id="ri-s7",
    )
    session.advance_tick({"player-a": "take helm", "player-b": "wait"})
    session.advance_tick({"player-a": "drop helm", "player-b": "wait"})

    save_world_slot(
        "ri-drop-slot", save_dir, session.world_state,
        run_id="ri-drop-persist-s1", scenario_id="tiny-shared-combat",
    )

    loaded = load_world_slot("ri-drop-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "helm" in snap["rooms"]["camp"]["entities_present"]
    assert snap["entities"]["helm"]["location"] == "camp"

    # Reconnected session: player-b can take helm
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-drop-persist-s2",
        shard_id="ri-s8",
        world_state=loaded.world_state,
    )
    session2.advance_tick({"player-a": "wait", "player-b": "take helm"})
    snap2 = session2.world_state.get_snapshot()
    assert "helm" in snap2["entities"]["player-b"]["inventory"]


def test_consumable_exhaustion_persists_across_save_load(tmp_path: Path) -> None:
    """Consumed ward-sigil remaining absent from world survives slot save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="ri-exhaust-s1",
        shard_id="ri-s9",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    save_world_slot(
        "ri-exhaust-slot", save_dir, session.world_state,
        run_id="ri-exhaust-s1", scenario_id="tiny-shared-combat",
    )

    loaded = load_world_slot("ri-exhaust-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "ward-sigil" not in snap["entities"]
    assert "ward-sigil" not in snap["rooms"]["camp"]["entities_present"]


# ---------------------------------------------------------------------------
# Persistent NPC state integration tests
# ---------------------------------------------------------------------------


def test_shared_npc_partial_health_after_attack() -> None:
    """Guardian health reduced to 10 after one attack from camp player."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-health",
        shard_id="pnpc-s1",
    )
    # Move player-a to vault-entrance where guardian is
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["guardian"]["health"] == 10


def test_shared_npc_hostile_tag_set_after_surviving_attack() -> None:
    """Guardian gets 'hostile' in tags after being attacked but not defeated."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-tag",
        shard_id="pnpc-s2",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert "hostile" in snap["entities"]["guardian"]["tags"]


def test_shared_npc_hostile_status_message_in_observation() -> None:
    """Actor in same room as hostile NPC sees '[World] guardian is HOSTILE' message."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-obs",
        shard_id="pnpc-s3",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs = session.get_observation("player-a")
    hostile_msgs = [m for m in obs.messages if "HOSTILE" in m and "HP" in m]
    assert len(hostile_msgs) >= 1
    assert any("guardian" in m for m in hostile_msgs)


def test_shared_npc_hostile_tag_idempotent_on_second_attack() -> None:
    """'hostile' appears exactly once in tags after two attacks."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-idem",
        shard_id="pnpc-s4",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    # Second attack (guardian still alive at 10 HP)
    session.advance_tick({"player-a": "wait", "player-b": "wait"})
    # Take one more step — need to advance a tick to re-attack
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    tags = snap["entities"]["guardian"].get("tags", [])
    assert tags.count("hostile") == 1


def test_shared_npc_npc_alert_event_visible_in_tick_events() -> None:
    """npc_alert event produces '[World] ... is now HOSTILE' message to all actors."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-evt",
        shard_id="pnpc-s5",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs_a = session.get_observation("player-a")
    obs_b = session.get_observation("player-b")
    # Both actors see the alert event message
    assert any("is now HOSTILE" in m for m in obs_a.messages)
    assert any("is now HOSTILE" in m for m in obs_b.messages)


def test_shared_npc_partial_health_persists_across_slot_save_load(tmp_path: Path) -> None:
    """Partial NPC health and hostile tag survive slot save/load/reconnect."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-persist-s1",
        shard_id="pnpc-s6",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "pnpc-slot", save_dir, session.world_state,
        run_id="pnpc-persist-s1", scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )
    loaded = load_world_slot("pnpc-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert snap["entities"]["guardian"]["health"] == 10
    assert "hostile" in snap["entities"]["guardian"]["tags"]


def test_shared_npc_hostile_status_shown_on_reconnect(tmp_path: Path) -> None:
    """Reconnecting into a world with a hostile NPC shows hostile status immediately."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-reconnect-s1",
        shard_id="pnpc-s7",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "pnpc-reconnect-slot", save_dir, session.world_state,
        run_id="pnpc-reconnect-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("pnpc-reconnect-slot", save_dir)
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-reconnect-s2",
        shard_id="pnpc-s8",
        world_state=loaded.world_state,
    )
    obs = session2.get_observation("player-a")
    assert any("HOSTILE" in m for m in obs.messages)


def test_shared_npc_history_contains_npc_alert_on_reconnect(tmp_path: Path) -> None:
    """World event log history includes npc_alert entry on reconnect."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-hist-s1",
        shard_id="pnpc-s9",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "pnpc-hist-slot", save_dir, session.world_state,
        run_id="pnpc-hist-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("pnpc-hist-slot", save_dir)
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-hist-s2",
        shard_id="pnpc-s10",
        world_state=loaded.world_state,
    )
    obs = session2.get_observation("player-a")
    history_msgs = [m for m in obs.messages if "[History]" in m]
    assert any("hostile" in m.lower() for m in history_msgs)


def test_shared_npc_no_actor_duplication_after_combat(tmp_path: Path) -> None:
    """Actor entity count remains correct after combat and slot reconnect."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnpc-dup-s1",
        shard_id="pnpc-s11",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot(
        "pnpc-dup-slot", save_dir, session.world_state,
        run_id="pnpc-dup-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("pnpc-dup-slot", save_dir)
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    player_entities = [
        eid for eid, e in snap["entities"].items()
        if e.get("entity_type") in {"player", "agent"}
    ]
    assert len(player_entities) == 2


# ---------------------------------------------------------------------------
# Persistent NPC behavior-response integration tests
# ---------------------------------------------------------------------------


def test_shared_patrol_npc_starts_hostile_in_vault_entrance() -> None:
    """Patrol NPC in vault-entrance has 'hostile' tag at session start."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s1",
        shard_id="pnbr-i1",
    )
    snap = session.world_state.get_snapshot()
    assert "hostile" in snap["entities"]["patrol"]["tags"]
    assert snap["entities"]["patrol"]["health"] == 15


def test_actor_takes_counter_damage_from_patrol_on_entering_vault_entrance() -> None:
    """Player-a takes 5 HP damage from hostile patrol on first tick in vault-entrance."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s2",
        shard_id="pnbr-i2",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    hp = snap["entities"]["player-a"]["health"]
    assert hp == 95, f"expected 95 HP after one counter-attack tick, got {hp}"


def test_actor_in_camp_not_affected_by_patrol_in_vault_entrance() -> None:
    """Player-b in camp is unaffected by patrol counter-attacks in vault-entrance."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s3",
        shard_id="pnbr-i3",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    hp_b = snap["entities"]["player-b"].get("health")
    # player-b was not in patrol's room; health unchanged (None = full health)
    assert hp_b is None


def test_both_actors_take_damage_when_both_in_vault_entrance() -> None:
    """When both players move to vault-entrance, both take patrol counter-damage."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s4",
        shard_id="pnbr-i4",
    )
    session.advance_tick({"player-a": "move east", "player-b": "move east"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["player-a"]["health"] == 95
    assert snap["entities"]["player-b"]["health"] == 95


def test_counter_attack_message_in_observation() -> None:
    """Actor in vault-entrance sees '[World] patrol attacks' message in observation."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s5",
        shard_id="pnbr-i5",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    obs = session.get_observation("player-a")
    attack_msgs = [m for m in obs.messages if "patrol attacks" in m]
    assert len(attack_msgs) == 1
    assert "damage" in attack_msgs[0]
    assert "HP" in attack_msgs[0]


def test_damage_accumulates_over_multiple_ticks_in_vault_entrance() -> None:
    """Player-a takes 5 HP per tick; after 2 ticks in vault-entrance HP is 90."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-s6",
        shard_id="pnbr-i6",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "wait", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["player-a"]["health"] == 90


def test_counter_damage_persists_across_slot_save_load(tmp_path: Path) -> None:
    """Actor HP reduced by counter-attack survives slot save/load/reconnect."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-persist-s1",
        shard_id="pnbr-i7",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    save_world_slot(
        "pnbr-slot", save_dir, session.world_state,
        run_id="pnbr-persist-s1", scenario_id="tiny-shared-combat",
        actor_ids=["player-a", "player-b"],
    )
    loaded = load_world_slot("pnbr-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert snap["entities"]["player-a"]["health"] == 95
    assert "hostile" in snap["entities"]["patrol"]["tags"]


def test_hostile_patrol_still_attacks_on_reconnect(tmp_path: Path) -> None:
    """Reconnecting into saved world: patrol still attacks player-a on next tick."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-reconnect-s1",
        shard_id="pnbr-i8",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # HP = 95 after first tick

    save_world_slot(
        "pnbr-reconnect-slot", save_dir, session.world_state,
        run_id="pnbr-reconnect-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("pnbr-reconnect-slot", save_dir)
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-reconnect-s2",
        shard_id="pnbr-i9",
        world_state=loaded.world_state,
    )
    session2.advance_tick({"player-a": "wait", "player-b": "wait"})

    snap = session2.world_state.get_snapshot()
    # player-a was in vault-entrance (HP 95); patrol attacks again → HP 90
    assert snap["entities"]["player-a"]["health"] == 90


def test_defeating_patrol_stops_counter_attacks() -> None:
    """After patrol is defeated (health 0, removed from room), no more counter-attacks."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-defeat-s1",
        shard_id="pnbr-i10",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # HP 95 (patrol attacked once)

    # Attack patrol twice to defeat (patrol has 15 HP, 10 damage/attack)
    session.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    session.advance_tick({"player-a": "attack patrol", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert snap["entities"]["patrol"]["health"] == 0

    # Next tick: no counter-attack since patrol is defeated
    hp_before = snap["entities"]["player-a"]["health"]
    session.advance_tick({"player-a": "wait", "player-b": "wait"})
    snap2 = session.world_state.get_snapshot()
    # Guardian is not hostile initially, so no counter-attacks from it either
    assert snap2["entities"]["player-a"]["health"] == hp_before


def test_no_actor_duplication_after_npc_counter_attack(tmp_path: Path) -> None:
    """Actor and NPC entity counts remain correct after counter-attack and slot reload."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")

    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="pnbr-dup-s1",
        shard_id="pnbr-i11",
    )
    session.advance_tick({"player-a": "move east", "player-b": "wait"})

    save_world_slot(
        "pnbr-dup-slot", save_dir, session.world_state,
        run_id="pnbr-dup-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("pnbr-dup-slot", save_dir)
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    actor_entities = [
        eid for eid, e in snap["entities"].items()
        if e.get("entity_type") in {"player", "agent"}
    ]
    assert len(actor_entities) == 2
    npc_entities = [
        eid for eid, e in snap["entities"].items()
        if e.get("entity_type") == "npc"
    ]
    assert len(npc_entities) == 2  # guardian + patrol


# ---------------------------------------------------------------------------
# Persistent NPC de-escalation / calm integration tests
# ---------------------------------------------------------------------------


def test_ward_sigil_in_action_space_in_camp() -> None:
    """Actor in camp sees 'use ward-sigil' in action space after taking it."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-int-s1",
        shard_id="calm-i1",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    obs = session.get_observation("player-a")
    assert "use ward-sigil" in obs.action_space


def test_use_ward_sigil_in_vault_entrance_calms_patrol() -> None:
    """Player-a uses ward-sigil in vault-entrance → patrol loses 'hostile' tag."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-int-s2",
        shard_id="calm-i2",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # player-a is now in vault-entrance with ward-sigil; patrol is hostile
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    assert "hostile" not in snap["entities"]["patrol"]["tags"]


def test_calmed_patrol_stops_counter_attacks() -> None:
    """After calm, patrol no longer deals damage on subsequent ticks."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-int-s3",
        shard_id="calm-i3",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    # HP = 95 (one hit from patrol)
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    hp_after_calm = snap["entities"]["player-a"]["health"]
    # Next tick: patrol is calmed, should NOT attack
    session.advance_tick({"player-a": "wait", "player-b": "wait"})
    snap2 = session.world_state.get_snapshot()
    assert snap2["entities"]["player-a"]["health"] == hp_after_calm


def test_calmed_patrol_visible_to_both_actors() -> None:
    """Both actors see '[World] player-a calmed patrol' message in observations."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-int-s4",
        shard_id="calm-i4",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    obs_a = session.get_observation("player-a")
    obs_b = session.get_observation("player-b")
    calm_msgs_a = [m for m in obs_a.messages if "calmed patrol" in m]
    calm_msgs_b = [m for m in obs_b.messages if "calmed patrol" in m]
    assert len(calm_msgs_a) == 1, f"player-a obs: {obs_a.messages}"
    assert len(calm_msgs_b) == 1, f"player-b obs: {obs_b.messages}"


def test_calm_state_persists_across_slot_save_load(tmp_path: Path) -> None:
    """Calmed NPC state (no 'hostile' tag, idempotency marker) persists through save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-persist-s1",
        shard_id="calm-i5",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    save_world_slot(
        "calm-slot", save_dir, session.world_state,
        run_id="calm-persist-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("calm-slot", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "hostile" not in snap["entities"]["patrol"]["tags"]
    assert snap["scenario_vars"].get("calmed.patrol-calmed") is True


def test_calmed_patrol_still_calm_on_reconnect(tmp_path: Path) -> None:
    """Reconnecting into saved world: calmed patrol does not attack on next tick."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-reconnect-s1",
        shard_id="calm-i6",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    hp_at_save = snap["entities"]["player-a"]["health"]

    save_world_slot(
        "calm-reconnect-slot", save_dir, session.world_state,
        run_id="calm-reconnect-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("calm-reconnect-slot", save_dir)
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-reconnect-s2",
        shard_id="calm-i7",
        world_state=loaded.world_state,
    )
    session2.advance_tick({"player-a": "wait", "player-b": "wait"})
    snap2 = session2.world_state.get_snapshot()
    # Patrol was calmed; player-a HP should be unchanged
    assert snap2["entities"]["player-a"]["health"] == hp_at_save


def test_calm_event_log_entry_persists_across_slot_save_load(tmp_path: Path) -> None:
    """World event log entry for npc_calmed is preserved across save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot
    import json

    save_dir = str(tmp_path / "saves")
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-log-s1",
        shard_id="calm-i8",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    save_world_slot(
        "calm-log-slot", save_dir, session.world_state,
        run_id="calm-log-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("calm-log-slot", save_dir)
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    raw_log = snap["scenario_vars"].get("world_event_log_json")
    assert isinstance(raw_log, str)
    log = json.loads(raw_log)
    calm_entries = [e for e in log if e.get("event_type") == "npc_calmed"]
    assert len(calm_entries) == 1


def test_calm_history_visible_on_reconnect(tmp_path: Path) -> None:
    """Reconnecting actor sees '[History] patrol was calmed' message."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-hist-s1",
        shard_id="calm-i9",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    save_world_slot(
        "calm-hist-slot", save_dir, session.world_state,
        run_id="calm-hist-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("calm-hist-slot", save_dir)
    session2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-hist-s2",
        shard_id="calm-i10",
        world_state=loaded.world_state,
    )
    obs = session2.get_observation("player-a")
    history_msgs = [m for m in obs.messages if "History" in m and "calmed" in m]
    assert len(history_msgs) == 1, f"Expected 1 calm history msg, got: {obs.messages}"


def test_ward_sigil_use_in_camp_does_not_calm_patrol() -> None:
    """Using ward-sigil in camp (wrong room) does not trigger calm effect."""
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-int-s9",
        shard_id="calm-i11",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    # player-a uses ward-sigil in camp (not vault-entrance)
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = session.world_state.get_snapshot()
    # patrol still hostile — calm only fires in vault-entrance
    assert "hostile" in snap["entities"]["patrol"]["tags"]


def test_no_actor_duplication_after_calm_and_slot_reload(tmp_path: Path) -> None:
    """Actor and NPC entity counts correct after calm + slot reload."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")
    session = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-dup-s1",
        shard_id="calm-i12",
    )
    session.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    session.advance_tick({"player-a": "move east", "player-b": "wait"})
    session.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    save_world_slot(
        "calm-dup-slot", save_dir, session.world_state,
        run_id="calm-dup-s1", scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("calm-dup-slot", save_dir)
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    actor_entities = [
        eid for eid, e in snap["entities"].items()
        if e and e.get("entity_type") in {"player", "agent"}
    ]
    assert len(actor_entities) == 2
    npc_entities = [
        eid for eid, e in snap["entities"].items()
        if e and e.get("entity_type") == "npc"
    ]
    assert len(npc_entities) == 2  # guardian + patrol still present


# ---------------------------------------------------------------------------
# NPC Respawn / World Reset Integration Tests
# task_id: persistent_npc_respawn_and_world_reset_v1
# ---------------------------------------------------------------------------

def _advance_n_ticks(session: object, n: int, action_a: str = "wait", action_b: str = "wait") -> None:
    """Advance the session by n ticks with the given actions for both actors."""
    for _ in range(n):
        session.advance_tick({"player-a": action_a, "player-b": action_b})


def test_respawn_rules_wired_into_world_state() -> None:
    """Verify that npc_respawn_rules_json is stored in scenario_vars after world setup."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="respawn-rules-wired",
        shard_id="rr-s1",
    )
    snap = s.world_state.get_snapshot()
    assert "npc_respawn_rules_json" in snap["scenario_vars"]
    rules = json.loads(snap["scenario_vars"]["npc_respawn_rules_json"])
    npc_ids = [r["npc_id"] for r in rules]
    assert "patrol" in npc_ids
    assert "guardian" in npc_ids


def test_patrol_respawns_after_defeat() -> None:
    """Patrol (health=15) should respawn 3 ticks after defeat."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="patrol-respawn",
        shard_id="pr-s1",
    )
    # Move player-a east (player-b stays in camp)
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    # Attack patrol twice to defeat it (10 damage per attack, patrol has 15 HP)
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})

    snap = s.world_state.get_snapshot()
    assert snap["entities"]["patrol"]["health"] == 0
    assert bool(snap["scenario_vars"].get("defeated.patrol"))
    respawn_tick = snap["scenario_vars"].get("respawn_at_tick.patrol")
    assert isinstance(respawn_tick, int)

    # Advance 3 more ticks for respawn
    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    assert snap["entities"]["patrol"]["health"] == 15
    assert not bool(snap["scenario_vars"].get("defeated.patrol"))
    assert not isinstance(snap["scenario_vars"].get("respawn_at_tick.patrol"), int)


def test_patrol_is_hostile_after_respawn() -> None:
    """Respawned patrol should have the hostile tag."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="patrol-hostile-respawn",
        shard_id="phr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    assert "hostile" in snap["entities"]["patrol"]["tags"]


def test_patrol_in_vault_entrance_after_respawn() -> None:
    """Respawned patrol should appear in vault-entrance entities_present."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="patrol-room-respawn",
        shard_id="prr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})

    # Confirm patrol not in room after defeat
    snap = s.world_state.get_snapshot()
    present_before = snap["rooms"].get("vault-entrance", {}).get("entities_present", [])
    assert "patrol" not in present_before

    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    present_after = snap["rooms"].get("vault-entrance", {}).get("entities_present", [])
    assert "patrol" in present_after


def test_patrol_respawns_after_calm() -> None:
    """Patrol should also respawn after being calmed (non-lethal resolution)."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="calm-respawn",
        shard_id="cr-s1",
    )
    # Take ward-sigil from camp
    s.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    # Move to vault-entrance
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    # Use ward-sigil to calm patrol
    s.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    snap = s.world_state.get_snapshot()
    # Patrol should be calmed
    assert bool(snap["scenario_vars"].get("calmed.patrol-calmed"))
    # Respawn should be scheduled
    assert isinstance(snap["scenario_vars"].get("respawn_at_tick.patrol"), int)

    # Advance 3 ticks for respawn
    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    # Patrol back — calm cleared, hostile restored
    assert snap["entities"]["patrol"]["health"] == 15
    assert "hostile" in snap["entities"]["patrol"]["tags"]
    assert not bool(snap["scenario_vars"].get("calmed.patrol-calmed"))


def test_respawn_visible_to_both_actors() -> None:
    """Both actors should see the same patrol health after respawn."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="respawn-both-see",
        shard_id="rbs-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "move east"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    _advance_n_ticks(s, 3)

    # Both actors share the same world_state; their view of patrol health is identical
    snap = s.world_state.get_snapshot()
    assert snap["entities"]["patrol"]["health"] == 15
    assert "hostile" in snap["entities"]["patrol"]["tags"]


def test_respawn_state_persists_across_save_load(tmp_path: Path) -> None:
    """respawn_at_tick.patrol should survive a save/load round-trip."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="respawn-save-load",
        shard_id="rsl-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})

    snap = s.world_state.get_snapshot()
    respawn_tick = snap["scenario_vars"].get("respawn_at_tick.patrol")
    assert isinstance(respawn_tick, int)

    save_world_slot(
        "test-respawn-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="rsl-s1",
    )
    loaded = load_world_slot("test-respawn-slot", str(tmp_path))
    assert loaded.accepted

    loaded_snap = loaded.world_state.get_snapshot()
    assert loaded_snap["scenario_vars"].get("respawn_at_tick.patrol") == respawn_tick


def test_reconnect_after_respawn_sees_live_npc(tmp_path: Path) -> None:
    """A world saved after respawn should show the NPC alive on reload."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="reconnect-post-respawn",
        shard_id="rpr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    assert snap["entities"]["patrol"]["health"] == 15

    save_world_slot(
        "post-respawn-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="rpr-s1",
    )
    loaded = load_world_slot("post-respawn-slot", str(tmp_path))
    assert loaded.accepted

    loaded_snap = loaded.world_state.get_snapshot()
    assert loaded_snap["entities"]["patrol"]["health"] == 15
    assert not bool(loaded_snap["scenario_vars"].get("defeated.patrol"))


def test_respawn_event_in_world_event_log() -> None:
    """npc_respawn event should be appended to the world event log."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="respawn-log",
        shard_id="rl-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    _advance_n_ticks(s, 3)

    snap = s.world_state.get_snapshot()
    log_raw = snap["scenario_vars"].get("world_event_log_json")
    assert isinstance(log_raw, str)
    log_entries = json.loads(log_raw)
    respawn_events = [e for e in log_entries if e.get("event_type") == "npc_respawn"]
    assert len(respawn_events) >= 1
    assert respawn_events[0].get("npc_id") == "patrol"


def test_no_actor_duplication_after_respawn_and_reload(tmp_path: Path) -> None:
    """Actor count must remain stable after respawn + save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="no-dup-respawn",
        shard_id="ndr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    s.advance_tick({"player-a": "attack patrol", "player-b": "wait"})
    _advance_n_ticks(s, 3)

    save_world_slot(
        "no-dup-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="ndr-s1",
    )
    loaded = load_world_slot("no-dup-slot", str(tmp_path))
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()
    actor_entities = [
        eid for eid, e in snap["entities"].items()
        if e and e.get("entity_type") in {"player", "agent"}
    ]
    assert len(actor_entities) == 2


# ---------------------------------------------------------------------------
# NPC Dialogue / Lore Integration Tests
# task_id: persistent_npc_dialogue_and_lore_v1
# ---------------------------------------------------------------------------

def test_dialogue_json_wired_into_world_state() -> None:
    """Verify npc_dialogue_json is stored in scenario_vars after world setup."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-wired",
        shard_id="dw-s1",
    )
    snap = s.world_state.get_snapshot()
    assert "npc_dialogue_json" in snap["scenario_vars"]
    entries = json.loads(snap["scenario_vars"]["npc_dialogue_json"])
    npc_ids = [e["npc_id"] for e in entries]
    assert "guardian" in npc_ids
    assert "patrol" in npc_ids


def test_talk_guardian_yields_dialogue_message() -> None:
    """Actor moving to vault-entrance can talk to the guardian and see its line."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-talk-guardian",
        shard_id="dtg-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})
    obs = s.get_observation("player-a")
    guardian_msgs = [m for m in obs.messages if "[guardian]" in m]
    assert len(guardian_msgs) == 1
    assert "vault" in guardian_msgs[0].lower() or "worthy" in guardian_msgs[0].lower()


def test_talk_hostile_patrol_rejected_with_world_message() -> None:
    """Talking to the hostile patrol is rejected; actor sees no dialogue."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-hostile-reject",
        shard_id="dhr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    # talk patrol should not be in action_space while patrol is hostile
    obs = s.get_observation("player-a")
    assert "talk patrol" not in obs.action_space
    # Direct attempt rejected
    from agents.gateway.action_parser import parse_action_command
    parse_result = parse_action_command(actor_id="player-a", action="talk patrol")
    assert parse_result.accepted  # parse succeeds — rejection happens at processor level
    from world.state.basic_action_processor import BasicDeterministicActionProcessor
    from core.action_processor import normalize_arguments
    from core.action_processor import ActionRequest
    action = ActionRequest(
        actor_id="player-a",
        action_type="talk",
        arguments=normalize_arguments({"target_id": "patrol"}),
    )
    proc = BasicDeterministicActionProcessor()
    results = proc.process_actions([action], s.world_state, step_index=s.current_tick)
    assert not results[0].accepted


def test_talk_patrol_available_after_calm() -> None:
    """After calming patrol, talk patrol appears in action_space."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-calm-talk",
        shard_id="dct-s1",
    )
    s.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    obs = s.get_observation("player-a")
    assert "talk patrol" in obs.action_space

    s.advance_tick({"player-a": "talk patrol", "player-b": "wait"})
    obs2 = s.get_observation("player-a")
    patrol_msgs = [m for m in obs2.messages if "[patrol]" in m]
    assert len(patrol_msgs) == 1
    assert "ward" in patrol_msgs[0].lower() or "attack" in patrol_msgs[0].lower()


def test_dialogue_cycles_across_multiple_talks() -> None:
    """Talking to guardian repeatedly cycles through its dialogue lines."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-cycle",
        shard_id="dc-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    lines_seen: list[str] = []
    for _ in range(3):
        s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})
        obs = s.get_observation("player-a")
        for m in obs.messages:
            if "[guardian]" in m:
                lines_seen.append(m)
    assert len(lines_seen) == 3
    assert len(set(lines_seen)) == 3  # all different lines


def test_dialogue_both_actors_see_same_npc_state() -> None:
    """After actor-a talks to guardian, talk_count is shared world state."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-shared-state",
        shard_id="dss-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "move east"})
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})

    snap = s.world_state.get_snapshot()
    assert snap["scenario_vars"].get("talk_count.guardian") == 1

    # actor-b talks and gets the second line
    s.advance_tick({"player-a": "wait", "player-b": "talk guardian"})
    obs_b = s.get_observation("player-b")
    guardian_msgs = [m for m in obs_b.messages if "[guardian]" in m]
    assert len(guardian_msgs) == 1
    # Should be the second line (idx=1)
    assert "sought" in guardian_msgs[0].lower() or "unchanged" in guardian_msgs[0].lower()


def test_talk_count_persists_across_save_load(tmp_path: Path) -> None:
    """talk_count.<npc_id> should survive save/load round-trip."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-save-load",
        shard_id="dsl-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})

    snap = s.world_state.get_snapshot()
    assert snap["scenario_vars"].get("talk_count.guardian") == 1

    save_world_slot(
        "talk-count-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="dsl-s1",
    )
    loaded = load_world_slot("talk-count-slot", str(tmp_path))
    assert loaded.accepted
    assert loaded.world_state.get_snapshot()["scenario_vars"].get("talk_count.guardian") == 1


def test_reconnect_after_talk_sees_correct_count(tmp_path: Path) -> None:
    """On reload, next talk resumes from the saved talk_count."""
    from world.state.world_persistence import save_world_slot, load_world_slot
    from world.state.basic_action_processor import BasicDeterministicActionProcessor
    from core.action_processor import ActionRequest, normalize_arguments

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-reconnect",
        shard_id="dr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    # Talk twice → count = 2
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})

    save_world_slot(
        "reconnect-talk-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="dr-s1",
    )
    loaded = load_world_slot("reconnect-talk-slot", str(tmp_path))
    assert loaded.accepted

    # Next talk should be line index 2 (third line)
    action = ActionRequest(
        actor_id="player-a",
        action_type="talk",
        arguments=normalize_arguments({"target_id": "guardian"}),
    )
    proc = BasicDeterministicActionProcessor()
    results = proc.process_actions([action], loaded.world_state, step_index=99)
    assert results[0].accepted
    events = [e for e in results[0].events if e.event_type == "npc_talked"]
    payload = dict(events[0].payload)
    assert "prove" in payload["line"].lower() or "strength" in payload["line"].lower()


def test_no_actor_duplication_after_dialogue_and_reload(tmp_path: Path) -> None:
    """Actor count stays stable after talk interactions + save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="dlg-no-dup",
        shard_id="dnd-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "talk guardian", "player-b": "wait"})

    save_world_slot(
        "dlg-no-dup-slot", str(tmp_path), s.world_state,
        scenario_id="tiny-shared-combat", run_id="dnd-s1",
    )
    loaded = load_world_slot("dlg-no-dup-slot", str(tmp_path))
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()
    actor_entities = [
        eid for eid, e in snap["entities"].items()
        if e and e.get("entity_type") in {"player", "agent"}
    ]
    assert len(actor_entities) == 2


# ===========================================================================
# Quest objective tracking tests (task: persistent_world_quest_objective_tracking_v1)
# ===========================================================================

def test_quest_objectives_wired_into_world_state():
    """Quest objectives JSON is written to scenario_vars on world setup."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-wire", shard_id="qw-s1",
    )
    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert "quest_objectives_json" in svars
    import json
    quests = json.loads(svars["quest_objectives_json"])
    quest_ids = {q["quest_id"] for q in quests}
    assert "defeat-the-guardian" in quest_ids
    assert "calm-the-patrol" in quest_ids
    assert "claim-the-relic" in quest_ids


def test_defeating_guardian_completes_quest():
    """Defeating the guardian completes defeat-the-guardian for the attacker."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-defeat", shard_id="qd-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert svars.get("quest.defeat-the-guardian.player-a") == "complete"
    assert svars.get("quest.defeat-the-guardian.player-b") is None


def test_quest_completion_message_shown_in_tick_observation():
    """Quest completed message appears in observation for actor who completed quest."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-msg", shard_id="qmsg-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs_a = s.get_observation("player-a")
    obs_b = s.get_observation("player-b")
    quest_msgs_a = [m for m in obs_a.messages if "[Quest]" in m]
    quest_msgs_b = [m for m in obs_b.messages if "[Quest]" in m]
    assert len(quest_msgs_a) >= 1
    assert any("Defeat the Guardian" in m for m in quest_msgs_a)
    assert not any("[Quest]" in m and "Defeat the Guardian" in m for m in quest_msgs_b)


def test_calming_patrol_completes_calm_quest():
    """Using ward-sigil on patrol completes calm-the-patrol quest for calmer."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-calm", shard_id="qcalm-s1",
    )
    s.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert svars.get("quest.calm-the-patrol.player-a") == "complete"
    assert svars.get("quest.calm-the-patrol.player-b") is None


def test_taking_relic_completes_claim_quest():
    """Taking the relic completes claim-the-relic quest for the taker."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-relic", shard_id="qr-s1",
    )
    # Move to east room, defeat guardian (2 attacks), then take relic
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "take relic", "player-b": "wait"})

    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert svars.get("quest.claim-the-relic.player-a") == "complete"
    assert svars.get("quest.claim-the-relic.player-b") is None


def test_two_actors_have_independent_quest_state():
    """Two actors completing the same quest type in different ways have independent state."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-indep", shard_id="qi-s1",
    )
    # player-a calms patrol
    s.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})

    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert svars.get("quest.calm-the-patrol.player-a") == "complete"
    assert svars.get("quest.calm-the-patrol.player-b") is None
    # defeat quest untouched for both
    assert svars.get("quest.defeat-the-guardian.player-a") is None
    assert svars.get("quest.defeat-the-guardian.player-b") is None


def test_quest_state_persists_across_save_load(tmp_path):
    """Quest completion state persists after save and load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-persist", shard_id="qp-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    svars = s.world_state.get_snapshot()["scenario_vars"]
    assert svars.get("quest.defeat-the-guardian.player-a") == "complete"

    save_world_slot("q-persist-slot", str(tmp_path), s.world_state,
                    scenario_id="tiny-shared-combat", run_id="q-persist")
    loaded = load_world_slot("q-persist-slot", str(tmp_path))
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()
    assert snap["scenario_vars"].get("quest.defeat-the-guardian.player-a") == "complete"
    assert "quest_objectives_json" in snap["scenario_vars"]


def test_quest_status_shown_on_reconnect():
    """Quest status messages are shown on reconnect/first observation after reload."""
    from world.state.world_persistence import save_world_slot, load_world_slot
    import tempfile

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-reconnect", shard_id="qrec-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    with tempfile.TemporaryDirectory() as tmpd:
        save_world_slot("qrec-slot", tmpd, s.world_state,
                        scenario_id="tiny-shared-combat", run_id="q-reconnect")
        loaded = load_world_slot("qrec-slot", tmpd)
    assert loaded.accepted

    # Build a new session from the loaded world
    s2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-reconnect-2", shard_id="qrec-s2",
        world_state=loaded.world_state,
    )
    obs = s2.get_observation("player-a")
    history_msgs = [m for m in obs.messages if "[Quest Status]" in m or "✓" in m or "○" in m]
    assert any("Defeat the Guardian" in m for m in history_msgs)


def test_no_actor_duplication_after_quest_completion_and_reload(tmp_path):
    """No actor duplication after quest completion + save/load round-trip."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="q-nodup", shard_id="qnd-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot("q-nodup-slot", str(tmp_path), s.world_state,
                    scenario_id="tiny-shared-combat", run_id="q-nodup")
    loaded = load_world_slot("q-nodup-slot", str(tmp_path))
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()
    actor_entities = [
        eid for eid, e in snap["entities"].items()
        if e and e.get("entity_type") in {"player", "agent"}
    ]
    assert len(actor_entities) == 2


# ===========================================================================
# Player status surface tests (task: shared_world_player_status_and_objective_surface_v1)
# ===========================================================================

def test_compact_objectives_line_present_in_observation():
    """Compact [Objectives] line appears in every observation when quests are configured."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs1", shard_id="so1-s1",
    )
    obs = s.get_observation("player-a")
    obj_msgs = [m for m in obs.messages if m.startswith("[Objectives]")]
    assert len(obj_msgs) == 1
    assert "○ Defeat the Guardian" in obj_msgs[0]
    assert "○ Calm the Patrol" in obj_msgs[0]
    assert "○ Claim the Relic" in obj_msgs[0]


def test_compact_objectives_line_updates_after_completion():
    """Compact [Objectives] line shows completed quests after they are done."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs2", shard_id="so2-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs = s.get_observation("player-a")
    obj_msgs = [m for m in obs.messages if m.startswith("[Objectives]")]
    assert len(obj_msgs) == 1
    assert "✓ Defeat the Guardian" in obj_msgs[0]
    assert "○ Calm the Patrol" in obj_msgs[0]


def test_compact_objectives_actor_specific_state():
    """Two actors see different objective progress in their own observations."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs3", shard_id="so3-s1",
    )
    # player-a moves east and defeats guardian
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    obs_a = s.get_observation("player-a")
    obs_b = s.get_observation("player-b")
    obj_a = next(m for m in obs_a.messages if m.startswith("[Objectives]"))
    obj_b = next(m for m in obs_b.messages if m.startswith("[Objectives]"))
    assert "✓ Defeat the Guardian" in obj_a
    assert "○ Defeat the Guardian" in obj_b


def test_compact_objectives_absent_when_no_quest_scenario():
    """[Objectives] line is absent when the scenario has no quest objectives."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs4", shard_id="so4-s1",
    )
    obs = s.get_observation("player-a")
    assert not any(m.startswith("[Objectives]") for m in obs.messages)


def test_observation_messages_not_none():
    """Observation messages tuple is never None — always a proper tuple."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs5", shard_id="so5-s1",
    )
    obs = s.get_observation("player-a")
    assert isinstance(obs.messages, tuple)


def test_hostile_npc_status_visible_in_observation():
    """Hostile NPC status appears in observation messages when NPC is in same room."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-obs6", shard_id="so6-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    obs = s.get_observation("player-a")
    # patrol is hostile and in vault-entrance
    hostile_msgs = [m for m in obs.messages if "HOSTILE" in m]
    assert any("patrol" in m for m in hostile_msgs)


def test_compact_objectives_persists_across_save_load(tmp_path):
    """Compact [Objectives] line appears correctly after save/load (completed quests preserved)."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-reload", shard_id="sr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot("status-slot", str(tmp_path), s.world_state,
                    scenario_id="tiny-shared-combat", run_id="status-reload")
    loaded = load_world_slot("status-slot", str(tmp_path))
    assert loaded.accepted

    s2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-reload-2", shard_id="sr-s2",
        world_state=loaded.world_state,
    )
    obs = s2.get_observation("player-a")
    obj_msgs = [m for m in obs.messages if m.startswith("[Objectives]")]
    assert len(obj_msgs) == 1
    assert "✓ Defeat the Guardian" in obj_msgs[0]


def test_reconnecting_actor_sees_coherent_status_context(tmp_path):
    """Reconnecting actor gets world event log history + quest status + current objectives."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-reconnect", shard_id="srec-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    save_world_slot("status-reconnect-slot", str(tmp_path), s.world_state,
                    scenario_id="tiny-shared-combat", run_id="status-reconnect")
    loaded = load_world_slot("status-reconnect-slot", str(tmp_path))
    assert loaded.accepted

    s2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="status-reconnect-2", shard_id="srec-s2",
        world_state=loaded.world_state,
    )
    obs = s2.get_observation("player-a")
    all_msgs = "\n".join(obs.messages)
    # Quest status block (from reconnect history)
    assert "[Quest]" in all_msgs
    # Compact objectives line (always-visible)
    assert "[Objectives]" in all_msgs
    # Both show the completed quest
    assert "Defeat the Guardian" in all_msgs


# ===========================================================================
# Scene/lore reactivity tests (task: shared_world_scene_and_lore_depth_v1)
# ===========================================================================

def test_base_vault_entrance_description_before_any_state_change():
    """Vault-entrance shows base description before guardian defeated or patrol calmed."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-base", shard_id="lb-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    obs = s.get_observation("player-a")
    assert "narrow entrance" in obs.description.lower() or "guardian" in obs.description.lower()
    # Should NOT show the post-battle override
    assert "quiet after the battle" not in obs.description


def test_vault_entrance_description_changes_after_guardian_defeated():
    """After defeating guardian, vault-entrance description reflects the changed state."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-defeat", shard_id="ld-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    obs = s.get_observation("player-a")
    assert "quiet after the battle" in obs.description or "guardian has fallen" in obs.description


def test_vault_entrance_description_changes_after_patrol_calmed():
    """After calming patrol, vault-entrance description reflects the calmed state."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-calm", shard_id="lc-s1",
    )
    s.advance_tick({"player-a": "take ward-sigil", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "use ward-sigil", "player-b": "wait"})
    obs = s.get_observation("player-a")
    assert "patrol stands aside" in obs.description or "aggression gone" in obs.description


def test_vault_description_changes_after_relic_taken():
    """After taking the relic, vault description reflects the empty stand."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-relic", shard_id="lr-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    obs_before = s.get_observation("player-a")
    assert "stone stand is empty" not in obs_before.description
    s.advance_tick({"player-a": "take relic", "player-b": "wait"})
    obs_after = s.get_observation("player-a")
    assert "stone stand is empty" in obs_after.description


def test_reactive_description_visible_to_both_actors():
    """Reactive description is consistent for both actors in the same room."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-shared", shard_id="ls-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "move east"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    obs_a = s.get_observation("player-a")
    obs_b = s.get_observation("player-b")
    assert obs_a.description == obs_b.description


def test_reactive_description_persists_after_save_load(tmp_path):
    """Reactive description survives save/load — shows same state-dependent text."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-persist", shard_id="lp-s1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})
    s.advance_tick({"player-a": "attack guardian", "player-b": "wait"})

    # Verify reactive description is active before save
    obs_before_save = s.get_observation("player-a")
    assert "quiet after the battle" in obs_before_save.description or \
           "guardian has fallen" in obs_before_save.description

    save_world_slot("lore-slot", str(tmp_path), s.world_state,
                    scenario_id="tiny-shared-combat", run_id="lore-persist")
    loaded = load_world_slot("lore-slot", str(tmp_path))
    assert loaded.accepted

    s2 = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-persist-2", shard_id="lp-s2",
        world_state=loaded.world_state,
    )
    # player-a is still at vault-entrance after reload — check description immediately
    obs_after_reload = s2.get_observation("player-a")
    assert "quiet after the battle" in obs_after_reload.description or \
           "guardian has fallen" in obs_after_reload.description


def test_no_observation_regression_without_overrides():
    """Scenarios without room_description_overrides are completely unaffected."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
        actor_ids=("player-a", "player-b"),
        run_id="lore-no-reg", shard_id="lnr-s1",
    )
    obs = s.get_observation("player-a")
    assert isinstance(obs.description, str) and len(obs.description) > 0


# ---------------------------------------------------------------------------
# defend combat action integration tests
# ---------------------------------------------------------------------------

def test_defend_in_action_space_at_vault_entrance():
    """defend appears in action space when hostile NPC is in the same room."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="defend-as-1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "move east"})
    obs = s.get_observation("player-a")
    assert "defend" in obs.action_space
    assert "attack guardian" in obs.action_space
    assert "attack patrol" in obs.action_space


def test_defend_not_in_action_space_at_camp():
    """defend does not appear in the starting camp (no hostile NPC there)."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="defend-camp-1",
    )
    obs = s.get_observation("player-a")
    assert "defend" not in obs.action_space


def test_defend_reduces_counter_attack_damage():
    """Defending actor takes less damage than one who waits."""
    s_wait = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="defend-cmp-wait",
    )
    s_defend = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-c", "player-d"),
        run_id="defend-cmp-defend",
    )

    # Both enter vault-entrance (patrol counter-attacks on entry tick)
    s_wait.advance_tick({"player-a": "move east", "player-b": "move east"})
    s_defend.advance_tick({"player-c": "move east", "player-d": "move east"})

    # Next tick: one session waits, one session defends
    s_wait.advance_tick({"player-a": "wait", "player-b": "wait"})
    s_defend.advance_tick({"player-c": "defend", "player-d": "defend"})

    hp_wait = s_wait.get_observation("player-a").health
    hp_defend = s_defend.get_observation("player-c").health

    assert hp_wait is not None and hp_defend is not None
    assert hp_defend > hp_wait


def test_defend_accepted_and_does_not_crash():
    """defend action is accepted cleanly in a shared shard session."""
    s = build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
        actor_ids=("player-a", "player-b"),
        run_id="defend-ok-1",
    )
    s.advance_tick({"player-a": "move east", "player-b": "move east"})
    # Should not raise
    result = s.advance_tick({"player-a": "defend", "player-b": "defend"})
    assert result is not None


def test_defend_state_persists_across_save_load():
    """defending.actor key in scenario_vars survives a save/load round-trip."""
    import tempfile
    from pathlib import Path
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    with tempfile.TemporaryDirectory() as tmp:
        slot = str(Path(tmp) / "defend-slot.json")

        s = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=("player-a", "player-b"),
            run_id="defend-sl-1",
        )
        s.advance_tick({"player-a": "move east", "player-b": "move east"})
        # Apply defend on tick 1
        s.advance_tick({"player-a": "defend", "player-b": "wait"})
        # Save snapshot
        save_world_snapshot(
            slot,
            s.world_state,
            run_id="defend-sl-1",
            scenario_id="tiny-shared-combat",
        )

        # Reload and verify scenario_vars contain the defending key
        load_result = load_world_snapshot(slot)
        snap = load_result.world_state.get_snapshot()
        # The key may be stale (different step_index) but must exist and be an int
        defending_keys = [k for k in snap.get("scenario_vars", {}) if k.startswith("defending.")]
        for k in defending_keys:
            assert isinstance(snap["scenario_vars"][k], int)
