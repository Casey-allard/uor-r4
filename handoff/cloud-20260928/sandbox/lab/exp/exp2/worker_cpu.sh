#!/bin/bash
# usage: worker_cpu.sh jobs_file core wait_for_json   (CPU-time limit 900 s instead of a wall-clock timeout)
cd "$(dirname "$0")"
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
export PYTHONPATH=$S/leadlib:$S/pylib XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1" OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
while [ -n "$3" ] && [ ! -f "$3" ]; do sleep 10; done
while IFS= read -r line; do
  [ -z "$line" ] && continue
  out=$(echo "$line" | sed -E 's/.*--out ([^ ]+).*/\1/'); log="${out%.json}.log"
  if [ -f "$out" ]; then continue; fi
  echo "CMD: python3 geoattn_exp2.py $line" > "$log"
  s=$(date +%s)
  ( ulimit -t 900; exec taskset -c $2 python3 geoattn_exp2.py $line ) >> "$log" 2>&1
  echo "EXIT $? WALL $(( $(date +%s) - s ))s" >> "$log"
done < "$1"
echo DONE > "$1.done"
