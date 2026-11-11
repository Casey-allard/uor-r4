#!/usr/bin/env bash
set -euo pipefail

base_artifact_sha=$1
development_report=$2
development_sha=$3
fresh_report=$4
fresh_sha=$5

test "$(sha256sum "$development_report" | awk '{print $1}')" = "$development_sha"
jq -e -f "$(dirname "$0")/scenario-preflight-gate.jq" \
  --arg split development \
  --arg base_artifact_sha256 "$base_artifact_sha" \
  --argjson corpus_context 256 \
  --argjson copy_hits 10 \
  "$development_report" >/dev/null

test "$(sha256sum "$fresh_report" | awk '{print $1}')" = "$fresh_sha"
jq -e -f "$(dirname "$0")/scenario-preflight-gate.jq" \
  --arg split fresh \
  --arg base_artifact_sha256 "$base_artifact_sha" \
  --argjson corpus_context 256 \
  --argjson copy_hits 11 \
  "$fresh_report" >/dev/null
