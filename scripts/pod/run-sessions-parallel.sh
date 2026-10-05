#!/usr/bin/env bash
# D19 grounded sessions (`m-world session`, world v2) for several models at once
# (#820): for each MODEL_ROOT (a dialogue-train report root holding report.json
# and model/), one sieve-on and one sieve-off session, run concurrently under a
# fixed CPU-thread budget and, with DEVICE=cuda, spread over the listed GPUs.
#
#   scripts/pod/run-sessions-parallel.sh MODEL_ROOT [MODEL_ROOT...]
#
# Each session claims its own new report root, which m-world seals and verifies
# at the end (report_output::claim/seal/verify). A sealed root is skipped; an
# existing unsealed one stops the script before anything starts (inspect it,
# then use a NEW OUT). A finished root without manifest.json is a failure.
#
# Environment (defaults in brackets):
#   MWORLD     m-world binary; built with --features cuda for DEVICE=cuda
#              [/root/target-evalgpu/release/examples/m-world]
#   T          tokenizer.json [/root/data/tokenizer.json]
#   COMPILER   saved compiler [/root/data/sieve/compiler-save-op-v25-rawtable/compiler.json]
#   TRUNK      op-model trunk [/root/data/sieve/op-model-v25/model]; empty: no trunk=
#   OP_POLICY  [unless_query] (needs TRUNK)
#   OUT        directory for the new roots [/root/runs/sessions]
#   TAG        suffix of each root name [the device]: OUT/session-RECALL-NAME-TAG
#   RECALLS    [sieve off]
#   DEVICE     cpu|cuda|metal [cpu]
#   GPUS       CUDA ordinals to round-robin when DEVICE=cuda [0]
#   THREADS    total CPU threads for all sessions together [8]
#   JOBS       sessions at once [number of sessions, at most THREADS]
#   CONVERSATIONS [300]  SEED [9101]  MAX_NEW_TOKENS [64]  ARMS [default]  RELOAD [0]
#   EXTRA      further key=value arguments for every session []
#
# Each job gets max(1, THREADS / JOBS) threads (RAYON_NUM_THREADS, and the
# BLAS/OpenMP variables). Logs go to ROOT.log; a summary line per root to
# OUT/sessions-parallel.log. The exit status is non-zero if any session failed.
set -euo pipefail

if [ $# -lt 1 ]; then
  sed -n '2,32p' "$0" >&2
  exit 2
fi

MWORLD=${MWORLD:-/root/target-evalgpu/release/examples/m-world}
T=${T:-/root/data/tokenizer.json}
COMPILER=${COMPILER:-/root/data/sieve/compiler-save-op-v25-rawtable/compiler.json}
TRUNK=${TRUNK-/root/data/sieve/op-model-v25/model}
OP_POLICY=${OP_POLICY:-unless_query}
OUT=${OUT:-/root/runs/sessions}
DEVICE=${DEVICE:-cpu}
TAG=${TAG:-$DEVICE}
read -r -a RECALL_LIST <<< "${RECALLS:-sieve off}"
read -r -a GPU_LIST <<< "$(echo "${GPUS:-0}" | tr ',' ' ')"
THREADS=${THREADS:-8}
CONVERSATIONS=${CONVERSATIONS:-300}
SEED=${SEED:-9101}
MAX_NEW_TOKENS=${MAX_NEW_TOKENS:-64}
ARMS=${ARMS:-default}
RELOAD=${RELOAD:-0}
read -r -a EXTRA_ARGS <<< "${EXTRA:-}"

case $DEVICE in cpu|cuda|metal) ;; *) echo "DEVICE must be cpu, cuda or metal" >&2; exit 2 ;; esac
[ -x "$MWORLD" ] || { echo "no m-world binary at $MWORLD" >&2; exit 2; }
for file in "$T" "$COMPILER"; do [ -f "$file" ] || { echo "missing $file" >&2; exit 2; }; done
TRUNK_ARGS=()
if [ -n "$TRUNK" ]; then
  [ -d "$TRUNK" ] || { echo "missing trunk $TRUNK" >&2; exit 2; }
  TRUNK_ARGS=("trunk=$TRUNK" "op_policy=$OP_POLICY")
fi
mkdir -p "$OUT"
log() { echo "$(date -u +%FT%TZ) $*" | tee -a "$OUT/sessions-parallel.log"; }

# The planned sessions: "MODEL_ROOT RECALL ROOT" per line; sealed roots are skipped.
PLAN=()
for model_root in "$@"; do
  [ -f "$model_root/report.json" ] && [ -d "$model_root/model" ] ||
    { echo "$model_root is not a dialogue-train root (report.json and model/)" >&2; exit 2; }
  name=$(basename "$model_root")
  for recall in "${RECALL_LIST[@]}"; do
    root=$OUT/session-$recall-$name-$TAG
    if [ -f "$root/manifest.json" ]; then log "skip $root (sealed)"; continue; fi
    if [ -e "$root" ]; then
      echo "$root exists but is not sealed: inspect it, then use a new OUT or TAG" >&2
      exit 1
    fi
    PLAN+=("$model_root $recall $root")
  done
done
if [ ${#PLAN[@]} -eq 0 ]; then log "nothing to run"; exit 0; fi

JOBS=${JOBS:-${#PLAN[@]}}
[ "$JOBS" -gt "$THREADS" ] && JOBS=$THREADS
[ "$JOBS" -lt 1 ] && JOBS=1
PER_JOB=$(( THREADS / JOBS ))
[ "$PER_JOB" -lt 1 ] && PER_JOB=1
log "plan ${#PLAN[@]} sessions, $JOBS at once, $PER_JOB threads each, device=$DEVICE gpus=${GPU_LIST[*]} conversations=$CONVERSATIONS"

run_one() {  # run_one MODEL_ROOT RECALL ROOT GPU
  local model_root=$1 recall=$2 root=$3 gpu=$4 started status
  started=$(date +%s)
  set +e
  CUDA_VISIBLE_DEVICES=$gpu RAYON_NUM_THREADS=$PER_JOB OMP_NUM_THREADS=$PER_JOB \
    OPENBLAS_NUM_THREADS=$PER_JOB MKL_NUM_THREADS=$PER_JOB VECLIB_MAXIMUM_THREADS=$PER_JOB \
    "$MWORLD" session out="$root" world=v2 model_root="$model_root" tokenizer="$T" \
    compiler="$COMPILER" ${TRUNK_ARGS[@]+"${TRUNK_ARGS[@]}"} log_recall="$recall" \
    conversations="$CONVERSATIONS" seed="$SEED" max_new_tokens="$MAX_NEW_TOKENS" \
    arms="$ARMS" reload="$RELOAD" device="$DEVICE" ${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"} \
    > "$root.log" 2>&1
  status=$?
  set -e
  local seconds=$(( $(date +%s) - started ))
  if [ $status -ne 0 ] || [ ! -f "$root/manifest.json" ]; then
    log "FAILED $root (exit $status, ${seconds}s; see $root.log)"
    return 1
  fi
  log "sealed $root (${seconds}s, gpu ${gpu}) $(grep -m1 '^arm default' "$root.log" || true)"
}

failures=0
index=0
running=0
for entry in "${PLAN[@]}"; do
  read -r model_root recall root <<< "$entry"
  gpu=${GPU_LIST[$(( index % ${#GPU_LIST[@]} ))]}
  if [ "$DEVICE" != cuda ]; then gpu=""; fi
  index=$(( index + 1 ))
  log "start $root (model $model_root, recall $recall${gpu:+, gpu $gpu})"
  run_one "$model_root" "$recall" "$root" "$gpu" &
  running=$(( running + 1 ))
  if [ "$running" -ge "$JOBS" ]; then
    wait -n || failures=$(( failures + 1 ))
    running=$(( running - 1 ))
  fi
done
while [ "$running" -gt 0 ]; do
  wait -n || failures=$(( failures + 1 ))
  running=$(( running - 1 ))
done
log "done: ${#PLAN[@]} sessions, $failures failed"
[ "$failures" -eq 0 ]
