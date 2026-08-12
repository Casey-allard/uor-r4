from __future__ import annotations

import json
import sys
from collections.abc import Mapping
from pathlib import Path

import pytest

from agents.direct_provider_runner import DirectProviderConfig
from cli.main import _SCENARIO_PRESETS, main
from agents.protocol.observation import Observation, ObservedEntity
from human_console_client import HumanConsoleSessionResult, HumanSharedShardSessionResult
from human_console_client import _resolve_console_action
from human_console_client import render_human_observation


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def _manifest_path_for(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".manifest.json")
    return Path(str(output_path) + ".manifest.json")


def _replay_path_for(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".replay.json")
    return Path(str(output_path) + ".replay.json")


def _write_deterministic_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "external_agent.py"
    script_path.write_text(
        (
            "import json\n"
            "import sys\n"
            "line = sys.stdin.readline()\n"
            "observation = json.loads(line)\n"
            "action_space = tuple(observation.get('action_space', ()))\n"
            "action = 'wait'\n"
            "for candidate in action_space:\n"
            "    if candidate.startswith('take '):\n"
            "        action = candidate\n"
            "        break\n"
            "else:\n"
            "    for candidate in action_space:\n"
            "        if candidate.startswith('move '):\n"
            "            action = candidate\n"
            "            break\n"
            "    else:\n"
            "        for candidate in action_space:\n"
            "            if candidate.startswith('attack '):\n"
            "                action = candidate\n"
            "                break\n"
            "        else:\n"
            "            action = 'look' if 'look' in action_space else 'wait'\n"
            "print(json.dumps({'action': action}, sort_keys=True, separators=(',', ':'), ensure_ascii=True))\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_broken_persistent_agent_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "broken_persistent_agent.py"
    script_path.write_text(
        (
            "import sys\n"
            "for line in sys.stdin:\n"
            "    sys.exit(1)\n"
        ),
        encoding="utf-8",
    )
    return script_path


def _write_external_agent_profile(
    tmp_path: Path,
    *,
    agent_id: str = "profile-agent",
    display_name: str = "Profile Wrapper",
    command: str,
    persistent_agent_session: bool = False,
    filename: str = "external-agent-profile.json",
) -> Path:
    profile_path = tmp_path / filename
    profile_path.write_text(
        json.dumps(
            {
                "agent_id": agent_id,
                "display_name": display_name,
                "command": command,
                "persistent_agent_session": persistent_agent_session,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return profile_path


def test_cli_run_default_executes_real_runtime_path_and_emits_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["run_id"] == "cli-run"
    assert payload["benchmark_id"] == "mudbench-cli"
    assert payload["scenario_id"] == "cli-minimal-scenario"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["scorecard"]["metadata"]["scoring_version"] == "phase3-v1"
    assert payload["replay"]["schema_version"] == "1.0"
    assert payload["replay"]["event_count"] >= payload["lifecycle"]["step_count"] * 2
    assert "timing_mode" not in payload
    assert "action_cadence_interval" not in payload


def test_cli_run_surfaces_replay_and_scorecard_references(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    refs = payload["replay"]["artifact_refs"]
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    assert refs[0]["ref"] == refs[1]["ref"]
    assert payload["scorecard"]["aggregate_score"] >= 0.0
    assert payload["scorecard"]["aggregate_score"] <= 1.0
    parity = payload["replay"]["parity"]
    for hash_key in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert isinstance(parity[hash_key], str)
        assert len(parity[hash_key]) == 64


def test_cli_run_supports_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "phase4-runtime-replay"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "phase4-runtime-replay-scenario"


def test_cli_run_supports_second_tiny_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "tiny-delayed-retrieval"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-delayed-retrieval"


def test_cli_run_supports_third_tiny_scenario_selection(capsys: pytest.CaptureFixture[str]) -> None:
    exit_code = main(["run", "--scenario", "tiny-hidden-key"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-hidden-key"


def test_cli_run_supports_social_trade_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-social-trade"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-social-trade"


def test_cli_run_supports_guarded_relic_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-guarded-relic"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-guarded-relic"
    assert payload["accepted"] is True
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_cli_run_supports_hazard_route_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-hazard-route"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-hazard-route"
    assert payload["accepted"] is True
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_cli_run_supports_delayed_cost_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-delayed-cost"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-delayed-cost"
    assert payload["accepted"] is True
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_cli_run_supports_context_pressure_tiny_scenario_selection(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario", "tiny-context-pressure"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["scenario_id"] == "tiny-context-pressure"
    assert payload["accepted"] is True
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 12
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_cli_run_rejects_routed_prompt_engine_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--prompt-engine", "geometric-routed"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "run_rejected",
        "reason": "prompt_engine_requires_direct_provider",
    }


def test_cli_run_rejects_prompt_dump_dir_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--direct-provider-prompt-dump-dir", "/tmp/direct-provider-dumps"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "run_rejected",
        "reason": "direct_provider_prompt_dump_dir_requires_direct_provider",
    }


def test_cli_run_rejects_router_variant_without_variant_aware_prompt_engine(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--router-variant", "legacy-phase4d_hopf_transport"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "run_rejected",
        "reason": "router_variant_requires_variant_prompt_engine",
    }


def test_cli_compare_playable_slices_supports_direct_provider_mode_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_configs: list[object] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        captured_configs.append(config)
        scenario = getattr(config, "scenario")
        scenario_id = str(scenario["scenario_id"])
        external_agent_command = getattr(config, "external_agent_command")
        if external_agent_command is None:
            mode = "built_in"
        elif tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            mode = "mock_wrapper"
        else:
            mode = "direct_provider"
        return _FakeResult(scenario_id, mode)

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        telemetry = None
        if mode == "direct_provider":
            telemetry = {
                "turn_count": 8,
                "repair_used_count": 1,
                "fail_closed_used_count": 0,
                "provider_request_count_total": 9,
                "provider_latency_ms_total": 12.5,
                "final_parse_status_counts": {"accepted": 7, "accepted_after_repair": 1},
                "failure_reasons": ["invalid_json"],
            }
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": mode == "built_in",
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": telemetry,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["comparison_schema"] == "playable_slice_comparison_v1"
    assert payload["scenario_ids"] == [
        "tiny-guarded-relic",
        "tiny-hazard-route",
        "tiny-delayed-cost",
        "tiny-context-pressure",
    ]
    assert payload["mode_ids"] == ["built_in", "mock_wrapper", "direct_provider"]
    assert payload["entry_count"] == 12
    assert len(captured_configs) == 12
    direct_entries = [entry for entry in payload["entries"] if entry["mode"] == "direct_provider"]
    assert len(direct_entries) == 4
    assert all(entry["agent_identity"] == "direct-provider:openai-chat-completions" for entry in direct_entries)
    assert all(entry["runtime_telemetry"]["repair_used_count"] == 1 for entry in direct_entries)
    assert all(entry["prompt_engine"] == "baseline" for entry in direct_entries)
    direct_provider_configs = [
        config for config in captured_configs if getattr(config, "external_agent_command") is not None
    ]
    assert all(getattr(config, "external_agent_timeout_seconds") is None for config in direct_provider_configs)


def test_cli_compare_playable_slices_can_include_routed_direct_provider_mode_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_commands: list[tuple[str, ...] | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        external_agent_command = getattr(config, "external_agent_command")
        captured_commands.append(None if external_agent_command is None else tuple(external_agent_command))
        scenario_id = str(getattr(config, "scenario")["scenario_id"])
        if external_agent_command is None:
            return _FakeResult(scenario_id, "built_in")
        if tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            return _FakeResult(scenario_id, "mock_wrapper")
        if "--prompt-engine" in tuple(external_agent_command):
            return _FakeResult(scenario_id, "direct_provider_routed")
        return _FakeResult(scenario_id, "direct_provider")

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": True,
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": {"repair_used_count": 0} if mode.startswith("direct_provider") else None,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--include-routed-prompt-engine",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["mode_ids"] == [
        "built_in",
        "mock_wrapper",
        "direct_provider",
        "direct_provider_routed",
    ]
    assert payload["entry_count"] == 16
    routed_entries = [entry for entry in payload["entries"] if entry["mode"] == "direct_provider_routed"]
    assert len(routed_entries) == 4
    assert all(entry["prompt_engine"] == "geometric-routed" for entry in routed_entries)
    assert any(command is not None and "--prompt-engine" in command for command in captured_commands)


def test_cli_compare_playable_slices_rejects_direct_provider_model_without_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["compare-playable-slices", "--direct-provider-model", "gpt-4.1-mini"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "direct_provider_model_requires_direct_provider",
    }


def test_cli_compare_playable_slices_rejects_timeout_override_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["compare-playable-slices", "--direct-provider-comparison-timeout-seconds", "5.0"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "direct_provider_comparison_timeout_requires_direct_provider",
    }


def test_cli_compare_playable_slices_rejects_prompt_dump_dir_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        ["compare-playable-slices", "--direct-provider-comparison-prompt-dump-dir", "/tmp/compare-dumps"]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "direct_provider_comparison_prompt_dump_dir_requires_direct_provider",
    }


def test_cli_compare_playable_slices_rejects_non_positive_timeout_override(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--direct-provider-comparison-timeout-seconds",
            "0",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "direct_provider_comparison_timeout_seconds_must_be_positive",
    }


def test_cli_compare_playable_slices_rejects_routed_mode_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["compare-playable-slices", "--include-routed-prompt-engine"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "include_routed_prompt_engine_requires_direct_provider",
    }


def test_cli_compare_playable_slices_can_include_canonical_angular_direct_provider_mode_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_commands: list[tuple[str, ...] | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        external_agent_command = getattr(config, "external_agent_command")
        captured_commands.append(None if external_agent_command is None else tuple(external_agent_command))
        scenario_id = str(getattr(config, "scenario")["scenario_id"])
        if external_agent_command is None:
            return _FakeResult(scenario_id, "built_in")
        if tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            return _FakeResult(scenario_id, "mock_wrapper")
        command_tuple = tuple(external_agent_command)
        if "--router-variant" in command_tuple:
            return _FakeResult(scenario_id, "direct_provider_angular_canonical")
        return _FakeResult(scenario_id, "direct_provider")

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": True,
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": {"repair_used_count": 0} if mode.startswith("direct_provider") else None,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--include-angular-canonical-prompt-engine",
            "--angular-router-variant",
            "angular-hopf-trans",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["mode_ids"] == [
        "built_in",
        "mock_wrapper",
        "direct_provider",
        "direct_provider_angular_canonical",
    ]
    assert payload["entry_count"] == 16
    router_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_angular_canonical"
    ]
    assert len(router_entries) == 4
    assert all(entry["prompt_engine"] == "angular-canonical" for entry in router_entries)
    assert all(entry["router_variant"] == "angular-hopf-trans" for entry in router_entries)
    assert any(command is not None and "--router-variant" in command for command in captured_commands)


def test_cli_compare_playable_slices_threads_timeout_override_into_direct_provider_runs(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_timeouts: list[float | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        external_agent_command = getattr(config, "external_agent_command")
        if external_agent_command is not None and tuple(external_agent_command)[1] != "examples/agents/mock_llm_wrapper.py":
            captured_timeouts.append(getattr(config, "external_agent_timeout_seconds"))
        scenario_id = str(getattr(config, "scenario")["scenario_id"])
        if external_agent_command is None:
            return _FakeResult(scenario_id, "built_in")
        if tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            return _FakeResult(scenario_id, "mock_wrapper")
        if "--router-variant" in tuple(external_agent_command):
            command_tuple = tuple(external_agent_command)
            if "legacy-router-backed" in command_tuple:
                return _FakeResult(scenario_id, "direct_provider_legacy_router_backed")
            return _FakeResult(scenario_id, "direct_provider_angular_canonical")
        if "--prompt-engine" in tuple(external_agent_command):
            return _FakeResult(scenario_id, "direct_provider_routed")
        return _FakeResult(scenario_id, "direct_provider")

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": True,
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": {"repair_used_count": 0} if mode.startswith("direct_provider") else None,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--include-routed-prompt-engine",
            "--include-angular-canonical-prompt-engine",
            "--angular-router-variant",
            "angular-hopf-base",
            "--include-legacy-router-backed-prompt-engine",
            "--legacy-router-variant",
            "legacy-phase4d_hopf_base",
            "--direct-provider-comparison-timeout-seconds",
            "7.5",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_timeouts == [7.5] * 16


def test_cli_compare_playable_slices_can_include_multiple_angular_variants_in_one_invocation(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_commands: list[tuple[str, ...] | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        external_agent_command = getattr(config, "external_agent_command")
        captured_commands.append(None if external_agent_command is None else tuple(external_agent_command))
        scenario_id = str(getattr(config, "scenario")["scenario_id"])
        if external_agent_command is None:
            return _FakeResult(scenario_id, "built_in")
        if tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            return _FakeResult(scenario_id, "mock_wrapper")
        if "--router-variant" in tuple(external_agent_command):
            command_tuple = tuple(external_agent_command)
            if "legacy-router-backed" in command_tuple:
                return _FakeResult(scenario_id, "direct_provider_legacy_router_backed")
            return _FakeResult(scenario_id, "direct_provider_angular_canonical")
        return _FakeResult(scenario_id, "direct_provider")

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": True,
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": {"repair_used_count": 0} if mode.startswith("direct_provider") else None,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--include-angular-canonical-prompt-engine",
            "--angular-router-variant",
            "angular-hopf-base",
            "--angular-router-variant",
            "angular-hopf-trans",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["mode_ids"] == [
        "built_in",
        "mock_wrapper",
        "direct_provider",
        "direct_provider_angular_canonical",
    ]
    assert payload["entry_count"] == 20
    baseline_entries = [entry for entry in payload["entries"] if entry["mode"] == "direct_provider"]
    angular_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_angular_canonical"
    ]
    assert len(baseline_entries) == 4
    assert len(angular_entries) == 8
    assert all(entry["prompt_engine"] == "angular-canonical" for entry in angular_entries)
    assert {entry["router_variant"] for entry in angular_entries} == {
        "angular-hopf-base",
        "angular-hopf-trans",
    }
    assert sum(1 for entry in angular_entries if entry["router_variant"] == "angular-hopf-base") == 4
    assert sum(1 for entry in angular_entries if entry["router_variant"] == "angular-hopf-trans") == 4
    assert any(command is not None and "angular-hopf-base" in command for command in captured_commands)
    assert any(command is not None and "angular-hopf-trans" in command for command in captured_commands)


def test_cli_compare_playable_slices_wires_prompt_dump_flags_into_direct_provider_rows(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_commands: list[tuple[str, ...] | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str, mode: str) -> None:
            self.scenario_id = scenario_id
            self.mode = mode

    def _fake_resolve_direct_provider_config(**_: object) -> _FakeProviderConfig:
        return _FakeProviderConfig()

    def _fake_build_direct_provider_command(
        _config: object,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        command = [python_executable, "src/agents/direct_provider_runner.py", "--provider", "openai-chat-completions"]
        if prompt_engine != "baseline":
            command.extend(("--prompt-engine", prompt_engine))
        if router_variant is not None:
            command.extend(("--router-variant", router_variant))
        return tuple(command)

    def _fake_run_benchmark_lifecycle(config: object) -> _FakeResult:
        external_agent_command = getattr(config, "external_agent_command")
        captured_commands.append(None if external_agent_command is None else tuple(external_agent_command))
        scenario_id = str(getattr(config, "scenario")["scenario_id"])
        if external_agent_command is None:
            return _FakeResult(scenario_id, "built_in")
        if tuple(external_agent_command)[1] == "examples/agents/mock_llm_wrapper.py":
            return _FakeResult(scenario_id, "mock_wrapper")
        return _FakeResult(scenario_id, "direct_provider")

    def _fake_build_playable_slice_comparison_entry(
        result: _FakeResult,
        *,
        mode: str,
        agent_identity: str,
        actor_id: str,
    ) -> dict[str, object]:
        return {
            "scenario_id": result.scenario_id,
            "mode": mode,
            "agent_identity": agent_identity,
            "actor_id": actor_id,
            "objective_completed": True,
            "aggregate_score": 0.5,
            "composite_score": 0.25,
            "runtime_telemetry": {"repair_used_count": 0} if mode.startswith("direct_provider") else None,
        }

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr(
        "cli.main.build_playable_slice_comparison_entry",
        _fake_build_playable_slice_comparison_entry,
    )

    exit_code = main(
        [
            "compare-playable-slices",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
            "--include-angular-canonical-prompt-engine",
            "--angular-router-variant",
            "angular-hopf-trans",
            "--include-legacy-router-backed-prompt-engine",
            "--legacy-router-variant",
            "legacy-phase4d_hopf_transport",
            "--direct-provider-comparison-prompt-dump-dir",
            "/tmp/compare-prompt-dumps",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    direct_commands = [
        command
        for command in captured_commands
        if command is not None and command[1] == "src/agents/direct_provider_runner.py"
    ]
    assert len(direct_commands) == 12
    assert all("--prompt-dump-dir" in command for command in direct_commands)
    assert all("--prompt-dump-actor-id" in command for command in direct_commands)
    assert all("--prompt-dump-scenario-id" in command for command in direct_commands)
    assert {command[command.index("--prompt-dump-scenario-id") + 1] for command in direct_commands} == {
        "tiny-guarded-relic",
        "tiny-hazard-route",
        "tiny-delayed-cost",
        "tiny-context-pressure",
    }


def test_cli_compare_playable_slices_rejects_angular_canonical_mode_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["compare-playable-slices", "--include-angular-canonical-prompt-engine"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "playable_slice_comparison_rejected",
        "reason": "include_angular_canonical_prompt_engine_requires_direct_provider",
    }


def test_cli_play_wires_human_console_session_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_console_session(**kwargs: object) -> HumanConsoleSessionResult:
        captured_args.update(kwargs)
        return HumanConsoleSessionResult(
            run_id="human-console",
            scenario_id="tiny-guarded-relic",
            actor_id="human-player",
            step_count=8,
            max_steps=8,
            objective_completed=True,
            quit_requested=False,
            final_location="reliquary",
            final_inventory=("relic", "relic-key"),
            action_history=("move east",),
        )

    monkeypatch.setattr("cli.main.run_human_console_session", _fake_run_human_console_session)

    exit_code = main(["play", "--scenario", "tiny-guarded-relic", "--actor-id", "human-player"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["actor_id"] == "human-player"
    assert captured_args["run_id"] == "human-console"
    assert captured_args["scenario"] == _SCENARIO_PRESETS["tiny-guarded-relic"]
    assert "Session Summary" in captured.out
    assert "Scenario: tiny-guarded-relic" in captured.out
    assert "Final Inventory: relic, relic-key" in captured.out


def test_cli_play_shared_shard_wires_shared_console_session_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "human-b"),
            step_count=3,
            max_steps=5,
            completed_actor_ids=("human-a",),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=3,
            last_world_tick_heartbeat="shared_shard_world_tick:0003",
            world_npc_stance_phase="settling",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(["play-shared-shard", "--scenario", "tiny-fetch-quest"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["run_id"] == "human-shared-shard"
    assert captured_args["shard_id"] == "shared-shard-local"
    assert captured_args["scenario"] == _SCENARIO_PRESETS["tiny-fetch-quest"]
    assert captured_args["actor_ids"] == ("human-a", "human-b")
    assert captured_args["mock_agent_actor_ids"] == ()
    assert captured_args["persistent_agent_session"] is False
    assert captured_args["external_agent_timeout_seconds"] is None
    assert "Session Summary" in captured.out
    assert "Shard: shared-shard-local" in captured.out
    assert "Actors: human-a, human-b" in captured.out
    assert "Completed Actors: human-a" in captured.out
    assert "World Tick Count: 3" in captured.out
    assert "Last World Tick Heartbeat: shared_shard_world_tick:0003" in captured.out
    assert "World NPC Stance Phase: settling" in captured.out


def test_cli_play_shared_shard_wires_action_cadence_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "human-b"),
            step_count=2,
            max_steps=5,
            completed_actor_ids=(),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=2,
            last_world_tick_heartbeat="shared_shard_world_tick:0002",
            world_npc_stance_phase="patrolling",
            action_cadence_interval=2,
            actor_action_cadence_overrides=(("human-b", 3),),
            actor_next_action_eligible_at=(("human-a", 2), ("human-b", 3)),
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--action-cadence-interval",
            "2",
            "--actor-action-cadence",
            "human-b=3",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["action_cadence_interval"] == 2
    assert captured_args["actor_action_cadence_overrides"] == {"human-b": 3}
    assert "Action Cadence Interval: 2" in captured.out
    assert "Actor Action Cadence Overrides: human-b=3" in captured.out
    assert "Next Action Eligible At: human-a=2, human-b=3" in captured.out


def test_cli_play_shared_shard_wires_timing_mode_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "human-b"),
            step_count=2,
            max_steps=5,
            completed_actor_ids=(),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=2,
            last_world_tick_heartbeat="shared_shard_world_tick:0002",
            world_npc_stance_phase="patrolling",
            timing_mode="human-parity",
            action_cadence_interval=2,
            actor_action_cadence_overrides=(),
            actor_next_action_eligible_at=(("human-a", 2), ("human-b", 2)),
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--timing-mode",
            "human-parity",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["timing_mode"] == "human-parity"
    assert captured_args["action_cadence_interval"] is None
    assert captured_args["actor_action_cadence_overrides"] == {}
    assert "Timing Mode: human-parity" in captured.out
    assert "Action Cadence Interval: 2" in captured.out


def test_cli_play_shared_shard_wires_mock_agent_participant_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "agent-b"),
            step_count=3,
            max_steps=5,
            completed_actor_ids=("agent-b",),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=3,
            last_world_tick_heartbeat="shared_shard_world_tick:0003",
            world_npc_stance_phase="settling",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--mock-agent-actor-id",
            "agent-b",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["actor_ids"] == ("human-a", "agent-b")
    assert captured_args["mock_agent_actor_ids"] == ("agent-b",)
    assert captured_args["persistent_agent_session"] is False
    assert "Actors: human-a, agent-b" in captured.out
    assert "Completed Actors: agent-b" in captured.out


def test_cli_play_shared_shard_wires_external_agent_participant_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "external-agent"),
            step_count=3,
            max_steps=5,
            completed_actor_ids=("external-agent",),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=3,
            last_world_tick_heartbeat="shared_shard_world_tick:0003",
            world_npc_stance_phase="settling",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["actor_ids"] == ("human-a", "external-agent")
    assert captured_args["mock_agent_actor_ids"] == ()
    assert captured_args["external_agent_commands_by_actor"] == {
        "external-agent": (sys.executable, "examples/agents/mock_llm_wrapper.py")
    }
    assert captured_args["persistent_agent_session"] is False
    assert "Actors: human-a, external-agent" in captured.out
    assert "Completed Actors: external-agent" in captured.out


def test_cli_play_shared_shard_wires_persistent_external_agent_participant_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "external-agent"),
            step_count=3,
            max_steps=5,
            completed_actor_ids=("external-agent",),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=3,
            last_world_tick_heartbeat="shared_shard_world_tick:0003",
            world_npc_stance_phase="settling",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py --persistent-session",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["actor_ids"] == ("human-a", "external-agent")
    assert captured_args["external_agent_commands_by_actor"] == {
        "external-agent": (
            sys.executable,
            "examples/agents/mock_llm_wrapper.py",
            "--persistent-session",
        )
    }
    assert captured_args["persistent_agent_session"] is True
    assert "Actors: human-a, external-agent" in captured.out
    assert "Completed Actors: external-agent" in captured.out


def test_cli_play_shared_shard_wires_direct_provider_participant_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}
    captured_provider: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "direct-provider"),
            step_count=3,
            max_steps=5,
            completed_actor_ids=("direct-provider",),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=3,
            last_world_tick_heartbeat="shared_shard_world_tick:0003",
            world_npc_stance_phase="settling",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    def _fake_resolve_direct_provider_config(
        *,
        provider: str,
        model: str | None = None,
        env: Mapping[str, str] | None = None,
    ) -> DirectProviderConfig:
        captured_provider["provider"] = provider
        captured_provider["model"] = model
        return DirectProviderConfig(
            provider=provider,
            model="gpt-4.1-mini",
            api_key="test-key",
            base_url="http://127.0.0.1:9999/v1/chat/completions",
        )

    def _fake_build_direct_provider_command(
        config: DirectProviderConfig,
        *,
        python_executable: str,
        prompt_engine: str = "baseline",
        router_variant: str | None = None,
    ) -> tuple[str, ...]:
        captured_provider["python_executable"] = python_executable
        captured_provider["base_url"] = config.base_url
        captured_provider["prompt_engine"] = prompt_engine
        captured_provider["router_variant"] = router_variant
        return (
            python_executable,
            "src/agents/direct_provider_runner.py",
            "--provider",
            config.provider,
            "--model",
            config.model,
            "--base-url",
            config.base_url,
        )

    monkeypatch.setattr("cli.main.resolve_direct_provider_config", _fake_resolve_direct_provider_config)
    monkeypatch.setattr("cli.main.build_direct_provider_command", _fake_build_direct_provider_command)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_provider == {
        "provider": "openai-chat-completions",
        "model": "gpt-4.1-mini",
        "python_executable": sys.executable,
        "base_url": "http://127.0.0.1:9999/v1/chat/completions",
        "prompt_engine": "baseline",
        "router_variant": None,
    }
    assert captured_args["actor_ids"] == ("human-a", "direct-provider")
    assert captured_args["external_agent_commands_by_actor"] == {
        "direct-provider": (
            sys.executable,
            "src/agents/direct_provider_runner.py",
            "--provider",
            "openai-chat-completions",
            "--model",
            "gpt-4.1-mini",
            "--base-url",
            "http://127.0.0.1:9999/v1/chat/completions",
        )
    }
    assert captured_args["persistent_agent_session"] is False
    assert "Actors: human-a, direct-provider" in captured.out
    assert "Completed Actors: direct-provider" in captured.out


def test_cli_play_shared_shard_wires_shared_external_agent_timeout_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_args: dict[str, object] = {}

    def _fake_run_human_shared_shard_session(**kwargs: object) -> HumanSharedShardSessionResult:
        captured_args.update(kwargs)
        return HumanSharedShardSessionResult(
            run_id="human-shared-shard",
            shard_id="shared-shard-local",
            scenario_id="tiny-fetch-quest",
            actor_ids=("human-a", "external-agent"),
            step_count=1,
            max_steps=5,
            completed_actor_ids=(),
            quit_requested=False,
            shard_mutation_generation=6,
            world_tick_count=1,
            last_world_tick_heartbeat="shared_shard_world_tick:0001",
            world_npc_stance_phase="watchful",
        )

    monkeypatch.setattr("cli.main.run_human_shared_shard_session", _fake_run_human_shared_shard_session)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
            "--shared-external-agent-timeout-seconds",
            "20",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert captured_args["external_agent_timeout_seconds"] == 20.0
    assert "Session Summary" in captured.out


def test_cli_play_shared_shard_rejects_persistent_session_without_agent_command_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"persistent_agent_session_requires_agent_command"}'
    )


def test_cli_play_shared_shard_rejects_actor_action_cadence_without_global_interval_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-action-cadence",
            "human-a=2",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"actor_action_cadence_requires_action_cadence_interval"}'
    )


def test_cli_play_shared_shard_rejects_explicit_cadence_with_native_speed_timing_mode_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--timing-mode",
            "native-speed",
            "--action-cadence-interval",
            "2",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"timing_mode_disallows_explicit_action_cadence"}'
    )


def test_cli_play_shared_shard_rejects_actor_override_with_equal_cadence_timing_mode_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--timing-mode",
            "equal-cadence",
            "--actor-action-cadence",
            "human-a=3",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"equal_cadence_timing_mode_disallows_actor_overrides"}'
    )


def test_cli_play_shared_shard_rejects_nonpositive_shared_external_agent_timeout_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--shared-external-agent-timeout-seconds",
            "0",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"shared_external_agent_timeout_seconds_must_be_positive"}'
    )


def test_cli_play_shared_shard_rejects_missing_direct_provider_config_machine_readably(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    monkeypatch.delenv("MUDBENCH_OPENAI_API_KEY", raising=False)
    monkeypatch.delenv("MUDBENCH_OPENAI_MODEL", raising=False)

    exit_code = main(
        [
            "play-shared-shard",
            "--scenario",
            "tiny-fetch-quest",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-model",
            "gpt-4.1-mini",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    assert captured.out.strip() == (
        '{"accepted":false,"error_type":"play_shared_shard_rejected",'
        '"reason":"direct_provider_missing_api_key:MUDBENCH_OPENAI_API_KEY"}'
    )


def test_human_console_rendering_is_deterministic() -> None:
    observation = Observation(
        run_id="human-console",
        step=2,
        location="watch-post",
        description="A narrow watch-post where a sentinel keeps vigil.",
        exits=("east", "south"),
        entities=(ObservedEntity(type="npc", name="sentinel"),),
        inventory=("relic-key",),
        health=100,
        messages=("The sentinel blocks the seal door.",),
        action_space=(
            "wait",
            "look",
            "move east",
            "move south",
            "use relic-key",
            "attack sentinel",
        ),
        remaining_steps=5,
    )

    rendered = render_human_observation(observation)

    assert rendered == "\n".join(
        (
            "Run: human-console",
            "Step: 3",
            "Remaining Steps: 5",
            "Location: watch-post",
            "Health: 100",
            "Description: A narrow watch-post where a sentinel keeps vigil.",
            "Exits: east, south",
            "NPCs: sentinel",
            "Inventory: relic-key",
            "Messages:",
            "  The sentinel blocks the seal door.",
            "Available Actions:",
            "  1. wait",
            "  2. look",
            "  3. move east",
            "  4. move south",
            "  5. use relic-key",
            "  6. attack sentinel",
            "Allowed Targets:",
            "  attack: sentinel",
            "  move: east, south",
            "  use: relic-key",
            "Enter an exact action, a 1-based action number, or a small alias like 'east', 'take key', or 'use key'.",
        )
    )


def test_human_console_alias_resolution_is_deterministic() -> None:
    observation = Observation(
        run_id="human-console",
        step=0,
        location="seal-door",
        description="A heavy sealed door blocks the final chamber.",
        exits=("west",),
        entities=(),
        inventory=("relic-key",),
        health=100,
        messages=(),
        action_space=("wait", "look", "move north", "move west", "use relic-key"),
        remaining_steps=3,
    )

    assert _resolve_console_action("north", observation) == "move north"
    assert _resolve_console_action("n", observation) == "move north"
    assert _resolve_console_action("west", observation) == "move west"
    assert _resolve_console_action("use key", observation) == "use relic-key"
    assert _resolve_console_action("look", observation) == "look"
    assert _resolve_console_action("5", observation) == "use relic-key"
    assert _resolve_console_action("move north", observation) == "move north"


def test_human_console_alias_resolution_rejects_ambiguous_or_invalid_input() -> None:
    observation = Observation(
        run_id="human-console",
        step=0,
        location="vault",
        description="A small vault with loose gear.",
        exits=("south",),
        entities=(),
        inventory=(),
        health=100,
        messages=(),
        action_space=("wait", "look", "take brass-key", "take silver-key", "move south"),
        remaining_steps=2,
    )

    with pytest.raises(ValueError, match="ambiguous_action_alias"):
        _resolve_console_action("take key", observation)
    with pytest.raises(ValueError, match="direction_not_available"):
        _resolve_console_action("north", observation)
    with pytest.raises(ValueError, match="action_alias_not_resolvable"):
        _resolve_console_action("use key", observation)


def test_cli_run_is_deterministic_for_identical_invocation(capsys: pytest.CaptureFixture[str]) -> None:
    first_exit = main(["run"])
    first_output = capsys.readouterr().out
    second_exit = main(["run"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output


def test_cli_run_rejects_invalid_scenario_selection() -> None:
    with pytest.raises(SystemExit) as exc_info:
        main(["run", "--scenario", "does-not-exist"])
    assert exc_info.value.code == 2


def test_cli_run_supports_scenario_file_loading(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario-file", "scenarios/canonical/tiny_fetch_quest.json"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"


def test_cli_run_rejects_missing_scenario_file_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--scenario-file", "scenarios/canonical/does-not-exist.json"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("scenario_file_read_failed:")


def test_cli_run_rejects_malformed_scenario_file_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    malformed_path = tmp_path / "bad-scenario.json"
    malformed_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(["run", "--scenario-file", str(malformed_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("scenario_file_invalid_json:")


def test_cli_run_external_local_agent_command_executes_deterministically(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"


def test_cli_run_examples_deterministic_rule_agent_executes_deterministically(
    capsys: pytest.CaptureFixture[str],
) -> None:
    command = f"{sys.executable} examples/agents/deterministic_rule_agent.py"

    first_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["run", "--scenario", "tiny-fetch-quest", "--actor-id", "agent-a", "--agent-command", command]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output


def test_cli_run_external_agent_label_surfaces_in_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["external_agent_label"] == "deterministic-wrapper"


def test_cli_run_agent_profile_executes_and_surfaces_profile_identity(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="rule-profile",
        display_name="Rule Profile",
        command=f"{sys.executable} {script_path}",
    )

    first_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["external_agent_profile_id"] == "rule-profile"
    assert payload["external_agent_label"] == "Rule Profile"


def test_cli_run_rejects_invalid_external_local_agent_command_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--agent-command", "/definitely/missing/mudbench-agent-binary"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert payload["reason"] == "external_agent_command_not_found:/definitely/missing/mudbench-agent-binary"


def test_cli_run_rejects_missing_agent_profile_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--agent-profile", "profiles/does-not-exist.json"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_run_rejects_malformed_agent_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    profile_path = tmp_path / "bad-profile.json"
    profile_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(["run", "--agent-profile", str(profile_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert str(payload["reason"]).startswith("json_payload_invalid:")


def test_cli_run_rejects_broken_persistent_agent_session_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_broken_persistent_agent_script(tmp_path)
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} {script_path}",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert "persistent_session_" in str(payload["reason"])


def test_cli_run_rejects_missing_direct_provider_config_machine_readably(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("MUDBENCH_OPENAI_API_KEY", raising=False)
    monkeypatch.delenv("MUDBENCH_OPENAI_MODEL", raising=False)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "run_rejected"
    assert payload["reason"] == "direct_provider_missing_api_key:MUDBENCH_OPENAI_API_KEY"


def test_cli_run_wires_direct_provider_through_external_agent_command_seam(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_config: dict[str, object] = {}

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
                "timing": {
                    "timing_mode": "human-parity",
                    "action_cadence_interval": 2,
                    "actor_action_cadence_overrides": [],
                    "actor_next_action_eligible_at": [
                        {"actor_id": "agent-a", "next_action_eligible_at": 2}
                    ],
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        captured_config["external_agent_command"] = config.external_agent_command
        captured_config["persistent_agent_session"] = config.persistent_agent_session
        captured_config["external_agent_timeout_seconds"] = config.external_agent_timeout_seconds
        captured_config["actor_ids"] = config.actor_ids
        captured_config["timing_mode"] = config.timing_mode
        captured_config["action_cadence_interval"] = config.action_cadence_interval
        captured_config["actor_action_cadence_overrides"] = config.actor_action_cadence_overrides
        return _FakeResult()

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["external_agent_label"] == "direct-provider:openai-chat-completions"
    assert payload["timing_mode"] == "human-parity"
    assert payload["action_cadence_interval"] == 2
    assert payload["actor_action_cadence_overrides"] == []
    assert payload["actor_next_action_eligible_at"] == [{"actor_id": "agent-a", "next_action_eligible_at": 2}]
    assert captured_config["persistent_agent_session"] is False
    assert captured_config["external_agent_timeout_seconds"] is None
    assert captured_config["actor_ids"] == ("agent-a",)
    assert captured_config["timing_mode"] is None
    assert captured_config["action_cadence_interval"] is None
    assert captured_config["actor_action_cadence_overrides"] is None
    external_agent_command = captured_config["external_agent_command"]
    assert isinstance(external_agent_command, tuple)
    assert external_agent_command[0] == sys.executable
    assert external_agent_command[1].endswith("src/agents/direct_provider_runner.py")
    assert external_agent_command[2:] == (
        "--provider",
        "openai-chat-completions",
        "--model",
        "gpt-4.1-mini",
        "--base-url",
        "https://api.openai.com/v1/chat/completions",
    )


def test_cli_run_wires_routed_prompt_engine_through_direct_provider_command(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_command: tuple[str, ...] | None = None

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        nonlocal captured_command
        captured_command = config.external_agent_command
        return _FakeResult()

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
            "--prompt-engine",
            "geometric-routed",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_command is not None
    assert captured_command[-2:] == ("--prompt-engine", "geometric-routed")


def test_cli_run_wires_direct_provider_prompt_dump_flags_through_command_seam(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_command: tuple[str, ...] | None = None

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        nonlocal captured_command
        captured_command = config.external_agent_command
        return _FakeResult()

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-prompt-dump-dir",
            "/tmp/direct-provider-dumps",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_command is not None
    assert captured_command[-6:] == (
        "--prompt-dump-dir",
        "/tmp/direct-provider-dumps",
        "--prompt-dump-actor-id",
        "agent-a",
        "--prompt-dump-scenario-id",
        "tiny-hidden-key",
    )


def test_cli_run_wires_angular_canonical_prompt_engine_and_variant_through_direct_provider_command(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_command: tuple[str, ...] | None = None

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        nonlocal captured_command
        captured_command = config.external_agent_command
        return _FakeResult()

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
            "--prompt-engine",
            "angular-canonical",
            "--router-variant",
            "angular-hopf-trans",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_command is not None
    assert captured_command[-4:] == (
        "--prompt-engine",
        "angular-canonical",
        "--router-variant",
        "angular-hopf-trans",
    )


def test_cli_run_threads_direct_provider_timeout_override_deterministically(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_timeout: float | None = None

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        nonlocal captured_timeout
        captured_timeout = config.external_agent_timeout_seconds
        return _FakeResult()

    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")
    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-timeout-seconds",
            "15",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_timeout == 15.0


def test_cli_run_rejects_direct_provider_timeout_without_direct_provider(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["run", "--direct-provider-timeout-seconds", "15"])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "run_rejected",
        "reason": "direct_provider_timeout_requires_direct_provider",
    }


def test_cli_run_rejects_non_positive_direct_provider_timeout(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("MUDBENCH_OPENAI_API_KEY", "test-key")
    monkeypatch.setenv("MUDBENCH_OPENAI_MODEL", "gpt-4.1-mini")

    exit_code = main(
        [
            "run",
            "--actor-id",
            "agent-a",
            "--direct-provider",
            "openai-chat-completions",
            "--direct-provider-timeout-seconds",
            "0",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "run_rejected",
        "reason": "direct_provider_timeout_seconds_must_be_positive",
    }


def test_cli_suite_tiny_emits_deterministic_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(["suite", "--suite", "tiny"])
    first_output = capsys.readouterr().out
    second_exit = main(["suite", "--suite", "tiny"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["benchmark_id"] == "mudbench-cli"
    assert payload["actor_ids"] == ["agent-a", "agent-b"]
    assert payload["report"]["schema_version"] == "tiny_suite_baseline_report_v1"
    assert payload["report"]["scenario_count"] == 5
    assert payload["report"]["entry_count"] == 10
    assert "timing_mode_aggregation" not in payload["report"]
    assert set(payload["report"]["entries"][0]) == {
        "scenario_id",
        "agent_id",
        "aggregate_score",
        "composite_score",
        "normalized_metrics",
        "contributions",
        "replay_ref",
        "parity_ref",
    }
    assert "timing_mode" not in payload
    assert "action_cadence_interval" not in payload


def test_cli_suite_tiny_with_timing_mode_emits_report_level_timing_aggregation_deterministically(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--timing-mode",
            "equal-cadence",
            "--action-cadence-interval",
            "3",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--timing-mode",
            "equal-cadence",
            "--action-cadence-interval",
            "3",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    timing_mode_aggregation = payload["report"]["timing_mode_aggregation"]
    assert timing_mode_aggregation == {
        "mode_count": 1,
        "run_count": 5,
        "entry_count": 10,
        "aggregate_score_total": timing_mode_aggregation["aggregate_score_total"],
        "aggregate_score_average": timing_mode_aggregation["aggregate_score_average"],
        "modes": [
            {
                "timing_mode": "equal-cadence",
                "run_count": 5,
                "entry_count": 10,
                "aggregate_score_total": timing_mode_aggregation["modes"][0]["aggregate_score_total"],
                "aggregate_score_average": timing_mode_aggregation["modes"][0]["aggregate_score_average"],
                "scenario_ids": [
                    "tiny-delayed-retrieval",
                    "tiny-fetch-quest",
                    "tiny-hidden-key",
                    "tiny-locked-path",
                    "tiny-social-trade",
                ],
            }
        ],
    }


def test_cli_suite_tiny_comparison_emits_deterministic_structured_output(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-a", "--candidate-agent", "agent-b"]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-a", "--candidate-agent", "agent-b"]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "agent-b"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    assert "timing_mode_aggregation" not in payload["report"]
    assert set(payload["report"]["comparisons"][0]["baseline"]) == {
        "scenario_id",
        "agent_id",
        "aggregate_score",
        "composite_score",
        "normalized_metrics",
        "contributions",
        "replay_ref",
        "parity_ref",
    }
    assert set(payload["report"]["comparisons"][0]["candidate"]) == {
        "scenario_id",
        "agent_id",
        "aggregate_score",
        "composite_score",
        "normalized_metrics",
        "contributions",
        "replay_ref",
        "parity_ref",
    }
    assert "composite_score_difference_total" in payload["report"]["summary"]


def test_cli_suite_tiny_comparison_with_timing_mode_emits_report_level_timing_aggregation(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--candidate-agent",
            "agent-b",
            "--timing-mode",
            "equal-cadence",
            "--action-cadence-interval",
            "3",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    timing_mode_aggregation = payload["report"]["timing_mode_aggregation"]
    assert timing_mode_aggregation["mode_count"] == 1
    assert timing_mode_aggregation["run_count"] == 5
    assert timing_mode_aggregation["entry_count"] == 10
    assert [entry["timing_mode"] for entry in timing_mode_aggregation["modes"]] == ["equal-cadence"]
    assert timing_mode_aggregation["modes"][0]["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]


def test_cli_suite_tiny_comparison_rejects_unsupported_actor_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        ["suite", "--suite", "tiny", "--baseline-agent", "agent-c", "--candidate-agent", "agent-b"]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert payload["reason"] == "unsupported baseline_agent: agent-c"


def test_cli_suite_tiny_external_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "external-local-agent"]
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "external-local-agent"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "external-local-agent"


def test_cli_run_wires_explicit_timing_mode_to_runner_config(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_config: dict[str, object] = {}

    class _FakeResult:
        def to_dict(self) -> dict[str, object]:
            return {
                "lifecycle_state": {
                    "run_id": "cli-run",
                    "scenario_id": "tiny-hidden-key",
                    "status": "finalized",
                    "step_index": 1,
                    "max_steps": 6,
                    "seed": 33,
                },
                "scorecard": {
                    "aggregate_score": 0.5,
                    "metadata": {
                        "benchmark_id": "mudbench-cli",
                        "scoring_version": "phase3-v1",
                    },
                },
                "replay_artifact_refs": [
                    {"name": "replay_artifact", "ref": "replay-1"},
                    {"name": "replay_checksum", "ref": "replay-1"},
                ],
                "replay_artifact": {
                    "envelope": {"schema_version": "1.0"},
                    "events": [{"event_type": "step"}],
                },
                "replay_parity_artifact": {
                    "terminal_step": 1,
                    "step_count": 1,
                    "terminal_state_hash": "a" * 64,
                    "applied_steps_hash": "b" * 64,
                    "score_summary_hash": "c" * 64,
                },
            }

    def _fake_run_benchmark_lifecycle(config):
        captured_config["timing_mode"] = config.timing_mode
        captured_config["action_cadence_interval"] = config.action_cadence_interval
        captured_config["actor_action_cadence_overrides"] = config.actor_action_cadence_overrides
        return _FakeResult()

    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--timing-mode",
            "equal-cadence",
            "--action-cadence-interval",
            "3",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    assert _read_json_output(captured.out)["accepted"] is True
    assert captured_config["timing_mode"] == "equal-cadence"
    assert captured_config["action_cadence_interval"] == 3
    assert captured_config["actor_action_cadence_overrides"] is None


def test_cli_suite_wires_explicit_timing_mode_to_all_runner_configs(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    captured_configs: list[object] = []

    class _FakeResult:
        def __init__(self, scenario_name: str) -> None:
            self.scenario_name = scenario_name

    def _fake_run_benchmark_lifecycle(config):
        captured_configs.append(config)
        return _FakeResult(config.scenario["scenario_id"])

    def _fake_build_tiny_suite_baseline_report(_results):
        return {
            "schema_version": "tiny_suite_baseline_report_v1",
            "benchmark_ids": ["mudbench-cli"],
            "scenario_count": 5,
            "entry_count": 10,
            "entries": [
                {
                    "scenario_id": "tiny-fetch-quest",
                    "agent_id": "agent-a",
                    "aggregate_score": 0.5,
                    "composite_score": 0.5,
                    "normalized_metrics": {"quest_completion": 0.5},
                    "contributions": {"quest_completion": 0.5},
                    "replay_ref": "sha256:abc",
                    "parity_ref": {
                        "terminal_state_hash": "a" * 64,
                        "applied_steps_hash": "b" * 64,
                        "score_summary_hash": "c" * 64,
                    },
                },
                {
                    "scenario_id": "tiny-fetch-quest",
                    "agent_id": "agent-b",
                    "aggregate_score": 0.5,
                    "composite_score": 0.5,
                    "normalized_metrics": {"quest_completion": 0.5},
                    "contributions": {"quest_completion": 0.5},
                    "replay_ref": "sha256:def",
                    "parity_ref": {
                        "terminal_state_hash": "a" * 64,
                        "applied_steps_hash": "b" * 64,
                        "score_summary_hash": "c" * 64,
                    },
                },
            ],
        }

    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr("cli.main.build_tiny_suite_baseline_report", _fake_build_tiny_suite_baseline_report)

    exit_code = main(["suite", "--suite", "tiny", "--timing-mode", "human-parity"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["timing_mode"] == "human-parity"
    assert payload["action_cadence_interval"] == 2
    assert payload["actor_action_cadence_overrides"] == []
    assert len(captured_configs) == 5
    for config in captured_configs:
        assert config.timing_mode == "human-parity"
        assert config.action_cadence_interval == 2
        assert config.actor_action_cadence_overrides is None


def test_cli_suite_rejects_explicit_cadence_with_native_speed_timing_mode_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--timing-mode",
            "native-speed",
            "--action-cadence-interval",
            "2",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "suite_rejected",
        "reason": "timing_mode_disallows_explicit_action_cadence",
    }


def test_cli_suite_tiny_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="rule-profile",
        display_name="Rule Profile",
        command=f"{sys.executable} {script_path}",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            str(profile_path),
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["agent-a", "rule-profile"]
    assert payload["report"]["candidate_agent_id"] == "rule-profile"
    assert payload["report"]["external_agent_profile_id"] == "rule-profile"
    assert payload["report"]["external_agent_label"] == "Rule Profile"


def test_cli_suite_tiny_dual_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    candidate_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="candidate-mock-profile",
        display_name="Candidate Mock Profile",
        command=f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        filename="candidate-agent-profile.json",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_label"] == "Baseline Rule Profile"
    assert payload["report"]["candidate_external_agent_label"] == "Candidate Mock Profile"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "baseline-rule-profile"
        assert entry["candidate"]["agent_id"] == "candidate-mock-profile"


def test_cli_suite_tiny_shared_dual_external_profile_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    candidate_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="candidate-mock-profile",
        display_name="Candidate Mock Profile",
        command=f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        filename="candidate-agent-profile.json",
    )

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
            "--external-agent-actor",
            "agent-b",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(candidate_profile_path),
            "--external-agent-actor",
            "agent-b",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert payload["report"]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert payload["report"]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "baseline-rule-profile"
        assert entry["candidate"]["agent_id"] == "candidate-mock-profile"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]


def test_cli_suite_tiny_external_comparison_rejects_invalid_external_command_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            "/definitely/missing/mudbench-agent-binary",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert payload["reason"] == "external_agent_command_not_found:/definitely/missing/mudbench-agent-binary"


def test_cli_suite_tiny_external_profile_rejects_missing_profile_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            "profiles/does-not-exist.json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_suite_tiny_dual_external_profile_rejects_missing_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            "profiles/missing-candidate-profile.json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("agent_profile_read_failed:")


def test_cli_suite_tiny_dual_external_profile_rejects_malformed_profile_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="baseline-rule-profile",
        display_name="Baseline Rule Profile",
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        filename="baseline-agent-profile.json",
    )
    malformed_profile_path = tmp_path / "bad-candidate-profile.json"
    malformed_profile_path.write_text("{not-json", encoding="utf-8")

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--baseline-agent-profile",
            str(baseline_profile_path),
            "--candidate-agent-profile",
            str(malformed_profile_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("json_payload_invalid:")


def test_cli_suite_tiny_mixed_external_comparison_emits_deterministic_structured_output(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_deterministic_agent_script(tmp_path)
    command = f"{sys.executable} {script_path}"

    first_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
        ]
    )
    first_output = capsys.readouterr().out
    second_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            command,
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
        ]
    )
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert payload["report"]["schema_version"] == "tiny_suite_comparison_report_v1"
    assert payload["report"]["baseline_agent_id"] == "agent-a"
    assert payload["report"]["candidate_agent_id"] == "deterministic-wrapper"
    assert payload["report"]["external_agent_label"] == "deterministic-wrapper"
    assert payload["report"]["scenario_count"] == 5
    assert len(payload["report"]["comparisons"]) == 5
    for entry in payload["report"]["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "deterministic-wrapper"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]


def test_cli_suite_output_file_rejects_unwritable_path_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["suite", "--suite", "tiny", "--output-file", str(tmp_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("output_file_write_failed:")


def test_cli_suite_output_manifest_rejects_unwritable_path_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)
    manifest_path.mkdir()

    exit_code = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert str(payload["reason"]).startswith("output_manifest_write_failed:")


def test_cli_suite_output_file_is_deterministic_for_identical_invocation(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

    first_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    first_stdout = capsys.readouterr().out
    first_file = output_path.read_text(encoding="utf-8")
    first_manifest = manifest_path.read_text(encoding="utf-8")
    first_replay = replay_path.read_text(encoding="utf-8")

    second_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    second_stdout = capsys.readouterr().out
    second_file = output_path.read_text(encoding="utf-8")
    second_manifest = manifest_path.read_text(encoding="utf-8")
    second_replay = replay_path.read_text(encoding="utf-8")

    assert first_exit == 0
    assert second_exit == 0
    assert first_stdout == second_stdout
    assert first_file == second_file
    assert first_manifest == second_manifest
    assert first_replay == second_replay


def test_cli_reports_list_and_show_are_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite.json"
    manifest_path = _manifest_path_for(output_path)

    write_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    capsys.readouterr()

    first_list_exit = main(["reports", "list", "--dir", str(tmp_path)])
    first_list_output = capsys.readouterr().out
    second_list_exit = main(["reports", "list", "--dir", str(tmp_path)])
    second_list_output = capsys.readouterr().out

    first_show_exit = main(["reports", "show", "--manifest", str(manifest_path)])
    first_show_output = capsys.readouterr().out
    second_show_exit = main(["reports", "show", "--manifest", str(manifest_path)])
    second_show_output = capsys.readouterr().out

    assert write_exit == 0
    assert first_list_exit == 0
    assert second_list_exit == 0
    assert first_list_output == second_list_output
    assert first_show_exit == 0
    assert second_show_exit == 0
    assert first_show_output == second_show_output

    list_payload = _read_json_output(first_list_output)
    assert list_payload["accepted"] is True
    assert list_payload["command"] == "reports_list"
    assert list_payload["artifact_count"] == 1

    show_payload = _read_json_output(first_show_output)
    assert show_payload["accepted"] is True
    assert show_payload["command"] == "reports_show"
    assert show_payload["artifact"]["manifest_path"] == str(manifest_path)
    assert show_payload["artifact"]["report_path"] == str(output_path)


def test_cli_reports_history_is_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite.json"
    external_output_path = tmp_path / "tiny-suite-mixed.json"

    write_baseline_exit = main(
        ["suite", "--suite", "tiny", "--output", "json", "--output-file", str(baseline_output_path)]
    )
    capsys.readouterr()
    write_external_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
            "--output-file",
            str(external_output_path),
        ]
    )
    capsys.readouterr()

    first_exit = main(["reports", "history", "--dir", str(tmp_path)])
    first_output = capsys.readouterr().out
    second_exit = main(["reports", "history", "--dir", str(tmp_path)])
    second_output = capsys.readouterr().out

    assert write_baseline_exit == 0
    assert write_external_exit == 0
    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["command"] == "reports_history"
    assert payload["artifact_count"] == 2
    assert len(payload["history"]) == 2
    assert len(payload["leaderboard"]) >= 2
    assert len(payload["identity_rollups"]) >= 2


def test_cli_reports_export_is_deterministic_for_identical_input(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite.json"
    external_output_path = tmp_path / "tiny-suite-mixed.json"

    write_baseline_exit = main(
        ["suite", "--suite", "tiny", "--output", "json", "--output-file", str(baseline_output_path)]
    )
    capsys.readouterr()
    write_external_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
            "--output-file",
            str(external_output_path),
        ]
    )
    capsys.readouterr()

    first_exit = main(["reports", "export", "--dir", str(tmp_path)])
    first_output = capsys.readouterr().out
    second_exit = main(["reports", "export", "--dir", str(tmp_path)])
    second_output = capsys.readouterr().out

    assert write_baseline_exit == 0
    assert write_external_exit == 0
    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["command"] == "reports_export"
    assert payload["viewmodel_version"] == "reports_export_viewmodel_v1"
    assert payload["artifact_count"] == 2
    assert len(payload["artifacts"]) == 2
    assert len(payload["history"]) == 2
    assert len(payload["leaderboard"]) >= 2
    assert len(payload["identity_rollups"]) >= 2
    assert len(payload["replay_drilldowns"]) == 2
    assert payload["coverage"]["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]
    assert payload["coverage"]["external_agent_labels"] == ["deterministic-wrapper"]
    assert payload["replay_drilldowns"][0]["replay_run_count"] == 5


def test_cli_reports_history_surfaces_unlabeled_raw_external_agent_gracefully(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite-external.json"

    write_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--output",
            "json",
            "--output-file",
            str(output_path),
        ]
    )
    capsys.readouterr()

    history_exit = main(["reports", "history", "--dir", str(tmp_path)])
    history_payload = _read_json_output(capsys.readouterr().out)
    export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert write_exit == 0
    assert history_exit == 0
    assert export_exit == 0
    assert history_payload["history"][0]["actor_ids"] == ["agent-a", "external-local-agent"]
    assert history_payload["history"][0]["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {"actor_id": "external-local-agent", "identity_type": "external_agent_command"},
    ]
    assert any(
        entry["actor_id"] == "external-local-agent" and entry["identity_type"] == "external_agent_command"
        for entry in history_payload["leaderboard"]
    )
    assert any(
        entry["identity_value"] == "external-local-agent"
        and entry["identity_type"] == "external_agent_command"
        and entry["comparison_artifact_count"] == 1
        and entry["has_comparison_artifacts"] is True
        for entry in history_payload["identity_rollups"]
    )
    assert export_payload["coverage"]["external_agent_labels"] == []
    assert export_payload["coverage"]["external_agent_profile_ids"] == []
    assert any(
        entry["identity_value"] == "external-local-agent"
        and entry["identity_type"] == "external_agent_command"
        and "has_shared_run_arena_artifacts" not in entry
        for entry in export_payload["identity_rollups"]
    )


def test_cli_reports_rejects_missing_or_malformed_inputs_machine_readably(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    missing_dir_exit = main(["reports", "list", "--dir", str(tmp_path / "missing-dir")])
    missing_dir_payload = _read_json_output(capsys.readouterr().out)
    missing_history_exit = main(["reports", "history", "--dir", str(tmp_path / "missing-dir")])
    missing_history_payload = _read_json_output(capsys.readouterr().out)
    missing_export_exit = main(["reports", "export", "--dir", str(tmp_path / "missing-dir")])
    missing_export_payload = _read_json_output(capsys.readouterr().out)

    malformed_manifest_path = tmp_path / "bad.manifest.json"
    malformed_manifest_path.write_text("{bad-json", encoding="utf-8")

    malformed_manifest_exit = main(["reports", "show", "--manifest", str(malformed_manifest_path)])
    malformed_manifest_payload = _read_json_output(capsys.readouterr().out)
    malformed_history_exit = main(["reports", "history", "--dir", str(tmp_path)])
    malformed_history_payload = _read_json_output(capsys.readouterr().out)
    malformed_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    malformed_export_payload = _read_json_output(capsys.readouterr().out)

    malformed_manifest_path.unlink()

    valid_output_path = tmp_path / "tiny-suite.json"
    write_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(valid_output_path)])
    capsys.readouterr()
    replay_path = _replay_path_for(valid_output_path)
    replay_path.unlink()
    missing_replay_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    missing_replay_export_payload = _read_json_output(capsys.readouterr().out)

    broken_output_path = tmp_path / "tiny-suite-broken.json"
    write_broken_exit = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(broken_output_path)])
    capsys.readouterr()
    broken_replay_path = _replay_path_for(broken_output_path)
    broken_replay_path.write_text("{bad-json", encoding="utf-8")
    broken_replay_export_exit = main(["reports", "export", "--dir", str(tmp_path)])
    broken_replay_export_payload = _read_json_output(capsys.readouterr().out)

    assert missing_dir_exit == 1
    assert missing_dir_payload["accepted"] is False
    assert missing_dir_payload["error_type"] == "reports_rejected"
    assert str(missing_dir_payload["reason"]).startswith("reports_dir_not_found:")

    assert missing_history_exit == 1
    assert missing_history_payload["accepted"] is False
    assert missing_history_payload["error_type"] == "reports_rejected"
    assert str(missing_history_payload["reason"]).startswith("reports_dir_not_found:")

    assert missing_export_exit == 1
    assert missing_export_payload["accepted"] is False
    assert missing_export_payload["error_type"] == "reports_rejected"
    assert str(missing_export_payload["reason"]).startswith("reports_dir_not_found:")

    assert malformed_manifest_exit == 1
    assert malformed_manifest_payload["accepted"] is False
    assert malformed_manifest_payload["error_type"] == "reports_rejected"
    assert str(malformed_manifest_payload["reason"]).startswith("json_payload_invalid:")

    assert malformed_history_exit == 1
    assert malformed_history_payload["accepted"] is False
    assert malformed_history_payload["error_type"] == "reports_rejected"
    assert str(malformed_history_payload["reason"]).startswith("json_payload_invalid:")

    assert malformed_export_exit == 1
    assert malformed_export_payload["accepted"] is False
    assert malformed_export_payload["error_type"] == "reports_rejected"
    assert str(malformed_export_payload["reason"]).startswith("json_payload_invalid:")

    assert write_exit == 0
    assert missing_replay_export_exit == 1
    assert missing_replay_export_payload["accepted"] is False
    assert missing_replay_export_payload["error_type"] == "reports_rejected"
    assert str(missing_replay_export_payload["reason"]).startswith("replay_drilldown_file_not_found:")

    assert write_broken_exit == 0
    assert broken_replay_export_exit == 1
    assert broken_replay_export_payload["accepted"] is False
    assert broken_replay_export_payload["error_type"] == "reports_rejected"
    assert str(broken_replay_export_payload["reason"]).startswith("json_payload_invalid:")


def test_cli_run_with_scenario_file_is_deterministic_for_identical_invocation(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(["run", "--scenario-file", "scenarios/canonical/tiny_hidden_key.json"])
    first_output = capsys.readouterr().out
    second_exit = main(["run", "--scenario-file", "scenarios/canonical/tiny_hidden_key.json"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output


# ── suite_timing_mode_eval_unblock_v1 tests ──────────────────────────────────


def test_cli_suite_human_parity_timing_mode_no_longer_rejects_on_lifecycle_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """human-parity must not raise via the live BenchmarkRunnerConfig/__post_init__ path."""
    exit_code = main(["suite", "--suite", "tiny", "--timing-mode", "human-parity"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["timing_mode"] == "human-parity"
    assert payload["action_cadence_interval"] == 2
    assert "timing_mode_aggregation" in payload["report"]
    agg = payload["report"]["timing_mode_aggregation"]
    assert agg["modes"][0]["timing_mode"] == "human-parity"


def test_cli_suite_direct_provider_timeout_wired_through_deterministically(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """--direct-provider-timeout-seconds must be threaded into every suite BenchmarkRunnerConfig."""
    captured_configs: list[object] = []

    def _fake_run_benchmark_lifecycle(config):
        captured_configs.append(config)

        class _R:
            scenario_name = config.scenario["scenario_id"]

        return _R()

    def _fake_build_tiny_suite_baseline_report(_results):
        return {
            "schema_version": "tiny_suite_baseline_report_v1",
            "benchmark_ids": ["mudbench-cli"],
            "scenario_count": 1,
            "entry_count": 2,
            "entries": [
                {
                    "scenario_id": "tiny-fetch-quest",
                    "agent_id": "agent-a",
                    "aggregate_score": 0.5,
                    "composite_score": 0.5,
                    "normalized_metrics": {"quest_completion": 0.5},
                    "contributions": {"quest_completion": 0.5},
                    "replay_ref": "sha256:abc",
                    "parity_ref": {
                        "terminal_state_hash": "a" * 64,
                        "applied_steps_hash": "b" * 64,
                        "score_summary_hash": "c" * 64,
                    },
                },
            ],
        }

    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr("cli.main.build_tiny_suite_baseline_report", _fake_build_tiny_suite_baseline_report)

    exit_code = main(
        ["suite", "--suite", "tiny", "--direct-provider-timeout-seconds", "30.0"]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert len(captured_configs) == 5
    for config in captured_configs:
        assert config.external_agent_timeout_seconds == 30.0


def test_cli_suite_default_timeout_preserved_when_not_specified(
    capsys: pytest.CaptureFixture[str],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Omitting --direct-provider-timeout-seconds leaves external_agent_timeout_seconds as None."""
    captured_configs: list[object] = []

    def _fake_run_benchmark_lifecycle(config):
        captured_configs.append(config)

        class _R:
            scenario_name = config.scenario["scenario_id"]

        return _R()

    def _fake_build_tiny_suite_baseline_report(_results):
        return {
            "schema_version": "tiny_suite_baseline_report_v1",
            "benchmark_ids": ["mudbench-cli"],
            "scenario_count": 1,
            "entry_count": 2,
            "entries": [
                {
                    "scenario_id": "tiny-fetch-quest",
                    "agent_id": "agent-a",
                    "aggregate_score": 0.5,
                    "composite_score": 0.5,
                    "normalized_metrics": {"quest_completion": 0.5},
                    "contributions": {"quest_completion": 0.5},
                    "replay_ref": "sha256:abc",
                    "parity_ref": {
                        "terminal_state_hash": "a" * 64,
                        "applied_steps_hash": "b" * 64,
                        "score_summary_hash": "c" * 64,
                    },
                },
            ],
        }

    monkeypatch.setattr("cli.main.run_benchmark_lifecycle", _fake_run_benchmark_lifecycle)
    monkeypatch.setattr("cli.main.build_tiny_suite_baseline_report", _fake_build_tiny_suite_baseline_report)

    exit_code = main(["suite", "--suite", "tiny"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert len(captured_configs) == 5
    for config in captured_configs:
        assert config.external_agent_timeout_seconds is None


def test_cli_suite_rejects_non_positive_direct_provider_timeout_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--direct-provider-timeout-seconds must be rejected machine-readably when <= 0."""
    for bad_value in ["0", "-1.5"]:
        exit_code = main(
            ["suite", "--suite", "tiny", "--direct-provider-timeout-seconds", bad_value]
        )
        captured = capsys.readouterr()
        assert exit_code == 1
        payload = _read_json_output(captured.out)
        assert payload == {
            "accepted": False,
            "error_type": "suite_rejected",
            "reason": "direct_provider_timeout_seconds_must_be_positive",
        }


def test_cli_suite_truly_invalid_timing_cadence_combination_still_rejects_machine_readably(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """native-speed + explicit cadence must still reject even after timing-mode fixes."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--timing-mode",
            "native-speed",
            "--action-cadence-interval",
            "3",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload == {
        "accepted": False,
        "error_type": "suite_rejected",
        "reason": "timing_mode_disallows_explicit_action_cadence",
    }


def test_cli_suite_row_level_output_unchanged_after_timing_mode_fixes(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Row-level entry fields must be preserved identically regardless of timing-mode fixes."""
    exit_code = main(["suite", "--suite", "tiny"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    entries = payload["report"]["entries"]
    assert len(entries) == 10
    required_fields = {
        "agent_id",
        "aggregate_score",
        "composite_score",
        "contributions",
        "normalized_metrics",
        "parity_ref",
        "replay_ref",
        "scenario_id",
    }
    for entry in entries:
        assert required_fields.issubset(set(entry.keys()))
    assert "timing_mode_aggregation" not in payload["report"]


def test_tiny_context_pressure_preset_carries_timing_consequence() -> None:
    """The hardcoded tiny-context-pressure CLI preset must include the cadence-efficiency
    timing consequence so CLI-driven evaluations activate the timing-sensitive scoring path."""
    from cli.main import _SCENARIO_PRESETS

    preset = _SCENARIO_PRESETS["tiny-context-pressure"]
    scenario_vars = preset["scenario_vars"]
    assert isinstance(scenario_vars, dict)
    assert scenario_vars.get("timing_consequence") == "cadence_efficiency"


# ---------------------------------------------------------------------------
# provider_budget_and_rate_controls tests
# ---------------------------------------------------------------------------


def test_provider_budget_default_absent_in_run_result(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """When --provider-min-turn-delay-seconds / --provider-max-actions are not given,
    provider_budget key must appear in the response with None/0 defaults."""
    exit_code = main(["run", "--scenario", "tiny-context-pressure"])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert "provider_budget" in payload
    pb = payload["provider_budget"]
    assert pb["provider_min_turn_delay_seconds"] is None
    assert pb["provider_max_actions"] is None
    assert isinstance(pb["provider_action_count"], int)


def test_provider_min_turn_delay_surfaces_in_result(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-min-turn-delay-seconds must be echoed back in provider_budget."""
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-context-pressure",
            "--provider-min-turn-delay-seconds",
            "0.0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    pb = payload["provider_budget"]
    assert pb["provider_min_turn_delay_seconds"] == 0.0
    assert pb["provider_max_actions"] is None


def test_provider_max_actions_surfaces_in_result(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions must be echoed back in provider_budget."""
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-context-pressure",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    pb = payload["provider_budget"]
    assert pb["provider_max_actions"] == 100


def test_provider_max_actions_zero_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions 0 must be rejected machine-readably."""
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-context-pressure",
            "--provider-max-actions",
            "0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert "provider_max_actions_must_be_positive" in payload["reason"]


def test_provider_min_turn_delay_negative_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Negative --provider-min-turn-delay-seconds must be rejected machine-readably."""
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-context-pressure",
            "--provider-min-turn-delay-seconds",
            "-1.0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert "provider_min_turn_delay_seconds_must_be_non_negative" in payload["reason"]


def test_provider_budget_does_not_affect_scripted_run(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """provider_max_actions with a scripted-only run must complete without error (scripted
    actors are not counted as provider actors so the cap never triggers)."""
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-context-pressure",
            "--provider-max-actions",
            "1",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    pb = payload["provider_budget"]
    assert pb["provider_action_count"] == 0
    assert pb["provider_max_actions"] == 1


def test_provider_budget_row_level_output_backward_compatible(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """provider_budget must be present and all core scorecard fields must survive unchanged."""
    exit_code = main(["run", "--scenario", "tiny-context-pressure"])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert "provider_budget" in payload
    assert "scorecard" in payload
    assert "aggregate_score" in payload["scorecard"]


def test_build_provider_actor_ids_no_external_command() -> None:
    """No external command → empty provider actor set."""
    from evaluation.benchmark_runner.runner import _build_provider_actor_ids

    result = _build_provider_actor_ids(
        actor_ids=["a1", "a2"],
        external_agent_command=None,
        external_agent_commands_by_actor=None,
        external_agent_actor_id=None,
    )
    assert result == frozenset()


def test_build_provider_actor_ids_single_external_command_no_actor_id() -> None:
    """Single external command with no actor_id restriction → all actor IDs."""
    from evaluation.benchmark_runner.runner import _build_provider_actor_ids

    result = _build_provider_actor_ids(
        actor_ids=["a1"],
        external_agent_command=["python", "agent.py"],
        external_agent_commands_by_actor=None,
        external_agent_actor_id=None,
    )
    assert result == frozenset({"a1"})


def test_build_provider_actor_ids_single_external_command_with_actor_id() -> None:
    """Single external command restricted to one actor_id → only that actor."""
    from evaluation.benchmark_runner.runner import _build_provider_actor_ids

    result = _build_provider_actor_ids(
        actor_ids=["a1", "a2"],
        external_agent_command=["python", "agent.py"],
        external_agent_commands_by_actor=None,
        external_agent_actor_id="a2",
    )
    assert result == frozenset({"a2"})


def test_build_provider_actor_ids_commands_by_actor() -> None:
    """Per-actor command map → provider set matches map keys exactly."""
    from evaluation.benchmark_runner.runner import _build_provider_actor_ids

    result = _build_provider_actor_ids(
        actor_ids=["a1", "a2", "a3"],
        external_agent_command=None,
        external_agent_commands_by_actor={"a1": ["python", "agent1.py"], "a3": ["python", "agent3.py"]},
        external_agent_actor_id=None,
    )
    assert result == frozenset({"a1", "a3"})


# ---------------------------------------------------------------------------
# provider_budget_suite_wiring tests
# ---------------------------------------------------------------------------


def test_suite_provider_budget_absent_by_default(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Default suite run must not include provider_budget key in response."""
    exit_code = main(["suite", "--suite", "tiny"])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" not in payload


def test_suite_provider_min_turn_delay_surfaces_in_response(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-min-turn-delay-seconds must appear in suite response provider_budget."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-min-turn-delay-seconds",
            "0.0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    pb = payload["provider_budget"]
    assert pb["provider_min_turn_delay_seconds"] == 0.0
    assert pb["provider_max_actions"] is None


def test_suite_provider_max_actions_surfaces_in_response(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions must appear in suite response provider_budget."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    pb = payload["provider_budget"]
    assert pb["provider_max_actions"] == 100
    assert pb["provider_min_turn_delay_seconds"] is None


def test_suite_provider_max_actions_zero_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions 0 must be rejected machine-readably on suite path."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-max-actions",
            "0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert "provider_max_actions_must_be_positive" in payload["reason"]


def test_suite_provider_min_turn_delay_negative_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Negative --provider-min-turn-delay-seconds must be rejected machine-readably on suite path."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-min-turn-delay-seconds",
            "-0.5",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "suite_rejected"
    assert "provider_min_turn_delay_seconds_must_be_non_negative" in payload["reason"]


def test_suite_provider_budget_does_not_affect_scripted_suite(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """provider_max_actions on a scripted-only suite must complete without early termination."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-max-actions",
            "1",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    assert payload["provider_budget"]["provider_max_actions"] == 1
    entries = payload["report"]["entries"]
    assert len(entries) == 10


def test_suite_provider_budget_row_level_output_backward_compatible(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Suite row-level fields must be unchanged when provider budget args are given."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    entries = payload["report"]["entries"]
    assert len(entries) == 10
    required_fields = {
        "agent_id",
        "aggregate_score",
        "composite_score",
        "contributions",
        "normalized_metrics",
        "parity_ref",
        "replay_ref",
        "scenario_id",
    }
    for entry in entries:
        assert required_fields.issubset(set(entry.keys()))


def test_suite_timing_mode_aggregation_intact_with_provider_budget(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """timing_mode_aggregation must still appear correctly when provider budget args are given."""
    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--timing-mode",
            "off",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "timing_mode_aggregation" in payload["report"]
    agg = payload["report"]["timing_mode_aggregation"]
    assert agg["mode_count"] == 1
    assert isinstance(agg["entry_count"], int)
    assert len(agg["modes"]) == 1
    assert agg["modes"][0]["timing_mode"] == "off"


# ---------------------------------------------------------------------------
# provider_budget_compare_wiring tests
# ---------------------------------------------------------------------------


def test_compare_provider_budget_absent_by_default(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Default compare run must not include provider_budget key in response."""
    exit_code = main(["compare-playable-slices"])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" not in payload


def test_compare_provider_min_turn_delay_surfaces_in_response(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-min-turn-delay-seconds must appear in compare response provider_budget."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-min-turn-delay-seconds",
            "0.0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    pb = payload["provider_budget"]
    assert pb["provider_min_turn_delay_seconds"] == 0.0
    assert pb["provider_max_actions"] is None


def test_compare_provider_max_actions_surfaces_in_response(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions must appear in compare response provider_budget."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    pb = payload["provider_budget"]
    assert pb["provider_max_actions"] == 100
    assert pb["provider_min_turn_delay_seconds"] is None


def test_compare_provider_max_actions_zero_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """--provider-max-actions 0 must be rejected machine-readably on compare path."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-max-actions",
            "0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "playable_slice_comparison_rejected"
    assert "provider_max_actions_must_be_positive" in payload["reason"]


def test_compare_provider_min_turn_delay_negative_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Negative --provider-min-turn-delay-seconds must be rejected machine-readably on compare path."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-min-turn-delay-seconds",
            "-1.0",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert payload["error_type"] == "playable_slice_comparison_rejected"
    assert "provider_min_turn_delay_seconds_must_be_non_negative" in payload["reason"]


def test_compare_provider_budget_does_not_affect_scripted_compare(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """provider_max_actions on a scripted-only compare must complete with all entries intact."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-max-actions",
            "1",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert "provider_budget" in payload
    assert payload["provider_budget"]["provider_max_actions"] == 1
    assert payload["entry_count"] > 0


def test_compare_row_level_output_backward_compatible_with_provider_budget(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Compare entries must have required fields when provider budget args are given."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-max-actions",
            "100",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    required_top_level = {"comparison_schema", "benchmark_id", "actor_id", "scenario_ids", "mode_ids", "entry_count", "entries"}
    assert required_top_level.issubset(set(payload.keys()))
    for entry in payload["entries"]:
        assert "mode" in entry
        assert "scenario_id" in entry
        assert "aggregate_score" in entry


def test_compare_provider_budget_both_controls_surfaced(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Both controls must appear in provider_budget when both are set."""
    exit_code = main(
        [
            "compare-playable-slices",
            "--provider-min-turn-delay-seconds",
            "0.5",
            "--provider-max-actions",
            "50",
        ]
    )
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    pb = payload["provider_budget"]
    assert pb["provider_min_turn_delay_seconds"] == 0.5
    assert pb["provider_max_actions"] == 50




# ---------------------------------------------------------------------------
# World persistence CLI argument tests
# ---------------------------------------------------------------------------


def test_run_world_save_path_accepted(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-save-path is accepted and saves a snapshot on run completion."""
    import os
    save_path = str(tmp_path / "world_snapshot.json")
    exit_code = main(["run", "--world-save-path", save_path])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert os.path.isfile(save_path), "world snapshot file must be created after run"


def test_run_world_load_path_nonexistent_rejected(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-load-path must reject a nonexistent file before run starts."""
    missing_path = str(tmp_path / "does_not_exist.json")
    exit_code = main(["run", "--world-load-path", missing_path])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert "world_load_path_not_found" in payload["reason"]


def test_run_world_persistence_default_behavior_unchanged(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Default run behavior (no persistence args) is unchanged."""
    exit_code = main(["run"])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True


def test_run_world_save_and_load_roundtrip(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """Save after first run; load path is accepted for second run."""
    import os
    save_path = str(tmp_path / "world.json")
    exit_code = main(["run", "--world-save-path", save_path])
    capsys.readouterr()
    assert exit_code == 0
    assert os.path.isfile(save_path)

    exit_code2 = main(["run", "--world-load-path", save_path])
    captured2 = capsys.readouterr()
    assert exit_code2 == 0
    payload2 = _read_json_output(captured2.out)
    assert payload2["accepted"] is True


def test_play_shared_shard_world_save_path_arg_accepted(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """--world-save-path is accepted by play-shared-shard subparser."""
    save_path = str(tmp_path / "shard_world.json")
    inputs = iter(["quit"])
    monkeypatch.setattr("builtins.input", lambda _: next(inputs))
    main(["play-shared-shard", "--world-save-path", save_path])
    captured = capsys.readouterr()
    assert "world_load_path_not_found" not in captured.out


def test_play_shared_shard_world_load_path_nonexistent_rejected(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-load-path must reject a nonexistent file on play-shared-shard."""
    missing_path = str(tmp_path / "no_shard.json")
    exit_code = main(["play-shared-shard", "--world-load-path", missing_path])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert "world_load_path_not_found" in payload["reason"]


# ---------------------------------------------------------------------------
# World save slot CLI argument tests
# ---------------------------------------------------------------------------


def test_run_world_save_slot_accepted(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-save-slot creates a named .json file in --save-dir after run."""
    import os
    save_dir = str(tmp_path / "saves")
    exit_code = main(["run", "--world-save-slot", "my-run", "--save-dir", save_dir])
    capsys.readouterr()
    assert exit_code == 0
    assert os.path.isfile(os.path.join(save_dir, "my-run.json"))


def test_run_world_load_slot_missing_rejects(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-load-slot for a nonexistent slot fails during lifecycle (not CLI validation)."""
    save_dir = str(tmp_path / "saves")
    exit_code = main(["run", "--world-load-slot", "ghost", "--save-dir", save_dir])
    capsys.readouterr()
    assert exit_code == 1


def test_run_world_save_slot_and_path_mutually_exclusive(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-save-slot and --world-save-path cannot be combined."""
    exit_code = main([
        "run",
        "--world-save-slot", "my-save",
        "--world-save-path", str(tmp_path / "snap.json"),
    ])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False


def test_run_world_load_slot_and_path_mutually_exclusive(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-load-slot and --world-load-path cannot be combined."""
    # Create a dummy snap file so --world-load-path does not fail on file-not-found
    snap_path = str(tmp_path / "snap.json")
    with open(snap_path, "w") as f:
        f.write("{}")
    exit_code = main([
        "run",
        "--world-load-slot", "my-save",
        "--world-load-path", snap_path,
    ])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False


def test_run_world_save_slot_invalid_name_rejected(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Invalid slot names are rejected clearly."""
    exit_code = main(["run", "--world-save-slot", "bad name!"])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
    assert "world_save_slot" in payload["reason"]


def test_run_world_save_slot_and_load_roundtrip(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """Save slot after first run, load slot in second run."""
    import os
    save_dir = str(tmp_path / "saves")
    exit_code = main(["run", "--world-save-slot", "session1", "--save-dir", save_dir])
    capsys.readouterr()
    assert exit_code == 0
    assert os.path.isfile(os.path.join(save_dir, "session1.json"))

    exit_code2 = main(["run", "--world-load-slot", "session1", "--save-dir", save_dir])
    captured2 = capsys.readouterr()
    assert exit_code2 == 0
    payload2 = _read_json_output(captured2.out)
    assert payload2["accepted"] is True


def test_list_saves_command_empty_dir(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """list-saves returns accepted with empty slots for an empty directory."""
    save_dir = str(tmp_path / "saves")
    exit_code = main(["list-saves", "--save-dir", save_dir])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["slots"] == []


def test_list_saves_command_shows_saved_slot(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """list-saves shows a slot that was saved via --world-save-slot."""
    save_dir = str(tmp_path / "saves")
    main(["run", "--world-save-slot", "mygame", "--save-dir", save_dir])
    capsys.readouterr()

    exit_code = main(["list-saves", "--save-dir", save_dir])
    captured = capsys.readouterr()
    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    slot_names = [s["slot_name"] for s in payload["slots"]]
    assert "mygame" in slot_names


def test_play_shared_shard_world_save_slot_accepted(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """--world-save-slot is accepted on play-shared-shard."""
    import os
    save_dir = str(tmp_path / "saves")
    inputs = iter(["quit"])
    monkeypatch.setattr("builtins.input", lambda _: next(inputs))
    main(["play-shared-shard", "--world-save-slot", "shardsave", "--save-dir", save_dir])
    capsys.readouterr()
    assert os.path.isfile(os.path.join(save_dir, "shardsave.json"))


def test_play_shared_shard_world_load_slot_missing_rejects(
    capsys: pytest.CaptureFixture[str],
    tmp_path: "Path",
) -> None:
    """--world-load-slot for nonexistent slot rejects on play-shared-shard."""
    save_dir = str(tmp_path / "saves")
    exit_code = main([
        "play-shared-shard",
        "--world-load-slot", "no-such-slot",
        "--save-dir", save_dir,
    ])
    captured = capsys.readouterr()
    assert exit_code == 1
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is False
