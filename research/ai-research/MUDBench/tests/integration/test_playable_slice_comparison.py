from __future__ import annotations

import json
import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def test_cli_compare_playable_slices_runs_built_in_and_mock_wrapper_modes_deterministically(
    capsys: pytest.CaptureFixture[str],
) -> None:
    first_exit = main(["compare-playable-slices"])
    first_output = capsys.readouterr().out
    second_exit = main(["compare-playable-slices"])
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["accepted"] is True
    assert payload["comparison_schema"] == "playable_slice_comparison_v1"
    assert payload["scenario_ids"] == [
        "tiny-guarded-relic",
        "tiny-hazard-route",
        "tiny-delayed-cost",
        "tiny-context-pressure",
    ]
    assert payload["mode_ids"] == ["built_in", "mock_wrapper"]
    assert payload["entry_count"] == 8
    assert "timing_mode_aggregation" not in payload

    entries = payload["entries"]
    assert [entry["scenario_id"] for entry in entries[0:8:2]] == [
        "tiny-guarded-relic",
        "tiny-hazard-route",
        "tiny-delayed-cost",
        "tiny-context-pressure",
    ]
    assert all(entry["aggregate_score"] > 0.0 for entry in entries)
    assert all(isinstance(entry["objective_completed"], bool) for entry in entries)

    built_in_entries = [entry for entry in entries if entry["mode"] == "built_in"]
    wrapper_entries = [entry for entry in entries if entry["mode"] == "mock_wrapper"]

    assert len(built_in_entries) == 4
    assert len(wrapper_entries) == 4
    assert all(entry["runtime_telemetry"] is None for entry in built_in_entries)
    assert all(entry["agent_identity"] == "mock-llm-wrapper" for entry in wrapper_entries)


def test_cli_compare_playable_slices_can_surface_baseline_and_routed_provider_modes_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str) -> None:
            self.scenario_id = scenario_id

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
        return _FakeResult(str(getattr(config, "scenario")["scenario_id"]))

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
            "objective_completed": mode == "built_in",
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

    argv = [
        "compare-playable-slices",
        "--direct-provider",
        "openai-chat-completions",
        "--direct-provider-model",
        "gpt-4.1-mini",
        "--include-routed-prompt-engine",
    ]
    first_exit = main(argv)
    first_output = capsys.readouterr().out
    second_exit = main(argv)
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
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


def test_cli_compare_playable_slices_can_surface_baseline_routed_canonical_angular_and_legacy_proxy_modes_deterministically(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str) -> None:
            self.scenario_id = scenario_id

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
        return _FakeResult(str(getattr(config, "scenario")["scenario_id"]))

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
            "objective_completed": mode == "built_in",
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

    argv = [
        "compare-playable-slices",
        "--direct-provider",
        "openai-chat-completions",
        "--direct-provider-model",
        "gpt-4.1-mini",
        "--include-routed-prompt-engine",
        "--include-angular-canonical-prompt-engine",
        "--angular-router-variant",
        "angular-hopf-trans",
        "--include-legacy-router-backed-prompt-engine",
        "--legacy-router-variant",
        "legacy-phase4d_hopf_transport",
    ]
    first_exit = main(argv)
    first_output = capsys.readouterr().out
    second_exit = main(argv)
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["mode_ids"] == [
        "built_in",
        "mock_wrapper",
        "direct_provider",
        "direct_provider_routed",
        "direct_provider_angular_canonical",
        "direct_provider_legacy_router_backed",
    ]
    assert payload["entry_count"] == 24
    angular_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_angular_canonical"
    ]
    legacy_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_legacy_router_backed"
    ]
    assert len(angular_entries) == 4
    assert len(legacy_entries) == 4
    assert all(entry["prompt_engine"] == "angular-canonical" for entry in angular_entries)
    assert all(entry["router_variant"] == "angular-hopf-trans" for entry in angular_entries)
    assert all(entry["prompt_engine"] == "legacy-router-backed" for entry in legacy_entries)
    assert all(
        entry["router_variant"] == "legacy-phase4d_hopf_transport" for entry in legacy_entries
    )


def test_cli_compare_playable_slices_threads_timeout_override_through_real_comparison_shape(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    observed_timeouts: list[float | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str) -> None:
            self.scenario_id = scenario_id

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
            observed_timeouts.append(getattr(config, "external_agent_timeout_seconds"))
        return _FakeResult(str(getattr(config, "scenario")["scenario_id"]))

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
            "objective_completed": mode == "built_in",
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
            "--angular-router-variant",
            "angular-hopf-trans",
            "--include-legacy-router-backed-prompt-engine",
            "--legacy-router-variant",
            "legacy-phase4d_hopf_product_phase",
            "--direct-provider-comparison-timeout-seconds",
            "6.0",
        ]
    )
    output = capsys.readouterr().out

    assert exit_code == 0
    payload = _read_json_output(output)
    assert payload["accepted"] is True
    assert observed_timeouts == [6.0] * 20


def test_cli_compare_playable_slices_can_surface_multiple_angular_variants_and_legacy_proxy_in_one_artifact(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str) -> None:
            self.scenario_id = scenario_id

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
        return _FakeResult(str(getattr(config, "scenario")["scenario_id"]))

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
            "objective_completed": mode == "built_in",
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

    argv = [
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
        "--include-legacy-router-backed-prompt-engine",
        "--legacy-router-variant",
        "legacy-phase4d_hopf_transport",
    ]
    first_exit = main(argv)
    first_output = capsys.readouterr().out
    second_exit = main(argv)
    second_output = capsys.readouterr().out

    assert first_exit == 0
    assert second_exit == 0
    assert first_output == second_output

    payload = _read_json_output(first_output)
    assert payload["mode_ids"] == [
        "built_in",
        "mock_wrapper",
        "direct_provider",
        "direct_provider_angular_canonical",
        "direct_provider_legacy_router_backed",
    ]
    assert payload["entry_count"] == 24
    baseline_entries = [entry for entry in payload["entries"] if entry["mode"] == "direct_provider"]
    angular_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_angular_canonical"
    ]
    legacy_entries = [
        entry for entry in payload["entries"] if entry["mode"] == "direct_provider_legacy_router_backed"
    ]
    assert len(baseline_entries) == 4
    assert len(angular_entries) == 8
    assert len(legacy_entries) == 4
    assert all(entry["prompt_engine"] == "angular-canonical" for entry in angular_entries)
    assert {entry["router_variant"] for entry in angular_entries} == {
        "angular-hopf-base",
        "angular-hopf-trans",
    }
    assert all(entry["prompt_engine"] == "legacy-router-backed" for entry in legacy_entries)
    assert all(
        entry["router_variant"] == "legacy-phase4d_hopf_transport" for entry in legacy_entries
    )


def test_cli_compare_playable_slices_threads_prompt_dump_flags_through_direct_provider_rows(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    captured_commands: list[tuple[str, ...] | None] = []

    class _FakeProviderConfig:
        provider = "openai-chat-completions"

    class _FakeResult:
        def __init__(self, scenario_id: str) -> None:
            self.scenario_id = scenario_id

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
        return _FakeResult(str(getattr(config, "scenario")["scenario_id"]))

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
            "--include-legacy-router-backed-prompt-engine",
            "--legacy-router-variant",
            "legacy-phase4d_hopf_transport",
            "--direct-provider-comparison-prompt-dump-dir",
            "/tmp/compare-prompt-dumps",
        ]
    )
    output = capsys.readouterr().out

    assert exit_code == 0
    assert _read_json_output(output)["accepted"] is True
    direct_commands = [
        command
        for command in captured_commands
        if command is not None and command[1] == "src/agents/direct_provider_runner.py"
    ]
    assert len(direct_commands) == 16
    assert all("--prompt-dump-dir" in command for command in direct_commands)
    assert all(command[command.index("--prompt-dump-dir") + 1] == "/tmp/compare-prompt-dumps" for command in direct_commands)
    assert {command[command.index("--prompt-dump-scenario-id") + 1] for command in direct_commands} == {
        "tiny-guarded-relic",
        "tiny-hazard-route",
        "tiny-delayed-cost",
        "tiny-context-pressure",
    }
