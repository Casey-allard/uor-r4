#!/usr/bin/env bash
# Step 2 deployment-parity MQAR bench: the full grid and its decision table
# (References #820; docs/research/barrier-assessment-2026-10-05/
# final-assessment.md Step 2 and completeness-critique.md points 2, 5, 6).
#
# Runs `mqar-bench layout=fact` for patterns rararr, rrarra (production) and
# aaaaaa (diagnostic) x arms none/f2/qk/conv (3 seeds) and identity/so4/
# wprev/qk_jj (2 seeds): 60 runs, K at a time, round-robin over the GPUs.
# Every run writes its own exclusively claimed, sealed report root under
# $OUT_ROOT/runs; a rerun of this script skips every (pattern, arm, seed)
# that already has a complete sealed root and gives a failed one a new
# attempt root (a root is never reused). Then `mode=decide` writes a new
# sealed decision root $OUT_ROOT/decision-<UTC stamp> with decision.json and
# decision.md.
#
# Required:  TOKENIZER=/path/to/#1017/tokenizer.json  OUT_ROOT=/new/or/resumed/dir
# Optional:  K=8 GPUS="0 1" DEVICE=cuda STEPS=4000 BATCH=16 LR=0.001
#            WARMUP=200 EVAL_EVERY=250 FINAL_SEQUENCES=256 MAX_SECONDS=3600
#            THREADS=2 CUDA_COMPUTE_CAP=89 SKIP_BUILD=0 REPO=<this checkout>
#            BIN=<prebuilt mqar-bench> (implies SKIP_BUILD=1)
#            PATTERNS="rararr rrarra aaaaaa"
#            ARMS="none:3 f2:3 qk:3 conv:3 identity:2 so4:2 wprev:2 qk_jj:2" (arm:seeds)
# A calibration run of one job: PATTERNS=rararr ARMS=none:1 K=1 (its root is
# then reused by the full grid, which skips complete roots).
set -euo pipefail

: "${TOKENIZER:?set TOKENIZER to the #1017 tokenizer.json}"
: "${OUT_ROOT:?set OUT_ROOT}"
REPO=${REPO:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}
K=${K:-8}
GPUS=${GPUS:-"0 1"}
DEVICE=${DEVICE:-cuda}
STEPS=${STEPS:-4000}
BATCH=${BATCH:-16}
LR=${LR:-0.001}
WARMUP=${WARMUP:-200}
EVAL_EVERY=${EVAL_EVERY:-250}
FINAL_SEQUENCES=${FINAL_SEQUENCES:-256}
MAX_SECONDS=${MAX_SECONDS:-3600}
THREADS=${THREADS:-2}
TOKENIZER_SHA256=d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89

sha256() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

got=$(sha256 "$TOKENIZER")
if [ "$got" != "$TOKENIZER_SHA256" ]; then
  echo "tokenizer sha256 $got is not the #1017 tokenizer ($TOKENIZER_SHA256)" >&2
  exit 1
fi

if [ -n "${BIN:-}" ]; then SKIP_BUILD=1; fi
BIN=${BIN:-"$REPO/target/release/examples/mqar-bench"}
if [ "${SKIP_BUILD:-0}" != 1 ]; then
  if [ "$DEVICE" = cuda ]; then
    export PATH=/usr/local/cuda/bin:$PATH
    (cd "$REPO" && CARGO_INCREMENTAL=0 CUDA_COMPUTE_CAP=${CUDA_COMPUTE_CAP:-89} \
      cargo build --release -p uor-r4-training --features cuda --example mqar-bench)
  else
    (cd "$REPO" && CARGO_INCREMENTAL=0 cargo build --release -p uor-r4-training --example mqar-bench)
  fi
fi

mkdir -p "$OUT_ROOT/runs" "$OUT_ROOT/logs"
{
  echo "utc_start $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "source $(git -C "$REPO" rev-parse HEAD) dirty_paths $(git -C "$REPO" status --porcelain | wc -l)"
  echo "binary_sha256 $(sha256 "$BIN")"
  echo "tokenizer_sha256 $got"
  echo "rustc $(rustc -V 2>/dev/null || echo UNAVAILABLE)"
  command -v nvidia-smi >/dev/null && nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
  echo "settings PATTERNS=${PATTERNS:-default} ARMS=${ARMS:-default} K=$K GPUS=$GPUS DEVICE=$DEVICE STEPS=$STEPS BATCH=$BATCH LR=$LR WARMUP=$WARMUP EVAL_EVERY=$EVAL_EVERY FINAL_SEQUENCES=$FINAL_SEQUENCES MAX_SECONDS=$MAX_SECONDS THREADS=$THREADS"
} >> "$OUT_ROOT/pod-manifest.txt"

complete() { [ -f "$1/manifest.json" ] && grep -q '"status": "complete"' "$1/report.json" 2>/dev/null; }

# One job: "<index> <pattern> <arm> <seed>".
run_job() {
  local index=$1 pattern=$2 arm=$3 seed=$4
  local name="$pattern-$arm-s$seed" root attempt=1
  for root in "$OUT_ROOT/runs/$name" "$OUT_ROOT/runs/$name"-a*; do
    if [ -d "$root" ] && complete "$root"; then
      echo "skip $name: complete at $root"
      return 0
    fi
  done
  root="$OUT_ROOT/runs/$name"
  while [ -e "$root" ]; do attempt=$((attempt + 1)); root="$OUT_ROOT/runs/$name-a$attempt"; done
  local gpus=($GPUS)
  local gpu=${gpus[$((index % ${#gpus[@]}))]}
  local start=$SECONDS
  echo "start $name on gpu $gpu -> $root"
  if CUDA_VISIBLE_DEVICES=$gpu RAYON_NUM_THREADS=$THREADS "$BIN" \
      "out=$root" layout=fact "tokenizer=$TOKENIZER" "device=$DEVICE" context=384 \
      pairs_per_bucket=4 "batch=$BATCH" "steps=$STEPS" "lr=$LR" "warmup=$WARMUP" \
      min_lr=0.1 weight_decay=0.1 clip=1.0 "eval_every=$EVAL_EVERY" curve_sequences=16 \
      "final_sequences=$FINAL_SEQUENCES" probe_sequences=16 probe_steps=final \
      "seed=$seed" "max_seconds=$MAX_SECONDS" save_model=true gaps=0,1,2,3 forms=rehearse,bare \
      arm=stack "pattern=$pattern" read=l2 rotation=true width=128 heads=4 mlp=384 \
      age=default "lineage=$arm" learned_init=lag1 \
      > "$OUT_ROOT/logs/$(basename "$root").stdout" 2> "$OUT_ROOT/logs/$(basename "$root").stderr"; then
    echo "done $name in $((SECONDS - start))s: $(grep '^final' "$OUT_ROOT/logs/$(basename "$root").stderr" || true)"
  else
    echo "FAILED $name after $((SECONDS - start))s (see $OUT_ROOT/logs/$(basename "$root").stderr)"
  fi
}
export -f run_job complete
export OUT_ROOT GPUS BIN TOKENIZER DEVICE BATCH STEPS LR WARMUP EVAL_EVERY FINAL_SEQUENCES MAX_SECONDS THREADS

jobs=()
index=0
for pattern in ${PATTERNS:-rararr rrarra aaaaaa}; do
  for spec in ${ARMS:-none:3 f2:3 qk:3 conv:3 identity:2 so4:2 wprev:2 qk_jj:2}; do
    arm=${spec%:*}
    for seed in $(seq 1 "${spec#*:}"); do
      jobs+=("$index $pattern $arm $seed")
      index=$((index + 1))
    done
  done
done
echo "${#jobs[@]} runs, $K at a time"
printf '%s\n' "${jobs[@]}" | xargs -P "$K" -L 1 bash -c 'run_job "$@"' _ | tee -a "$OUT_ROOT/grid.log"

stamp=$(date -u +%Y%m%dT%H%M%SZ)
"$BIN" mode=decide "runs=$OUT_ROOT/runs" "out=$OUT_ROOT/decision-$stamp" 2> "$OUT_ROOT/logs/decision-$stamp.stderr"
cat "$OUT_ROOT/decision-$stamp/decision.md"
echo "utc_end $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$OUT_ROOT/pod-manifest.txt"
