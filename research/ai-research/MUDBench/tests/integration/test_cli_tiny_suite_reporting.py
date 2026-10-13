from __future__ import annotations

import json
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


def test_cli_tiny_suite_reporting_emits_expected_report_shape(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(["suite", "--suite", "tiny", "--output", "json"])
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)

    assert payload["accepted"] is True
    assert payload["suite_id"] == "tiny"
    assert payload["benchmark_id"] == "mudbench-cli"
    assert payload["actor_ids"] == ["agent-a", "agent-b"]

    report = payload["report"]
    assert report["schema_version"] == "tiny_suite_baseline_report_v1"
    assert report["benchmark_ids"] == ["mudbench-cli"]
    assert report["scenario_count"] == 5
    assert report["entry_count"] == 10

    scenario_ids = sorted({entry["scenario_id"] for entry in report["entries"]})
    assert scenario_ids == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    for entry in report["entries"]:
        assert entry["agent_id"] in {"agent-a", "agent-b"}
        assert isinstance(entry["aggregate_score"], float)
        assert isinstance(entry["composite_score"], float)
        assert "quest_completion" in entry["normalized_metrics"]
        assert "quest_completion" in entry["contributions"]
        assert entry["replay_ref"].startswith("sha256:")
        assert set(entry["parity_ref"].keys()) == {
            "applied_steps_hash",
            "score_summary_hash",
            "terminal_state_hash",
        }


def test_cli_tiny_suite_reporting_writes_output_file_matching_stdout(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    output_path = tmp_path / "tiny-suite-report.json"
    manifest_path = _manifest_path_for(output_path)
    replay_path = _replay_path_for(output_path)

    exit_code = main(["suite", "--suite", "tiny", "--output", "json", "--output-file", str(output_path)])
    captured = capsys.readouterr()

    assert exit_code == 0
    stdout_payload = _read_json_output(captured.out)
    file_payload = _read_json_output(output_path.read_text(encoding="utf-8"))
    manifest_payload = _read_json_output(manifest_path.read_text(encoding="utf-8"))
    replay_payload = _read_json_output(replay_path.read_text(encoding="utf-8"))

    assert stdout_payload == file_payload
    assert manifest_payload == {
        "artifact_type": "suite_report_manifest_v1",
        "command_mode": "suite_baseline",
        "artifact_path": str(output_path),
        "benchmark_id": "mudbench-cli",
        "suite_id": "tiny",
        "scenario_ids": [
            "tiny-delayed-retrieval",
            "tiny-fetch-quest",
            "tiny-hidden-key",
            "tiny-locked-path",
            "tiny-social-trade",
        ],
        "actor_ids": ["agent-a", "agent-b"],
        "has_replay_refs": True,
        "has_parity_refs": True,
        "report_schema_version": "tiny_suite_baseline_report_v1",
    }
    assert replay_payload["schema_version"] == "suite_replay_drilldown_v1"
    assert replay_payload["artifact_type"] == "suite_replay_drilldown_v1"
    assert replay_payload["artifact_path"] == str(replay_path)
    assert replay_payload["report_path"] == str(output_path)
    assert replay_payload["suite_id"] == "tiny"
    assert replay_payload["benchmark_id"] == "mudbench-cli"
    assert replay_payload["run_count"] == 5
    assert len(replay_payload["runs"]) == 5
    first_run = replay_payload["runs"][0]
    assert first_run["scenario_id"] == "tiny-delayed-retrieval"
    assert first_run["source"] == "baseline"
    assert first_run["replay_ref"].startswith("sha256:")
    assert "state_snapshot" in first_run["event_types"]
    assert first_run["final_state_summary"]["step_index"] >= 0
    assert len(first_run["events"]) == first_run["event_count"]

    list_exit = main(["reports", "list", "--dir", str(tmp_path), "--output", "json"])
    list_payload = _read_json_output(capsys.readouterr().out)
    show_exit = main(["reports", "show", "--manifest", str(manifest_path), "--output", "json"])
    show_payload = _read_json_output(capsys.readouterr().out)

    assert list_exit == 0
    assert list_payload == {
        "accepted": True,
        "command": "reports_list",
        "directory": str(tmp_path),
        "artifact_count": 1,
        "artifacts": [
            {
                "report_path": str(output_path),
                "manifest_path": str(manifest_path),
                "artifact_type": "suite_report_manifest_v1",
                "command_mode": "suite_baseline",
                "suite_id": "tiny",
                "benchmark_id": "mudbench-cli",
            }
        ],
    }

    assert show_exit == 0
    assert show_payload == {
        "accepted": True,
        "command": "reports_show",
        "artifact": list_payload["artifacts"][0],
        "manifest": manifest_payload,
        "report": file_payload,
    }

    history_exit = main(["reports", "history", "--dir", str(tmp_path), "--output", "json"])
    history_payload = _read_json_output(capsys.readouterr().out)

    assert history_exit == 0
    assert history_payload["accepted"] is True
    assert history_payload["command"] == "reports_history"
    assert history_payload["directory"] == str(tmp_path)
    assert history_payload["artifact_count"] == 1
    assert len(history_payload["history"]) == 1
    history_entry = history_payload["history"][0]
    assert history_entry["report_path"] == str(output_path)
    assert history_entry["manifest_path"] == str(manifest_path)
    assert history_entry["artifact_type"] == "suite_report_manifest_v1"
    assert history_entry["command_mode"] == "suite_baseline"
    assert history_entry["suite_id"] == "tiny"
    assert history_entry["benchmark_id"] == "mudbench-cli"
    assert history_entry["scenario_ids"] == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]
    assert history_entry["actor_ids"] == ["agent-a", "agent-b"]
    assert history_entry["identity_summary"] == [
        {"actor_id": "agent-a", "identity_type": "built_in_actor"},
        {"actor_id": "agent-b", "identity_type": "built_in_actor"},
    ]
    assert history_entry["score_summary"]["report_schema_version"] == "tiny_suite_baseline_report_v1"
    assert history_entry["score_summary"]["entry_count"] == 10
    assert history_entry["score_summary"]["aggregate_score_max"] >= history_entry["score_summary"]["aggregate_score_min"]
    assert history_entry["score_summary"]["composite_score_max"] >= history_entry["score_summary"]["composite_score_min"]
    leaderboard = history_payload["leaderboard"]
    assert [entry["actor_id"] for entry in leaderboard] == ["agent-a", "agent-b"]
    for entry in leaderboard:
        assert entry["artifact_count"] == 1
        assert entry["identity_type"] == "built_in_actor"
        assert entry["suite_ids"] == ["tiny"]
        assert entry["benchmark_ids"] == ["mudbench-cli"]
        assert isinstance(entry["score_total"], float)
    assert history_payload["identity_rollups"] == [
        {
            "identity_type": "built_in_actor",
            "identity_value": "agent-a",
            "actor_id": "agent-a",
            "scenario_ids": history_entry["scenario_ids"],
            "scenario_coverage_count": 5,
            "artifact_count": 1,
            "run_count": 5,
            "suite_ids": ["tiny"],
            "benchmark_ids": ["mudbench-cli"],
            "score_total": leaderboard[0]["score_total"],
            "score_average_per_artifact": leaderboard[0]["score_total"],
            "comparison_artifact_count": 0,
            "has_comparison_artifacts": False,
        },
        {
            "identity_type": "built_in_actor",
            "identity_value": "agent-b",
            "actor_id": "agent-b",
            "scenario_ids": history_entry["scenario_ids"],
            "scenario_coverage_count": 5,
            "artifact_count": 1,
            "run_count": 5,
            "suite_ids": ["tiny"],
            "benchmark_ids": ["mudbench-cli"],
            "score_total": leaderboard[1]["score_total"],
            "score_average_per_artifact": leaderboard[1]["score_total"],
            "comparison_artifact_count": 0,
            "has_comparison_artifacts": False,
        },
    ]

    export_exit = main(["reports", "export", "--dir", str(tmp_path), "--output", "json"])
    export_payload = _read_json_output(capsys.readouterr().out)

    assert export_exit == 0
    assert export_payload["accepted"] is True
    assert export_payload["command"] == "reports_export"
    assert export_payload["viewmodel_version"] == "reports_export_viewmodel_v1"
    assert export_payload["artifact_count"] == 1
    assert export_payload["coverage"]["scenario_ids"] == history_entry["scenario_ids"]
    assert export_payload["coverage"]["actor_ids"] == ["agent-a", "agent-b"]
    assert export_payload["coverage"]["external_agent_labels"] == []
    assert export_payload["coverage"]["external_agent_profile_ids"] == []
    assert export_payload["artifacts"] == [
        {
            "report_path": str(output_path),
            "manifest_path": str(manifest_path),
            "artifact_type": "suite_report_manifest_v1",
            "command_mode": "suite_baseline",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
        }
    ]
    assert export_payload["history"] == history_payload["history"]
    assert export_payload["leaderboard"] == history_payload["leaderboard"]
    assert export_payload["identity_rollups"] == history_payload["identity_rollups"]
    assert export_payload["replay_drilldowns"] == [
        {
            "manifest_path": str(manifest_path),
            "report_path": str(output_path),
            "replay_drilldown_path": str(replay_path),
            "command_mode": "suite_baseline",
            "suite_id": "tiny",
            "benchmark_id": "mudbench-cli",
            "scenario_ids": history_entry["scenario_ids"],
            "actor_ids": ["agent-a", "agent-b"],
            "replay_run_count": 5,
            "runs": replay_payload["runs"],
        }
    ]
