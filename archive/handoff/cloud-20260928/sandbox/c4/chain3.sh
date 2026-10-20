#!/usr/bin/env bash
# Replaces chain2.sh after its seed-2 Lorentz/Dot pair started (2026-09-27 07:46Z). The reads-only Lorentz
# ablation (2.5531) beat every other arm, including the RoPE control (2.7680), so the attribution between
# the Lorentz score and the read layer's learned age bias / NoRead needs the reads-only Dot arm. After the
# running seed-2 pair: reads-only Dot (seed 1) beside reads-only Lorentz (seed 2), then the main comparison.
set -euo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-b87acd63
D=$S/c3/data/code
until [ -f $S/c4/runs/ablation-seed2/lorentz/report.json ] && [ -f $S/c4/runs/ablation-seed2/dot/report.json ]; do sleep 30; done
echo "seed2 done $(date -u +%FT%TZ)" >> $S/c4/runs/ablation-seed2/launch.log
OUT=$S/c4/runs/ablation-readsonly
mkdir -p $OUT
echo "source b87acd63; binary $(sha256sum $B | cut -d' ' -f1); lr 0.004; started $(date -u +%FT%TZ)" >> $OUT/launch.log
common=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt arch=geometric pattern=aaaaaa rotation=false
  steps=1000 batch=16 lr=0.004 warmup=100 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=250 sample_tokens=96)
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/dot_s1 read=dot seed=1 > $OUT/dot_s1.log 2>&1 &
RAYON_NUM_THREADS=2 $B train "${common[@]}" out=$OUT/lorentz_s2 read=lorentz seed=2 > $OUT/lorentz_s2.log 2>&1 &
wait
echo "readsonly pair done $(date -u +%FT%TZ)" >> $OUT/launch.log
echo "chain3: reads-only pair done, main starting $(date -u +%FT%TZ)" >> $S/c4/chain.log
$S/c4/launch_main.sh 0.002 0.004 1
echo "chain3 done $(date -u +%FT%TZ)" >> $S/c4/chain.log
