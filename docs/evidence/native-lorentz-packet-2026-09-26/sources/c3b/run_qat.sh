#!/bin/bash
# Wave 1: quantization-aware fine-tunes; wave 2: float fine-tune controls.
set -u
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b
BIN=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/target/release/examples/joint-read-geometry
COMMON="train=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/data/code/train.u16 valid=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/data/code/valid.u16 lens=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/data/code/lens.u16 width=128 context=256 batch=8 steps=300 lr=0.0003 eval_every=100 eval_windows=64 final_windows=512 checkpoint_every=100 save_model=true"
run() { # name geometry seed parent extra
  $BIN $COMMON out=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_$1 geometry=$2 seed=$3 init=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/runs/ctx256/$4/model $5 > /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_$1.log 2>&1
  echo "$1 exit $?" >> /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_status.log
}
for wave in qat float; do
  if [ $wave = qat ]; then extra="quantize_ramp=100"; else extra=""; fi
  run ${wave}_lorentzflat_s1 lorentz 1 lorentzflat_s1_r1 "$extra" &
  run ${wave}_dot_s1 dot 1 dot_s1_r1 "$extra" &
  run ${wave}_lorentzflat_s2 lorentz 2 lorentzflat_s2_r1 "$extra" &
  run ${wave}_dot_s2 dot 2 dot_s2_r1 "$extra" &
  wait
done
echo ALL_DONE >> /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_status.log
