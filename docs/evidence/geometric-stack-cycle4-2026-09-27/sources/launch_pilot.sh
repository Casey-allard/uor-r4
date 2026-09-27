#!/usr/bin/env bash
# Cycle 4 learning-rate pilot: 1,000 updates (4,096,000 target visits) on the repository code corpus,
# transformer control against the geometric stack (rrarra, Lorentz, rotation), lr 2e-3, 1e-3, 4e-3.
# Two runs at a time with two threads each.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-dca1b790
D=$S/c3/data/code
OUT=$S/c4/runs/pilot
mkdir -p $OUT
echo "source dca1b790; binary $(sha256sum $B | cut -d' ' -f1); started $(date -u +%FT%TZ)" >> $OUT/launch.log
for lr in 0.002 0.001 0.004; do
  for arch in transformer geometric; do
    RAYON_NUM_THREADS=2 $B train train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt \
      out=$OUT/${arch}_lr$lr arch=$arch seed=1 steps=1000 batch=16 lr=$lr warmup=100 eval_every=250 \
      eval_windows=64 final_windows=512 checkpoint_every=250 sample_tokens=96 > $OUT/${arch}_lr$lr.log 2>&1 &
  done
  wait
  echo "round lr=$lr done $(date -u +%FT%TZ)" >> $OUT/launch.log
done
echo "pilot done $(date -u +%FT%TZ)" >> $OUT/launch.log
