#!/usr/bin/env bash
# Static gates for the reply-SFT pod job and controller.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
JOB=${1:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"}
LAUNCHER=${2:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/run-s1.sh"}
TRAINER="$ROOT/crates/uor-r4-float-train/src/in_space_long.rs"
SHEET=${3:-"$ROOT/docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md"}
EXTENSION_SHEET=${4:-"$ROOT/docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md"}
HARDWARE_LIB=${5:-"$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/hardware-evidence.sh"}
CONTENT_SHEET=${6:-"$ROOT/docs/harness/prep/one-model-end-to-end-content-ratio-2026-11-08.md"}
HARDWARE_TEST="$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/test-hardware-evidence.sh"
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
case_block() {
  local label=$1 file=$2
  awk -v label="$label" '
    {
      line=$0
      sub(/^[ \t]+/, "", line)
      sub(/[ \t]+$/, "", line)
      if (substr(line, 1, length(label)) == label) inside=1
    }
    inside {
      print
      if (line == ";;") exit
    }
  ' "$file"
}
require_case_block() {
  local label=$1 file=$2
  shift 2
  local block want
  block=$(case_block "$label" "$file")
  [ -n "$block" ] || fail "$file lacks case block $label"
  printf '%s\n' "$block" | awk 'END { exit(found ? 0 : 1) } { line=$0; sub(/^[ \t]+/, "", line); sub(/[ \t]+$/, "", line); if (line == ";;") found=1 }' ||
    fail "$file case block $label is unterminated"
  for want in "$@"; do
    printf '%s\n' "$block" | awk -v want="$want" '
      { line=$0; sub(/^[ \t]+/, "", line); sub(/[ \t]+$/, "", line); if (line == want) found=1 }
      END { exit(found ? 0 : 1) }
    ' || fail "$file case block $label lacks complete line: $want"
  done
  checks=$((checks + 1))
}
reject_case_block_line() {
  local label=$1 file=$2 want=$3 block
  block=$(case_block "$label" "$file")
  [ -n "$block" ] || fail "$file lacks case block $label"
  if printf '%s\n' "$block" | awk -v want="$want" '
    { line=$0; sub(/^[ \t]+/, "", line); sub(/[ \t]+$/, "", line); if (line == want) found=1 }
    END { exit(found ? 0 : 1) }
  '; then
    fail "$file case block $label has forbidden complete line: $want"
  fi
  checks=$((checks + 1))
}

bash -n "$JOB"
bash -n "$LAUNCHER"
bash -n "$HARDWARE_LIB"
bash -n "$HARDWARE_TEST"
checks=$((checks + 4))
if command -v shellcheck >/dev/null; then
  shellcheck "$JOB" "$LAUNCHER" "$HARDWARE_LIB" "$HARDWARE_TEST"
  checks=$((checks + 1))
fi
[ -x "$HARDWARE_TEST" ] || fail "$HARDWARE_TEST is not executable"
"$HARDWARE_TEST" "$HARDWARE_LIB" >/dev/null
checks=$((checks + 1))

require_line '2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;' "$JOB"
require_line '*) echo "SEED must be 1; seed 2 is a later decision" >&2; exit 2 ;;' "$JOB"
require_line 'got=$(sha256sum "$path" | cut -d'"'"' '"'"' -f1)' "$JOB"
require_line '[ "$got" = "$want" ] || die "$path hashes to $got, not $want"' "$JOB"
require_line 'RECIPE=${UOR_SFT_RECIPE:-legacy}' "$JOB"
require_line 'case "$RECIPE:$STEPS:$RUN_LABEL" in' "$JOB"
require_line 'legacy:1024:s1|legacy:2048:s1-2048|content-ratio:2048:s1-content-ratio) ;;' "$JOB"
require_line '*) echo "recipe, steps and run label do not form a registered SFT pair" >&2; exit 2 ;;' "$JOB"
for forbidden_override in UOR_SFT_OUT_STORE UOR_SFT_PREP_SHEET UOR_SFT_REPORT UOR_SFT_TRAIN_ROOT UOR_SFT_TRAIN_SHA; do
  if grep -Fq "$forbidden_override" "$JOB" "$LAUNCHER"; then
    fail "runtime exposes unregistered recipe override $forbidden_override"
  fi
  checks=$((checks + 1))
done
require_line 'STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07' "$JOB"
require_line 'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08' "$JOB"
require_line 'OUT_STORE="$STORE_TOPIC/runs/$RUN_LABEL-$RUN_ID"' "$JOB"
require_line 'store_listing=$("$STORE" ls "$STORE_TOPIC") ||' "$JOB"
require_line 'die "cannot verify the store topic before training: $STORE_TOPIC"' "$JOB"
require_line 'if [ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS); then' "$JOB"
require_line 'if "$STORE" put "$RUN" "$OUT_STORE"; then' "$JOB"
require_line '"$STORE" put "$RUN.readback-status" "$OUT_STORE/READBACK" ||' "$JOB"

require_line 'fetch_file "$STORE_TOPIC/inputs/tokens.u16" "$RUN/inputs/mix/tokens.u16"' "$JOB"
require_line 'fetch_file "$STORE_TOPIC/inputs/response_mask.u8" "$RUN/inputs/mix/response_mask.u8"' "$JOB"
require_line 'fetch_file "$STORE_TOPIC/inputs/manifest.json" "$RUN/inputs/mix/manifest.json"' "$JOB"
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

require_line 'TRAIN_TOKENS_SHA=3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3' "$JOB"
require_line 'TRAIN_MASK_SHA=67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b' "$JOB"
require_line 'TRAIN_MANIFEST_SHA=1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe' "$JOB"
require_line 'TRAIN_DOCUMENTS=25071' "$JOB"
require_line 'TRAIN_ASSISTANT_REPLIES=79679' "$JOB"
require_line 'TRAIN_TOKENS_SHA=3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34' "$JOB"
require_line 'TRAIN_MASK_SHA=83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf' "$JOB"
require_line 'TRAIN_MANIFEST_SHA=8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd' "$JOB"
require_line 'TRAIN_DOCUMENTS=65071' "$JOB"
require_line 'TRAIN_ASSISTANT_REPLIES=223469' "$JOB"
require_line 'check_file "$TRAIN_TOKENS_SHA" "$RUN/inputs/mix/tokens.u16"' "$JOB"
require_line 'check_file "$TRAIN_MASK_SHA" "$RUN/inputs/mix/response_mask.u8"' "$JOB"
require_line 'check_file "$TRAIN_MANIFEST_SHA" "$RUN/inputs/mix/manifest.json"' "$JOB"
require_line '.rows_used == 65071 and .tokens == 11786293 and' "$JOB"
require_line '.response_tokens == 5710018 and' "$JOB"
require_line '["oasst2-reply", "mworld-repeat-1", "mworld-repeat-2", "mworld-repeat-3"]'"'"' \' "$JOB"
require_line 'die "content-ratio corpus manifest differs from the registered split"' "$JOB"
require_case_block 'legacy)' "$JOB" \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07' \
  'TRAIN_TOKENS_SHA=3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3' \
  'TRAIN_MASK_SHA=67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b' \
  'TRAIN_MANIFEST_SHA=1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe' \
  'TRAIN_DOCUMENTS=25071' 'TRAIN_ASSISTANT_REPLIES=79679'
reject_case_block_line 'legacy)' "$JOB" \
  'TRAIN_TOKENS_SHA=3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34'
require_case_block 'content-ratio)' "$JOB" \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08' \
  'TRAIN_TOKENS_SHA=3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34' \
  'TRAIN_MASK_SHA=83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf' \
  'TRAIN_MANIFEST_SHA=8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd' \
  'TRAIN_DOCUMENTS=65071' 'TRAIN_ASSISTANT_REPLIES=223469'
reject_case_block_line 'content-ratio)' "$JOB" \
  'TRAIN_TOKENS_SHA=3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3'

for hash in \
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
require_line '## Bar' "$EXTENSION_SHEET"
require_line '## Data split' "$EXTENSION_SHEET"
require_line '## Steps' "$EXTENSION_SHEET"
require_line '## Stop rule' "$EXTENSION_SHEET"
require_line '## Bar' "$CONTENT_SHEET"
require_line '## Data split' "$CONTENT_SHEET"
require_line '## Recipe' "$CONTENT_SHEET"
require_line '## Stop rule' "$CONTENT_SHEET"
require_line '| `tokens.u16` | `3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34` |' "$CONTENT_SHEET"
require_line '| `response_mask.u8` | `83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf` |' "$CONTENT_SHEET"
require_line '| `manifest.json` | `8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd` |' "$CONTENT_SHEET"
require_line '`control/post-use-inputs.json`, and then removes only those two raw payloads.' "$CONTENT_SHEET"
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
require_line 'verify_grade_labels "$RUN/control/base-grade.json" base' "$JOB"
require_line 'bind_grade_hardware "$RUN/control/base-grade.json"' "$JOB"
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
require_line '.counts.documents == $documents and .counts.assistant_replies == $replies and' "$JOB"
require_line '--argjson documents "$TRAIN_DOCUMENTS" \' "$JOB"
require_line '--argjson replies "$TRAIN_ASSISTANT_REPLIES" '"'"'' "$JOB"
grep -Fq '.counts.exact_accepted_assistant_reply == 0 and' "$JOB" ||
  fail "$JOB does not reject accepted-answer overlap"
checks=$((checks + 1))
grep -Fq '.counts.assistant_replies_with_panel_6gram == 0' "$JOB" ||
  fail "$JOB does not require the registered overlap count"
checks=$((checks + 1))
require_line 'die "corpus-panel overlap audit differs from the registered reading"' "$JOB"
require_line 'die "checkpoint $next is missing without a registered stop"' "$JOB"
require_line 'verify_grade_labels "$grade_dir.json" "checkpoint $step"' "$JOB"
require_line 'bind_grade_hardware "$grade_dir.json"' "$JOB"
require_line 'train_and_score 16 attempt-1 "$RUN/run"' "$JOB"
require_line 'if [ "$TRAIN_FAILURE" -ne 0 ]; then' "$JOB"
require_line 'TRAIN_FAILURE=$train_rc' "$JOB"
require_line 'compact_failed_attempt attempt-1 "$TRAIN_RUN"' "$JOB"
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
require_line '--train-sha256 "$TRAIN_TOKENS_SHA" \' "$JOB"
require_line '--train-mask-sha256 "$TRAIN_MASK_SHA" \' "$JOB"
require_line 'STEPS=${UOR_SFT_STEPS:-1024}' "$JOB"
require_line '--batch 64 --micro-batch "$micro" --steps "$STEPS" \' "$JOB"
require_line 'if [ "$next" -le "$STEPS" ]; then' "$JOB"
require_line 'while [ "$next" -le "$STEPS" ]; do' "$JOB"
require_line 'log "training completed all $STEPS steps"' "$JOB"
require_line '--eval-every 256 --samples 16 --reply-tokens 64 --seed 1 \' "$JOB"
require_line 'BUILD_ROOT=${UOR_SFT_BUILD_ROOT:-/workspace/uor-r4/build/deepseek-sft-$RUN_ID}' "$JOB"
require_line 'PANEL_TARGET=${UOR_SFT_PANEL_TARGET:-$BUILD_ROOT/panel-target}' "$JOB"
require_line 'SNAPSHOT_ROOT=${UOR_SFT_SNAPSHOT_ROOT:-$BUILD_ROOT/snapshots}' "$JOB"
require_line 'SEAL_LIMIT_BYTES=104857600' "$JOB"
require_line 'compact_successful_inputs() {' "$HARDWARE_LIB"
require_line '[ "$tokens_sha" = "$TRAIN_TOKENS_SHA" ] &&' "$HARDWARE_LIB"
require_line '[ "$mask_sha" = "$TRAIN_MASK_SHA" ] || return 1' "$HARDWARE_LIB"
require_line 'output="$run/control/post-use-inputs.json"' "$HARDWARE_LIB"
require_line 'reason:"successful-run seal compaction",' "$HARDWARE_LIB"
require_line 'mv "$staging" "$output" || return 1' "$HARDWARE_LIB"
require_line '[ -f "$output" ] || return 1' "$HARDWARE_LIB"
require_line 'rm -f "$tokens" "$mask" || return 1' "$HARDWARE_LIB"
require_line 'compact_successful_inputs "$RUN" ||' "$JOB"
require_line '{ echo "could not compact the used training inputs for sealing" >&2; rc=1; }' "$JOB"
require_line 'exec >> "$RUN/job.log" 2>&1' "$JOB"
require_line 'if ! seal_bytes=$(measure_seal_bytes "$RUN"); then' "$JOB"
require_line 'if ! actual_seal_bytes=$(measure_all_bytes "$RUN"); then' "$JOB"
require_line 'source "$HARDWARE_LIB"' "$JOB"
require_line 'capture_cpu_hardware "$RUN"' "$JOB"
require_line 'verify_grade_hardware_bindings "$RUN" || rc=1' "$JOB"
require_line 'write_seal_manifest "$RUN"' "$JOB"
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
require_line 'RECIPE=${UOR_SFT_RECIPE:-legacy}' "$LAUNCHER"
require_line 'STEPS=${UOR_SFT_STEPS:-1024}' "$LAUNCHER"
require_line 'case "$RECIPE:$STEPS:$RUN_LABEL" in' "$LAUNCHER"
require_line 'legacy:1024:s1)' "$LAUNCHER"
require_line 'legacy:2048:s1-2048)' "$LAUNCHER"
require_line 'content-ratio:2048:s1-content-ratio)' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-25/REPORT.md' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-29/REPORT.md' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-content-ratio-2026-11-08.md' "$LAUNCHER"
require_line 'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-31/REPORT.md' "$LAUNCHER"
require_line 'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08' "$LAUNCHER"
require_line 'OUT_STORE="$STORE_TOPIC/runs/$RUN_LABEL-$RUN_ID"' "$LAUNCHER"
require_case_block 'legacy:1024:s1)' "$LAUNCHER" \
  'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md' \
  'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-25/REPORT.md' \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07'
require_case_block 'legacy:2048:s1-2048)' "$LAUNCHER" \
  'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md' \
  'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-29/REPORT.md' \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07'
require_case_block 'content-ratio:2048:s1-content-ratio)' "$LAUNCHER" \
  'RUNTIME_EXCLUDE_FIT=docs/harness/prep/one-model-end-to-end-content-ratio-2026-11-08.md' \
  'RUNTIME_EXCLUDE_REPORT=docs/loop/tasks/one-model-end-to-end/deepseek-31/REPORT.md' \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08'
reject_case_block_line 'legacy:1024:s1)' "$LAUNCHER" \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08'
reject_case_block_line 'legacy:2048:s1-2048)' "$LAUNCHER" \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08'
reject_case_block_line 'content-ratio:2048:s1-content-ratio)' "$LAUNCHER" \
  'STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07'
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
require_line 'UOR_SFT_RECIPE="$RECIPE" UOR_SFT_STEPS="$STEPS" \' "$LAUNCHER"
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
