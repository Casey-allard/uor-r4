#!/usr/bin/env bash
# Integer (D10) serving of the cycle-4 pilot pair and seed-1 ablations with the binary built from commit 2a681bb7:
# export each trained model, then score the integer engine beside the float model on the 512 evenly spaced
# development windows of the final evaluation (131,072 targets). One thread each, niced, beside the training
# pipeline, so the tokens-per-second figures are for a shared machine.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-2a681bb7
L=$S/c4/lut/v1
D=$S/c3/data/code
echo "start $(date -u +%FT%TZ) source 2a681bb7 binary $(sha256sum $B | cut -d' ' -f1)" >> $L/run.log
for spec in pilot/geometric_lr0.004 pilot/transformer_lr0.002 ablation/dot ablation/norot ablation/readsonly; do
  name=${spec//\//-}
  [ -f $L/export-$name/manifest.json ] || RAYON_NUM_THREADS=1 nice -n 5 $B export model=$S/c4/runs/$spec/model out=$L/export-$name
  [ -f $L/eval512-$name/manifest.json ] || RAYON_NUM_THREADS=1 nice -n 5 $B lut-evaluate artifact=$L/export-$name/model.lut \
    valid=$D/valid.u16 lens=$D/lens.u16 model=$S/c4/runs/$spec/model windows=512 threads=1 out=$L/eval512-$name
  echo "done $name $(date -u +%FT%TZ)" >> $L/run.log
done
echo "end $(date -u +%FT%TZ)" >> $L/run.log
