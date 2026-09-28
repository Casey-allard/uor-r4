#!/usr/bin/env bash
# Curvature-homotopy conversion pilot for SmolLM2 on an Apple-silicon Mac (M1a + M1b of the lab roadmap).
#
# Usage, from the repository root:
#   scripts/kappa-m1-pilot.sh TEXT_FILE [OUT_PARENT]
#
# TEXT_FILE is any plain-text corpus of at least a few MB (for example TinyStories or exported chat turns); its first
# 95% of lines train and the last 5% validate. OUT_PARENT (default reports/kappa-m1-<timestamp>) must not exist; every
# run below claims and seals its own report root inside it. The checkpoints are the registered sources
# (models/smollm2-{135m,360m}-instruct.json); override with S135=... S360=....
#
# Stages (each stage's outputs are independent report roots, so an interrupted pilot can be resumed by hand):
#   1. build with Metal; tokenize with the checkpoint's own byte-level BPE
#   2. M1a: flat-limit probes (intrinsic and key_norm) and a greedy chat sample, dot vs converted
#   3. M1b: KL distillation from SmolLM2-360M-Instruct into the converted 135M, trainable query/key projections,
#      paired data order, seeds 1-2: dot; intrinsic learnable; intrinsic annealed to log_eps=-2; key_norm annealed.
#
# Environment knobs: STEPS (500), BATCH (4), TIME (256), DEVICE (metal), ANNEAL_TO (-2), SEEDS ("1 2").
# Memory: batch 4 x 256 with a 360M teacher needs roughly 8 GB of unified memory; use BATCH=2 on an 8 GB machine.
set -euo pipefail

TEXT=${1:?usage: scripts/kappa-m1-pilot.sh TEXT_FILE [OUT_PARENT]}
OUT=${2:-reports/kappa-m1-$(date +%Y%m%d-%H%M%S)}
S135=${S135:-.uor-models/sources/smollm2-135m-instruct}
S360=${S360:-.uor-models/sources/smollm2-360m-instruct}
STEPS=${STEPS:-500}
BATCH=${BATCH:-4}
TIME=${TIME:-256}
DEVICE=${DEVICE:-metal}
ANNEAL_TO=${ANNEAL_TO:--2}
SEEDS=${SEEDS:-"1 2"}
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
cargo build --release "${features[@]}" -p uor-r4-training --example kappa-conversion

lines=$(wc -l < "$TEXT")
cut_at=$(( lines * 95 / 100 ))
head -n "$cut_at" "$TEXT" > "$OUT/train.txt"
tail -n +"$(( cut_at + 1 ))" "$TEXT" > "$OUT/valid.txt"
"$BIN" mode=tokenize model="$S135" text="$OUT/train.txt" out="$OUT/train.u16"
"$BIN" mode=tokenize model="$S135" text="$OUT/valid.txt" out="$OUT/valid.u16"
rm -f "$OUT/train.txt" "$OUT/valid.txt"

# M1a: is the conversion exact on the real model?
for score in intrinsic key_norm; do
  "$BIN" mode=probe device="$DEVICE" model="$S135" tokens="$OUT/valid.u16" score="$score" out="$OUT/probe-$score"
done
prompt="Explain in two sentences why the sky is blue."
{
  echo "dot:";       "$BIN" mode=sample device="$DEVICE" model="$S135" prompt="$prompt" score=dot
  echo "intrinsic:"; "$BIN" mode=sample device="$DEVICE" model="$S135" prompt="$prompt" score=intrinsic
} > "$OUT/samples.txt"

# M1b: does curvature earn its place against the matched dot control?
common=(mode=train device="$DEVICE" student="$S135" teacher="$S360" train="$OUT/train.u16" valid="$OUT/valid.u16"
        trainable=query_key steps="$STEPS" batch="$BATCH" time="$TIME" eval_every=100 eval_windows=16)
for seed in $SEEDS; do
  "$BIN" "${common[@]}" seed="$seed" score=dot out="$OUT/train-dot-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=intrinsic out="$OUT/train-intrinsic-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=intrinsic anneal_to="$ANNEAL_TO" out="$OUT/train-intrinsic-anneal-s$seed"
  "$BIN" "${common[@]}" seed="$seed" score=key_norm anneal_to="$ANNEAL_TO" out="$OUT/train-key_norm-anneal-s$seed"
done
echo "done: $OUT (send the probe-*/probe.json, train-*/report.json and samples.txt back to the lab)"
