#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION_GATE_TESTS=(
  "tests/unit/test_replay_log_format.py::test_parse_replay_log_envelope_rejects_missing_version_provenance"
  "tests/unit/test_metadata_index.py::test_parse_run_metadata_index_rejects_partial_records_explicitly"
  "tests/unit/test_scorecard.py::test_scorecard_metadata_rejects_mismatched_scoring_version_aliases"
  "tests/unit/test_observation_schema.py::test_observation_rejects_unsupported_protocol_version_with_machine_readable_reason"
  "tests/unit/test_http_runner_client.py::test_http_runner_client_rejects_incompatible_observation_protocol_version"
  "tests/unit/test_http_runner_client.py::test_http_runner_client_rejects_unsupported_action_envelope_protocol_version"
  "tests/unit/test_local_runner_bridge.py::test_local_process_runner_rejects_incompatible_observation_protocol_version"
  "tests/unit/test_local_runner_bridge.py::test_local_process_runner_rejects_unsupported_action_envelope_protocol_version"
)

echo "[version-gate-local] running version/protocol gate suite"
echo "[version-gate-local] test_count=${#VERSION_GATE_TESTS[@]}"
PYTHONPATH=src python -m pytest --maxfail=1 -q "${VERSION_GATE_TESTS[@]}"
