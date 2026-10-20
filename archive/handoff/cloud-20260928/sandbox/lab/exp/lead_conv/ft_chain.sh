#!/bin/bash
# stage 2a: waits for the conversion chains, then runs fine-tunes sequentially (1 thread)
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/lead_conv
export PYTHONPATH=$S/leadlib:$S/pylib OMP_NUM_THREADS=1 XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1"
until [ -f "$1" ]; do sleep 30; done
shift
for job in "$@"; do
  var=${job%_s*}; seed=${job##*_s}
  [ -f ft_$job.json ] && continue
  python3 ft.py --var $var --seed $seed --steps 600 --out ft_$job.json > ft_$job.log 2>&1
  echo "EXIT $? $job" >> ft_$job.log
done
