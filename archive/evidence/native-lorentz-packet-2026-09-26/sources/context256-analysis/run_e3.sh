#!/bin/bash
# after each ctx256 run seals: dump exact read q/k with math2's probe, then per-event analysis for both seed pairs
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/lead_ctx256
export OMP_NUM_THREADS=1 RAYON_NUM_THREADS=1
for m in dot_s1 dot_s2 lorentzflat_s1 lorentzflat_s2; do
  root=$(ls -d $S/c3/runs/ctx256/${m}*/ 2>/dev/null | while read d; do [ -d $d/model ] && echo $d; done | tail -1)
  until [ -n "$root" ]; do sleep 120; root=$(ls -d $S/c3/runs/ctx256/${m}*/ 2>/dev/null | while read d; do [ -d $d/model ] && echo $d; done | tail -1); done
  [ -f e3/$m.params.json ] || nice $S/target/release/probe ${root}model $S/c3/data/code/valid.u16 800 64 e3/$m >> e3/jobs.log 2>&1
done
PYTHONPATH=$S/pylib nice python3 e3_analyze_ctx256.py > e3_analysis.log 2>&1
echo done > e3/ALL_DONE
