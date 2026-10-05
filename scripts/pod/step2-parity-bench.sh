#!/usr/bin/env bash
# Step 2 deployment-parity MQAR bench: the full grid and its decision table
# (References #820; docs/research/barrier-assessment-2026-10-05/
# final-assessment.md Step 2 and completeness-critique.md points 2, 5, 6).
#
# Runs `mqar-bench layout=fact` for patterns rararr, rrarra (production) and
# aaaaaa (diagnostic) x arms none/f2/qk/conv/qk_jj (3 seeds) and identity/
# so4/wprev (2 seeds): 63 runs, K at a time, round-robin over the GPUs.
# Every run writes its own exclusively claimed, sealed report root under
# $OUT_ROOT/runs. A rerun of this script skips every (pattern, arm, seed)
# that already has a sealed root which trained its full budget
# (status complete, stopped_early_at_max_seconds=false and
# steps_completed == STEPS, read from its report.json). A sealed root that
# stopped short of that budget (for example at MAX_SECONDS) is moved aside
# to <root>.incomplete-<UTC stamp>, never deleted, and the seed is re-run:
# `mode=decide` drops such roots, so skipping them would leave the seed
# missing for good. A failed or unsealed root keeps its place and the seed
# gets a new attempt root (a root path is never reused). A report.json that
# cannot be read stops the grid. Then `mode=decide` writes a new
# sealed decision root $OUT_ROOT/decision-<UTC stamp> with decision.json and
# decision.md.
#
# Required:  TOKENIZER=/path/to/#1017/tokenizer.json  OUT_ROOT=/new/or/resumed/dir
# Optional:  K=8 GPUS="0 1" DEVICE=cuda STEPS=4000 BATCH=16 LR=0.001
#            WARMUP=200 EVAL_EVERY=250 FINAL_SEQUENCES=256 MAX_SECONDS=3600
#            THREADS=2 CUDA_COMPUTE_CAP=89 SKIP_BUILD=0 REPO=<this checkout>
#            BIN=<prebuilt mqar-bench> (implies SKIP_BUILD=1)
#            PATTERNS="rararr rrarra aaaaaa"
#            ARMS="none:3 f2:3 qk:3 conv:3 qk_jj:3 identity:2 so4:2 wprev:2" (arm:seeds)
# A calibration run of one job: PATTERNS=rararr ARMS=none:1 K=1 (its root is
# then reused by the full grid when it trained the same STEPS; otherwise it
# is moved aside and re-run).
set -euo pipefail

command -v python3 >/dev/null || { echo "python3 is required to read report.json" >&2; exit 1; }

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

# The state of one existing root: prints `full` (sealed, status complete,
# not stopped at max_seconds, steps_completed == STEPS), `truncated ...`
# (sealed and complete but short of that budget), `failed` (sealed, status
# not complete) or `unsealed`. Returns non-zero when a sealed report.json
# cannot be read or lacks the budget fields: never guess.
root_state() {
  if [ ! -f "$1/manifest.json" ] || [ ! -f "$1/report.json" ]; then echo unsealed; return 0; fi
  python3 -c '
import json, sys
path, steps = sys.argv[1], int(sys.argv[2])
report = json.load(open(path))
if report.get("status") != "complete":
    print("failed")
    sys.exit(0)
results = report["results"]
early, done = results["stopped_early_at_max_seconds"], results["steps_completed"]
if not isinstance(early, bool) or not isinstance(done, int) or isinstance(done, bool):
    sys.exit(f"{path}: stopped_early_at_max_seconds / steps_completed missing or mistyped")
if not early and done == steps:
    print("full")
else:
    print(f"truncated steps_completed={done} of {steps} stopped_early_at_max_seconds={str(early).lower()}")
' "$1/report.json" "$STEPS"
}

# One job: "<index> <pattern> <arm> <seed>".
run_job() {
  local index=$1 pattern=$2 arm=$3 seed=$4
  local name="$pattern-$arm-s$seed" root attempt=1 state aside
  for root in "$OUT_ROOT/runs/$name" "$OUT_ROOT/runs/$name"-a*; do
    [ -d "$root" ] || continue
    case $root in *.incomplete-*) continue ;; esac  # already moved aside
    if ! state=$(root_state "$root"); then
      echo "ERROR $name: cannot read the budget of $root/report.json; stopping the grid" >&2
      exit 255  # makes xargs stop launching jobs
    fi
    case $state in
      full)
        echo "skip $name: full budget at $root"
        return 0 ;;
      truncated*)
        aside="$root.incomplete-$(date -u +%Y%m%dT%H%M%SZ)"
        if [ -e "$aside" ] || ! mv -- "$root" "$aside"; then
          echo "ERROR $name: could not move $root aside to $aside; stopping the grid" >&2
          exit 255
        fi
        echo "moved $name aside ($state): $root -> $aside" ;;
    esac
  done
  # A path is never reused, including one whose root was moved aside.
  root="$OUT_ROOT/runs/$name"
  while [ -e "$root" ] || compgen -G "$root.incomplete-*" >/dev/null; do
    attempt=$((attempt + 1)); root="$OUT_ROOT/runs/$name-a$attempt"
  done
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
export -f run_job root_state
export OUT_ROOT GPUS BIN TOKENIZER DEVICE BATCH STEPS LR WARMUP EVAL_EVERY FINAL_SEQUENCES MAX_SECONDS THREADS

jobs=()
index=0
for pattern in ${PATTERNS:-rararr rrarra aaaaaa}; do
  for spec in ${ARMS:-none:3 f2:3 qk:3 conv:3 qk_jj:3 identity:2 so4:2 wprev:2}; do
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
