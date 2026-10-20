#!/bin/bash
cd "$(dirname "$0")"
while [ ! -f run_extra.done ]; do sleep 30; done
PYTHONPATH=../../leadlib taskset -c 2 python3 textlm.py --kind gru --steps 1000 --seed 0 --out full_gru_s0.json > full_gru_s0.log 2>&1
echo DONE > run_extra2.done
