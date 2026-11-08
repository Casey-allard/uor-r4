#!/usr/bin/env bash
# Run the reviewed reply-masked SFT unit on a bootstrapped UOR-R4 Vast pod.
#
# Usage:
#   UOR_SFT_PIN=<40-hex-merge-pin> UOR_SFT_RUN=/workspace/uor-r4/runs/sft-s1 \
#     UOR_SFT_RECIPE=legacy UOR_SFT_STEPS=1024 UOR_SFT_RUN_LABEL=s1 \
#     bash /workspace/one-model-reply-sft-job.sh 1
#
# The caller owns the pod, GPU lease and teardown. This script fetches every input,
# runs the base controls, trains the registered 1,024- or 2,048-step dose with
# checkpoints every 256 steps, exports and scores each checkpoint, applies the
# registered 512-step stop, then seals and reads the evidence back through uor-store.
set -euo pipefail

SEED=${1:?usage: job.sh SEED}
case "$SEED" in
  1) ;;
  2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;
  *) echo "SEED must be 1; seed 2 is a later decision" >&2; exit 2 ;;
esac
[ "$#" -eq 1 ] || { echo "usage: job.sh 1" >&2; exit 2; }
RECIPE=${UOR_SFT_RECIPE:-legacy}
STEPS=${UOR_SFT_STEPS:-1024}
RUN_LABEL=${UOR_SFT_RUN_LABEL:-s1}
case "$RECIPE:$STEPS:$RUN_LABEL" in
  legacy:1024:s1|legacy:2048:s1-2048|content-ratio:2048:s1-content-ratio) ;;
  *) echo "recipe, steps and run label do not form a registered SFT pair" >&2; exit 2 ;;
esac

PIN=${UOR_SFT_PIN:?UOR_SFT_PIN must name the reviewed 40-hex merge commit}
case "$PIN" in
  *[!0-9a-f]*|'') echo "UOR_SFT_PIN is not lowercase 40-hex" >&2; exit 2 ;;
esac
[ "${#PIN}" -eq 40 ] || { echo "UOR_SFT_PIN is not 40 hex characters" >&2; exit 2; }

RUN_ID=${UOR_SFT_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')}
RUN=${UOR_SFT_RUN:-/workspace/uor-r4/runs/deepseek-sft-s$SEED}
case "$RECIPE" in
  legacy)
    STORE_TOPIC=results/deepseek/one-model-end-to-end-reply-sft-2026-11-07
    TRAIN_TOKENS_SHA=3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3
    TRAIN_MASK_SHA=67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b
    TRAIN_MANIFEST_SHA=1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe
    TRAIN_DOCUMENTS=25071
    TRAIN_ASSISTANT_REPLIES=79679
    ;;
  content-ratio)
    STORE_TOPIC=results/deepseek/one-model-end-to-end-content-ratio-2026-11-08
    TRAIN_TOKENS_SHA=3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34
    TRAIN_MASK_SHA=83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf
    TRAIN_MANIFEST_SHA=8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd
    TRAIN_DOCUMENTS=65071
    TRAIN_ASSISTANT_REPLIES=223469
    ;;
esac
OUT_STORE="$STORE_TOPIC/runs/$RUN_LABEL-$RUN_ID"
STORE=/root/uor-store
REPO=${UOR_REPO:-/root/uor-sft-src-$PIN}
TARGET=${CARGO_TARGET_DIR:-/workspace/uor-r4/target-deepseek-sft}
BUILD_ROOT=${UOR_SFT_BUILD_ROOT:-/workspace/uor-r4/build/deepseek-sft-$RUN_ID}
PANEL_TARGET=${UOR_SFT_PANEL_TARGET:-$BUILD_ROOT/panel-target}
SNAPSHOT_ROOT=${UOR_SFT_SNAPSHOT_ROOT:-$BUILD_ROOT/snapshots}
SEAL_LIMIT_BYTES=104857600
export RAYON_NUM_THREADS=${UOR_SFT_CPU_THREADS:-8}
export CARGO_TARGET_DIR=$TARGET

case "$RUN_ID" in
  ''|*..*|*[!A-Za-z0-9._-]*) echo "UOR_SFT_RUN_ID contains unsafe characters" >&2; exit 2 ;;
esac
case "$RUN" in
  /workspace/*) ;;
  *) echo "UOR_SFT_RUN must stay under /workspace" >&2; exit 2 ;;
esac
[ ! -e "$RUN" ] || { echo "refusing an existing run root: $RUN" >&2; exit 2; }

die() {
  echo "one-model-reply-sft: $*" >&2
  exit 1
}
log() {
  printf '%s\n' "$*"
}
check_file() {
  local want=$1 path=$2 got
  got=$(sha256sum "$path" | cut -d' ' -f1)
  [ "$got" = "$want" ] || die "$path hashes to $got, not $want"
}
check_path_absent() {
  local path=$1
  [ ! -e "$path" ] || die "refusing an existing output: $path"
}
fail() {
  die "$*"
}

command -v git >/dev/null || die "git is required"
command -v jq >/dev/null || die "jq is required"
command -v sha256sum >/dev/null || die "sha256sum is required"
command -v nvidia-smi >/dev/null || die "nvidia-smi is required"
[ -x "$STORE" ] || die "uor-store is not installed"

store_listing=$("$STORE" ls "$STORE_TOPIC") ||
  die "cannot verify the store topic before training: $STORE_TOPIC"
if printf '%s\n' "$store_listing" |
  awk -v path="$OUT_STORE/" 'index($0, path) { found = 1 } END { exit(found ? 0 : 1) }'; then
  die "refusing a nonempty store output: $OUT_STORE"
fi

install -d "$RUN/inputs" "$RUN/control" "$RUN/artifacts" \
  "$RUN/grades" "$RUN/panel" "$RUN/scores" "$RUN/download" "$RUN/logs"
exec >> "$RUN/job.log" 2>&1

TRAIN_PID=
STOP_REASON=
TRAIN_RUN=
ATTEMPT=
TRAIN_FAILURE=0
on_exit() {
  local rc=$?
  # Detach the seal from RUN/job.log, whose bytes are about to be hashed. The job log
  # is written directly rather than through an asynchronous process substitution.
  exec >> "$RUN.seal.log" 2>&1
  if [ -n "$TRAIN_PID" ] && kill -0 "$TRAIN_PID" 2>/dev/null; then
    kill -TERM "$TRAIN_PID" 2>/dev/null || true
    kill -CONT "$TRAIN_PID" 2>/dev/null || true
    wait "$TRAIN_PID" 2>/dev/null || true
  fi
  if [ "$rc" -eq 0 ]; then
    compact_successful_inputs "$RUN" ||
      { echo "could not compact the used training inputs for sealing" >&2; rc=1; }
  fi
  if declare -F verify_grade_hardware_bindings >/dev/null; then
    verify_grade_hardware_bindings "$RUN" || rc=1
  fi
  printf '%s\n' "$rc" > "$RUN/EXIT"
  printf '%s\n' "${STOP_REASON:-natural-or-failure}" > "$RUN/STOP_REASON"
  local seal_bytes actual_seal_bytes
  if ! seal_bytes=$(measure_seal_bytes "$RUN"); then
    echo "could not measure the seal" >&2
    rc=1
    seal_bytes=0
  fi
  printf '%s\n' "$seal_bytes" > "$RUN/BYTES"
  (
    cd "$RUN"
    write_seal_manifest "$RUN"
  ) || rc=1
  if ! actual_seal_bytes=$(measure_all_bytes "$RUN"); then
    echo "could not measure the completed seal" >&2
    rc=1
    actual_seal_bytes=$SEAL_LIMIT_BYTES
  fi
  if [ "$actual_seal_bytes" -gt "$SEAL_LIMIT_BYTES" ]; then
    echo "seal is $actual_seal_bytes bytes, over the $SEAL_LIMIT_BYTES-byte limit" >&2
    rc=1
  fi
  READBACK_STATUS=failed
  if [ "$rc" -eq 0 ]; then
    READBACK_STATUS=not-attempted
    if "$STORE" put "$RUN" "$OUT_STORE"; then
      rm -rf "$RUN.readback"
      install -d "$RUN.readback"
      if "$STORE" get "$OUT_STORE/" "$RUN.readback"; then
        local sealed="$RUN.readback/$OUT_STORE"
        if [ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS); then
          READBACK_STATUS=verified
        fi
      fi
    fi
  fi
  printf '%s\n' "$READBACK_STATUS" > "$RUN.readback-status"
  if [ "$READBACK_STATUS" = verified ]; then
    "$STORE" put "$RUN.readback-status" "$OUT_STORE/READBACK" ||
      { echo "could not upload READBACK marker" >&2; rc=1; }
  else
    echo "store readback was not verified; retain the pod" >&2
    rc=1
  fi
  exit "$rc"
}
trap on_exit EXIT

fetch_file() {
  local stored=$1 destination=$2
  "$STORE" get "$stored" "$RUN/download" || die "store get $stored"
  local source="$RUN/download/$stored"
  [ -f "$source" ] || die "store get did not create $source"
  install -D "$source" "$destination"
}

log "pin $PIN"
log "run $RUN"
log "store output $OUT_STORE"
nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader |
  tee "$RUN/gpu.txt"

if [ ! -d "$REPO/.git" ]; then
  install -d "$REPO"
  git -C "$REPO" init --quiet
  git -C "$REPO" remote add origin https://github.com/UOR-Foundation/uor-r4.git
fi
git -C "$REPO" fetch --quiet --depth 1 origin "$PIN" || die "could not fetch $PIN"
git -C "$REPO" checkout --quiet --detach FETCH_HEAD || die "could not checkout $PIN"
actual=$(git -C "$REPO" rev-parse HEAD)
[ "$actual" = "$PIN" ] || die "source is $actual, not $PIN"
[ -z "$(git -C "$REPO" status --porcelain)" ] || die "source checkout is dirty"
HARDWARE_LIB="$REPO/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/hardware-evidence.sh"
[ -f "$HARDWARE_LIB" ] || die "missing pinned hardware-evidence library"
# shellcheck disable=SC1091
# shellcheck source=hardware-evidence.sh
source "$HARDWARE_LIB"
capture_cpu_hardware "$RUN"

fetch_file "$STORE_TOPIC/inputs/tokens.u16" "$RUN/inputs/mix/tokens.u16"
fetch_file "$STORE_TOPIC/inputs/response_mask.u8" "$RUN/inputs/mix/response_mask.u8"
fetch_file "$STORE_TOPIC/inputs/manifest.json" "$RUN/inputs/mix/manifest.json"
fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/tokens.u16" \
  "$RUN/inputs/oasst2-heldout/tokens.u16"
fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/response_mask.u8" \
  "$RUN/inputs/oasst2-heldout/response_mask.u8"
fetch_file "results/claude/real-text-corpus-2026-11-05/oasst2/heldout/manifest.json" \
  "$RUN/inputs/oasst2-heldout/manifest.json"
fetch_file "results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/d3-accum-s1.tar.gz" \
  "$RUN/inputs/d3-accum-s1.tar.gz"
fetch_file "data/m2-fresh256/inputs.json" "$RUN/inputs/m2-fresh256/inputs.json"
fetch_file "data/m2-fresh256/labels.json" "$RUN/inputs/m2-fresh256/labels.json"
fetch_file "data/m2-fresh256/plan.json" "$RUN/inputs/m2-fresh256/plan.json"
fetch_file "data/m2-fresh256/manifest.json" "$RUN/inputs/m2-fresh256/manifest.json"
fetch_file "w2-inputs/2026-10-25/driver-data/tokenizer.json" "$RUN/inputs/data/tokenizer.json"
fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16" \
  "$RUN/inputs/data/ft/dev/tokens.u16"
fetch_file "w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8" \
  "$RUN/inputs/data/ft/dev/response_mask.u8"

check_file "$TRAIN_TOKENS_SHA" "$RUN/inputs/mix/tokens.u16"
check_file "$TRAIN_MASK_SHA" "$RUN/inputs/mix/response_mask.u8"
check_file "$TRAIN_MANIFEST_SHA" "$RUN/inputs/mix/manifest.json"
if [ "$RECIPE" = content-ratio ]; then
  jq -e --arg tokens "$TRAIN_TOKENS_SHA" --arg mask "$TRAIN_MASK_SHA" '
    .tokens_sha256 == $tokens and
    .mask_sha256 == $mask and
    .rows_used == 65071 and .tokens == 11786293 and
    .response_tokens == 5710018 and
    (.files | map(.label)) ==
      ["oasst2-reply", "mworld-repeat-1", "mworld-repeat-2", "mworld-repeat-3"]' \
    "$RUN/inputs/mix/manifest.json" >/dev/null ||
    die "content-ratio corpus manifest differs from the registered split"
fi
check_file 02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e \
  "$RUN/inputs/oasst2-heldout/tokens.u16"
check_file 9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833 \
  "$RUN/inputs/oasst2-heldout/response_mask.u8"
check_file 7474ba9331f242148dd9833d5cb085307c8e0d86314d13f285a0ab1c42b8b081 \
  "$RUN/inputs/oasst2-heldout/manifest.json"
check_file f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba \
  "$RUN/inputs/d3-accum-s1.tar.gz"
check_file 1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a \
  "$RUN/inputs/m2-fresh256/inputs.json"
check_file 3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be \
  "$RUN/inputs/m2-fresh256/labels.json"
check_file 0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715 \
  "$RUN/inputs/m2-fresh256/plan.json"
check_file 7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964 \
  "$RUN/inputs/m2-fresh256/manifest.json"
check_file d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89 \
  "$RUN/inputs/data/tokenizer.json"
check_file 0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b \
  "$RUN/inputs/data/ft/dev/tokens.u16"
check_file a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078 \
  "$RUN/inputs/data/ft/dev/response_mask.u8"

install -d "$RUN/inputs/base"
tar -xzf "$RUN/inputs/d3-accum-s1.tar.gz" -C "$RUN/inputs/base" \
  oh/run/b64L-s1/models/step-016384 oh/run/b64L-s1/config.json
B64L="$RUN/inputs/base/oh/run/b64L-s1/models/step-016384"
B64L_CONFIG="$RUN/inputs/base/oh/run/b64L-s1/config.json"
[ -d "$B64L" ] || die "the base archive has no $B64L"
check_file 6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de \
  "$B64L/model.safetensors"
check_file 1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2 \
  "$B64L/config.json"
check_file 672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd \
  "$B64L/hamming_read.safetensors"
check_file 6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2 \
  "$B64L_CONFIG"
rm -rf "$RUN/download"
rm -f "$RUN/inputs/d3-accum-s1.tar.gz"

log "building pinned binaries"
(
  cd "$REPO"
  cargo build --release --locked -p uor-r4-float-train --features cuda \
    --bin train-in-space
  cargo build --release --locked -p uor-r4-cpu-train \
    --bin cpu-checkpoint-grade --bin cpu-fbs-export
  cargo test --release --locked -p uor-r4-float-train --features cuda --lib \
    in_space_long::tests::checkpoint_initialization
  cargo test --release --locked -p uor-r4-float-train --features cuda --lib \
    in_space_long::tests::a_step_saved_as_served
)
(
  cd "$REPO/docs/labs/one-model-end-to-end/instrument"
  CARGO_TARGET_DIR="$PANEL_TARGET" \
    cargo build --release --locked --features cpu-parity \
      --bin served-reply-panel --bin corpus-panel-overlap
)

TRAIN_BIN="$TARGET/release/train-in-space"
GRADE_BIN="$TARGET/release/cpu-checkpoint-grade"
EXPORT_BIN="$TARGET/release/cpu-fbs-export"
PANEL_BIN="$PANEL_TARGET/release/served-reply-panel"
OVERLAP_BIN="$PANEL_TARGET/release/corpus-panel-overlap"
for binary in "$TRAIN_BIN" "$GRADE_BIN" "$EXPORT_BIN" "$PANEL_BIN" "$OVERLAP_BIN"; do
  [ -x "$binary" ] || die "build did not produce $binary"
done
sha256sum "$TRAIN_BIN" "$GRADE_BIN" "$EXPORT_BIN" "$PANEL_BIN" "$OVERLAP_BIN" \
  > "$RUN/binaries.sha256"

check_progress() {
  local progress=$1
  [ -s "$progress" ] || die "training wrote no progress"
  jq -e -s '(map(select(.step == 0)) | length) == 1 and
    ([.[] | select(.step > 0)] | length) > 0 and
    all(.[];
      ((.dev_loss | type) == "number" and (.dev_loss | isfinite)) and
      ((.dev_token_margin | type) == "number" and
        (.dev_token_margin | isfinite)) and
      ((.dev_address_margin | type) == "number" and
        (.dev_address_margin | isfinite)) and
      (if .step == 0 then .train_loss_mean == null
       else ((.train_loss_mean | type) == "number" and
         (.train_loss_mean | isfinite))
       end))' \
    "$progress" >/dev/null || die "progress has a non-finite loss"
}

grade_checkpoint() {
  local step=$1 checkpoint=$2 grade_dir artifact_dir snapshot
  grade_dir="$RUN/grades/$ATTEMPT/step-$(printf '%06d' "$step")"
  artifact_dir="$RUN/artifacts/$ATTEMPT/step-$(printf '%06d' "$step")"
  snapshot="$SNAPSHOT_ROOT/$ATTEMPT/step-$(printf '%06d' "$step").snapshot"
  install -d "$artifact_dir"
  install -d "$(dirname "$snapshot")"
  "$GRADE_BIN" \
    --checkpoint "$checkpoint" \
    --run-config "$TRAIN_RUN/config.json" \
    --snapshot-out "$snapshot" \
    --artifact-out "$artifact_dir/model.uorlut" \
    --tokens "$RUN/inputs/oasst2-heldout/tokens.u16" \
    --mask "$RUN/inputs/oasst2-heldout/response_mask.u8" \
    --manifest "$RUN/inputs/oasst2-heldout/manifest.json" \
    --max-evaluation 512 --max-replies 1 --reply-tokens 1 \
    --raw-gates --raw-transition \
    --out "$grade_dir.json" ||
    die "checkpoint $step did not export and grade"
  jq -e '.evaluation.top1_correct >= 0' "$grade_dir.json" >/dev/null ||
    die "checkpoint $step grade has no top-1 count"
  jq -e '.evaluation.samples == 512 and
    .evaluation.sample_fingerprint ==
    "b724656f29bdade21aecb372d0568c48c81d78c80f43fe4de8ec667a7824ccc4"' \
    "$grade_dir.json" >/dev/null ||
    die "checkpoint $step grade did not use the registered 512-row sample"
  verify_grade_labels "$grade_dir.json" "checkpoint $step"
  bind_grade_hardware "$grade_dir.json"
  rm -f "$snapshot" || die "could not remove transient snapshot for checkpoint $step"
}

run_panel() {
  local step=$1 artifact=$2 rendering=$3 teacher=$4 label=$5
  local out
  out="$RUN/panel/${ATTEMPT:+$ATTEMPT/}step-$(printf '%06d' "$step")-$label.json"
  "$PANEL_BIN" \
    --artifact "$artifact" \
    --tokenizer "$RUN/inputs/data/tokenizer.json" \
    --inputs "$RUN/inputs/m2-fresh256/inputs.json" \
    --labels "$RUN/inputs/m2-fresh256/labels.json" \
    --plan "$RUN/inputs/m2-fresh256/plan.json" \
    --manifest "$RUN/inputs/m2-fresh256/manifest.json" \
    --rendering "$rendering" --teacher-forced "$teacher" \
    --reply-tokens 64 --out "$out" || return 1
  printf '%s\n' "$out"
}

score_checkpoint() {
  local step=$1 checkpoint=$2
  local marker artifact
  marker="$RUN/scores/$ATTEMPT/step-$(printf '%06d' "$step").scored"
  [ ! -e "$marker" ] || return 0
  [ -d "$checkpoint" ] || die "checkpoint $step is missing"
  install -d "$RUN/scores/$ATTEMPT"
  check_progress "$TRAIN_RUN/progress.jsonl"
  grade_checkpoint "$step" "$checkpoint" ||
    die "checkpoint $step export and grade failed"
  artifact="$RUN/artifacts/$ATTEMPT/step-$(printf '%06d' "$step")/model.uorlut"
  run_panel "$step" "$artifact" per-turn/v1 1 per-turn-teacher >/dev/null ||
    die "checkpoint $step per-turn teacher panel failed"
  run_panel "$step" "$artifact" per-turn/v1 0 per-turn-generated >/dev/null ||
    die "checkpoint $step per-turn generation panel failed"
  run_panel "$step" "$artifact" one-user-turn/v1 0 one-user-generated >/dev/null ||
    die "checkpoint $step one-user panel failed"

  local guard eos_top1 eos_p50 free_complete free_eos free_pairs
  guard=$(jq -er '.evaluation.top1_correct | numbers' \
    "$RUN/grades/$ATTEMPT/step-$(printf '%06d' "$step").json") ||
    die "checkpoint $step grade has no numeric top-1 reading"
  eos_top1=$(jq -er '.teacher_forced.totals.eos_top1 | numbers' \
    "$RUN/panel/$ATTEMPT/step-$(printf '%06d' "$step")-per-turn-teacher.json") ||
    die "checkpoint $step teacher panel has no numeric EOS top-1 reading"
  eos_p50=$(jq -er '.teacher_forced.totals.eos_rank_p50 | numbers' \
    "$RUN/panel/$ATTEMPT/step-$(printf '%06d' "$step")-per-turn-teacher.json") ||
    die "checkpoint $step teacher panel has no numeric EOS median reading"
  free_complete=$(jq -er '.totals.complete | numbers' \
    "$RUN/panel/$ATTEMPT/step-$(printf '%06d' "$step")-per-turn-generated.json") ||
    die "checkpoint $step free panel has no numeric complete-row reading"
  free_eos=$(jq -er '.totals.eos_stops | numbers' \
    "$RUN/panel/$ATTEMPT/step-$(printf '%06d' "$step")-per-turn-generated.json") ||
    die "checkpoint $step free panel has no numeric EOS-stop reading"
  free_pairs=$(jq -er '.totals.pairs_both_complete_distinct | numbers' \
    "$RUN/panel/$ATTEMPT/step-$(printf '%06d' "$step")-per-turn-generated.json") ||
    die "checkpoint $step free panel has no numeric complete-pair reading"
  jq -n --argjson step "$step" --argjson guard "$guard" \
    --argjson eos_top1 "$eos_top1" --argjson eos_p50 "$eos_p50" \
    --argjson free_complete "$free_complete" --argjson free_eos "$free_eos" \
    --argjson free_pairs "$free_pairs" \
    '{step:$step, guard_top1:$guard, eos_top1:$eos_top1,
      eos_rank_p50:$eos_p50, free_complete_rows:$free_complete,
      free_eos_stops:$free_eos, free_complete_pairs:$free_pairs}' \
    >> "$RUN/checkpoint-readings-$ATTEMPT.jsonl" ||
    die "could not record checkpoint $step readings"
  if [ "$step" -eq 512 ] && [ "$eos_top1" -eq 0 ] && [ "$eos_p50" -ge 7 ]; then
    STOP_REASON="no-ranking-movement-step-512"
    kill "$TRAIN_PID" 2>/dev/null || true
  fi
  touch "$marker" || die "could not mark checkpoint $step scored"
}

select_best_checkpoint() {
  local readings=$1
  jq -e -s '
    . as $rows
    | ($rows | any(.[]; .guard_top1 >= 93)) as $guard_safe
    | ([$rows[] | select(($guard_safe | not) or .guard_top1 >= 93)]
       | sort_by([
           (if $guard_safe then 0 else -(.guard_top1) end),
           -(.eos_top1),
           -(.free_eos_stops),
           -(.free_complete_rows),
           .eos_rank_p50,
           -(.guard_top1),
           .step
         ])) as $ranked
    | {schema:"uor-r4.reply-sft-best-checkpoint/1",
       guard_safe_candidates:$guard_safe,
       free_generation_rendering:"per-turn/v1",
       selected:$ranked[0],
       ranked:$ranked}' \
    "$readings" > "$RUN/best-checkpoint.json" ||
    die "could not select the registered best checkpoint"
}

log "panel-overlap control"
"$OVERLAP_BIN" \
  --tokens "$RUN/inputs/mix/tokens.u16" \
  --mask "$RUN/inputs/mix/response_mask.u8" \
  --tokenizer "$RUN/inputs/data/tokenizer.json" \
  --plan "$RUN/inputs/m2-fresh256/plan.json" \
  --labels "$RUN/inputs/m2-fresh256/labels.json" \
  --out "$RUN/control/corpus-panel-overlap.json"
jq -e --arg tokens "$TRAIN_TOKENS_SHA" --arg mask "$TRAIN_MASK_SHA" \
  --argjson documents "$TRAIN_DOCUMENTS" \
  --argjson replies "$TRAIN_ASSISTANT_REPLIES" '
  .pass == true and
  .tokens_sha256 == $tokens and
  .mask_sha256 == $mask and
  .tokenizer_sha256 ==
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89" and
  .plan.sha256 ==
    "0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715" and
  .labels.sha256 ==
    "3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be" and
  .counts.documents == $documents and .counts.assistant_replies == $replies and
  .counts.exact_panel_in_document == 0 and
  .counts.exact_accepted_assistant_reply == 0 and
  .counts.documents_with_panel_6gram == 0 and
  .counts.assistant_replies_with_panel_6gram == 0' \
  "$RUN/control/corpus-panel-overlap.json" >/dev/null ||
  die "corpus-panel overlap audit differs from the registered reading"

log "base controls"
BASE_ARTIFACT="$RUN/control/b64L-s1.uorlut"
BASE_SNAPSHOT="$SNAPSHOT_ROOT/base/b64L-s1.snapshot"
install -d "$(dirname "$BASE_SNAPSHOT")"
"$EXPORT_BIN" --checkpoint "$B64L" --run-config "$B64L_CONFIG" \
  --snapshot-out "$BASE_SNAPSHOT" --out "$BASE_ARTIFACT" \
  --raw-gates --raw-transition
check_file 9a370566cd9da163191c0196763c68ddb395927fcf61fdecc1736b04941a4bd9 \
  "$BASE_ARTIFACT"
rm -f "$BASE_SNAPSHOT" || die "could not remove the transient base snapshot"
run_panel 0 "$BASE_ARTIFACT" per-turn/v1 1 base-per-turn-teacher >/dev/null
run_panel 0 "$BASE_ARTIFACT" per-turn/v1 0 base-per-turn-generated >/dev/null
run_panel 0 "$BASE_ARTIFACT" one-user-turn/v1 1 base-one-user-teacher >/dev/null
run_panel 0 "$BASE_ARTIFACT" one-user-turn/v1 0 base-one-user-generated >/dev/null
"$GRADE_BIN" --artifact "$BASE_ARTIFACT" \
  --tokens "$RUN/inputs/oasst2-heldout/tokens.u16" \
  --mask "$RUN/inputs/oasst2-heldout/response_mask.u8" \
  --manifest "$RUN/inputs/oasst2-heldout/manifest.json" \
  --max-evaluation 512 --max-replies 1 --reply-tokens 1 \
  --out "$RUN/control/base-grade.json"
jq -e '.evaluation.samples == 512 and
  .evaluation.sample_fingerprint ==
    "b724656f29bdade21aecb372d0568c48c81d78c80f43fe4de8ec667a7824ccc4"' \
  "$RUN/control/base-grade.json" >/dev/null ||
  die "base continuation grade did not use the registered 512-row sample"
verify_grade_labels "$RUN/control/base-grade.json" base
bind_grade_hardware "$RUN/control/base-grade.json"
base_totals() {
  jq -e "$1" "$RUN/panel/step-000000-$2.json" >/dev/null ||
    die "base $2 control differs from its registered reading"
}
base_totals '.teacher_forced.totals |
  .eos_rank_min == 5 and .eos_rank_p50 == 7 and .eos_rank_max == 13 and
  .eos_top1 == 0 and .eos_top8 == 198' base-per-turn-teacher
base_totals '.teacher_forced.totals |
  .eos_rank_min == 18 and .eos_rank_p50 == 138 and .eos_rank_max == 873 and
  .eos_top1 == 0 and .eos_top8 == 0' base-one-user-teacher
base_totals '.totals |
  .complete == 0 and .eos_stops == 0 and .bos_stops == 0 and
  .context_stops == 0 and .cap_stops == 256' base-one-user-generated
base_totals '.totals |
  .complete == 0 and .eos_stops == 0 and .bos_stops == 0 and
  .context_stops == 0 and .cap_stops == 256' base-per-turn-generated
base_guard=$(jq -r '.evaluation.top1_correct' "$RUN/control/base-grade.json")
[ "$base_guard" -eq 132 ] ||
  die "base continuation guard is $base_guard, not the registered 132/512"

check_initialization() {
  local run=$1
  [ -f "$run/init.json" ] ||
    die "training wrote no checkpoint-initialization record"
  jq -e '.mode == "checkpoint" and .optimizer == "fresh" and .schedule == "fresh" and
    .run_config.sha256 ==
      "6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2" and
    .files["model.safetensors"].sha256 ==
      "6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de" and
    .files["config.json"].sha256 ==
      "1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2" and
    .files["hamming_read.safetensors"].sha256 ==
      "672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd"' \
    "$run/init.json" >/dev/null ||
    die "checkpoint-initialization record differs from the registered base"
}

train_and_score() {
  local micro=$1 label=$2 run_dir=$3
  local next=256 train_rc=0 checkpoint init_wait=0 init_approval
  ATTEMPT=$label
  TRAIN_RUN=$run_dir
  init_approval="$RUN/control/init-approved-$label"
  [ ! -e "$init_approval" ] ||
    die "initialization approval for $label already exists"
  install -d "$RUN/grades/$label" "$RUN/artifacts/$label" \
    "$RUN/scores/$label" "$RUN/panel/$label"
  UOR_TRAIN_INIT_APPROVAL="$init_approval" \
  "$TRAIN_BIN" \
    --data-root "$RUN/inputs/data" \
    --init-from "$B64L" \
    --train-split "$RUN/inputs/mix" \
    --train-sha256 "$TRAIN_TOKENS_SHA" \
    --train-mask-sha256 "$TRAIN_MASK_SHA" \
    --out "$run_dir" \
    --objective nearest --retain-rates 1 --decay-mult 1 \
    --lr 0.0002 --lr-end 0 --train-as-served 1 \
    --context 192 --width 256 --hamming-read 4 \
    --batch 64 --micro-batch "$micro" --steps "$STEPS" \
    --eval-every 256 --samples 16 --reply-tokens 64 --seed 1 \
    > "$RUN/logs/train-$label.log" 2>&1 &
  TRAIN_PID=$!

  while [ ! -f "$run_dir/.init-ready" ]; do
    if ! kill -0 "$TRAIN_PID" 2>/dev/null; then
      wait "$TRAIN_PID" || train_rc=$?
      TRAIN_PID=
      [ "$train_rc" -ne 0 ] ||
        die "trainer exited before publishing its initialization record"
      if [ -z "$STOP_REASON" ]; then
        TRAIN_FAILURE=$train_rc
        return 0
      fi
      die "trainer exited before initialization after a registered stop"
    fi
    [ "$init_wait" -lt 120 ] ||
      die "trainer did not publish initialization within 120 seconds"
    sleep 1
    init_wait=$((init_wait + 1))
  done
  check_initialization "$run_dir"
  touch "$init_approval" ||
    die "could not approve the validated initialization"

  while kill -0 "$TRAIN_PID" 2>/dev/null; do
    if [ "$next" -le "$STEPS" ]; then
      checkpoint="$run_dir/models/step-$(printf '%06d' "$next")"
      if [ -d "$checkpoint" ] && [ -f "$checkpoint/.checkpoint-ready" ]; then
        kill -STOP "$TRAIN_PID" 2>/dev/null || true
        score_checkpoint "$next" "$checkpoint"
        if [ -z "$STOP_REASON" ]; then
          kill -CONT "$TRAIN_PID" 2>/dev/null || true
        else
          kill -TERM "$TRAIN_PID" 2>/dev/null || true
          kill -CONT "$TRAIN_PID" 2>/dev/null || true
        fi
        next=$((next + 256))
      fi
    fi
    sleep 5
  done
  wait "$TRAIN_PID" || train_rc=$?
  TRAIN_PID=

  if [ "$train_rc" -ne 0 ] && [ -z "$STOP_REASON" ]; then
    TRAIN_FAILURE=$train_rc
    return 0
  fi
  while [ "$next" -le "$STEPS" ]; do
    checkpoint="$run_dir/models/step-$(printf '%06d' "$next")"
    if [ -d "$checkpoint" ]; then
      [ -f "$checkpoint/.checkpoint-ready" ] ||
        die "checkpoint $next has no completion marker"
      score_checkpoint "$next" "$checkpoint"
    elif [ -z "$STOP_REASON" ]; then
      die "checkpoint $next is missing without a registered stop"
    fi
    next=$((next + 256))
  done
  check_progress "$run_dir/progress.jsonl"
  select_best_checkpoint "$RUN/checkpoint-readings-$label.jsonl"
  if [ -f "$RUN/checkpoint-readings-$label.jsonl" ]; then
    cp "$RUN/checkpoint-readings-$label.jsonl" "$RUN/checkpoint-readings.jsonl" ||
      die "could not publish checkpoint readings from $label"
  fi
  return 0
}

log "training"
TRAIN_FAILURE=0
train_and_score 16 attempt-1 "$RUN/run"
if [ "$TRAIN_FAILURE" -ne 0 ]; then
  if grep -Eiq 'out of memory|CUDA_ERROR_OUT_OF_MEMORY|\bOOM\b' \
    "$RUN/logs/train-attempt-1.log"; then
    log "micro-batch 16 ran out of memory; retrying once at micro-batch 8"
    compact_failed_attempt attempt-1 "$TRAIN_RUN"
    STOP_REASON=
    TRAIN_FAILURE=0
    train_and_score 8 attempt-2 "$RUN/run-mb8"
    [ "$TRAIN_FAILURE" -eq 0 ] ||
      die "trainer failed after the registered micro-batch-8 retry"
  else
    die "trainer failed before a registered stop"
  fi
fi
if [ -n "$STOP_REASON" ]; then
  log "registered stop: $STOP_REASON"
else
  log "training completed all $STEPS steps"
fi
log "job complete"
