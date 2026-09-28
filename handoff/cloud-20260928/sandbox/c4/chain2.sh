#!/usr/bin/env bash
# Replaces the tail of chain.sh (2026-09-27 07:22Z). After the ablations finish, a second seed of the
# Lorentz-versus-Dot pair (the first seed's -0.021 nat Dot edge is below the declared 0.03 threshold,
# so the declared rule calls for another seed), then the main comparison at the rule-chosen rates.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-b87acd63
D=$S/c3/data/code
until grep -q "ablations done" $S/c4/runs/ablation/launch.log 2>/dev/null; do sleep 30; done
OUT=$S/c4/runs/ablation-seed2
mkdir -p $OUT
echo "source b87acd63; binary $(sha256sum $B | cut -d' ' -f1); lr 0.004; seed 2; started $(date -u +%FT%TZ)" >> $OUT/launch.log
common=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt arch=geometric seed=2
  steps=1000 batch=16 lr=0.004 warmup=100 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=250 sample_tokens=96)
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/lorentz pattern=rrarra read=lorentz rotation=true > $OUT/lorentz.log 2>&1 &
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/dot pattern=rrarra read=dot rotation=true > $OUT/dot.log 2>&1 &
wait
echo "seed2 done $(date -u +%FT%TZ)" >> $OUT/launch.log
echo "chain2: seed-2 pair done, main starting $(date -u +%FT%TZ)" >> $S/c4/chain.log
$S/c4/launch_main.sh 0.002 0.004 1
echo "chain2 done $(date -u +%FT%TZ)" >> $S/c4/chain.log
