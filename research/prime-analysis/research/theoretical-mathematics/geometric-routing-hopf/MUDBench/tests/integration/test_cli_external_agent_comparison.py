from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

from cli.main import main


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


def _write_external_agent_profile(
    tmp_path: Path,
    *,
    agent_id: str,
    display_name: str,
    command: str,
    filename: str,
) -> Path:
    profile_path = tmp_path / filename
    profile_path.write_text(
        json.dumps(
            {
                "agent_id": agent_id,
                "display_name": display_name,
                "command": command,
                "persistent_agent_session": False,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    return profile_path


def test_cli_tiny_suite_builtin_vs_external_comparison_emits_expected_report_shape(
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
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--output",
            "json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)

    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "external-local-agent"]
    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_comparison_report_v1"
    assert report["baseline_agent_id"] == "agent-a"
    assert report["candidate_agent_id"] == "external-local-agent"
    assert report["scenario_count"] == 5
    assert len(report["comparisons"]) == 5

    scenario_ids = [entry["scenario_id"] for entry in report["comparisons"]]
    assert scenario_ids == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    difference_total = 0.0
    for entry in report["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "external-local-agent"
        expected_difference = (
            float(entry["candidate"]["composite_score"]) - float(entry["baseline"]["composite_score"])
        )
        assert entry["composite_score_difference"] == expected_difference
        difference_total += expected_difference

    assert report["summary"]["composite_score_difference_total"] == pytest.approx(difference_total)


def test_cli_tiny_suite_builtin_vs_external_profile_comparison_writes_profile_identity_context(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    command = f"{sys.executable} examples/agents/deterministic_rule_agent.py"
    profile_path = _write_external_agent_profile(
        tmp_path,
        agent_id="comparison-rule-profile",
        display_name="Comparison Rule Profile",
        command=command,
        filename="comparison-agent-profile.json",
    )
    output_path = tmp_path / "tiny-suite-profile-comparison.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

    exit_code = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--baseline-agent",
            "agent-a",
            "--agent-profile",
            str(profile_path),
            "--output",
            "json",
            "--output-file",
            str(output_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))
    replay_payload = _read_json_output(replay_path.read_text(encoding="utf-8"))

    assert stdout_payload["accepted"] is True
    assert stdout_payload["actor_ids"] == ["agent-a", "comparison-rule-profile"]
    assert stdout_payload["report"]["candidate_agent_id"] == "comparison-rule-profile"
    assert stdout_payload["report"]["external_agent_profile_id"] == "comparison-rule-profile"
    assert stdout_payload["report"]["external_agent_label"] == "Comparison Rule Profile"
    assert manifest_payload["actor_ids"] == ["agent-a", "comparison-rule-profile"]
    assert manifest_payload["external_agent_profile_id"] == "comparison-rule-profile"
    assert manifest_payload["external_agent_label"] == "Comparison Rule Profile"
    assert replay_payload["run_count"] == 10

    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert export_exit == 0
    assert export_payload["history"][0]["external_agent_profile_id"] == "comparison-rule-profile"
    assert export_payload["history"][0]["external_agent_label"] == "Comparison Rule Profile"
    assert export_payload["history"][0]["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {
            "actor_id": "comparison-rule-profile",
            "identity_type": "external_agent_profile",
            "external_agent_label": "Comparison Rule Profile",
            "external_agent_profile_id": "comparison-rule-profile",
        },
    ]
    assert len(export_payload["leaderboard"]) == 2
    assert any(
        entry
        == {
            "actor_id": "comparison-rule-profile",
            "identity_type": "external_agent_profile",
            "artifact_count": 1,
            "suite_ids": ["tiny"],
            "benchmark_ids": ["mudbench-cli"],
            "score_total": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
            "external_agent_profile_id": "comparison-rule-profile",
            "external_agent_label": "Comparison Rule Profile",
        }
        for entry in export_payload["leaderboard"]
    )
    assert any(
        entry
        == {
            "actor_id": "agent-a",
            "identity_type": "built_in_actor",
            "artifact_count": 1,
            "suite_ids": ["tiny"],
            "benchmark_ids": ["mudbench-cli"],
            "score_total": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
        }
        for entry in export_payload["leaderboard"]
    )
    profile_rollups = {entry["identity_value"]: entry for entry in export_payload["identity_rollups"]}
    assert profile_rollups["agent-a"] == {
        "identity_type": "built_in_actor",
        "identity_value": "agent-a",
        "actor_id": "agent-a",
        "scenario_ids": [
            "tiny-delayed-retrieval",
            "tiny-fetch-quest",
            "tiny-hidden-key",
            "tiny-locked-path",
            "tiny-social-trade",
        ],
        "scenario_coverage_count": 5,
        "artifact_count": 1,
        "run_count": 5,
        "suite_ids": ["tiny"],
        "benchmark_ids": ["mudbench-cli"],
        "score_total": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
        "score_average_per_artifact": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
        "comparison_artifact_count": 1,
        "has_comparison_artifacts": True,
    }
    assert profile_rollups["comparison-rule-profile"] == {
        "identity_type": "external_agent_profile",
        "identity_value": "comparison-rule-profile",
        "actor_id": "comparison-rule-profile",
        "external_agent_profile_id": "comparison-rule-profile",
        "external_agent_label": "Comparison Rule Profile",
        "scenario_ids": [
            "tiny-delayed-retrieval",
            "tiny-fetch-quest",
            "tiny-hidden-key",
            "tiny-locked-path",
            "tiny-social-trade",
        ],
        "scenario_coverage_count": 5,
        "artifact_count": 1,
        "run_count": 5,
        "suite_ids": ["tiny"],
        "benchmark_ids": ["mudbench-cli"],
        "score_total": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
        "score_average_per_artifact": export_payload["history"][0]["score_summary"]["candidate_composite_score_total"],
        "comparison_artifact_count": 1,
        "has_comparison_artifacts": True,
    }
    assert export_payload["coverage"]["external_agent_profile_ids"] == ["comparison-rule-profile"]
    assert export_payload["replay_drilldowns"][0]["external_agent_profile_id"] == "comparison-rule-profile"


def test_cli_tiny_suite_external_profile_vs_profile_comparison_preserves_profile_identities(
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
    output_path = tmp_path / "tiny-suite-profile-vs-profile.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

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
            str(candidate_profile_path),
            "--output",
            "json",
            "--output-file",
            str(output_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))
    replay_payload = _read_json_output(replay_path.read_text(encoding="utf-8"))

    assert stdout_payload["accepted"] is True
    assert stdout_payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert stdout_payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert stdout_payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert stdout_payload["report"]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert stdout_payload["report"]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert stdout_payload["report"]["baseline_external_agent_label"] == "Baseline Rule Profile"
    assert stdout_payload["report"]["candidate_external_agent_label"] == "Candidate Mock Profile"
    assert manifest_payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert manifest_payload["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert manifest_payload["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert manifest_payload["baseline_external_agent_label"] == "Baseline Rule Profile"
    assert manifest_payload["candidate_external_agent_label"] == "Candidate Mock Profile"
    assert replay_payload["run_count"] == 10

    history_exit = main(["reports", "history", "--dir", str(tmp_path), "--output", "json"])
    history_payload = _read_json_output(capsys.readouterr().out)
    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert history_exit == 0
    assert export_exit == 0
    assert history_payload["history"][0]["identity_summary"] == [
        {
            "actor_id": "baseline-rule-profile",
            "identity_type": "external_agent_profile",
            "external_agent_profile_id": "baseline-rule-profile",
            "external_agent_label": "Baseline Rule Profile",
        },
        {
            "actor_id": "candidate-mock-profile",
            "identity_type": "external_agent_profile",
            "external_agent_profile_id": "candidate-mock-profile",
            "external_agent_label": "Candidate Mock Profile",
        },
    ]
    assert export_payload["coverage"]["external_agent_profile_ids"] == [
        "baseline-rule-profile",
        "candidate-mock-profile",
    ]
    assert export_payload["coverage"]["external_agent_labels"] == [
        "Baseline Rule Profile",
        "Candidate Mock Profile",
    ]
    assert any(
        entry["actor_id"] == "baseline-rule-profile"
        and entry["identity_type"] == "external_agent_profile"
        and entry["external_agent_profile_id"] == "baseline-rule-profile"
        for entry in export_payload["leaderboard"]
    )
    assert any(
        entry["actor_id"] == "candidate-mock-profile"
        and entry["identity_type"] == "external_agent_profile"
        and entry["external_agent_profile_id"] == "candidate-mock-profile"
        for entry in export_payload["leaderboard"]
    )
    assert export_payload["replay_drilldowns"][0]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert export_payload["replay_drilldowns"][0]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert {
        entry["identity_value"]: entry
        for entry in export_payload["identity_rollups"]
    }["baseline-rule-profile"]["external_agent_profile_id"] == "baseline-rule-profile"
    assert {
        entry["identity_value"]: entry
        for entry in export_payload["identity_rollups"]
    }["candidate-mock-profile"]["external_agent_profile_id"] == "candidate-mock-profile"
    assert {
        entry["identity_value"]: entry
        for entry in export_payload["identity_rollups"]
    }["baseline-rule-profile"].get("has_shared_run_arena_artifacts") is None
    assert {
        entry["identity_value"]: entry
        for entry in export_payload["identity_rollups"]
    }["candidate-mock-profile"].get("has_shared_run_arena_artifacts") is None


def test_cli_tiny_suite_shared_external_profile_vs_profile_comparison_reflects_shared_run_results(
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
    output_path = tmp_path / "tiny-suite-shared-profile-vs-profile.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

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
            str(candidate_profile_path),
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
            "--output-file",
            str(output_path),
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))
    replay_payload = _read_json_output(replay_path.read_text(encoding="utf-8"))

    assert stdout_payload["accepted"] is True
    assert stdout_payload["actor_ids"] == ["baseline-rule-profile", "candidate-mock-profile"]
    assert stdout_payload["report"]["baseline_agent_id"] == "baseline-rule-profile"
    assert stdout_payload["report"]["candidate_agent_id"] == "candidate-mock-profile"
    assert manifest_payload["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert manifest_payload["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    assert replay_payload["run_count"] == 5
    assert replay_payload["runs"][0]["source"] == "shared"

    comparisons = stdout_payload["report"]["comparisons"]
    assert len(comparisons) == 5
    for entry in comparisons:
        assert entry["baseline"]["agent_id"] == "baseline-rule-profile"
        assert entry["candidate"]["agent_id"] == "candidate-mock-profile"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]

    history_exit = main(["reports", "history", "--dir", str(tmp_path), "--output", "json"])
    history_payload = _read_json_output(capsys.readouterr().out)
    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert history_exit == 0
    assert export_exit == 0
    assert history_payload["history"][0]["identity_summary"] == [
        {
            "actor_id": "baseline-rule-profile",
            "identity_type": "external_agent_profile",
            "external_agent_profile_id": "baseline-rule-profile",
            "external_agent_label": "Baseline Rule Profile",
        },
        {
            "actor_id": "candidate-mock-profile",
            "identity_type": "external_agent_profile",
            "external_agent_profile_id": "candidate-mock-profile",
            "external_agent_label": "Candidate Mock Profile",
        },
    ]
    assert export_payload["coverage"]["external_agent_profile_ids"] == [
        "baseline-rule-profile",
        "candidate-mock-profile",
    ]
    assert export_payload["replay_drilldowns"][0]["baseline_external_agent_profile_id"] == "baseline-rule-profile"
    assert export_payload["replay_drilldowns"][0]["candidate_external_agent_profile_id"] == "candidate-mock-profile"
    shared_rollups = {entry["identity_value"]: entry for entry in export_payload["identity_rollups"]}
    assert shared_rollups["baseline-rule-profile"]["has_shared_run_arena_artifacts"] is True
    assert shared_rollups["candidate-mock-profile"]["has_shared_run_arena_artifacts"] is True
    assert shared_rollups["baseline-rule-profile"]["shared_run_arena_artifact_count"] == 1
    assert shared_rollups["candidate-mock-profile"]["shared_run_arena_artifact_count"] == 1


def test_cli_tiny_suite_mixed_builtin_vs_external_comparison_reflects_shared_run_results(
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
            f"{sys.executable} examples/agents/deterministic_rule_agent.py",
            "--agent-label",
            "deterministic-wrapper",
            "--external-agent-actor",
            "agent-b",
            "--output",
            "json",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)

    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_comparison_report_v1"
    assert report["baseline_agent_id"] == "agent-a"
    assert report["candidate_agent_id"] == "deterministic-wrapper"
    assert report["external_agent_label"] == "deterministic-wrapper"
    assert report["scenario_count"] == 5
    assert len(report["comparisons"]) == 5

    scenario_ids = [entry["scenario_id"] for entry in report["comparisons"]]
    assert scenario_ids == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    difference_total = 0.0
    for entry in report["comparisons"]:
        assert entry["baseline"]["agent_id"] == "agent-a"
        assert entry["candidate"]["agent_id"] == "deterministic-wrapper"
        assert entry["baseline"]["replay_ref"] == entry["candidate"]["replay_ref"]
        assert entry["baseline"]["parity_ref"] == entry["candidate"]["parity_ref"]
        expected_difference = (
            float(entry["candidate"]["composite_score"]) - float(entry["baseline"]["composite_score"])
        )
        assert entry["composite_score_difference"] == expected_difference
        difference_total += expected_difference

    assert report["summary"]["composite_score_difference_total"] == pytest.approx(difference_total)


def test_cli_tiny_suite_mixed_external_comparison_writes_labeled_manifest_context(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    baseline_output_path = tmp_path / "tiny-suite-baseline.json"
    output_path = tmp_path / "tiny-suite-mixed-comparison.json"
    manifest_path = _manifest_path_for(output_path)
    baseline_manifest_path = _manifest_path_for(baseline_output_path)
    replay_path = _replay_path_for(output_path)
    baseline_replay_path = _replay_path_for(baseline_output_path)

    baseline_exit = main(
        [
            "suite",
            "--suite",
            "tiny",
            "--output",
            "json",
            "--output-file",
            str(baseline_output_path),
        ]
    )
    capsys.readouterr()

    exit_code = main(
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
            str(output_path),
        ]
    )
    captured = capsys.readouterr()

    assert baseline_exit == 0
    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    file_payload = _read_json_output(output_path.read_text(encoding="utf-8"))
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))
    replay_payload = _read_json_output(replay_path.read_text(encoding="utf-8"))

    assert stdout_payload == file_payload
    assert file_payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert file_payload["report"]["candidate_agent_id"] == "deterministic-wrapper"
    assert file_payload["report"]["external_agent_label"] == "deterministic-wrapper"
    assert manifest_payload["command_mode"] == "suite_comparison"
    assert manifest_payload["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert manifest_payload["external_agent_label"] == "deterministic-wrapper"
    assert replay_payload["run_count"] == 5
    assert replay_payload["runs"][0]["source"] == "shared"
    assert replay_payload["runs"][0]["final_state_summary"]["step_index"] >= 0

    history_exit = main(["reports", "history", "--dir", str(tmp_path), "--output", "json"])
    history_payload = _read_json_output(capsys.readouterr().out)

    assert history_exit == 0
    assert history_payload["accepted"] is True
    assert history_payload["command"] == "reports_history"
    assert history_payload["artifact_count"] == 2
    assert [entry["manifest_path"] for entry in history_payload["history"]] == [
        str(baseline_manifest_path),
        str(manifest_path),
    ]
    assert history_payload["history"][0]["command_mode"] == "suite_baseline"
    assert history_payload["history"][0]["actor_ids"] == ["agent-a", "agent-b"]
    assert history_payload["history"][0]["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {"actor_id": "agent-b", "identity_type": "built_in_actor"},
    ]
    assert "external_agent_label" not in history_payload["history"][0]
    assert history_payload["history"][1]["command_mode"] == "suite_comparison"
    assert history_payload["history"][1]["actor_ids"] == ["agent-a", "deterministic-wrapper"]
    assert history_payload["history"][1]["external_agent_label"] == "deterministic-wrapper"
    assert history_payload["history"][1]["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {
            "actor_id": "deterministic-wrapper",
            "identity_type": "external_agent_label",
            "external_agent_label": "deterministic-wrapper",
        },
    ]
    assert any(
        entry["actor_id"] == "deterministic-wrapper" and entry["identity_type"] == "external_agent_label"
        for entry in history_payload["leaderboard"]
    )

    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert export_exit == 0
    assert export_payload["accepted"] is True
    assert export_payload["command"] == "reports_export"
    assert export_payload["viewmodel_version"] == "reports_export_viewmodel_v1"
    assert export_payload["artifact_count"] == 2
    assert export_payload["coverage"]["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]
    assert export_payload["coverage"]["actor_ids"] == ["agent-a", "agent-b", "deterministic-wrapper"]
    assert export_payload["coverage"]["external_agent_labels"] == ["deterministic-wrapper"]
    assert export_payload["coverage"]["external_agent_profile_ids"] == []
    assert export_payload["artifacts"] == [
        {
            "report_path": str(baseline_output_path),
            "manifest_path": str(baseline_manifest_path),
            "artifact_type": "suite_report_manifest_v1",
            "command_mode": "suite_baseline",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
        },
        {
            "report_path": str(output_path),
            "manifest_path": str(manifest_path),
            "artifact_type": "suite_report_manifest_v1",
            "command_mode": "suite_comparison",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
        },
    ]
    assert export_payload["history"] == history_payload["history"]
    assert export_payload["leaderboard"] == history_payload["leaderboard"]
    label_rollups = {entry["identity_value"]: entry for entry in export_payload["identity_rollups"]}
    assert label_rollups["deterministic-wrapper"]["identity_type"] == "external_agent_label"
    assert label_rollups["deterministic-wrapper"]["external_agent_label"] == "deterministic-wrapper"
    assert label_rollups["deterministic-wrapper"]["has_shared_run_arena_artifacts"] is True
    assert label_rollups["deterministic-wrapper"]["shared_run_arena_artifact_count"] == 1
    assert label_rollups["agent-a"]["comparison_artifact_count"] == 1
    assert export_payload["replay_drilldowns"] == [
        {
            "manifest_path": str(baseline_manifest_path),
            "report_path": str(baseline_output_path),
            "replay_drilldown_path": str(baseline_replay_path),
            "command_mode": "suite_baseline",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
            "scenario_ids": [
                "tiny-delayed-retrieval",
                "tiny-fetch-quest",
                "tiny-hidden-key",
                "tiny-locked-path",
                "tiny-social-trade",
            ],
            "actor_ids": ["agent-a", "agent-b"],
            "replay_run_count": 5,
            "runs": _read_json_output(baseline_replay_path.read_text(encoding="utf-8"))["runs"],
        },
        {
            "manifest_path": str(manifest_path),
            "report_path": str(output_path),
            "replay_drilldown_path": str(replay_path),
            "command_mode": "suite_comparison",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
            "scenario_ids": [
                "tiny-delayed-retrieval",
                "tiny-fetch-quest",
                "tiny-hidden-key",
                "tiny-locked-path",
                "tiny-social-trade",
            ],
            "actor_ids": ["agent-a", "deterministic-wrapper"],
            "external_agent_label": "deterministic-wrapper",
            "replay_run_count": 5,
            "runs": replay_payload["runs"],
        },
    ]
