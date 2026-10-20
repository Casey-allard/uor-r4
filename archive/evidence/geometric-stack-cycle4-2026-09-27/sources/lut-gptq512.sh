#!/usr/bin/env bash
# GPTQ (calibrated 4-bit) integer serving of the cycle-4 pilot pair and seed-1 ablations with the binary built
# from commit 6bc7dd6d: export each trained model with input moments from 64 evenly spaced 256-token windows of
# its own training split (damp 0.01), then score the integer engine, the artifact's grid reference and the float
# model on the 512 evenly spaced development windows of the final evaluation (131,072 targets). One thread each,
# niced, beside the training pipeline.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-6bc7dd6d
L=$S/c4/lut/v1
D=$S/c3/data/code
echo "gptq start $(date -u +%FT%TZ) source 6bc7dd6d binary $(sha256sum $B | cut -d' ' -f1)" >> $L/run.log
for spec in pilot/geometric_lr0.004 pilot/transformer_lr0.002 ablation/dot ablation/norot ablation/readsonly; do
  name=${spec//\//-}
  [ -f $L/gptq-export-$name/manifest.json ] || RAYON_NUM_THREADS=1 nice -n 5 $B export model=$S/c4/runs/$spec/model \
    calibration=$D/train.u16 calibration_windows=64 damp=0.01 out=$L/gptq-export-$name
  [ -f $L/gptq-eval512-$name/manifest.json ] || RAYON_NUM_THREADS=1 nice -n 5 $B lut-evaluate \
    artifact=$L/gptq-export-$name/model.lut valid=$D/valid.u16 lens=$D/lens.u16 model=$S/c4/runs/$spec/model \
    windows=512 threads=1 reference=true out=$L/gptq-eval512-$name
  echo "gptq done $name $(date -u +%FT%TZ)" >> $L/run.log
done
echo "gptq end $(date -u +%FT%TZ)" >> $L/run.log
