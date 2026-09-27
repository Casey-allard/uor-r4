#!/usr/bin/env bash
# Cycle 4 ablations at the pilot's scale: 1,000 updates (4,096,000 target visits) on the repository code
# corpus, seed 1, the geometric arm's pilot learning rate. Each changes one thing in the geometric stack:
#   dot       Dot read instead of Lorentz
#   norot     identity transport (real gated linear recurrence)
#   readsonly six Lorentz read layers, no recurrence (and no RoPE)
# usage: launch_ablations.sh LR_GEOMETRIC
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-b87acd63
D=$S/c3/data/code
LG=${1:?geometric lr}
OUT=$S/c4/runs/ablation
mkdir -p $OUT
echo "source b87acd63; binary $(sha256sum $B | cut -d' ' -f1); lr $LG; started $(date -u +%FT%TZ)" >> $OUT/launch.log
common=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt arch=geometric seed=1
  steps=1000 batch=16 lr=$LG warmup=100 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=250 sample_tokens=96)
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/dot pattern=rrarra read=dot rotation=true > $OUT/dot.log 2>&1 &
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/norot pattern=rrarra read=lorentz rotation=false > $OUT/norot.log 2>&1 &
wait
RAYON_NUM_THREADS=4 $B train "${common[@]}" out=$OUT/readsonly pattern=aaaaaa read=lorentz rotation=false > $OUT/readsonly.log 2>&1
echo "ablations done $(date -u +%FT%TZ)" >> $OUT/launch.log
