#!/usr/bin/env bash
# Run the reviewed reply-masked SFT unit on a bootstrapped UOR-R4 Vast pod.
#
# Usage:
#   UOR_SFT_PIN=<40-hex-merge-pin> UOR_SFT_RUN=/workspace/uor-r4/runs/sft-s1 \
#     bash /workspace/one-model-reply-sft-job.sh 1
#
# The caller owns the pod, GPU lease and teardown. This script fetches every input,
# runs the base controls, trains once with checkpoints every 256 steps, exports and
# scores each checkpoint, applies the registered 512-step stop, then seals and reads
# the evidence back through uor-store.
set -euo pipefail

SEED=${1:?usage: job.sh SEED}
case "$SEED" in
  1) ;;
  2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;
  *) echo "SEED must be 1; seed 2 is a later decision" >&2; exit 2 ;;
esac
[ "$#" -eq 1 ] || { echo "usage: job.sh 1" >&2; exit 2; }

PIN=${UOR_SFT_PIN:?UOR_SFT_PIN must name the reviewed 40-hex merge commit}
case "$PIN" in
  *[!0-9a-f]*|'') echo "UOR_SFT_PIN is not lowercase 40-hex" >&2; exit 2 ;;
esac
[ "${#PIN}" -eq 40 ] || { echo "UOR_SFT_PIN is not 40 hex characters" >&2; exit 2; }

RUN_ID=${UOR_SFT_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')}
RUN=${UOR_SFT_RUN:-/workspace/uor-r4/runs/deepseek-sft-s$SEED}
OUT_STORE=${UOR_SFT_OUT_STORE:-results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1}
STORE=/root/uor-store
REPO=${UOR_REPO:-/root/uor-sft-src-$PIN}
TARGET=${CARGO_TARGET_DIR:-/workspace/uor-r4/target-deepseek-sft}
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

store_listing=$("$STORE" ls "$OUT_STORE" 2>/dev/null || true)
[ -z "$store_listing" ] || die "refusing a nonempty store output: $OUT_STORE"

install -d "$RUN/inputs" "$RUN/control" "$RUN/checkpoints" "$RUN/artifacts" \
  "$RUN/grades" "$RUN/panel" "$RUN/download" "$RUN/logs"
exec > >(tee -a "$RUN/job.log") 2>&1

TRAIN_PID=
STOP_REASON=
on_exit() {
  local rc=$?
  # Detach the seal from RUN/job.log, whose bytes are about to be hashed.
  exec >> "$RUN.seal.log" 2>&1
  if [ -n "$TRAIN_PID" ] && kill -0 "$TRAIN_PID" 2>/dev/null; then
    kill -TERM "$TRAIN_PID" 2>/dev/null || true
    kill -CONT "$TRAIN_PID" 2>/dev/null || true
    wait "$TRAIN_PID" 2>/dev/null || true
  fi
  printf '%s\n' "$rc" > "$RUN/EXIT"
  printf '%s\n' "${STOP_REASON:-natural-or-failure}" > "$RUN/STOP_REASON"
  (
    cd "$RUN"
    find . -type f ! -name 'SHA256SUMS*' -print0 |
      sort -z | xargs -0 sha256sum > SHA256SUMS.next
    mv SHA256SUMS.next SHA256SUMS
    sha256sum -c SHA256SUMS
  ) || rc=1
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

fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/tokens.u16" \
  "$RUN/inputs/mix/tokens.u16"
fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/response_mask.u8" \
  "$RUN/inputs/mix/response_mask.u8"
fetch_file "results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/manifest.json" \
  "$RUN/inputs/mix/manifest.json"
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

check_file 3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  "$RUN/inputs/mix/tokens.u16"
check_file 67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \
  "$RUN/inputs/mix/response_mask.u8"
check_file 1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe \
  "$RUN/inputs/mix/manifest.json"
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
tar -xzf "$RUN/inputs/d3-accum-s1.tar.gz" -C "$RUN/inputs/base"
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
  CARGO_TARGET_DIR="$RUN/panel-target" \
    cargo build --release --locked --features cpu-parity --bin served-reply-panel
)

TRAIN_BIN="$TARGET/release/train-in-space"
GRADE_BIN="$TARGET/release/cpu-checkpoint-grade"
EXPORT_BIN="$TARGET/release/cpu-fbs-export"
PANEL_BIN="$RUN/panel-target/release/served-reply-panel"
for binary in "$TRAIN_BIN" "$GRADE_BIN" "$EXPORT_BIN" "$PANEL_BIN"; do
  [ -x "$binary" ] || die "build did not produce $binary"
done
sha256sum "$TRAIN_BIN" "$GRADE_BIN" "$EXPORT_BIN" "$PANEL_BIN" > "$RUN/binaries.sha256"

grade_checkpoint() {
  local step=$1 checkpoint=$2 grade_dir artifact_dir
  grade_dir="$RUN/grades/step-$(printf '%06d' "$step")"
  artifact_dir="$RUN/artifacts/step-$(printf '%06d' "$step")"
  install -d "$artifact_dir"
  "$GRADE_BIN" \
    --checkpoint "$checkpoint" \
    --run-config "$RUN/run/config.json" \
    --snapshot-out "$artifact_dir/model.snapshot" \
    --artifact-out "$artifact_dir/model.uorlut" \
    --tokens "$RUN/inputs/oasst2-heldout/tokens.u16" \
    --mask "$RUN/inputs/oasst2-heldout/response_mask.u8" \
    --manifest "$RUN/inputs/oasst2-heldout/manifest.json" \
    --max-evaluation 512 --max-replies 1 --reply-tokens 1 \
    --out "$grade_dir.json"
  jq -e '.evaluation.top1_correct >= 0' "$grade_dir.json" >/dev/null ||
    die "checkpoint $step grade has no top-1 count"
}

run_panel() {
  local step=$1 artifact=$2 rendering=$3 teacher=$4 label=$5
  local out
  out="$RUN/panel/step-$(printf '%06d' "$step")-$label.json"
  "$PANEL_BIN" \
    --artifact "$artifact" \
    --tokenizer "$RUN/inputs/data/tokenizer.json" \
    --inputs "$RUN/inputs/m2-fresh256/inputs.json" \
    --labels "$RUN/inputs/m2-fresh256/labels.json" \
    --plan "$RUN/inputs/m2-fresh256/plan.json" \
    --manifest "$RUN/inputs/m2-fresh256/manifest.json" \
    --rendering "$rendering" --teacher-forced "$teacher" \
    --reply-tokens 64 --out "$out"
  printf '%s\n' "$out"
}

score_checkpoint() {
  local step=$1 checkpoint=$2
  local marker artifact
  marker="$RUN/checkpoints/step-$(printf '%06d' "$step").scored"
  [ ! -e "$marker" ] || return 0
  [ -d "$checkpoint" ] || die "checkpoint $step is missing"
  cp -a "$checkpoint" "$RUN/checkpoints/step-$(printf '%06d' "$step")"
  grade_checkpoint "$step" "$checkpoint"
  artifact="$RUN/artifacts/step-$(printf '%06d' "$step")/model.uorlut"
  run_panel "$step" "$artifact" per-turn/v1 1 per-turn-teacher >/dev/null
  run_panel "$step" "$artifact" per-turn/v1 0 per-turn-generated >/dev/null
  run_panel "$step" "$artifact" one-user-turn/v1 0 one-user-generated >/dev/null
  touch "$marker"

  local guard eos_top1 eos_p50
  guard=$(jq -r '.evaluation.top1_correct' "$RUN/grades/step-$(printf '%06d' "$step").json")
  eos_top1=$(jq -r '.teacher_forced.totals.eos_top1' \
    "$RUN/panel/step-$(printf '%06d' "$step")-per-turn-teacher.json")
  eos_p50=$(jq -r '.teacher_forced.totals.eos_rank_p50' \
    "$RUN/panel/step-$(printf '%06d' "$step")-per-turn-teacher.json")
  jq -n --argjson step "$step" --argjson guard "$guard" \
    --argjson eos_top1 "$eos_top1" --argjson eos_p50 "$eos_p50" \
    '{step:$step, guard_top1:$guard, eos_top1:$eos_top1, eos_rank_p50:$eos_p50}' \
    >> "$RUN/checkpoint-readings.jsonl"
  if [ "$guard" -lt 93 ]; then
    STOP_REASON="continuation-failure-step-$step"
    kill "$TRAIN_PID" 2>/dev/null || true
  elif [ "$step" -eq 512 ] && [ "$eos_top1" -eq 0 ] && [ "$eos_p50" -ge 7 ]; then
    STOP_REASON="no-ranking-movement-step-512"
    kill "$TRAIN_PID" 2>/dev/null || true
  fi
}

log "base controls"
BASE_ARTIFACT="$RUN/control/b64L-s1.uorlut"
"$EXPORT_BIN" --checkpoint "$B64L" --run-config "$B64L_CONFIG" \
  --snapshot-out "$RUN/control/b64L-s1.snapshot" --out "$BASE_ARTIFACT"
run_panel 0 "$BASE_ARTIFACT" per-turn/v1 1 base-per-turn-teacher >/dev/null
run_panel 0 "$BASE_ARTIFACT" per-turn/v1 0 base-per-turn-generated >/dev/null
run_panel 0 "$BASE_ARTIFACT" one-user-turn/v1 0 base-one-user-generated >/dev/null
"$GRADE_BIN" --artifact "$BASE_ARTIFACT" \
  --tokens "$RUN/inputs/oasst2-heldout/tokens.u16" \
  --mask "$RUN/inputs/oasst2-heldout/response_mask.u8" \
  --manifest "$RUN/inputs/oasst2-heldout/manifest.json" \
  --max-evaluation 512 --max-replies 1 --reply-tokens 1 \
  --out "$RUN/control/base-grade.json"
base_guard=$(jq -r '.evaluation.top1_correct' "$RUN/control/base-grade.json")
[ "$base_guard" -ge 93 ] || die "base continuation guard is $base_guard, below 93"

log "training"
"$TRAIN_BIN" \
  --data-root "$RUN/inputs/data" \
  --init-from "$B64L" \
  --train-split "$RUN/inputs/mix" \
  --train-sha256 3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  --train-mask-sha256 67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \
  --out "$RUN/run" \
  --objective nearest --retain-rates 1 --decay-mult 1 \
  --lr 0.0002 --lr-end 0 --train-as-served 1 \
  --context 192 --width 256 --hamming-read 4 \
  --batch 64 --micro-batch 16 --steps 1024 \
  --eval-every 256 --samples 16 --reply-tokens 64 --seed 1 \
  > "$RUN/logs/train.log" 2>&1 &
TRAIN_PID=$!

next=256
while kill -0 "$TRAIN_PID" 2>/dev/null; do
  if [ "$next" -le 1024 ]; then
    checkpoint="$RUN/run/models/step-$(printf '%06d' "$next")"
    if [ -d "$checkpoint" ]; then
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
train_rc=0
wait "$TRAIN_PID" || train_rc=$?
TRAIN_PID=

while [ "$next" -le 1024 ]; do
  checkpoint="$RUN/run/models/step-$(printf '%06d' "$next")"
  [ ! -d "$checkpoint" ] || score_checkpoint "$next" "$checkpoint"
  next=$((next + 256))
done

if [ "$train_rc" -ne 0 ] && [ -z "$STOP_REASON" ]; then
  die "trainer exited $train_rc without a registered stop"
fi
if [ -n "$STOP_REASON" ]; then
  log "registered stop: $STOP_REASON"
else
  log "training completed all 1,024 steps"
fi
[ -s "$RUN/run/progress.jsonl" ] || die "training wrote no progress"
jq -e 'select(.train_loss_mean == null or (.train_loss_mean | isfinite))' \
  "$RUN/run/progress.jsonl" >/dev/null || die "progress has a non-finite loss"
log "job complete"
