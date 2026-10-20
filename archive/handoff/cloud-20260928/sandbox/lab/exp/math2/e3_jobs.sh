#!/bin/bash
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/math2
export OMP_NUM_THREADS=1 RAYON_NUM_THREADS=1
for m in code_dot_s3_resumed code_dot_s4_resumed code_lorentzflat_s4_resumed code_dot_s2_r2 code_lorentzflat_s2_r2 code_lorentz_s3_resumed; do
  o=e3/$(echo $m | sed 's/_resumed//; s/_r2//')
  [ -f $o.params.json ] || $S/target/release/probe $S/c3/runs/full/$m/model $S/c3/data/code/valid.u16 800 64 $o >> e3/jobs.log 2>&1
done
for m in wiki_dot_s1 wiki_lorentzflat_s1; do
  [ -f e3/$m.params.json ] || $S/target/release/probe $S/c3/runs/full/$m/model $S/c3/data/wiki/valid.u16 800 64 e3/$m >> e3/jobs.log 2>&1
done
echo done > e3/ALL_DONE
