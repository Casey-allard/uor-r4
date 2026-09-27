#!/usr/bin/env bash
# Curvature-homotopy conversion pilot for SmolLM2 on an Apple-silicon Mac.
#
# Usage, from the repository root:
#   scripts/kappa-m1-pilot.sh TEXT_FILE [OUT_PARENT]
#
# TEXT_FILE is any plain-text corpus of at least a few MB (for example exported chat turns or TinyStories); its first
# 95% of lines train and the last 5% validate. OUT_PARENT (default reports/kappa-m1-<timestamp>) must not exist; every
# run below claims and seals its own report root inside it. The checkpoints are the registered sources
# (models/smollm2-{135m,360m}-instruct.json); override with S135=... S360=....
#
# Stages:
#   1. build with Metal; tokenize with the checkpoint's own byte-level BPE
#   2. flat-limit probes (intrinsic and key_norm) over a grid of dimensionless curvature t
#   3. the zero-training curvature drive with its pre-registered rule (teacher: SmolLM2-360M-Instruct)
#   4. only if the rule passes (or FORCE_TRAIN=1): distillation from 360M into the converted 135M, trainable
#      query/key projections, paired data order, arms dot / intrinsic_linear (matched first-order control) /
#      intrinsic learnable / intrinsic annealed to t = 1 / key_norm annealed to t = 1, for each seed in SEEDS.
#
# Knobs: STEPS (500), BATCH (1), ACCUMULATE (4), TIME (256; use 128 on an 8 GB machine), DEVICE (metal),
# SEEDS ("1 2 3"), FORCE_TRAIN (0).
set -euo pipefail

TEXT=${1:?usage: scripts/kappa-m1-pilot.sh TEXT_FILE [OUT_PARENT]}
OUT=${2:-reports/kappa-m1-$(date +%Y%m%d-%H%M%S)}
S135=${S135:-.uor-models/sources/smollm2-135m-instruct}
S360=${S360:-.uor-models/sources/smollm2-360m-instruct}
STEPS=${STEPS:-500}
BATCH=${BATCH:-1}
ACCUMULATE=${ACCUMULATE:-4}
TIME=${TIME:-256}
DEVICE=${DEVICE:-metal}
SEEDS=${SEEDS:-"1 2 3"}
FORCE_TRAIN=${FORCE_TRAIN:-0}
BIN=target/release/examples/kappa-conversion

for dir in "$S135" "$S360"; do
  for file in config.json model.safetensors tokenizer.json; do
    [ -f "$dir/$file" ] || { echo "missing $dir/$file (download the checkpoint first)" >&2; exit 1; }
  done
done
[ -e "$OUT" ] && { echo "$OUT exists; choose a new output parent" >&2; exit 1; }
mkdir -p "$OUT"

features=()
[ "$DEVICE" = metal ] && features=(--features metal)
cargo build --release ${features[@]+"${features[@]}"} -p uor-r4-training --example kappa-conversion

lines=$(wc -l < "$TEXT")
cut_at=$(( lines * 95 / 100 ))
head -n "$cut_at" "$TEXT" > "$OUT/train.txt"
tail -n +"$(( cut_at + 1 ))" "$TEXT" > "$OUT/valid.txt"
"$BIN" mode=tokenize model="$S135" text="$OUT/train.txt" out="$OUT/train.u16"
"$BIN" mode=tokenize model="$S135" text="$OUT/valid.txt" out="$OUT/valid.u16"
rm -f "$OUT/train.txt" "$OUT/valid.txt"

# Is the flat-limit conversion within rounding of the checkpoint, and how fast does curvature cost?
for score in intrinsic key_norm; do
  "$BIN" mode=probe device="$DEVICE" model="$S135" tokens="$OUT/valid.u16" score="$score" time="$TIME" \
    out="$OUT/probe-$score"
done
"$BIN" mode=sample device="$DEVICE" model="$S135" prompt="Explain in two sentences why the sky is blue." \
  > "$OUT/sample-dot.txt"

# Does any head's objective want curvature? (zero training)
"$BIN" mode=drive device="$DEVICE" model="$S135" teacher="$S360" tokens="$OUT/valid.u16" time="$TIME" \
  out="$OUT/drive"
if grep -q '"any_layer_passes": true' "$OUT/drive/drive.json"; then
  echo "drive rule: at least one layer passes; running the curvature arms"
elif [ "$FORCE_TRAIN" = 1 ]; then
  echo "drive rule: no layer passes; FORCE_TRAIN=1, running the curvature arms anyway"
else
  echo "drive rule: no layer passes. Per the pre-registered rule the curvature arms are skipped: keep dot heads and"
  echo "report the backbone as a quantized transformer. Send $OUT/drive/drive.json and $OUT/probe-*/probe.json back."
  exit 0
fi

common=(mode=train device="$DEVICE" student="$S135" teacher="$S360" train="$OUT/train.u16" valid="$OUT/valid.u16"
        trainable=query_key steps="$STEPS" batch="$BATCH" accumulate="$ACCUMULATE" time="$TIME"
        eval_every=100 eval_windows=16)
for seed in $SEEDS; do
  "$BIN" "${common[@]}" seed="$seed" score=dot out="$OUT/train-dot-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=intrinsic_linear out="$OUT/train-intrinsic_linear-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=intrinsic out="$OUT/train-intrinsic-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=intrinsic anneal_t=1 out="$OUT/train-intrinsic-anneal-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=key_norm anneal_t=1 out="$OUT/train-key_norm-anneal-s$seed"
done
echo "done: $OUT (send drive/drive.json, probe-*/probe.json and train-*/report.json back to the lab)"
