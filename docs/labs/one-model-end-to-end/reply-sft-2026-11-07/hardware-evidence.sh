#!/usr/bin/env bash
# CPU-identity, grade-binding, retry-compaction and seal helpers for the job.
set -euo pipefail

validate_cpu_identity() {
  local model=$1 arch=$2 logical=$3
  case "$model" in
    ''|unknown|UNKNOWN) die "could not identify the CPU hardware for D84 evaluation" ;;
  esac
  case "$arch" in
    ''|unknown|UNKNOWN) die "could not identify the CPU architecture" ;;
  esac
  case "$logical" in
    ''|0|*[!0-9]*) die "could not identify the CPU's logical processor count" ;;
  esac
}

capture_cpu_hardware() {
  local run=$1
  cpu_model=$(sed -n 's/^model name[[:space:]]*:[[:space:]]*//p' /proc/cpuinfo | head -n 1)
  cpu_arch=$(uname -m)
  cpu_logical=$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)
  validate_cpu_identity "$cpu_model" "$cpu_arch" "$cpu_logical"
  jq -n --arg model "$cpu_model" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \
    '{schema:"uor-r4.cpu-hardware/1",model:$model,architecture:$arch,logical_cpus:$logical}' \
    > "$run/cpu.json"
  {
    printf 'model: %s\narchitecture: %s\nlogical_cpus: %s\n' \
      "$cpu_model" "$cpu_arch" "$cpu_logical"
    uname -a
    if command -v lscpu >/dev/null; then
      lscpu
    else
      sed -n '/^processor[[:space:]]*:/p;/^model name[[:space:]]*:/p' /proc/cpuinfo
    fi
  } > "$run/cpu.txt"
}

verify_grade_labels() {
  local grade=$1 label=$2
  jq -e '.substrate == "cpu-development-research" and
    .qualification == "gpu-required"' "$grade" >/dev/null ||
    die "$label grade is not labelled D84 CPU development evidence"
}

bind_grade_hardware() {
  local grade=$1 relative grade_sha
  relative=${grade#"$RUN/"}
  grade_sha=$(sha256sum "$grade" | cut -d' ' -f1)
  jq -n \
    --arg path "$relative" --arg grade_sha "$grade_sha" \
    --arg model "$cpu_model" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \
    '{schema:"uor-r4.cpu-grade-hardware/1",
      grade:{path:$path,sha256:$grade_sha},
      hardware:{model:$model,architecture:$arch,logical_cpus:$logical}}' \
    > "$grade.hardware.json"
  jq -e \
    --arg path "$relative" --arg grade_sha "$grade_sha" \
    --arg model "$cpu_model" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \
    '.grade.path == $path and .grade.sha256 == $grade_sha and
     .hardware.model == $model and .hardware.architecture == $arch and
     .hardware.logical_cpus == $logical' "$grade.hardware.json" >/dev/null ||
    die "grade $relative is not bound to the recorded CPU hardware"
}

verify_grade_hardware_binding() {
  local run=$1 grade=$2 sidecar relative actual
  sidecar="$grade.hardware.json"
  [ -f "$sidecar" ] || die "grade $grade has no CPU hardware binding"
  relative=${grade#"$run/"}
  actual=$(sha256sum "$grade" | cut -d' ' -f1)
  jq -e \
    --arg path "$relative" --arg grade_sha "$actual" \
    --arg model "$cpu_model" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \
    '.grade.path == $path and .grade.sha256 == $grade_sha and
     .hardware.model == $model and .hardware.architecture == $arch and
     .hardware.logical_cpus == $logical' "$sidecar" >/dev/null ||
    die "grade $relative no longer matches its CPU hardware binding"
}

verify_grade_hardware_bindings() {
  local run=$1 sidecar grade
  if [ -f "$run/control/base-grade.json" ]; then
    verify_grade_hardware_binding "$run" "$run/control/base-grade.json"
  fi
  while IFS= read -r -d '' grade; do
    verify_grade_hardware_binding "$run" "$grade"
  done < <(find "$run/grades" -type f -name '*.json' ! -name '*.hardware.json' -print0 2>/dev/null)
  while IFS= read -r -d '' sidecar; do
    grade=${sidecar%.hardware.json}
    [ -f "$grade" ] || die "hardware sidecar has no grade: $sidecar"
    verify_grade_hardware_binding "$run" "$grade"
  done < <(find "$run/control" "$run/grades" -type f -name '*.hardware.json' -print0 2>/dev/null)
}

compact_failed_attempt() {
  local label=$1 run_dir=$2 manifest path files bytes
  local -a paths
  manifest="$RUN/control/$label-partial-SHA256SUMS"
  paths=("${run_dir#"$RUN/"}/models" "artifacts/$label" "grades/$label" \
    "panel/$label" "scores/$label")
  (
    cd "$RUN"
    for path in "${paths[@]}"; do
      [ ! -d "$path" ] || find "$path" -type f -print0
    done | sort -z | xargs -0 -r sha256sum > "$manifest"
  ) || die "could not hash compacted $label evidence"
  files=$(wc -l < "$manifest")
  bytes=$(
    cd "$RUN"
    for path in "${paths[@]}"; do
      [ ! -d "$path" ] || find "$path" -type f -printf '%s\n'
    done | awk '{ total += $1 } END { print total + 0 }'
  )
    jq -e \
    -n --arg label "$label" --arg reason "oom-retry" \
    --arg manifest "$(basename "$manifest")" --argjson files "$files" \
    --argjson bytes "$bytes" \
    '{schema:"uor-r4.compacted-attempt/1",label:$label,reason:$reason,
      file_manifest:$manifest,files:$files,bytes:$bytes}' \
    > "$RUN/control/$label-partial.json"
  for path in "${paths[@]}"; do
    [ ! -e "$RUN/$path" ] || find "$RUN/$path" -depth -delete
  done
  log "compacted $label bulky evidence: $files files, $bytes bytes retained as hashes"
}

compact_successful_inputs() {
  local run=$1 tokens mask output staging tokens_sha mask_sha tokens_bytes mask_bytes
  tokens="$run/inputs/mix/tokens.u16"
  mask="$run/inputs/mix/response_mask.u8"
  output="$run/control/post-use-inputs.json"
  staging="$output.tmp"
  [ -f "$tokens" ] && [ -f "$mask" ] || return 1
  [ ! -e "$output" ] || return 1
  tokens_sha=$(sha256sum "$tokens" | cut -d' ' -f1) || return 1
  mask_sha=$(sha256sum "$mask" | cut -d' ' -f1) || return 1
  [ "$tokens_sha" = "$TRAIN_TOKENS_SHA" ] &&
    [ "$mask_sha" = "$TRAIN_MASK_SHA" ] || return 1
  tokens_bytes=$(stat -c '%s' "$tokens") || return 1
  mask_bytes=$(stat -c '%s' "$mask") || return 1
  jq -n --arg recipe "$RECIPE" --arg tokens_sha "$tokens_sha" \
    --arg mask_sha "$mask_sha" --arg manifest_sha "$TRAIN_MANIFEST_SHA" \
    --argjson tokens_bytes "$tokens_bytes" --argjson mask_bytes "$mask_bytes" \
    '{schema:"uor-r4.reply-sft-post-use-inputs/1",recipe:$recipe,
      reason:"successful-run seal compaction",
      retained_manifest_sha256:$manifest_sha,
      files:{tokens:{sha256:$tokens_sha,bytes:$tokens_bytes},
        response_mask:{sha256:$mask_sha,bytes:$mask_bytes}}}' \
    > "$staging" || return 1
  mv "$staging" "$output" || return 1
  [ -f "$output" ] || return 1
  rm -f "$tokens" "$mask" || return 1
  [ ! -e "$tokens" ] && [ ! -e "$mask" ]
}

measure_seal_bytes() {
  find "$1" -type f ! -name 'SHA256SUMS*' -printf '%s\n' |
    awk '{ total += $1 } END { print total + 0 }'
}

measure_all_bytes() {
  find "$1" -type f -printf '%s\n' |
    awk '{ total += $1 } END { print total + 0 }'
}

write_seal_manifest() {
  local run=$1 required
  cd "$run"
  find . -type f ! -name 'SHA256SUMS*' -print0 |
    sort -z | xargs -0 -r sha256sum > SHA256SUMS.next
  mv SHA256SUMS.next SHA256SUMS
  for required in ./cpu.json ./cpu.txt; do
    [ -f "$required" ] || die "seal is missing required CPU evidence: $required"
    grep -F "  $required" SHA256SUMS >/dev/null ||
      die "seal manifest omits required CPU evidence: $required"
  done
  sha256sum -c SHA256SUMS
}
