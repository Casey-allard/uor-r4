#!/usr/bin/env bash
# Static gates for the reply-SFT pod job and controller.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
JOB=${1:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"}
LAUNCHER=${2:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/run-s1.sh"}
TRAINER="$ROOT/crates/uor-r4-float-train/src/in_space_long.rs"
SHEET="$ROOT/docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md"
checks=0

fail() {
  echo "FAIL: $*" >&2
  exit 1
}
require_line() {
  local text=$1 file=$2
  awk -v want="$text" '
    {
      line=$0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == want) found = 1
    }
    END { exit(found ? 0 : 1) }
  ' "$file" || fail "$file lacks complete line: $text"
  checks=$((checks + 1))
}

bash -n "$JOB"
bash -n "$LAUNCHER"
checks=$((checks + 2))
if command -v shellcheck >/dev/null; then
  shellcheck "$JOB" "$LAUNCHER"
  checks=$((checks + 1))
fi

require_line '2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;' "$JOB"
require_line '*) echo "SEED must be 1; seed 2 is a later decision" >&2; exit 2 ;;' "$JOB"
require_line 'got=$(sha256sum "$path" | cut -d'"'"' '"'"' -f1)' "$JOB"
require_line '[ "$got" = "$want" ] || die "$path hashes to $got, not $want"' "$JOB"
require_line 'OUT_STORE=${UOR_SFT_OUT_STORE:-results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1}' "$JOB"
require_line 'store_listing=$("$STORE" ls "$STORE_TOPIC") ||' "$JOB"
require_line 'die "cannot verify the store topic before training: $STORE_TOPIC"' "$JOB"
require_line 'if [ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS); then' "$JOB"
require_line 'if "$STORE" put "$RUN" "$OUT_STORE"; then' "$JOB"
require_line '"$STORE" put "$RUN.readback-status" "$OUT_STORE/READBACK" ||' "$JOB"

require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/tokens.u16" \' "$JOB"
require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/response_mask.u8" \' "$JOB"
require_line 'fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/manifest.json" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/tokens.u16" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/response_mask.u8" \' "$JOB"
require_line 'fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/manifest.json" \' "$JOB"
require_line 'fetch_file "results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/d3-accum-s1.tar.gz" \' "$JOB"
require_line 'fetch_file "data/m2-fresh256/inputs.json" "$RUN/inputs/m2-fresh256/inputs.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/labels.json" "$RUN/inputs/m2-fresh256/labels.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/plan.json" "$RUN/inputs/m2-fresh256/plan.json"' "$JOB"
require_line 'fetch_file "data/m2-fresh256/manifest.json" "$RUN/inputs/m2-fresh256/manifest.json"' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/tokenizer.json" "$RUN/inputs/data/tokenizer.json"' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16" \' "$JOB"
require_line 'fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8" \' "$JOB"

for hash in \
  3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \
  1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe \
  02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e \
  9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833 \
  7474ba9331f242148dd9833d5cb085307c8e0d86314d13f285a0ab1c42b8b081 \
  f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba \
  1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a \
  3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be \
  0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715 \
  7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964 \
  d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89 \
  0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de \
  1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 \
  672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd \
  6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2
do
  require_line "check_file $hash \\" "$JOB"
done

grade_flag_lines=$(
  awk '
    {
      line=$0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (line == "--max-evaluation 512 --max-replies 1 --reply-tokens 1 \\") found++
    }
    END { print found }
  ' "$JOB"
)
[ "$grade_flag_lines" -eq 2 ] ||
  fail "$JOB does not pin both checkpoint and base-control grade flags"
checks=$((checks + 1))
raw_flag_lines=$(grep -Fc -- '--raw-gates --raw-transition' "$JOB")
[ "$raw_flag_lines" -eq 2 ] ||
  fail "$JOB does not pass raw served snapshots to both exports"
checks=$((checks + 1))
require_line 'kill -STOP "$TRAIN_PID" 2>/dev/null || true' "$JOB"
require_line 'if [ -d "$checkpoint" ] && [ -f "$checkpoint/.checkpoint-ready" ]; then' "$JOB"
require_line '[ -f "$checkpoint/.checkpoint-ready" ] ||' "$JOB"
require_line 'elif [ -z "$STOP_REASON" ]; then' "$JOB"
grep -Fq 'mark_checkpoint_ready(&directory)?;' "$TRAINER" ||
  fail "$TRAINER does not mark a checkpoint complete after saving it"
checks=$((checks + 1))
termination_count=$(
  awk '
  {
    line=$0
    sub(/^[ \t]+/, "", line)
    sub(/[ \t]+$/, "", line)
    if (previous == "kill -TERM \"$TRAIN_PID\" 2>/dev/null || true" &&
        line == "kill -CONT \"$TRAIN_PID\" 2>/dev/null || true") found++
    previous = line
  }
  END { print found }
' "$JOB"
)
[ "$termination_count" -eq 2 ] ||
  fail "$JOB does not continue both stopped-trainer SIGTERM paths"
checks=$((checks + 1))
if grep -Fq 'STOP_REASON="continuation-failure-step-$step"' "$JOB"; then
  fail "$JOB stops on the continuation guard instead of applying row 1 at the best checkpoint"
fi
checks=$((checks + 1))
require_line 'if [ "$step" -eq 512 ] && [ "$eos_top1" -eq 0 ] && [ "$eos_p50" -ge 7 ]; then' "$JOB"
require_line 'run_panel "$step" "$artifact" per-turn/v1 1 per-turn-teacher >/dev/null ||' "$JOB"
require_line 'run_panel "$step" "$artifact" per-turn/v1 0 per-turn-generated >/dev/null ||' "$JOB"
require_line 'run_panel "$step" "$artifact" one-user-turn/v1 0 one-user-generated >/dev/null ||' "$JOB"
require_line 'touch "$marker" || die "could not mark checkpoint $step scored"' "$JOB"
require_line '--reply-tokens 64 --out "$out" || return 1' "$JOB"
require_line 'check_progress "$TRAIN_RUN/progress.jsonl"' "$JOB"
require_line 'check_progress "$run_dir/progress.jsonl"' "$JOB"
require_line 'select_best_checkpoint "$RUN/checkpoint-readings-$label.jsonl"' "$JOB"
require_line 'free_complete=$(jq -er '"'"'.totals.complete | numbers'"'"' \' "$JOB"
require_line '"$RUN/panel/$ATTEMPT/step-$(printf '"'"'%06d'"'"' "$step")-per-turn-generated.json") ||' "$JOB"
require_line '| ($rows | any(.[]; .guard_top1 >= 93)) as $guard_safe' "$JOB"
require_line '| ([$rows[] | select(($guard_safe | not) or .guard_top1 >= 93)]' "$JOB"
require_line '(if $guard_safe then 0 else -(.guard_top1) end),' "$JOB"
require_line '-(.eos_top1),' "$JOB"
require_line '-(.free_eos_stops),' "$JOB"
require_line '-(.free_complete_rows),' "$JOB"
require_line '.eos_rank_p50,' "$JOB"
require_line '-(.guard_top1),' "$JOB"
require_line '.step' "$JOB"
require_line 'selected:$ranked[0],' "$JOB"
require_line 'free_generation_rendering:"per-turn/v1",' "$JOB"
require_line 'The free-generation values used by selection and the outcome map are the' "$SHEET"
require_line '`per-turn/v1` generated panel'"'"'s EOS stops and complete accepted rows. The' "$SHEET"
require_line '- Apply row 2 before continuing past 512. Apply rows 1 and 3-7 to the selected' "$SHEET"
require_line 'init_approval="$RUN/control/init-approved-$label"' "$JOB"
require_line '[ ! -e "$init_approval" ] ||' "$JOB"
require_line 'UOR_TRAIN_INIT_APPROVAL="$init_approval" \' "$JOB"
require_line 'while [ ! -f "$run_dir/.init-ready" ]; do' "$JOB"
require_line 'check_initialization "$run_dir"' "$JOB"
require_line 'touch "$init_approval" ||' "$JOB"
grep -Fq 'fs::write(out.join(INIT_READY_MARKER), b"ready\n")?;' "$TRAINER" ||
  fail "$TRAINER does not publish initialization readiness after writing the record"
checks=$((checks + 1))
grep -Fq 'wait_for_initialization_approval(out, approval.as_deref())?;' "$TRAINER" ||
  fail "$TRAINER does not wait for initialization approval before training"
checks=$((checks + 1))
require_line 'run_panel 0 "$BASE_ARTIFACT" one-user-turn/v1 1 base-one-user-teacher >/dev/null' "$JOB"
require_line '.eos_rank_min == 5 and .eos_rank_p50 == 7 and .eos_rank_max == 13 and' "$JOB"
require_line '.eos_rank_min == 18 and .eos_rank_p50 == 138 and .eos_rank_max == 873 and' "$JOB"
require_line '.complete == 0 and .eos_stops == 0 and .bos_stops == 0 and' "$JOB"
per_turn_free_lines=$(grep -Fc "' base-per-turn-generated" "$JOB")
[ "$per_turn_free_lines" -eq 1 ] ||
  fail "$JOB does not reproduce the per-turn free-generation control"
checks=$((checks + 1))
require_line '[ "$base_guard" -eq 132 ] ||' "$JOB"
grep -Fq 'die "base continuation grade did not use the registered 512-row sample"' "$JOB" ||
  fail "$JOB does not pin the base-grade sample fingerprint"
checks=$((checks + 1))
grep -Fq '.evaluation.samples == 512 and' "$JOB" ||
  fail "$JOB does not require the registered 512-row sample count"
checks=$((checks + 1))
require_line '.evaluation.sample_fingerprint ==' "$JOB"
grep -Fq 'b724656f29bdade21aecb372d0568c48c81d78c80f43fe4de8ec667a7824ccc4' "$JOB" ||
  fail "$JOB does not pin the registered 512-row sample fingerprint"
checks=$((checks + 1))
require_line '.run_config.sha256 ==' "$JOB"
require_line '.files["model.safetensors"].sha256 ==' "$JOB"
require_line '[ -f "$run/init.json" ] ||' "$JOB"
require_line 'check_file 9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9 \' "$JOB"
require_line '.counts.documents == 25071 and .counts.assistant_replies == 79679 and' "$JOB"
grep -Fq '.counts.exact_accepted_assistant_reply == 0 and' "$JOB" ||
  fail "$JOB does not reject accepted-answer overlap"
checks=$((checks + 1))
grep -Fq '.counts.assistant_replies_with_panel_6gram == 0' "$JOB" ||
  fail "$JOB does not require the registered overlap count"
checks=$((checks + 1))
require_line 'die "corpus-panel overlap audit differs from the registered reading"' "$JOB"
require_line 'die "checkpoint $next is missing without a registered stop"' "$JOB"
require_line 'train_and_score 16 attempt-1 "$RUN/run"' "$JOB"
require_line 'if [ "$TRAIN_FAILURE" -ne 0 ]; then' "$JOB"
require_line 'TRAIN_FAILURE=$train_rc' "$JOB"
require_line 'train_and_score 8 attempt-2 "$RUN/run-mb8"' "$JOB"
require_line '[ "$TRAIN_FAILURE" -eq 0 ] ||' "$JOB"
require_line "jq -e -s '(map(select(.step == 0)) | length) == 1 and" "$JOB"
require_line '([.[] | select(.step > 0)] | length) > 0 and' "$JOB"
require_line 'all(.[];' "$JOB"
require_line '((.dev_loss | type) == "number" and (.dev_loss | isfinite)) and' "$JOB"
require_line '((.dev_token_margin | type) == "number" and' "$JOB"
require_line '(.dev_token_margin | isfinite)) and' "$JOB"
require_line '((.dev_address_margin | type) == "number" and' "$JOB"
require_line '(.dev_address_margin | isfinite)) and' "$JOB"
require_line '(if .step == 0 then .train_loss_mean == null' "$JOB"
require_line 'else ((.train_loss_mean | type) == "number" and' "$JOB"
require_line '(.train_loss_mean | isfinite))' "$JOB"
grep -Fq 'CUDA_ERROR_OUT_OF_MEMORY|\bOOM\b' "$JOB" ||
  fail "$JOB does not classify the registered OOM retry"
checks=$((checks + 1))
if grep -Fq 'if ! train_and_score' "$JOB"; then
  fail "$JOB runs train_and_score in a negated context"
fi
checks=$((checks + 1))
require_line '--init-from "$B64L" \' "$JOB"
require_line '--train-split "$RUN/inputs/mix" \' "$JOB"
require_line '--train-sha256 3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \' "$JOB"
require_line '--train-mask-sha256 67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \' "$JOB"
require_line '--batch 64 --micro-batch "$micro" --steps 1024 \' "$JOB"
require_line '--eval-every 256 --samples 16 --reply-tokens 64 --seed 1 \' "$JOB"
require_line 'BUILD_ROOT=${UOR_SFT_BUILD_ROOT:-/workspace/uor-r4/build/deepseek-sft-$RUN_ID}' "$JOB"
require_line 'PANEL_TARGET=${UOR_SFT_PANEL_TARGET:-$BUILD_ROOT/panel-target}' "$JOB"
require_line 'SNAPSHOT_ROOT=${UOR_SFT_SNAPSHOT_ROOT:-$BUILD_ROOT/snapshots}' "$JOB"
require_line 'SEAL_LIMIT_BYTES=104857600' "$JOB"
require_line 'exec >> "$RUN/job.log" 2>&1' "$JOB"
require_line 'CARGO_TARGET_DIR="$PANEL_TARGET" \' "$JOB"
require_line 'PANEL_BIN="$PANEL_TARGET/release/served-reply-panel"' "$JOB"
require_line 'oh/run/b64L-s1/models/step-016384 oh/run/b64L-s1/config.json' "$JOB"
require_line 'rm -rf "$RUN/download"' "$JOB"
require_line 'rm -f "$RUN/inputs/d3-accum-s1.tar.gz"' "$JOB"
require_line '--snapshot-out "$snapshot" \' "$JOB"
require_line '--snapshot-out "$BASE_SNAPSHOT" --out "$BASE_ARTIFACT" \' "$JOB"
require_line 'rm -f "$BASE_SNAPSHOT" || die "could not remove the transient base snapshot"' "$JOB"
require_line 'READBACK_STATUS=verified' "$JOB"
grep -Fq "printf '%s\\n' \"\$seal_bytes\" > \"\$RUN/BYTES\"" "$JOB" ||
  fail "$JOB does not record the seal size"
checks=$((checks + 1))
require_line 'if [ "$actual_seal_bytes" -gt "$SEAL_LIMIT_BYTES" ]; then' "$JOB"

require_line 'created=$("$POD_TOOL" up --lab deepseek --session "$SESSION" --gpu 5090 \' "$LAUNCHER"
require_line '[ "$(git rev-parse HEAD)" = "$PIN" ] ||' "$LAUNCHER"
require_line '[ -z "$(git status --porcelain --untracked-files=no)" ] ||' "$LAUNCHER"
require_line 'git merge-base --is-ancestor "$PIN" origin/main ||' "$LAUNCHER"
require_line 'REVIEWED_RUNTIME=${UOR_SFT_REVIEWED_RUNTIME:?UOR_SFT_REVIEWED_RUNTIME must name the reviewed runtime identity}' "$LAUNCHER"
require_line 'git ls-tree -r "$PIN" |' "$LAUNCHER"
require_line "path != fit && path != report'" "$LAUNCHER"
require_line 'git show "$PIN:$RUNTIME_EXCLUDE_FIT" |' "$LAUNCHER"
require_line '/^TEST FITNESS:/ { print "TEST FITNESS: RECORDED"; next }' "$LAUNCHER"
require_line '/^PRE-RUN:/ { print "PRE-RUN: RECORDED"; next }' "$LAUNCHER"
require_line '[ "$runtime_sha" = "$REVIEWED_RUNTIME" ] ||' "$LAUNCHER"
require_line 'model_time_state=absent' "$LAUNCHER"
require_line '[ "$free_kib" -ge "$MIN_FREE_KIB" ] ||' "$LAUNCHER"
require_line 'export UOR_POD_MAX_PODS=4' "$LAUNCHER"
require_line 'export UOR_POD_MAX_RATE=8.00' "$LAUNCHER"
require_line 'export UOR_POD_PROVIDER=vast' "$LAUNCHER"
require_line 'export UOR_POD_PROVIDERS=vast' "$LAUNCHER"
require_line 'VAST_ADAPTER="$CONTROLLER/provider-target/debug/uor-r4-pod-provider"' "$LAUNCHER"
require_line 'cargo build --offline --locked -p uor-r4-pod-provider' "$LAUNCHER"
require_line 'export UOR_POD_VAST_BIN="$VAST_ADAPTER"' "$LAUNCHER"
require_line 'sha256sum "$VAST_ADAPTER" > "$CONTROLLER/provider.sha256"' "$LAUNCHER"
require_line 'if ! listings=$("$STORE" ls "$STORE_TOPIC"); then' "$LAUNCHER"
grep -Fq 'cannot verify the store topic $STORE_TOPIC; no pod will be rented' "$LAUNCHER" ||
  fail "$LAUNCHER does not fail closed when the store preflight cannot be read"
checks=$((checks + 1))
require_line '--count 1 --hours 2 --ref "$PIN" \' "$LAUNCHER"
require_line 'if ! "$POD_TOOL" run "$POD" --lab deepseek --session "$SESSION" --gpu 0 -- \' "$LAUNCHER"
grep -Fqx "JOB_LOG=\$(awk '/Log: / {" "$LAUNCHER" ||
  fail "$LAUNCHER does not begin parsing uor-pod's job-log path"
checks=$((checks + 1))
grep -Fq 'for (i = 1; i <= NF; i++) if ($i == "Log:") { print $(i + 1); exit }' \
  "$LAUNCHER" || fail "$LAUNCHER does not parse the log path from uor-pod's one-line success message"
checks=$((checks + 1))
require_line '*) cat "$CONTROLLER/run.out" >&2; echo "uor-pod did not report a job log" >&2; exit 1 ;;' "$LAUNCHER"
grep -Fq "date -u +%Y%m%dT%H%M%SZ | tr '[:upper:]' '[:lower:]'" "$LAUNCHER" ||
  fail "$LAUNCHER does not normalize the uor-pod session to lowercase"
checks=$((checks + 1))
grep -Fq 'job_exit=${probe#exit }; break ;;' "$LAUNCHER" ||
  fail "$LAUNCHER does not stop on the terminal remote exit"
checks=$((checks + 1))
require_line '[ "$job_exit" -eq 0 ] || {' "$LAUNCHER"
grep -Fq 'elif ! flock -n /root/gpu0.lock true; then printf '"'"'running\\n'"'"'' \
  "$LAUNCHER" || fail "$LAUNCHER does not distinguish a live GPU job from a stopped one"
checks=$((checks + 1))
require_line 'echo "$POD is running, but GPU 0 has no live job and $JOB_LOG has no terminal exit" >&2' "$LAUNCHER"
require_line 'if [ $((remote_now - last_progress)) -ge "$STALL_SEC" ]; then' "$LAUNCHER"
require_line 'last_progress=$started' "$LAUNCHER"
require_line 'if [ "$latest_progress" -ne 0 ]; then' "$LAUNCHER"
require_line 'stopped) failures=$((failures + 1)); unreachable=0 ;;' "$LAUNCHER"
require_line '*) failures=0; unreachable=$((unreachable + 1)) ;;' "$LAUNCHER"
require_line 'if [ $((now - started)) -lt 300 ]; then' "$LAUNCHER"
require_line 'if [ "$unreachable" -ge 15 ]; then' "$LAUNCHER"
require_line 'if stop_remote_job; then' "$LAUNCHER"
require_line '"$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" --to-store || rc=1' "$LAUNCHER"
require_line 'SUBMISSION_ATTEMPTED=1' "$LAUNCHER"
require_line '[ "$SUBMISSION_ATTEMPTED" -ne 1 ] || {' "$LAUNCHER"
grep -Fq -- "-name '*-"'$SESSION'"-gpu0.sh' -print" "$LAUNCHER" ||
  fail "$LAUNCHER cannot discover a submitted job when uor-pod loses its log-path output"
checks=$((checks + 1))
if grep -Fq 'WRAPPER_PID' "$LAUNCHER"; then
  fail "$LAUNCHER still waits on the detached-job submission wrapper"
fi
checks=$((checks + 1))
require_line 'if ! "$POD_TOOL" renew "$POD" --lab deepseek --session "$SESSION" --hours 2 \' "$LAUNCHER"
require_line '"$STORE" get "$OUT_STORE/READBACK" "$CONTROLLER/readback" >/dev/null' "$LAUNCHER"
require_line '[ -f "$READBACK_FILE" ] || { echo "no READBACK file was returned" >&2; exit 1; }' "$LAUNCHER"
require_line '[ "$(cat "$READBACK_FILE")" = verified ] || {' "$LAUNCHER"
require_line '[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ] || {' "$LAUNCHER"
require_line '--confirm-archived || rc=1' "$LAUNCHER"
require_line 'STATUS=SEALED' "$LAUNCHER"
require_line '[ "$STATUS" = SEALED ] || rc=1' "$LAUNCHER"
require_line 'discover_owned_pod' "$LAUNCHER"
require_line 'for lease in "$POD_STATE"/leases/*/"deepseek-$SESSION.json"; do' "$LAUNCHER"
require_line 'discover_submitted_job' "$LAUNCHER"
if grep -Fq 'tee -a "$RUN/job.log"' "$JOB"; then
  fail "$JOB still uses an asynchronous tee for the sealed job log"
fi
checks=$((checks + 1))
if grep -Fq 'cp -a "$checkpoint"' "$JOB" ||
  grep -Fq '"$RUN/checkpoints"' "$JOB"; then
  fail "$JOB duplicates immutable trainer checkpoints inside the seal"
fi
checks=$((checks + 1))

echo "$checks checks passed"
