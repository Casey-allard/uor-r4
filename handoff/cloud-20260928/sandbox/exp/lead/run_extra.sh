#!/bin/bash
cd "$(dirname "$0")"
while [ ! -f run_all.done ]; do sleep 30; done
for spec in "quatint 0" "quat 1" "diag 1"; do
  set -- $spec
  PYTHONPATH=../../leadlib taskset -c 3 python3 textlm.py --kind $1 --steps 1000 --seed $2 --out full_${1}_s$2.json > full_${1}_s$2.log 2>&1
done
echo DONE > run_extra.done
