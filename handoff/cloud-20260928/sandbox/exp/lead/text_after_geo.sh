#!/bin/bash
cd "$(dirname "$0")"
until [ -f geo/worker_0.done ] && [ -f geo/worker_1.done ] && [ -f geo/worker_2.done ] && [ -f geo/worker_3.done ]; do sleep 15; done
c=0
for k in hyb_dot hyb_ham hyb_e8 hyb_cos; do
  PYTHONPATH=../../leadlib taskset -c $c python3 textlm.py --kind $k --steps 1000 --seed 0 --out full_${k}_s0.json > full_${k}_s0.log 2>&1 &
  c=$((c+1))
done
wait
echo DONE > text_hyb.done
