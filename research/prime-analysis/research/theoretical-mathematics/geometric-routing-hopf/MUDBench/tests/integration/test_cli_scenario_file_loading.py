from __future__ import annotations

import json

import pytest

from cli.main import main


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def test_cli_scenario_file_loading_runs_canonical_locked_path(
    capsys: pytest.CaptureFixture[str],
) -> None:
    exit_code = main(
        ["run", "--scenario-file", "scenarios/canonical/tiny_locked_path.json", "--output", "json"]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-locked-path"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["scorecard"]["metadata"]["scenario_id"] == "tiny-locked-path"
    assert payload["replay"]["schema_version"] == "1.0"
    assert [entry["name"] for entry in payload["replay"]["artifact_refs"]] == [
        "replay_artifact",
        "replay_checksum",
    ]
