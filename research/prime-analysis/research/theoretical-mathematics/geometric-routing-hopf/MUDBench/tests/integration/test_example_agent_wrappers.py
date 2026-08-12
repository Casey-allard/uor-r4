from __future__ import annotations

import json
import sys
from pathlib import Path
import builtins

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def _write_external_agent_profile(tmp_path: Path, *, command: str) -> Path:
    profile_path = tmp_path / "example-agent-profile.json"
    profile_path.write_text(
        json.dumps(
            {
                "agent_id": "example-rule-profile",
                "display_name": "Example Rule Profile",
                "command": command,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return profile_path


def test_example_deterministic_rule_agent_runs_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["replay"]["event_count"] > 0
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_example_deterministic_rule_agent_profile_runs_through_real_cli_runtime_path(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    profile_path = _write_external_agent_profile(
        tmp_path,
        command=f"{sys.executable} examples/agents/deterministic_rule_agent.py",
    )

    exit_code = main(
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
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["external_agent_profile_id"] == "example-rule-profile"
    assert payload["external_agent_label"] == "Example Rule Profile"


def test_example_mock_llm_wrapper_runs_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-hidden-key"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["replay"]["event_count"] > 0


def test_example_mock_llm_wrapper_repairs_one_invalid_first_output_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py --emit-invalid-first",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-hidden-key"
    assert payload["lifecycle"]["status"] == "finalized"


def test_example_mock_llm_wrapper_fails_closed_after_invalid_repair_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hidden-key",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py --emit-invalid-always",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-hidden-key"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] > 0


def test_example_mock_llm_wrapper_meaningfully_attempts_guarded_relic_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-guarded-relic",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-guarded-relic"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_example_mock_llm_wrapper_meaningfully_attempts_hazard_route_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-hazard-route",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-hazard-route"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_example_mock_llm_wrapper_meaningfully_attempts_delayed_cost_through_real_cli_runtime_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-delayed-cost",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} examples/agents/mock_llm_wrapper.py",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-delayed-cost"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] == 8
    assert payload["replay"]["event_count"] > payload["lifecycle"]["step_count"] * 2
    assert payload["scorecard"]["aggregate_score"] > 0.0


def test_example_mock_llm_wrapper_participates_through_real_shared_shard_cli_path(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

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
    assert "Actor Turn: external-agent" in captured.out
    assert "External Agent Selected Action: move east" in captured.out
    assert "External Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: external-agent" in captured.out


def test_example_mock_llm_wrapper_participates_through_real_persistent_shared_shard_cli_path(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    scripted_inputs = iter(
        (
            "east",
            "east",
            "wait",
        )
    )
    monkeypatch.setattr(builtins, "input", lambda _prompt="": next(scripted_inputs))

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
    assert "Actor Turn: external-agent" in captured.out
    assert "External Agent Selected Action: move east" in captured.out
    assert "External Agent Selected Action: take golden-key" in captured.out
    assert "Objective complete for: external-agent" in captured.out
