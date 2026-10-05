#!/usr/bin/env bash
# Phase 1 bf16 parity gate (#820): the frozen decision rule of
# docs/compute/bf16-parity-gate.md, executed on the 2 x RTX 4090 pod.
#
#   scripts/pod/bf16-parity-gate.sh STAGE
#
# STAGE:
#   build    build the geometric-stack and m-world binaries of this checkout,
#            record their SHA-256, and run the two focused test binaries
#   base     4 base runs (arms f32, bf16 x seeds 1, 2), 2 in parallel
#   ft       4 Arm C fine-tunes from those bases, at the base's precision
#   session  4 D19 grounded sessions (log_recall=off) of the fine-tunes
#   tabulate the frozen decision (scripts/pod/bf16-parity-gate.py)
#   all      build base ft session tabulate
#
# A sealed report root is never reused; an unsealed one stops the stage (a run
# capped by max_seconds is resumed from its checkpoint into the same root).
set -euo pipefail

STAGE=${1:?usage: bf16-parity-gate.sh build|base|ft|session|tabulate|all}
REPO=${REPO:-/root/bf16/src}
D=${D:-/root/data}
R=${R:-/root/runs/bf16-gate}
THREADS=${THREADS:-8}
CUDA_COMPUTE_CAP=${CUDA_COMPUTE_CAP:-89}
CUDA_DIR=${CUDA_DIR:-/usr/local/cuda-12.8}
export PATH=$CUDA_DIR/bin:$HOME/.cargo/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_DIR/lib64 CUDA_HOME=$CUDA_DIR
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/root/target-bf16}
T=$D/tokenizer.json
GS=$CARGO_TARGET_DIR/release/examples/geometric-stack
MWORLD=$CARGO_TARGET_DIR/release/examples/m-world
ARMS=(f32 bf16)
SEEDS=(1 2)
BASE_STEPS=70609
FT_STEPS=4000

log() { echo "$(date -u +%FT%TZ) $*" | tee -a "$R/gate.log"; }
fail() { log "FAILED $1 (see $1.log)"; exit 1; }

fresh() {  # fresh ROOT -> 0 when the work must run
  if [ -f "$1/manifest.json" ]; then log "skip $1 (sealed)"; return 1; fi
  if [ -e "$1" ]; then
    echo "$1 exists but is not sealed: resume it with resume=$1/checkpoint or move it aside" >&2
    exit 1
  fi
  return 0
}

completed() {  # completed ROOT STEPS
  python3 - "$1/report.json" "$2" <<'PY'
import json, sys
try:
    r = json.load(open(sys.argv[1]))
except Exception:
    sys.exit(1)
sys.exit(0 if r.get("completed_steps") == int(sys.argv[2]) and not r.get("stopped_early") else 1)
PY
}

precision_arg() {  # precision_arg ARM
  [ "$1" = bf16 ] && echo "precision=bf16" || echo ""
}

stage_build() {
  mkdir -p "$R"
  ( cd "$REPO" && cargo build --release -p uor-r4-training --features cuda \
      --example geometric-stack --example m-world )
  sha256sum "$GS" "$MWORLD" | tee "$R/binaries.sha256"
  ( cd "$REPO" && cargo test --release -p uor-r4-training --features cuda \
      --test cuda_stack_ops_parity 2>&1 | tail -n 3 )
  if [ -f "$REPO/crates/uor-r4-training/tests/cuda_stack_ops_bf16_parity.rs" ]; then
    ( cd "$REPO" && cargo test --release -p uor-r4-training --features cuda \
        --test cuda_stack_ops_bf16_parity 2>&1 | tail -n 3 )
  else
    log "bf16 parity test not in this checkout yet"
  fi
  log "build done"
}

base_run() {  # base_run ARM SEED
  local arm=$1 seed=$2
  local out=$R/base-$arm-s$seed
  fresh "$out" || return 0
  log "base $arm seed $seed on GPU $((seed - 1))"
  CUDA_VISIBLE_DEVICES=$((seed - 1)) RAYON_NUM_THREADS=$THREADS "$GS" train \
    out="$out" seed="$seed" lr=0.0005 $(precision_arg "$arm") \
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

# The two seeds of an arm run on the two GPUs at once; the arms run in sequence
# so like-for-like share the pod's clocks.
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

ft_run() {  # ft_run ARM SEED
  local arm=$1 seed=$2
  local base=$R/base-$arm-s$seed
  local out=$R/ft-$arm-s$seed
  completed "$base" $BASE_STEPS || {
    echo "base-$arm-s$seed has not completed $BASE_STEPS steps: run the base stage first" >&2
    exit 1
  }
  fresh "$out" || return 0
  log "ft $arm seed $seed on GPU $((seed - 1))"
  CUDA_VISIBLE_DEVICES=$((seed - 1)) RAYON_NUM_THREADS=$THREADS "$GS" dialogue-train \
    out="$out" tokenizer="$T" $(precision_arg "$arm") \
    train_tokens="$D/ft/mixed-c/tokens.u16" train_mask="$D/ft/mixed-c/response_mask.u8" \
    train_manifest="$D/ft/mixed-c/manifest.json" \
    dev_tokens="$D/ft/dev/tokens.u16" dev_mask="$D/ft/dev/response_mask.u8" \
    dev_manifest="$D/ft/dev/manifest.json" \
    init="$base/model" \
    pointer=32 protocol=2 context=384 policy=full_prefix data_seed="$seed" \
    steps=$FT_STEPS batch=16 lr=0.0003 warmup=100 min_lr=0.1 weight_decay=0.1 clip=1.0 \
    eval_every=500 checkpoint_every=4000 dev_seed=20260930 dev_per_source=32 \
    max_seconds=10800 device=cuda tf32=true \
    > "$out.log" 2>&1 || fail "$out"
}

stage_ft() {
  for arm in "${ARMS[@]}"; do
    ft_run "$arm" 1 & local p1=$!
    ft_run "$arm" 2 & local p2=$!
    wait $p1 || exit 1
    wait $p2 || exit 1
    log "ft $arm done"
  done
  log "ft stage done"
}

session_run() {  # session_run ARM SEED
  local arm=$1 seed=$2
  local ft=$R/ft-$arm-s$seed
  local out=$R/session-off-$arm-s$seed
  completed "$ft" $FT_STEPS || {
    echo "ft-$arm-s$seed has not completed $FT_STEPS steps: run the ft stage first" >&2
    exit 1
  }
  fresh "$out" || return 0
  log "session off $arm seed $seed"
  RAYON_NUM_THREADS=$THREADS "$MWORLD" session \
    out="$out" world=v2 model_root="$ft" tokenizer="$T" \
    compiler="$D/sieve/compiler-save-op-v25-rawtable/compiler.json" \
    trunk="$D/sieve/op-model-v25/model" op_policy=unless_query \
    log_recall=off max_new_tokens=64 arms=default reload=0 \
    > "$out.log" 2>&1 || fail "$out"
}

# The four sessions are CPU-only and independent: all four at once.
stage_session() {
  local pids=()
  for arm in "${ARMS[@]}"; do
    for seed in "${SEEDS[@]}"; do
      session_run "$arm" "$seed" & pids+=($!)
    done
  done
  for pid in "${pids[@]}"; do wait "$pid" || exit 1; done
  log "session stage done"
}

stage_tabulate() {
  python3 "$REPO/scripts/pod/bf16-parity-gate.py" "$R" | tee "$R/decision.txt"
  log "tabulate done"
}

mkdir -p "$R"
case $STAGE in
  build|base|ft|session|tabulate) "stage_$STAGE" ;;
  all) for s in build base ft session tabulate; do "stage_$s"; done ;;
  *) echo "unknown stage $STAGE" >&2; exit 2 ;;
esac
