#!/bin/bash
# Start diag s1 on core 0 once the GRU frees it (claim via mkdir so the queue workers skip it),
# then rerun quat s0 with the patched script on core 2 once quat s1 frees it.
cd "$(dirname "$0")"
until [ -f full_gru_s0.json ]; do sleep 10; done
if mkdir claim_diag_s1 2>/dev/null; then
  PYTHONPATH=../../leadlib taskset -c 0 python3 textlm.py --kind diag --steps 1000 --seed 1 --out full_diag_s1.json > full_diag_s1.log 2>&1 &
fi
until [ -f full_quat_s1.json ]; do sleep 10; done
PYTHONPATH=../../leadlib taskset -c 2 python3 textlm.py --kind quat --steps 1000 --seed 0 --out full_quat_s0_rerun.json > full_quat_s0_rerun.log 2>&1
wait
echo DONE > extra_sched.done
