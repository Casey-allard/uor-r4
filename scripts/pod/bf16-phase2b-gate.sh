#!/usr/bin/env bash
# bf16 Phase 2b gate — the fused-read rewrite (flash-style read).
#
# Frozen rule: docs/compute/bf16-phase2b-gate.md. This script implements it; it
# does not define it. Do not change the recipe, the arms or the acceptance rule
# after the first gate run starts — a later change is a new gate.
#
#   scripts/pod/bf16-phase2b-gate.sh build|base|ab|tabulate|all
#
# Arms (both bf16, evaluation stays f32 in both):
#   current — the stored-read path that ships today
#   flash   — the same binary with UOR_R4_CUDA_READ=flash
#
# The gate needs the shared corpora under $D (EUR-NO-1 canonical volume).
set -euo pipefail

STAGE=${1:?usage: bf16-phase2b-gate.sh build|base|ab|tabulate|all}
REPO=${REPO:-/root/ds-uor-r4}
D=${D:-/root/data}
R=${R:-/root/runs/phase2b-gate}
THREADS=${THREADS:-8}
CUDA_COMPUTE_CAP=${CUDA_COMPUTE_CAP:-120}
CUDA_DIR=${CUDA_DIR:-/usr/local/cuda-12.8}
export CUDA_COMPUTE_CAP
export LD_LIBRARY_PATH=$CUDA_DIR/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/root/target-deepseek}

# One binary for all four runs; the arms differ only in the read switch.
GS=$CARGO_TARGET_DIR/release/examples/geometric-stack
ARMS=(current flash)
SEEDS=(1 2)
BASE_STEPS=70609
SPEED_STEPS=300
SPEED_ROUNDS=3
SPEED_FLOOR_PCT=5

T=$D/tokenizer.json

log()  { echo "[$(date -u +%H:%M:%S)] $*" >&2; }
fail() { echo "FAILED: $1 (see $1.log)" >&2; exit 1; }

# Refuse to overwrite a sealed root: every retry gets a new root (report_output rule).
fresh() {  # fresh OUT -> 0 if it may be created
  if [ -e "$1" ]; then log "skip: $1 exists (never reuse a report root)"; return 1; fi
  return 0
}

completed() {  # completed OUT STEPS -> 0 if the root finished the full run
  local r=$1/report.json
  [ -f "$r" ] || return 1
  python3 - "$r" "$2" <<'PY'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(1)
ok = d.get("completed_steps") == int(sys.argv[2]) and not d.get("stopped_early", False)
sys.exit(0 if ok else 1)
PY
}

read_switch() {  # read_switch ARM -> env assignment for the flash arm (empty otherwise)
  [ "$1" = flash ] && echo "UOR_R4_CUDA_READ=flash" || echo ""
}

stage_build() {
  mkdir -p "$R"
  ( cd "$REPO" && cargo build --release -p uor-r4-training --features cuda \
      --example geometric-stack --example m-world )
  sha256sum "$GS" | tee "$R/binaries.sha256"
  # Frozen op parity: test_flash_read_parity over Dot/Lorentz/L2 x NoRead x age,
  # bound 1e-4 + 1e-3*|ref|, bit-identical at tile 1, determinism, existing 32 green.
  # Single-threaded: the read selector is process-global.
  ( cd "$REPO" && UOR_REQUIRE_CUDA=1 cargo test --release -p uor-r4-training \
      --features cuda --test cuda_stack_ops_parity -- --test-threads=1 2>&1 | tail -n 5 )
  log "build + op parity done"
}

base_run() {  # base_run ARM SEED
  local arm=$1 seed=$2
  local out=$R/base-$arm-s$seed
  fresh "$out" || return 0
  log "base $arm seed $seed on GPU $((seed - 1))"
  CUDA_VISIBLE_DEVICES=$((seed - 1)) RAYON_NUM_THREADS=$THREADS \
    env $(read_switch "$arm") "$GS" train \
    out="$out" seed="$seed" lr=0.0005 precision=bf16 \
    train="$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16" \
    train_weights=0.6,0.15,0.25 \
    valid="$D/corpora/ts-valid/tokens.u16" tokenizer="$T" \
    arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 \
    read=l2 rotation=true key_shift=false \
    steps=$BASE_STEPS batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
    eval_every=5000 eval_windows=64 final_windows=512 \
    checkpoint_every=10000 max_seconds=21600 \
    device=cuda tf32=true data_parallel=1 \
    > "$out.log" 2>&1 || fail "$out"
}

# The two seeds of an arm run on the two GPUs at once; the arms run in sequence so
# like-for-like share the pod's clocks. Four 70,609-step runs: ~2 h on 2x5090.
stage_base() {
  for arm in "${ARMS[@]}"; do
    base_run "$arm" 1 & local p1=$!
    base_run "$arm" 2 & local p2=$!
    wait $p1 || exit 1
    wait $p2 || exit 1
    log "base $arm done"
  done
  log "base stage done"
}

# Alternating 300-step A/B on one GPU, 3 rounds, for the >=5% end-to-end floor.
stage_ab() {
  local out=$R/ab
  mkdir -p "$out"
  for round in $(seq 1 "$SPEED_ROUNDS"); do
    for arm in "${ARMS[@]}"; do
      local root=$out/$arm-r$round
      log "ab round $round $arm"
      CUDA_VISIBLE_DEVICES=0 RAYON_NUM_THREADS=$THREADS \
        env $(read_switch "$arm") "$GS" train \
        out="$root" seed=7 lr=0.0005 precision=bf16 \
        train="$D/corpora/ts-train/tokens.u16,$D/corpora/td-train/tokens.u16,$D/chat-v0-p2/train/tokens.u16" \
        train_weights=0.6,0.15,0.25 \
        valid="$D/corpora/ts-valid/tokens.u16" tokenizer="$T" \
        arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 \
        read=l2 rotation=true key_shift=false \
        steps=$SPEED_STEPS batch=16 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1.0 \
        eval_every=$SPEED_STEPS eval_windows=8 final_windows=8 \
        checkpoint_every=0 sample_tokens=0 \
        device=cuda tf32=true data_parallel=1 \
        > "$root.log" 2>&1 || fail "$root"
    done
  done
  log "ab stage done"
}

stage_tabulate() {
  python3 "$(dirname "$0")/bf16-phase2b-gate.py" "$R" | tee "$R/GATE.txt"
}

case $STAGE in
  build)    stage_build ;;
  base)     stage_base ;;
  ab)       stage_ab ;;
  tabulate) stage_tabulate ;;
  all)      stage_build; stage_base; stage_ab; stage_tabulate ;;
  *)        echo "unknown stage '$STAGE'" >&2; exit 2 ;;
esac
