#!/usr/bin/env bash
# Exercise the CPU evidence helpers without a pod, model or paid compute.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
LIB=${1:-"$HERE/hardware-evidence.sh"}
TMP=$(mktemp -d /dev/shm/uor-hardware-evidence.XXXXXX)
trap 'find "$TMP" -depth -delete' EXIT

die() {
  printf '%s\n' "$*" >&2
  exit 1
}
log() {
  printf '%s\n' "$*"
}

cpu_model=
cpu_arch=
cpu_logical=
zero_pin=$(printf '0%.0s' {1..64})
# shellcheck disable=SC1091
# shellcheck source=hardware-evidence.sh
source "$LIB"

RUN="$TMP/run"
install -d "$RUN/control" "$RUN/grades/attempt-1" \
  "$RUN/artifacts/attempt-1/step-000256" "$RUN/panel/attempt-1" \
  "$RUN/scores/attempt-1" "$RUN/run/models/step-000256"

capture_cpu_hardware "$RUN"
jq -e '.schema == "uor-r4.cpu-hardware/1" and
  (.model | length) > 0 and (.architecture | length) > 0 and
  .logical_cpus > 0' "$RUN/cpu.json" >/dev/null
test -s "$RUN/cpu.txt"
validate_cpu_identity "$cpu_model" "$cpu_arch" "$cpu_logical"
if (validate_cpu_identity unknown "$cpu_arch" "$cpu_logical") >/dev/null 2>&1; then
  die "unknown CPU model escaped validation"
fi
if (validate_cpu_identity "$cpu_model" "$cpu_arch" 0) >/dev/null 2>&1; then
  die "zero logical CPUs escaped validation"
fi

grade="$RUN/grades/attempt-1/step-000256.json"
printf '%s\n' '{"substrate":"cpu-development-research","qualification":"gpu-required"}' \
  > "$grade"
verify_grade_labels "$grade" checkpoint
printf '%s\n' '{"substrate":"cpu-development-research"}' > "$grade"
if (verify_grade_labels "$grade" checkpoint) >/dev/null 2>&1; then
  die "unlabelled checkpoint grade escaped validation"
fi
printf '%s\n' '{"substrate":"cpu-development-research","qualification":"gpu-required"}' \
  > "$grade"
bind_grade_hardware "$grade"
verify_grade_hardware_bindings "$RUN"
jq -e --arg sha "$(sha256sum "$grade" | cut -d' ' -f1)" --arg model "$cpu_model" \
  '.grade.sha256 == $sha and .hardware.model == $model' \
  "$grade.hardware.json" >/dev/null
unbound="$RUN/grades/attempt-1/unbound.json"
cp "$grade" "$unbound"
if (verify_grade_hardware_bindings "$RUN") >/dev/null 2>&1; then
  die "grade without a hardware sidecar escaped validation"
fi
unlink "$unbound"

printf '%s\n' '{"tampered":true}' > "$grade"
if (verify_grade_hardware_bindings "$RUN") >/dev/null 2>&1; then
  die "tampered grade escaped its hardware binding"
fi
printf '%s\n' '{"substrate":"cpu-development-research","qualification":"gpu-required"}' \
  > "$grade"
bind_grade_hardware "$grade"
verify_grade_hardware_bindings "$RUN"

printf 'model\n' > "$RUN/run/models/step-000256/model.safetensors"
printf 'artifact\n' > "$RUN/artifacts/attempt-1/step-000256/model.uorlut"
printf 'panel\n' > "$RUN/panel/attempt-1/step-000256.json"
printf 'score\n' > "$RUN/scores/attempt-1/step-000256.scored"
compact_failed_attempt attempt-1 "$RUN/run"
jq -e '.files == 6 and .bytes > 0 and .reason == "oom-retry"' \
  "$RUN/control/attempt-1-partial.json" >/dev/null
test ! -e "$RUN/run/models"
test ! -e "$RUN/artifacts/attempt-1"
test ! -e "$RUN/grades/attempt-1"
test ! -e "$RUN/panel/attempt-1"
test ! -e "$RUN/scores/attempt-1"

install -d "$RUN/inputs/mix"
printf 'tokens\n' > "$RUN/inputs/mix/tokens.u16"
printf 'mask\n' > "$RUN/inputs/mix/response_mask.u8"
printf 'manifest\n' > "$RUN/inputs/mix/manifest.json"
TRAIN_TOKENS_SHA=$(sha256sum "$RUN/inputs/mix/tokens.u16" | cut -d' ' -f1)
TRAIN_MASK_SHA=$(sha256sum "$RUN/inputs/mix/response_mask.u8" | cut -d' ' -f1)
TRAIN_MANIFEST_SHA=$(sha256sum "$RUN/inputs/mix/manifest.json" | cut -d' ' -f1)
# shellcheck disable=SC2034 # consumed by compact_successful_inputs
RECIPE=content-ratio
compact_successful_inputs "$RUN"
jq -e --arg tokens "$TRAIN_TOKENS_SHA" --arg mask "$TRAIN_MASK_SHA" \
  --arg manifest "$TRAIN_MANIFEST_SHA" '
  .recipe == "content-ratio" and
  .reason == "successful-run seal compaction" and
  .retained_manifest_sha256 == $manifest and
  .files.tokens.sha256 == $tokens and .files.tokens.bytes == 7 and
  .files.response_mask.sha256 == $mask and .files.response_mask.bytes == 5' \
  "$RUN/control/post-use-inputs.json" >/dev/null
test ! -e "$RUN/inputs/mix/tokens.u16"
test ! -e "$RUN/inputs/mix/response_mask.u8"
test -f "$RUN/inputs/mix/manifest.json"
if (compact_successful_inputs "$RUN") >/dev/null 2>&1; then
  die "successful-input compaction ran twice"
fi

BAD_INPUTS="$TMP/bad-inputs"
install -d "$BAD_INPUTS/inputs/mix" "$BAD_INPUTS/control"
printf 'tokens\n' > "$BAD_INPUTS/inputs/mix/tokens.u16"
printf 'mask\n' > "$BAD_INPUTS/inputs/mix/response_mask.u8"
if (
  TRAIN_TOKENS_SHA=$zero_pin
  compact_successful_inputs "$BAD_INPUTS"
) >/dev/null 2>&1; then
  die "successful-input compaction accepted a mismatched token hash"
fi
test -f "$BAD_INPUTS/inputs/mix/tokens.u16"
test ! -e "$BAD_INPUTS/control/post-use-inputs.json"

printf 'log\n' > "$RUN/job.log"
MEASURE="$TMP/measure"
install -d "$MEASURE/control"
printf 'abcd' > "$MEASURE/root"
printf 'ef' > "$MEASURE/control/inner"
printf '0123456789' > "$MEASURE/SHA256SUMS"
[ "$(measure_seal_bytes "$MEASURE")" -eq 6 ]
[ "$(measure_all_bytes "$MEASURE")" -eq 16 ]
MISSING_CPU="$TMP/missing-cpu"
install -d "$MISSING_CPU"
printf 'other\n' > "$MISSING_CPU/other"
if (write_seal_manifest "$MISSING_CPU") >/dev/null 2>&1; then
  die "seal without CPU evidence escaped validation"
fi
write_seal_manifest "$RUN"
grep -F '  ./cpu.json' "$RUN/SHA256SUMS" >/dev/null
grep -F '  ./cpu.txt' "$RUN/SHA256SUMS" >/dev/null
printf 'hardware_evidence_tests=pass\n'
