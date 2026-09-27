#!/usr/bin/env bash
# The native D8 model with the hyperbolic (Lorentz) read, from training to integer serving, on an Apple-silicon
# Mac. It runs the full-scale comparison proposed in docs/integration/hyperbolic-cycle3-2026-09-26.md §9.2 and
# serves each trained model with integers (§11).
#
# Usage, from the repository root:
#   scripts/native-lorentz-m1.sh TRAIN.u16 DEV.u16 [OUT_PARENT]
#
# TRAIN.u16 and DEV.u16 are little-endian u16 token files in the model's 4096-token vocabulary, for example the
# retained TinyStories training and development stores. OUT_PARENT (default reports/native-lorentz-<timestamp>)
# must not exist; every run claims and seals its own report root inside it.
#
# Stages:
#   1. build joint-read-geometry and joint-integer-parity (with Accelerate on macOS) and the training CLI
#   2. for each seed in SEEDS, train geometry=dot and geometry=lorentz lorentz_start=flat from scratch
#      (STEPS updates, resumable: a stopped run continues with resume=ROOT/checkpoint in a new root)
#   3. fine-tune each trained model for QAT_STEPS updates with quantize_ramp (the packed format's frozen 4-bit
#      scales, straight-through, ramped over QAT_RAMP updates), and, with CONTROL=1, the same fine-tune in float
#   4. export the integer tables once (with the arcosh table the Lorentz read needs)
#   5. joint-integer-parity for each fine-tuned model on DEV.u16: float, packed emulator and integer NLL, read on
#      and off, and integer-against-emulator drift position by position
#   6. only with TOKENIZER (the tokenizer.json that produced the token files, with <|bos|> = 0 and <|eos|> = 1):
#      pack each fine-tuned model as a development serving bundle and generate greedy and sampled continuations
#      of PROMPT with the integer runtime (uor-r4-integer generate)
#
# Knobs: WIDTH (256), CONTEXT (256), BATCH (16), SHARDS (2), STEPS (7324), LR (0.001), SEEDS ("1 2"),
# EVAL_EVERY (250), EVAL_WINDOWS (64), FINAL_WINDOWS (256), QAT_STEPS (300), QAT_RAMP (100), QAT_LR (0.0003),
# CONTROL (0), WINDOWS (64, parity windows), LENS (a u16 file of per-token byte lengths, for bits per byte),
# FEATURES (cpu-accelerate on macOS, none elsewhere), TOKENIZER, PROMPT ("Once upon a time"), NEW_TOKENS (64).
#
# Cost: a from-scratch run is STEPS updates of BATCH x CONTEXT tokens; at width 256 expect hours per run on an
# M1 (unmeasured). Try STEPS=500 first to time an update. Send OUT_PARENT back for the record.
set -euo pipefail

TRAIN=${1:?usage: scripts/native-lorentz-m1.sh TRAIN.u16 DEV.u16 [OUT_PARENT]}
DEV=${2:?development token file required}
OUT=${3:-reports/native-lorentz-$(date +%Y%m%d-%H%M%S)}
WIDTH=${WIDTH:-256}
CONTEXT=${CONTEXT:-256}
BATCH=${BATCH:-16}
SHARDS=${SHARDS:-2}
STEPS=${STEPS:-7324}
LR=${LR:-0.001}
SEEDS=${SEEDS:-"1 2"}
EVAL_EVERY=${EVAL_EVERY:-250}
EVAL_WINDOWS=${EVAL_WINDOWS:-64}
FINAL_WINDOWS=${FINAL_WINDOWS:-256}
QAT_STEPS=${QAT_STEPS:-300}
QAT_RAMP=${QAT_RAMP:-100}
QAT_LR=${QAT_LR:-0.0003}
CONTROL=${CONTROL:-0}
WINDOWS=${WINDOWS:-64}
PROMPT=${PROMPT:-"Once upon a time"}
NEW_TOKENS=${NEW_TOKENS:-64}
if [ -z "${FEATURES+x}" ]; then
  if [ "$(uname -s)" = Darwin ]; then FEATURES=cpu-accelerate; else FEATURES=; fi
fi
TARGET=${CARGO_TARGET_DIR:-target}
TRAINER=$TARGET/release/examples/joint-read-geometry
PARITY=$TARGET/release/examples/joint-integer-parity
CLI=$TARGET/release/uor-r4-training
SERVE=$TARGET/release/uor-r4-integer

for file in "$TRAIN" "$DEV"; do
  [ -f "$file" ] || { echo "missing token file $file" >&2; exit 1; }
done
[ -z "${TOKENIZER:-}" ] || [ -f "$TOKENIZER" ] || { echo "missing tokenizer $TOKENIZER" >&2; exit 1; }
[ "$(cd "$(dirname "$TRAIN")" && pwd)/$(basename "$TRAIN")" != \
  "$(cd "$(dirname "$DEV")" && pwd)/$(basename "$DEV")" ] ||
  { echo "training and development files must differ" >&2; exit 1; }
[ -e "$OUT" ] && { echo "$OUT exists; choose a new output parent" >&2; exit 1; }
mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

features=()
[ -n "$FEATURES" ] && features=(--features "$FEATURES")
cargo build --release ${features[@]+"${features[@]}"} -p uor-r4-training --example joint-read-geometry \
  --example joint-integer-parity --bin uor-r4-training
cargo build --release -p uor-r4-integer --bin uor-r4-integer

lens=()
[ -n "${LENS:-}" ] && lens=(lens="$LENS")
common=(train="$TRAIN" valid="$DEV" width="$WIDTH" context="$CONTEXT" batch="$BATCH" eval_every="$EVAL_EVERY"
  eval_windows="$EVAL_WINDOWS" final_windows="$FINAL_WINDOWS" save_model=true ${lens[@]+"${lens[@]}"})

for seed in $SEEDS; do
  for arm in dot lorentz; do
    start=()
    [ "$arm" = lorentz ] && start=(lorentz_start=flat)
    name=$arm-s$seed
    "$TRAINER" "${common[@]}" out="$OUT/train-$name" geometry="$arm" seed="$seed" steps="$STEPS" lr="$LR" \
      shards="$SHARDS" checkpoint_every="$EVAL_EVERY" ${start[@]+"${start[@]}"} 2>&1 | tee "$OUT/train-$name.log"
    modes=(qat)
    [ "$CONTROL" = 1 ] && modes+=(float)
    for mode in "${modes[@]}"; do
      ramp=()
      [ "$mode" = qat ] && ramp=(quantize_ramp="$QAT_RAMP")
      "$TRAINER" "${common[@]}" out="$OUT/$mode-$name" geometry="$arm" seed="$seed" steps="$QAT_STEPS" \
        lr="$QAT_LR" shards="$SHARDS" init="$OUT/train-$name/model" ${ramp[@]+"${ramp[@]}"} 2>&1 | tee "$OUT/$mode-$name.log"
    done
  done
done

"$CLI" joint-integer-tables "$OUT/tables"
for seed in $SEEDS; do
  for arm in dot lorentz; do
    "$PARITY" model="$OUT/qat-$arm-s$seed/model" tables="$OUT/tables" tokens="$DEV" out="$OUT/parity-$arm-s$seed" \
      windows="$WINDOWS" ${lens[@]+"${lens[@]}"} > "$OUT/parity-$arm-s$seed.log" 2>&1
    python3 -c 'import json,sys; r=json.load(open(sys.argv[1])); a=r["read"]
print(sys.argv[2], "float %.4f emulator %.4f integer %.4f nats; integer-emulator %+.5f; top-1 %.4f" % (a["float_nll"],
  a["emulator_nll"], a["integer_nll"], a["integer_minus_emulator_nll"], a["top1_agreement_integer_emulator"]))' \
      "$OUT/parity-$arm-s$seed/report.json" "$arm-s$seed" | tee -a "$OUT/summary.txt"
  done
done

if [ -n "${TOKENIZER:-}" ]; then
  python3 -c 'import json,sys; p=sys.argv[1]; n=int(sys.argv[2])
print(json.dumps([{"prompt":p,"max_new_tokens":n},
  {"prompt":p,"max_new_tokens":n,"selection":{"kind":"categorical","top_k":40,"seed":1}}]))' \
    "$PROMPT" "$NEW_TOKENS" > "$OUT/requests.json"
  for seed in $SEEDS; do
    for arm in dot lorentz; do
      "$SERVE" pack-development "$OUT/parity-$arm-s$seed/packed" "$OUT/tables" "$TOKENIZER" "$OUT/bundle-$arm-s$seed"
      "$SERVE" generate "$OUT/bundle-$arm-s$seed" "$OUT/requests.json" "$OUT/generate-$arm-s$seed" \
        > "$OUT/generate-$arm-s$seed.log" 2>&1
      python3 -c 'import json,sys
for line in open(sys.argv[1]):
    g=json.loads(line)["generation"]
    print(sys.argv[2], g["selection"]["kind"], "|", sys.argv[3] + g["raw_decoded"].replace("\n", " "))' \
        "$OUT/generate-$arm-s$seed/generations.jsonl" "$arm-s$seed" "$PROMPT" | tee -a "$OUT/samples.txt"
    done
  done
fi

echo "done: train-*/report.json, qat-*/report.json, parity-*/report.json, summary.txt and samples.txt are in $OUT"
