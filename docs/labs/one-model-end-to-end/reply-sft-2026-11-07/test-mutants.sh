#!/usr/bin/env bash
# Prove the static gates catch changes to the registered stop and launch rules.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
JOB="$HERE/job.sh"
LAUNCHER="$HERE/run-s1.sh"
SHEET="$HERE/../../../harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md"
EXTENSION_SHEET="$HERE/../../../harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md"
HARDWARE_LIB="$HERE/hardware-evidence.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
caught=0
attempted=0

"$HERE/gate.sh" >/dev/null
zero_pin=$(printf '0%.0s' {1..40})
if UOR_SFT_PIN="$zero_pin" bash "$JOB" 2 >"$TMP/seed-two.out" 2>&1; then
  echo "unmodified job accepted seed 2" >&2
  exit 1
fi
grep -Fq 'seed 2 is conditional and is not launched by this script' "$TMP/seed-two.out" || {
  echo "unmodified job refused seed 2 for the wrong reason" >&2
  exit 1
}
actual_log=$(
  printf '%s\n' \
    'Started on vast-example gpu 0 (queues behind any job holding the GPU lock). Log: /workspace/uor-r4/jobs/deepseek/example.log' |
    awk '/Log: / {
      for (i = 1; i <= NF; i++) if ($i == "Log:") { print $(i + 1); exit }
    }'
)
[ "$actual_log" = /workspace/uor-r4/jobs/deepseek/example.log ] || {
  echo "the controller does not parse uor-pod's actual job-log line" >&2
  exit 1
}
remote_process=$(
  printf '%s\n' \
    '123 456 bash /workspace/uor-r4/jobs/deepseek/example.sh' |
    awk -v script=/workspace/uor-r4/jobs/deepseek/example.sh '
      /^[[:space:]]*[0-9]+[[:space:]]+[0-9]+[[:space:]]/ {
        for (i = 3; i <= NF; i++) if ($i == script) { print $1, $2; exit }
      }'
)
[ "$remote_process" = '123 456' ] || {
  echo "the controller does not identify the exact remote process group" >&2
  exit 1
}

mutate() {
  local name=$1 file=$2 search=$3 replacement=$4
  attempted=$((attempted + 1))
  local target="$TMP/$name"
  cp "$file" "$target"
  SEARCH=$search REPLACE=$replacement perl -0pi -e \
    's/\Q$ENV{SEARCH}\E/$ENV{REPLACE}/g' "$target"
  cmp -s "$file" "$target" && {
    echo "mutation $name did not change its file" >&2
    exit 1
  }
  if "$HERE/gate.sh" \
    "$([ "$file" = "$JOB" ] && printf '%s' "$target" || printf '%s' "$JOB")" \
    "$([ "$file" = "$LAUNCHER" ] && printf '%s' "$target" || printf '%s' "$LAUNCHER")" \
    "$([ "$file" = "$SHEET" ] && printf '%s' "$target" || printf '%s' "$SHEET")" \
    "$([ "$file" = "$EXTENSION_SHEET" ] && printf '%s' "$target" || printf '%s' "$EXTENSION_SHEET")" \
    "$([ "$file" = "$HARDWARE_LIB" ] && printf '%s' "$target" || printf '%s' "$HARDWARE_LIB")" \
    >/dev/null 2>&1; then
    echo "mutation $name escaped" >&2
    exit 1
  fi
  caught=$((caught + 1))
}

mutate_first() {
  local name=$1 file=$2 search=$3 replacement=$4
  attempted=$((attempted + 1))
  local target="$TMP/$name"
  cp "$file" "$target"
  SEARCH=$search REPLACE=$replacement perl -0pi -e \
    's/\Q$ENV{SEARCH}\E/$ENV{REPLACE}/' "$target"
  cmp -s "$file" "$target" && {
    echo "mutation $name did not change its file" >&2
    exit 1
  }
  if "$HERE/gate.sh" \
    "$([ "$file" = "$JOB" ] && printf '%s' "$target" || printf '%s' "$JOB")" \
    "$([ "$file" = "$LAUNCHER" ] && printf '%s' "$target" || printf '%s' "$LAUNCHER")" \
    "$([ "$file" = "$SHEET" ] && printf '%s' "$target" || printf '%s' "$SHEET")" \
    "$([ "$file" = "$EXTENSION_SHEET" ] && printf '%s' "$target" || printf '%s' "$EXTENSION_SHEET")" \
    "$([ "$file" = "$HARDWARE_LIB" ] && printf '%s' "$target" || printf '%s' "$HARDWARE_LIB")" \
    >/dev/null 2>&1; then
    echo "mutation $name escaped" >&2
    exit 1
  fi
  caught=$((caught + 1))
}

mutate wrong-stop-step "$JOB" '[ "$step" -eq 512 ]' '[ "$step" -eq 768 ]'
mutate disabled-stop "$JOB" '[ "$eos_p50" -ge 7 ]; then' '[ "$eos_p50" -ge 7 ] && false; then'
mutate weak-eos-p50 "$JOB" '[ "$eos_p50" -ge 7 ]' '[ "$eos_p50" -ge 0 ]'
mutate wrong-best-no-guard "$JOB" \
  '(if $guard_safe then 0 else -(.guard_top1) end),' \
  '(if $guard_safe then 0 else $rows end),'
mutate weak-best-guard "$JOB" \
  '($rows | any(.[]; .guard_top1 >= 93))' \
  '($rows | any(.[]; .guard_top1 >= 0))'
mutate unfiltered-best-guard "$JOB" \
  'select(($guard_safe | not) or .guard_top1 >= 93)' 'select(true)'
mutate weak-best-eos-top1 "$JOB" '-(.eos_top1),' '(.eos_top1),'
mutate weak-best-free-eos "$JOB" '-(.free_eos_stops),' '(.free_eos_stops),'
mutate weak-best-complete "$JOB" \
  '-(.free_complete_rows),' '(.free_complete_rows),'
mutate missing-free-complete "$JOB" \
  '.totals.complete | numbers' '.totals.complete | null'
mutate one-user-best-source "$JOB" \
  '-per-turn-generated.json") ||' '-one-user-generated.json") ||'
mutate wrong-best-rendering "$JOB" \
  'free_generation_rendering:"per-turn/v1",' \
  'free_generation_rendering:"one-user-turn/v1",'
mutate last-best-checkpoint "$JOB" \
  'selected:$ranked[0],' 'selected:$ranked[-1],'
mutate early-guard-stop "$JOB" \
  'touch "$marker" || die "could not mark checkpoint $step scored"' \
  'if [ "$guard" -lt 93 ]; then STOP_REASON="continuation-failure-step-$step"; kill "$TRAIN_PID" 2>/dev/null || true; fi
  touch "$marker" || die "could not mark checkpoint $step scored"'
mutate weak-base-guard "$JOB" '[ "$base_guard" -eq 132 ]' '[ "$base_guard" -ge 0 ]'
mutate wrong-base-rank "$JOB" '.eos_rank_min == 5' '.eos_rank_min == 0'
mutate missing-per-turn-free "$JOB" \
  "' base-per-turn-generated" "' base-one-user-generated"
mutate no-freeze "$JOB" 'kill -STOP "$TRAIN_PID"' 'kill -CONT "$TRAIN_PID"'
mutate partial-checkpoint "$JOB" \
  'if [ -d "$checkpoint" ] && [ -f "$checkpoint/.checkpoint-ready" ]; then' \
  'if [ -d "$checkpoint" ]; then'
mutate missing-checkpoint "$JOB" \
  'elif [ -z "$STOP_REASON" ]; then' 'elif false; then'
mutate no-termination-cont "$JOB" 'kill -CONT "$TRAIN_PID" 2>/dev/null || true' ''
mutate wrong-eval "$JOB" '--eval-every 256' '--eval-every 2560'
mutate wrong-dose "$JOB" '  1024|2048) ;;' '  1024) ;;'
mutate fixed-dose "$JOB" '--steps "$STEPS" \' '--steps 1024 \'
mutate wrong-seed "$JOB" '--seed 1 \' '--seed 2 \'
mutate seed-two-enabled "$JOB" '2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;' '2) ;;'
mutate zero-replies "$JOB" '--max-replies 1 --reply-tokens 1' \
  '--max-replies 0 --reply-tokens 1'
mutate_first checkpoint-zero-replies "$JOB" \
  '--max-evaluation 512 --max-replies 1 --reply-tokens 1' \
  '--max-evaluation 512 --max-replies 0 --reply-tokens 1'
mutate_first raw-export-flags "$JOB" \
  '--raw-gates --raw-transition \' '--raw-gates \'
mutate wrong-panel-target "$JOB" \
  'PANEL_BIN="$PANEL_TARGET/release/served-reply-panel"' \
  'PANEL_BIN="$REPO/docs/labs/one-model-end-to-end/instrument/target/release/served-reply-panel"'
mutate mixed-hash "$JOB" \
  3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  0000000000000000000000000000000000000000000000000000000000000000
mutate forge-hash "$JOB" \
  'got=$(sha256sum "$path" | cut -d'"'"' '"'"' -f1)' \
  'got=$want'
mutate wrong-sample "$JOB" \
  b724656f29bdade21aecb372d0568c48c81d78c80f43fe4de8ec667a7824ccc4 \
  0000000000000000000000000000000000000000000000000000000000000000
mutate missing-init-check "$JOB" '[ -f "$run/init.json" ] ||' 'true ||'
mutate shared-init-approval "$JOB" \
  'init_approval="$RUN/control/init-approved-$label"' \
  'init_approval="$RUN/control/init-approved"'
mutate missing-oom-retry "$JOB" 'TRAIN_FAILURE=$train_rc' 'TRAIN_FAILURE=0'
mutate masked-panel-exit "$JOB" \
  '--reply-tokens 64 --out "$out" || return 1' \
  '--reply-tokens 64 --out "$out"'
mutate ignored-panel-exit "$JOB" \
  'run_panel "$step" "$artifact" per-turn/v1 1 per-turn-teacher >/dev/null ||' \
  'run_panel "$step" "$artifact" per-turn/v1 1 per-turn-teacher >/dev/null'
mutate async-seal-log "$JOB" \
  'exec >> "$RUN/job.log" 2>&1' \
  'exec > >(tee -a "$RUN/job.log") 2>&1'
mutate duplicate-checkpoints "$JOB" \
  'install -d "$RUN/scores/$ATTEMPT"' \
  'install -d "$RUN/scores/$ATTEMPT"; cp -a "$checkpoint" "$RUN/scores/$ATTEMPT/copy"'
mutate missing-seal-cap "$JOB" \
  'if [ "$actual_seal_bytes" -gt "$SEAL_LIMIT_BYTES" ]; then' \
  'if false; then'
mutate weak-progress-finite "$JOB" 'all(.[];' 'any(.[];'
mutate unchecked-single-step-zero "$JOB" \
  '(map(select(.step == 0)) | length) == 1 and' \
  '(map(select(.step == 0)) | length) >= 1 and'
mutate unchecked-positive-progress "$JOB" \
  '([.[] | select(.step > 0)] | length) > 0 and' 'true and'
mutate unchecked-dev-loss "$JOB" \
  '((.dev_loss | type) == "number" and (.dev_loss | isfinite)) and' \
  '(.dev_loss != null) and'
mutate unchecked-dev-token-margin "$JOB" \
  '(.dev_token_margin | isfinite)' 'true'
mutate unchecked-dev-address-margin "$JOB" \
  '(.dev_address_margin | isfinite)' 'true'
mutate unchecked-step-zero-train-loss "$JOB" \
  'then .train_loss_mean == null' 'then true'
mutate unchecked-positive-train-loss "$JOB" \
  'else ((.train_loss_mean | type) == "number" and
         (.train_loss_mean | isfinite))' \
  'else true'
mutate missing-cpu-hardware-id "$HARDWARE_LIB" \
  'cpu_model=$(sed -n '"'"'s/^model name[[:space:]]*:[[:space:]]*//p'"'"' /proc/cpuinfo | head -n 1)' \
  'cpu_model=unknown'
mutate unchecked-grade-labels "$HARDWARE_LIB" \
  'die "$label grade is not labelled D84 CPU development evidence"' \
  'true'
mutate missing-checkpoint-hardware-binding "$JOB" \
  'bind_grade_hardware "$grade_dir.json"' 'true'
mutate missing-base-hardware-binding "$JOB" \
  'bind_grade_hardware "$RUN/control/base-grade.json"' 'true'
mutate cpu-hardware-excluded-from-seal "$HARDWARE_LIB" \
  "find . -type f ! -name 'SHA256SUMS*' -print0 |" \
  "find . -type f ! -name 'SHA256SUMS*' ! -name 'cpu.*' -print0 |"
mutate forged-grade-hardware-sha "$HARDWARE_LIB" \
  'grade_sha=$(sha256sum "$grade" | cut -d'"'"' '"'"' -f1)' \
  'grade_sha=0000000000000000000000000000000000000000000000000000000000000000'
mutate unbound-grade-hardware-model "$HARDWARE_LIB" \
  '--arg model "$cpu_model" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \' \
  '--arg model "unknown" --arg arch "$cpu_arch" --argjson logical "$cpu_logical" \'
mutate unchecked-grade-hardware-drift "$HARDWARE_LIB" \
  'die "grade $relative no longer matches its CPU hardware binding"' 'true'
mutate unknown-cpu-identity-accepted "$HARDWARE_LIB" \
  "''|unknown|UNKNOWN) die \"could not identify the CPU hardware for D84 evaluation\" ;;" \
  "''|UNKNOWN) die \"could not identify the CPU hardware for D84 evaluation\" ;;"
mutate zero-logical-cpus-accepted "$HARDWARE_LIB" \
  "''|0|*[!0-9]*) die \"could not identify the CPU's logical processor count\" ;;" \
  "''|*[!0-9]*) die \"could not identify the CPU's logical processor count\" ;;"
mutate retry-model-evidence-omitted "$HARDWARE_LIB" \
  'paths=("${run_dir#"$RUN/"}/models" "artifacts/$label" "grades/$label" \' \
  'paths=("artifacts/$label" "grades/$label" \'
mutate retry-hash-partial "$HARDWARE_LIB" \
  'for path in "${paths[@]}"; do' \
  'for path in "${paths[@]:0:2}"; do'
mutate retry-deletion-disabled "$HARDWARE_LIB" \
  '[ ! -e "$RUN/$path" ] || find "$RUN/$path" -depth -delete' 'true'
mutate seal-size-ignores-cpu "$HARDWARE_LIB" \
  "find \"\$1\" -type f ! -name 'SHA256SUMS*' -printf '%s\\n' |" \
  "find \"\$1/control\" -type f ! -name 'SHA256SUMS*' -printf '%s\\n' |"
mutate uncompacted-oom-retry "$JOB" \
  'compact_failed_attempt attempt-1 "$TRAIN_RUN"' 'true'
mutate checkpoint-progress-unchecked "$JOB" \
  'check_progress "$TRAIN_RUN/progress.jsonl"' 'true'
mutate base-sample-unchecked "$JOB" \
  'die "base continuation grade did not use the registered 512-row sample"' \
  'true'
mutate overlap-accepted "$JOB" \
  '.counts.exact_accepted_assistant_reply == 0 and' \
  '.counts.exact_accepted_assistant_reply >= 0 and'
mutate no-readback "$JOB" 'READBACK_STATUS=verified' 'READBACK_STATUS=unknown'
mutate directory-readback "$JOB" \
  '[ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS)' \
  '[ -d "$sealed" ]'
mutate two-gpus "$LAUNCHER" '--count 1 --hours 2' '--count 2 --hours 2'
mutate uppercase-session "$LAUNCHER" \
  " | tr '[:upper:]' '[:lower:]'" ''
mutate dirty-launch "$LAUNCHER" \
  '[ -z "$(git status --porcelain --untracked-files=no)" ] ||' \
  'true ||'
mutate unmerged-pin "$LAUNCHER" \
  'git merge-base --is-ancestor "$PIN" origin/main ||' 'true ||'
mutate reviewed-runtime-unchecked "$LAUNCHER" \
  '[ "$runtime_sha" = "$REVIEWED_RUNTIME" ] ||' 'true ||'
mutate weak-global-cap "$LAUNCHER" \
  'export UOR_POD_MAX_RATE=8.00' 'export UOR_POD_MAX_RATE=80.00'
mutate store-preflight-fail-open "$LAUNCHER" \
  'if ! listings=$("$STORE" ls "$STORE_TOPIC"); then' 'if false; then'
mutate submit-only-wait "$LAUNCHER" \
  "'exit '*) job_exit=\${probe#exit }; break ;;" '*) break ;;'
mutate dead-job-ignored "$LAUNCHER" \
  'elif ! flock -n /root/gpu0.lock true; then printf '"'"'running\\n'"'"'' \
  'elif false; then printf '"'"'running\\n'"'"''
mutate ignored-remote-exit "$LAUNCHER" '[ "$job_exit" -eq 0 ] || {' 'true || {'
mutate missing-job-log "$LAUNCHER" \
  '*) cat "$CONTROLLER/run.out" >&2; echo "uor-pod did not report a job log" >&2; exit 1 ;;' \
  '*) ;;'
mutate no-renew "$LAUNCHER" 'if ! "$POD_TOOL" renew "$POD"' 'if ! true'
mutate no-stall-stop "$LAUNCHER" \
  'if [ $((remote_now - last_progress)) -ge "$STALL_SEC" ]; then' \
  'if false; then'
mutate epoch-zero-stall "$LAUNCHER" 'last_progress=$started' 'last_progress=0'
mutate unreachable-as-stopped "$LAUNCHER" \
  '*) failures=0; unreachable=$((unreachable + 1)) ;;' \
  '*) failures=$((failures + 1)); unreachable=0 ;;'
mutate no-rental-discovery "$LAUNCHER" 'discover_owned_pod' 'true'
mutate no-active-stop "$LAUNCHER" 'if stop_remote_job; then' 'if true; then'
mutate teardown-after-failed-stop "$LAUNCHER" \
  '"$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" --to-store || rc=1' \
  'true'
mutate unknown-submission-success "$LAUNCHER" \
  '[ "$SUBMISSION_ATTEMPTED" -ne 1 ] || {' \
  'false || {'
mutate no-remote-job-discovery "$LAUNCHER" \
  "-name '*-"'$SESSION'"-gpu0.sh' -print" \
  "-name '*.sh' -print"
mutate missing-remote-dose "$LAUNCHER" \
  'UOR_SFT_OUT_STORE="$OUT_STORE" UOR_SFT_STEPS="$STEPS" \' \
  'UOR_SFT_OUT_STORE="$OUT_STORE" \'
mutate wrong-extension-report "$LAUNCHER" \
  '2048:docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md:docs/loop/tasks/one-model-end-to-end/deepseek-29/REPORT.md) ;;' \
  '2048:docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md:docs/loop/tasks/one-model-end-to-end/deepseek-25/REPORT.md) ;;'
mutate signal-cleanup-success "$LAUNCHER" \
  '[ "$STATUS" = SEALED ] || rc=1' \
  '[ "$STATUS" = SEALED ] || true'
mutate missing-provider-build "$LAUNCHER" \
  'VAST_ADAPTER="$CONTROLLER/provider-target/debug/uor-r4-pod-provider"' \
  'VAST_ADAPTER=/missing/uor-r4-pod-provider'
mutate launcher-readback-true "$LAUNCHER" \
  '[ "$(cat "$READBACK_FILE")" = verified ]' 'true'
mutate launcher-exit-true "$LAUNCHER" \
  '[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ]' 'true'
mutate_first missing-extension-bar "$EXTENSION_SHEET" '## Bar' '## Removed bar'
mutate_first missing-extension-split "$EXTENSION_SHEET" '## Data split' '## Removed split'
mutate_first missing-extension-steps "$EXTENSION_SHEET" '## Steps' '## Removed steps'
mutate_first missing-extension-stop "$EXTENSION_SHEET" '## Stop rule' '## Removed stop'

[ "$caught" -eq "$attempted" ]
echo "$caught of $attempted mutations caught"
