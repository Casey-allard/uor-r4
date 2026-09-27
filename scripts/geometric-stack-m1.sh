#!/usr/bin/env bash
# The geometric stack against a transformer control on an Apple-silicon Mac, on the retained TinyStories stores.
# Both arms have #1017's parameter count (7,155,360; the geometric stack's MLP width is chosen to match) and train
# on identical windows. Each trained model is then scored with the retained evaluator's block protocol on the
# development store, where #1017 itself scores 1.574024 nats on the 912-block comparison tail and the retained
# native model's continuation finals score 1.975-1.996 (docs/integration/current-state.md). See
# docs/integration/geometric-stack-cycle4-2026-09-27.md.
#
# Usage, from the repository root:
#   scripts/geometric-stack-m1.sh TRAIN.u16[,TRAIN2.u16] DEV.u16 TOKENIZER.json [OUT_PARENT]
#
# TRAIN and DEV are little-endian u16 token files in the retained 4,096-token vocabulary, for example the
# issue-1014 and issue-1017 training stores (comma-separated; windows are drawn from each in proportion to its
# length unless TRAIN_WEIGHTS is set) and the issue-1017 development store. TOKENIZER is the retained
# tokenizer.json, used only to decode samples. OUT_PARENT (default reports/geometric-stack-<timestamp>) must not
# exist; every run claims and seals its own report root inside it.
#
# Stages:
#   1. build the geometric-stack example (with Accelerate on macOS)
#   2. for each seed in SEEDS: train the transformer control and the geometric stack for STEPS updates of
#      BATCH x 256 tokens, both arms at once with THREADS threads each (resumable: a stopped run continues with
#      resume=ROOT/checkpoint in a new root)
#   3. evaluate each trained model on DEV with the block protocol (tune, comparison and full means)
#   4. write summary.txt: comparison-tail NLL per arm next to #1017's 1.574024, and greedy/sampled continuations
#   5. unless INTEGER=0: export each trained model to its integer serving artifact (owner decision D10: 4-bit table
#      weight maps, no floating point), score it with the same block protocol beside the float model, and sample
#      from it with the integer sampler; summary.txt gains the integer comparison tail and the engine's tokens per
#      second on this machine (one thread)
#
# Knobs: SEEDS ("1"), STEPS (7324), BATCH (16), THREADS (4), SEQUENTIAL (0), LR_TRANSFORMER (0.002) and
# LR_GEOMETRIC (0.004), the lab pilot's choices on code (section 5 of the note), WARMUP (200), PATTERN (rrarra),
# READ (lorentz), ROTATION (true), TRAIN_WEIGHTS (unset: proportional), EVAL_EVERY (250), INTEGER (1), FEATURES
# (cpu-accelerate on macOS, none elsewhere).
#
# Shared machine: by default both arms run at once with THREADS threads each (8 threads). Before launching, check
# #973 for other labs' active fits and do not start beside one; SEQUENTIAL=1 runs the arms one after the other,
# and THREADS lowers each arm's share.
#
# Cost: unmeasured on the M1. The lab sandbox (4 x86 cores, two arms at once, two threads each) trained about
# 1,100 tokens per second per arm, so STEPS=7324 is about 7.6 hours there. Try STEPS=250 first to time an update.
# Send OUT_PARENT back for the record.
set -euo pipefail

TRAIN=${1:?usage: scripts/geometric-stack-m1.sh TRAIN.u16[,TRAIN2.u16] DEV.u16 TOKENIZER.json [OUT_PARENT]}
DEV=${2:?development token file required}
TOKENIZER=${3:?tokenizer.json required (for decoding samples)}
OUT=${4:-reports/geometric-stack-$(date +%Y%m%d-%H%M%S)}
SEEDS=${SEEDS:-"1"}
SEQUENTIAL=${SEQUENTIAL:-0}
STEPS=${STEPS:-7324}
BATCH=${BATCH:-16}
THREADS=${THREADS:-4}
LR_TRANSFORMER=${LR_TRANSFORMER:-0.002}
LR_GEOMETRIC=${LR_GEOMETRIC:-0.004}
WARMUP=${WARMUP:-200}
PATTERN=${PATTERN:-rrarra}
READ=${READ:-lorentz}
ROTATION=${ROTATION:-true}
EVAL_EVERY=${EVAL_EVERY:-250}
INTEGER=${INTEGER:-1}
if [ -z "${FEATURES+x}" ]; then
  if [ "$(uname -s)" = Darwin ]; then FEATURES=cpu-accelerate; else FEATURES=; fi
fi
TARGET=${CARGO_TARGET_DIR:-target}
STACK=$TARGET/release/examples/geometric-stack

IFS=, read -r -a train_files <<< "$TRAIN"
for file in "${train_files[@]}" "$DEV" "$TOKENIZER"; do
  [ -f "$file" ] || { echo "missing $file" >&2; exit 1; }
done
[ -e "$OUT" ] && { echo "$OUT exists; choose a new output parent" >&2; exit 1; }
mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

features=()
[ -n "$FEATURES" ] && features=(--features "$FEATURES")
cargo build --release ${features[@]+"${features[@]}"} -p uor-r4-training --example geometric-stack

weights=()
[ -n "${TRAIN_WEIGHTS:-}" ] && weights=(train_weights="$TRAIN_WEIGHTS")
common=(train="$TRAIN" valid="$DEV" tokenizer="$TOKENIZER" steps="$STEPS" batch="$BATCH" warmup="$WARMUP"
  eval_every="$EVAL_EVERY" eval_windows=64 final_windows=512 checkpoint_every="$EVAL_EVERY" sample_tokens=128
  ${weights[@]+"${weights[@]}"})

for seed in $SEEDS; do
  RAYON_NUM_THREADS=$THREADS "$STACK" train "${common[@]}" out="$OUT/transformer-s$seed" arch=transformer \
    seed="$seed" lr="$LR_TRANSFORMER" > "$OUT/transformer-s$seed.log" 2>&1 &
  [ "$SEQUENTIAL" = 1 ] && wait
  RAYON_NUM_THREADS=$THREADS "$STACK" train "${common[@]}" out="$OUT/geometric-s$seed" arch=geometric \
    pattern="$PATTERN" read="$READ" rotation="$ROTATION" seed="$seed" lr="$LR_GEOMETRIC" \
    > "$OUT/geometric-s$seed.log" 2>&1 &
  wait
done

for seed in $SEEDS; do
  for arm in transformer geometric; do
    "$STACK" evaluate model="$OUT/$arm-s$seed/model" tokens="$DEV" out="$OUT/evaluate-$arm-s$seed" tune_blocks=64
    python3 -c 'import json,sys; e=json.load(open(sys.argv[1])); c=e["comparison"]
print(sys.argv[2], "comparison tail %.6f nats on %d targets (#1017: 1.574024); full %.6f" % (c["nll"], c["targets"], e["full"]["nll"]))' \
      "$OUT/evaluate-$arm-s$seed/evaluation.json" "$arm-s$seed" | tee -a "$OUT/summary.txt"
    python3 -c 'import json,sys
for row in json.load(open(sys.argv[1]))["samples"]["rows"]:
    print(sys.argv[2], "|", (row["prompt"] or "") + " ->", (row["sampled"] or "").replace("\n", " "))' \
      "$OUT/$arm-s$seed/report.json" "$arm-s$seed" | tee -a "$OUT/samples.txt"
  done
done

if [ "$INTEGER" = 1 ]; then
  for seed in $SEEDS; do
    for arm in transformer geometric; do
      "$STACK" export model="$OUT/$arm-s$seed/model" out="$OUT/integer-$arm-s$seed"
      "$STACK" lut-evaluate artifact="$OUT/integer-$arm-s$seed/model.lut" valid="$DEV" model="$OUT/$arm-s$seed/model" \
        blocks=true tune_blocks=64 threads=1 out="$OUT/integer-evaluate-$arm-s$seed"
      python3 -c 'import json,sys; e=json.load(open(sys.argv[1])); c=e["comparison"]; g=e["engine"]
print(sys.argv[2], "integer comparison tail %.6f nats (float %.6f; #1017: 1.574024); top-1 agreement %.4f; %.0f tokens/s in engine steps on one thread (%s)" % (c["integer"]["nll"], c["float"]["nll"], e["top1_agreement"], g["tokens_per_second"], g["backend"]))' \
        "$OUT/integer-evaluate-$arm-s$seed/evaluation.json" "$arm-s$seed" | tee -a "$OUT/summary.txt"
      "$STACK" lut-sample artifact="$OUT/integer-$arm-s$seed/model.lut" valid="$DEV" tokenizer="$TOKENIZER" \
        out="$OUT/integer-samples-$arm-s$seed"
      python3 -c 'import json,sys
for row in json.load(open(sys.argv[1]))["rows"]:
    print(sys.argv[2], "integer |", (row["prompt"] or "") + " ->", (row["sampled"] or "").replace("\n", " "))' \
        "$OUT/integer-samples-$arm-s$seed/samples.json" "$arm-s$seed" | tee -a "$OUT/samples.txt"
    done
  done
fi

echo "done: */report.json, evaluate-*/evaluation.json, integer-*/, summary.txt and samples.txt are in $OUT"
