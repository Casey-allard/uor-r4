#!/bin/bash
# usage: chain.sh var1 var2 ... ; runs sequentially, 1 thread, 400 KD steps per stage
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/lead_conv
export PYTHONPATH=$S/leadlib:$S/pylib OMP_NUM_THREADS=1 XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1"
for v in "$@"; do
  [ -f $v.json ] && continue
  python3 conv.py --var $v --steps 400 --out $v.json > $v.log 2>&1
  echo "EXIT $? $v" >> $v.log
done
