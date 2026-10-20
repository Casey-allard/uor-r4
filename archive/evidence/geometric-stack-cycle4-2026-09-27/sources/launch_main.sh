#!/usr/bin/env bash
# Cycle 4 main comparison: 7,324 updates (29,999,104 target visits), repository code and registry code drawn
# with equal probability, the transformer control against the geometric stack (rrarra, Lorentz, rotation),
# each at its pilot-selected learning rate. Two runs at a time with two threads each.
# usage: launch_main.sh LR_TRANSFORMER LR_GEOMETRIC [SEED]
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-b87acd63
D=$S/c3/data/code
R=$S/c4/data/registry.u16
LT=${1:?transformer lr}
LG=${2:?geometric lr}
SEED=${3:-1}
OUT=$S/c4/runs/main
mkdir -p $OUT
echo "source b87acd63; binary $(sha256sum $B | cut -d' ' -f1); lr transformer $LT geometric $LG seed $SEED; started $(date -u +%FT%TZ)" >> $OUT/launch.log
common=(train=$D/train.u16,$R train_weights=1,1 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt seed=$SEED
  steps=7324 batch=16 warmup=200 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=250 sample_tokens=128)
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/transformer_s$SEED arch=transformer lr=$LT > $OUT/transformer_s$SEED.log 2>&1 &
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/geometric_s$SEED arch=geometric pattern=rrarra read=lorentz rotation=true lr=$LG > $OUT/geometric_s$SEED.log 2>&1 &
wait
echo "main done $(date -u +%FT%TZ)" >> $OUT/launch.log
