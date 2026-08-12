from __future__ import annotations

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE_PATH = REPO_ROOT / "Makefile"
VERSION_GATE_SCRIPT_PATH = REPO_ROOT / "scripts" / "version_gate_local.sh"


def test_ci_local_pipeline_includes_version_gate_stage() -> None:
    makefile = MAKEFILE_PATH.read_text(encoding="utf-8")
    assert "ci-local: lint-local version-gate-local test-local determinism-local" in makefile


def test_version_gate_script_is_fail_fast_and_machine_readable() -> None:
    script = VERSION_GATE_SCRIPT_PATH.read_text(encoding="utf-8")
    assert "set -euo pipefail" in script
    assert '[version-gate-local] running version/protocol gate suite' in script
    assert "python -m pytest --maxfail=1 -q" in script


def test_version_gate_script_covers_required_provenance_and_protocol_checks() -> None:
    script = VERSION_GATE_SCRIPT_PATH.read_text(encoding="utf-8")
    required_node_ids = (
        "test_parse_replay_log_envelope_rejects_missing_version_provenance",
        "test_parse_run_metadata_index_rejects_partial_records_explicitly",
        "test_scorecard_metadata_rejects_mismatched_scoring_version_aliases",
        "test_observation_rejects_unsupported_protocol_version_with_machine_readable_reason",
        "test_http_runner_client_rejects_incompatible_observation_protocol_version",
        "test_http_runner_client_rejects_unsupported_action_envelope_protocol_version",
        "test_local_process_runner_rejects_incompatible_observation_protocol_version",
        "test_local_process_runner_rejects_unsupported_action_envelope_protocol_version",
    )
    for node_id in required_node_ids:
        assert node_id in script
