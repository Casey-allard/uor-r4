#!/usr/bin/env bash
# Receipt transfer/validation only. Never invokes a model binary or GPU job.
set -euo pipefail
if [[ $# != 3 ]]; then
  echo 'usage: score-quarter.sh COMPLETED_CURSOR ARCHIVED_PILOT_REPORT NEW_LOCAL_DIRECTORY' >&2
  exit 2
fi
case "$1" in 4101|8202|12303|16404) ;; *) echo 'Not a registered completed cursor' >&2; exit 2;; esac
cursor=$1
pilot=$2
out=$3
root=$(git rev-parse --show-toplevel)
[[ "$root" != /Users/casey.allard/uor-r4 ]] || { echo 'Use your owned worktree (R10)' >&2; exit 2; }
[[ -f "$pilot" ]] || { echo 'Missing archived pilot report' >&2; exit 2; }
[[ -n "${UOR_LAB:-}" ]] || { echo 'Set UOR_LAB' >&2; exit 2; }
provider=${UOR_POD_VAST_BIN:-$root/local/pod-provider-target/debug/uor-r4-pod-provider}
[[ -x "$provider" ]] || { echo 'Build the Rust pod provider or set UOR_POD_VAST_BIN to an existing Rust adapter' >&2; exit 2; }
scorer=${SCORING_AUDIT_BIN:-$root/local/scoring-target/release/completion-scoring-audit}
[[ -x "$scorer" ]] || { echo 'Build the audited scorer first' >&2; exit 2; }
[[ $(shasum -a 256 "$pilot" | cut -d ' ' -f1) == e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3 ]] || { echo 'Pilot baseline hash mismatch' >&2; exit 1; }
# Exclusive output; failures leave evidence intact. Retry in a fresh directory.
mkdir "$out"
out=$(cd "$out" && pwd)
printf -v suffix '%05d' "$cursor"
remote=/workspace/uor-r4/rung0-completion-20261014
evaluation=seed1-run-state/evaluation-$suffix.json
pin=seed1-run-state/evaluation-$suffix.pin.json
checkpoint=seed1-attempt-1/checkpoint-$suffix.json
# These paths describe Claude's current attempt. A resumed attempt with a new
# checkpoint FilePin path requires an explicit runbook update, never a guessed path.
"$root/scripts/pod/uor-pod" ssh vast-55939635 \
  "cd '$remote' && tar -cf - '$pin' '$evaluation' '$checkpoint'" > "$out/sealed-quarter.tar"
tar -xf "$out/sealed-quarter.tar" -C "$out"
"$root/scripts/pod/uor-pod" ssh vast-55939635 "cat '$remote/$pin'" > "$out/pin-after-pull.json"
cmp "$out/$pin" "$out/pin-after-pull.json"
jq -e --arg p "$remote/$evaluation" '.path == $p' "$out/$pin" >/dev/null
actual=$(shasum -a 256 "$out/$evaluation" | cut -d ' ' -f1)
[[ "$actual" == "$(jq -er .sha256 "$out/$pin")" ]] || { echo 'Evaluation pin mismatch' >&2; exit 1; }
jq -e --arg p "$remote/$checkpoint" --argjson n "$cursor" \
  '.checkpoint.path == $p and .completed_batches == $n and .admission_sha256 == "2063239bd9c65f6ccdd1fcdec21c1e6898e076dbd6cc9d171f6de9f582e51203"' "$out/$evaluation" >/dev/null
actual=$(shasum -a 256 "$out/$checkpoint" | cut -d ' ' -f1)
[[ "$actual" == "$(jq -er .checkpoint.sha256 "$out/$evaluation")" ]] || { echo 'Checkpoint pin mismatch' >&2; exit 1; }
plan=$root/docs/labs/rung0-completion-release-2026-10-14/SEED1_PLAN.json
shasum -a 256 "$out/$pin" "$out/$evaluation" "$out/$checkpoint" "$pilot" "$plan" "$scorer" > "$out/SHA256SUMS"
SCORING_AUDIT_CHECKPOINT_PATH="$out/$checkpoint" "$scorer" completion \
  "$out/$evaluation" "$pilot" "$plan" "$out/audit.json" \
  > "$out/scorer.stdout" 2> "$out/scorer.stderr"
cat "$out/scorer.stdout"
jq '{observation, completed_batches, inputs, metrics, overall, sensitivity, snapshot_metadata, registered_comparisons, authorizes_model_compute}' "$out/audit.json"
echo 'Q1-Q3 are diagnostics only. Q4 is single-seed evidence, not a two-seed verdict or seed-2 release.'
