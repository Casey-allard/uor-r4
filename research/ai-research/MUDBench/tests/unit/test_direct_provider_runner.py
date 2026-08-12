from __future__ import annotations

import json
import sys
from io import StringIO
from pathlib import Path

import pytest

from agents.direct_provider_runner import (
    _DEFAULT_OPENAI_BASE_URL,
    DirectProviderConfig,
    _build_prompt_dump_output_path,
    build_direct_provider_command,
    build_openai_chat_completions_request,
    request_openai_chat_completions,
    resolve_direct_provider_config,
    _build_runtime_telemetry_output_path,
    run_direct_provider_benchmark_turn,
)
from agents.protocol.observation import Observation


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "tiny-fetch-quest-run",
            "step": 2,
            "location": "corridor",
            "description": "A narrow stone corridor.",
            "exits": ["east", "west"],
            "entities": [
                {"type": "npc", "name": "guard-1"},
                {"type": "item", "name": "golden-key"},
            ],
            "inventory": [],
            "health": 100,
            "messages": ["The guard blocks part of the hall."],
            "action_space": [
                "move east",
                "move west",
                "attack guard-1",
                "take golden-key",
                "look",
                "wait",
            ],
            "remaining_steps": 3,
            "protocol_version": "1.0",
        }
    )


def _sample_config() -> DirectProviderConfig:
    return DirectProviderConfig(
        provider="openai-chat-completions",
        model="gpt-4.1-mini",
        api_key="test-key",
        base_url=_DEFAULT_OPENAI_BASE_URL,
    )


def test_resolve_direct_provider_config_rejects_missing_api_key() -> None:
    with pytest.raises(ValueError, match="direct_provider_missing_api_key:MUDBENCH_OPENAI_API_KEY"):
        resolve_direct_provider_config(
            provider="openai-chat-completions",
            model="gpt-4.1-mini",
            env={},
        )


def test_resolve_direct_provider_config_rejects_missing_model() -> None:
    with pytest.raises(ValueError, match="direct_provider_missing_model:MUDBENCH_OPENAI_MODEL"):
        resolve_direct_provider_config(
            provider="openai-chat-completions",
            env={"MUDBENCH_OPENAI_API_KEY": "test-key"},
        )


def test_build_openai_chat_completions_request_is_deterministic() -> None:
    prompt = "Return JSON only."
    config = _sample_config()

    first = build_openai_chat_completions_request(prompt=prompt, config=config)
    second = build_openai_chat_completions_request(prompt=prompt, config=config)

    assert first == second
    assert first["url"] == _DEFAULT_OPENAI_BASE_URL
    assert first["headers"]["Authorization"] == "Bearer test-key"
    assert first["body"] == {
        "model": "gpt-4.1-mini",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0,
    }


def test_request_openai_chat_completions_extracts_message_content() -> None:
    prompt = "Return JSON only."
    config = _sample_config()
    requests: list[dict[str, object]] = []

    def _transport(request_payload: dict[str, object]) -> dict[str, object]:
        requests.append(request_payload)
        return {
            "choices": [
                {
                    "message": {
                        "content": '{"action":"wait"}',
                    }
                }
            ]
        }

    content = request_openai_chat_completions(
        prompt=prompt,
        config=config,
        transport=_transport,
    )

    assert content == '{"action":"wait"}'
    assert len(requests) == 1
    assert requests[0]["body"] == {
        "model": "gpt-4.1-mini",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0,
    }


def test_run_direct_provider_benchmark_turn_reuses_canonical_prompt_and_repairs_once() -> None:
    observation = _sample_observation()
    config = _sample_config()
    prompts: list[str] = []
    responses = iter(("not-json", '{"action":"take golden-key"}'))
    time_points = iter((100.0, 100.012, 101.0, 101.018))

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        prompts.append(prompt)
        return next(responses)

    monkeypatch = pytest.MonkeyPatch()
    monkeypatch.setattr("agents.direct_provider_runner.monotonic", lambda: next(time_points))

    try:
        result = run_direct_provider_benchmark_turn(
            observation,
            config=config,
            completion_request=_completion,
        )
    finally:
        monkeypatch.undo()

    assert len(prompts) == 2
    assert "Invariant runtime guardrails:" in prompts[0]
    assert "Allowed actions:" in prompts[0]
    assert "golden-key" in prompts[0]
    assert "Your previous response was invalid for MUDBench." in prompts[1]
    assert result.initial_parse_result.reason == "invalid_json"
    assert result.action_submission.action == "take golden-key"
    assert result.used_fail_closed_fallback is False
    assert result.runtime_telemetry == {
        "repair_used": True,
        "fail_closed_used": False,
        "final_parse_status": "accepted_after_repair",
        "failure_reason": "invalid_json",
        "provider_name": "openai-chat-completions",
        "provider_request_count": 2,
        "provider_latency_ms": 30.0,
    }


def test_run_direct_provider_benchmark_turn_fails_closed_after_invalid_repair() -> None:
    observation = _sample_observation()
    config = _sample_config()
    time_points = iter((10.0, 10.005, 20.0, 20.007))

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        return "still-not-json"

    monkeypatch = pytest.MonkeyPatch()
    monkeypatch.setattr("agents.direct_provider_runner.monotonic", lambda: next(time_points))

    try:
        result = run_direct_provider_benchmark_turn(
            observation,
            config=config,
            completion_request=_completion,
        )
    finally:
        monkeypatch.undo()

    assert result.used_fail_closed_fallback is True
    assert result.fail_closed_reason == "model_output_rejected_after_repair:invalid_json"
    assert result.action_submission.action == "wait"
    assert result.runtime_telemetry == {
        "repair_used": True,
        "fail_closed_used": True,
        "final_parse_status": "fail_closed",
        "failure_reason": "model_output_rejected_after_repair:invalid_json",
        "provider_name": "openai-chat-completions",
        "provider_request_count": 2,
        "provider_latency_ms": 12.0,
    }


def test_run_direct_provider_benchmark_turn_preserves_shared_observation_guardrails() -> None:
    observation = Observation.from_dict(
        {
            "run_id": "shared-loop-run",
            "step": 1,
            "location": "corridor",
            "description": "A narrow stone corridor.",
            "exits": ["east", "west"],
            "entities": [{"type": "npc", "name": "guard-1"}],
            "inventory": [],
            "health": 100,
            "messages": [
                "A distant sentinel grows watchful in the quiet corridors.",
                "Hint: the sharp watch makes a careful look feel safer than rushing.",
                "Consequence: the exposed west passage is pinned under watch; move west is unavailable.",
            ],
            "action_space": [
                "move east",
                "attack guard-1",
                "look",
                "wait",
            ],
            "remaining_steps": 4,
            "protocol_version": "1.0",
        }
    )
    config = _sample_config()
    prompts: list[str] = []

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_direct_provider_benchmark_turn(
        observation,
        config=config,
        completion_request=_completion,
    )

    assert len(prompts) == 1
    assert "Invariant runtime guardrails:" in prompts[0]
    assert "Consequence: the exposed west passage is pinned under watch; move west is unavailable." in prompts[0]
    assert '{"allowed_actions":["move east","attack guard-1","look","wait"]}' in prompts[0]
    assert result.action_submission.action == "move east"
    assert result.used_fail_closed_fallback is False


def test_run_direct_provider_benchmark_turn_supports_geometric_routed_prompt_engine() -> None:
    observation = _sample_observation()
    config = _sample_config()
    prompts: list[str] = []

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_direct_provider_benchmark_turn(
        observation,
        config=config,
        completion_request=_completion,
        prompt_engine="geometric-routed",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "geometric-routed"
    assert result.routing_plan is not None
    assert "Routing plan:" in prompts[0]
    assert "Loop layer: immediate_action" in prompts[0]
    assert result.action_submission.action == "move east"


def test_run_direct_provider_benchmark_turn_supports_angular_canonical_prompt_engine() -> None:
    observation = _sample_observation()
    config = _sample_config()
    prompts: list[str] = []

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_direct_provider_benchmark_turn(
        observation,
        config=config,
        completion_request=_completion,
        prompt_engine="angular-canonical",
        router_variant="angular-hopf-trans",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "angular-canonical"
    assert result.router_variant == "angular-hopf-trans"
    assert result.routing_plan is not None
    assert result.routing_plan["router_variant"] == "angular-hopf-trans"
    assert "Angular canonical plan:" in prompts[0]
    assert '"router_variant":"angular-hopf-trans"' in prompts[0]
    assert result.action_submission.action == "move east"


def test_run_direct_provider_benchmark_turn_supports_legacy_router_backed_prompt_engine() -> None:
    observation = _sample_observation()
    config = _sample_config()
    prompts: list[str] = []

    def _completion(prompt: str, runtime_config: DirectProviderConfig) -> str:
        assert runtime_config == config
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_direct_provider_benchmark_turn(
        observation,
        config=config,
        completion_request=_completion,
        prompt_engine="legacy-router-backed",
        router_variant="legacy-phase4d_hopf_transport",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "legacy-router-backed"
    assert result.router_variant == "legacy-phase4d_hopf_transport"
    assert result.routing_plan is not None
    assert result.routing_plan["router_variant"] == "legacy-phase4d_hopf_transport"
    assert "Legacy router-backed proxy plan:" in prompts[0]
    assert '"router_variant":"legacy-phase4d_hopf_transport"' in prompts[0]
    assert result.action_submission.action == "move east"


def test_build_direct_provider_command_is_deterministic() -> None:
    config = _sample_config()

    first = build_direct_provider_command(config, python_executable="/usr/bin/python3")
    second = build_direct_provider_command(config, python_executable="/usr/bin/python3")

    assert first == second
    assert first[:2] == ("/usr/bin/python3", first[1])
    assert first[2:] == (
        "--provider",
        "openai-chat-completions",
        "--model",
        "gpt-4.1-mini",
        "--base-url",
        _DEFAULT_OPENAI_BASE_URL,
    )


def test_build_direct_provider_command_appends_routed_prompt_engine_only_when_requested() -> None:
    config = _sample_config()

    baseline = build_direct_provider_command(config, python_executable="/usr/bin/python3")
    routed = build_direct_provider_command(
        config,
        python_executable="/usr/bin/python3",
        prompt_engine="geometric-routed",
    )

    assert "--prompt-engine" not in baseline
    assert routed[-2:] == ("--prompt-engine", "geometric-routed")


def test_build_direct_provider_command_appends_router_variant_for_variant_aware_modes() -> None:
    config = _sample_config()

    angular_command = build_direct_provider_command(
        config,
        python_executable="/usr/bin/python3",
        prompt_engine="angular-canonical",
        router_variant="angular-hopf-trans",
    )
    legacy_command = build_direct_provider_command(
        config,
        python_executable="/usr/bin/python3",
        prompt_engine="legacy-router-backed",
        router_variant="legacy-phase4d_hopf_product_phase",
    )

    assert angular_command[-4:] == (
        "--prompt-engine",
        "angular-canonical",
        "--router-variant",
        "angular-hopf-trans",
    )
    assert legacy_command[-4:] == (
        "--prompt-engine",
        "legacy-router-backed",
        "--router-variant",
        "legacy-phase4d_hopf_product_phase",
    )


def test_direct_provider_runner_main_repairs_one_invalid_provider_response(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    prompts: list[str] = []
    responses = iter(("not-json", '{"action":"move east"}'))
    time_points = iter((1.0, 1.01, 2.0, 2.015))

    def _request(prompt: str, config: DirectProviderConfig) -> str:
        assert config.provider == "openai-chat-completions"
        prompts.append(prompt)
        return next(responses)

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setattr("agents.direct_provider_runner.request_openai_chat_completions", _request)
    monkeypatch.setattr("agents.direct_provider_runner.monotonic", lambda: next(time_points))
    stdin = StringIO(json.dumps(_sample_observation().to_dict()) + "\n")
    stdout = StringIO()
    stderr = StringIO()
    monkeypatch.setattr(sys, "stdin", stdin)
    monkeypatch.setattr(sys, "stdout", stdout)
    monkeypatch.setattr(sys, "stderr", stderr)

    from agents import direct_provider_runner

    exit_code = direct_provider_runner.main(
        [
            "--provider",
            "openai-chat-completions",
            "--model",
            "gpt-4.1-mini",
            "--telemetry-dir",
            str(tmp_path),
            "--telemetry-actor-id",
            "agent-a",
        ]
    )

    assert exit_code == 0
    assert len(prompts) == 2
    assert "Invariant runtime guardrails:" in prompts[0]
    assert "Your previous response was invalid for MUDBench." in prompts[1]
    assert json.loads(stdout.getvalue().strip()) == {"action": "move east"}
    assert stderr.getvalue() == ""
    telemetry_path = _build_runtime_telemetry_output_path(
        telemetry_dir=tmp_path,
        run_id="tiny-fetch-quest-run",
        step=2,
        actor_id="agent-a",
    )
    assert json.loads(telemetry_path.read_text(encoding="utf-8")) == {
        "actor_id": "agent-a",
        "fail_closed_used": False,
        "failure_reason": "invalid_json",
        "final_parse_status": "accepted_after_repair",
        "provider_latency_ms": 25.0,
        "provider_name": "openai-chat-completions",
        "provider_request_count": 2,
        "repair_used": True,
        "run_id": "tiny-fetch-quest-run",
        "step": 2,
        "telemetry_schema": "llm_runtime_turn_v1",
    }


def test_direct_provider_runner_main_writes_prompt_dump_when_requested(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    responses = iter(("not-json", '{"action":"move east"}'))
    time_points = iter((1.0, 1.01, 2.0, 2.015))

    def _request(prompt: str, config: DirectProviderConfig) -> str:
        assert config.provider == "openai-chat-completions"
        return next(responses)

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setattr("agents.direct_provider_runner.request_openai_chat_completions", _request)
    monkeypatch.setattr("agents.direct_provider_runner.monotonic", lambda: next(time_points))
    stdin = StringIO(json.dumps(_sample_observation().to_dict()) + "\n")
    stdout = StringIO()
    stderr = StringIO()
    monkeypatch.setattr(sys, "stdin", stdin)
    monkeypatch.setattr(sys, "stdout", stdout)
    monkeypatch.setattr(sys, "stderr", stderr)

    from agents import direct_provider_runner

    exit_code = direct_provider_runner.main(
        [
            "--provider",
            "openai-chat-completions",
            "--model",
            "gpt-4.1-mini",
            "--prompt-engine",
            "angular-canonical",
            "--router-variant",
            "angular-hopf-trans",
            "--prompt-dump-dir",
            str(tmp_path),
            "--prompt-dump-actor-id",
            "agent-a",
            "--prompt-dump-scenario-id",
            "tiny-fetch-quest",
        ]
    )

    assert exit_code == 0
    assert json.loads(stdout.getvalue().strip()) == {"action": "move east"}
    assert stderr.getvalue() == ""

    dump_path = _build_prompt_dump_output_path(
        prompt_dump_dir=tmp_path,
        run_id="tiny-fetch-quest-run",
        step=2,
        actor_id="agent-a",
    )
    dump_payload = json.loads(dump_path.read_text(encoding="utf-8"))
    assert dump_payload["dump_schema"] == "direct_provider_prompt_dump_v1"
    assert dump_payload["scenario_id"] == "tiny-fetch-quest"
    assert dump_payload["prompt_engine"] == "angular-canonical"
    assert dump_payload["router_variant"] == "angular-hopf-trans"
    assert dump_payload["raw_provider_response_text"] == "not-json"
    assert dump_payload["repair_raw_provider_response_text"] == '{"action":"move east"}'
    assert dump_payload["final_action"] == {"action": "move east"}
    assert dump_payload["provider_turns"] == [
        {
            "phase": "initial",
            "prompt_text": dump_payload["prompt_text"],
            "raw_provider_response_text": "not-json",
        },
        {
            "phase": "repair",
            "prompt_text": dump_payload["repair_prompt_text"],
            "raw_provider_response_text": '{"action":"move east"}',
        },
    ]


def test_direct_provider_runner_main_rejects_prompt_dump_dir_without_actor_id(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    stdout = StringIO()
    stderr = StringIO()
    monkeypatch.setattr(sys, "stdout", stdout)
    monkeypatch.setattr(sys, "stderr", stderr)

    from agents import direct_provider_runner

    exit_code = direct_provider_runner.main(
        [
            "--provider",
            "openai-chat-completions",
            "--model",
            "gpt-4.1-mini",
            "--prompt-dump-dir",
            "/tmp/prompt-dumps",
        ]
    )

    assert exit_code == 1
    assert stdout.getvalue() == ""
    assert stderr.getvalue().strip() == (
        "direct_provider_runner error: prompt_dump_dir_requires_prompt_dump_actor_id"
    )


def test_direct_provider_runner_main_fails_closed_after_invalid_repair(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    time_points = iter((5.0, 5.004, 6.0, 6.006))

    def _request(prompt: str, config: DirectProviderConfig) -> str:
        assert config.model == "gpt-4.1-mini"
        return "still-not-json"

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setattr("agents.direct_provider_runner.request_openai_chat_completions", _request)
    monkeypatch.setattr("agents.direct_provider_runner.monotonic", lambda: next(time_points))
    stdin = StringIO(json.dumps(_sample_observation().to_dict()) + "\n")
    stdout = StringIO()
    stderr = StringIO()
    monkeypatch.setattr(sys, "stdin", stdin)
    monkeypatch.setattr(sys, "stdout", stdout)
    monkeypatch.setattr(sys, "stderr", stderr)

    from agents import direct_provider_runner

    exit_code = direct_provider_runner.main(
        [
            "--provider",
            "openai-chat-completions",
            "--model",
            "gpt-4.1-mini",
            "--telemetry-dir",
            str(tmp_path),
            "--telemetry-actor-id",
            "agent-a",
        ]
    )

    assert exit_code == 0
    assert json.loads(stdout.getvalue().strip()) == {"action": "wait"}
    assert stderr.getvalue() == ""
    telemetry_path = _build_runtime_telemetry_output_path(
        telemetry_dir=tmp_path,
        run_id="tiny-fetch-quest-run",
        step=2,
        actor_id="agent-a",
    )
    assert json.loads(telemetry_path.read_text(encoding="utf-8")) == {
        "actor_id": "agent-a",
        "fail_closed_used": True,
        "failure_reason": "model_output_rejected_after_repair:invalid_json",
        "final_parse_status": "fail_closed",
        "provider_latency_ms": 10.0,
        "provider_name": "openai-chat-completions",
        "provider_request_count": 2,
        "repair_used": True,
        "run_id": "tiny-fetch-quest-run",
        "step": 2,
        "telemetry_schema": "llm_runtime_turn_v1",
    }
