#!/bin/bash
cd "$(dirname "$0")"
for k in quat diag quat2i complex attn; do
  PYTHONPATH=../../leadlib taskset -c 3 python3 textlm.py --kind $k --steps 1000 --seed 0 --out full_${k}_s0.json > full_${k}_s0.log 2>&1
done
echo DONE > run_all.done
