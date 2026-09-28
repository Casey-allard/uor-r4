#!/bin/bash
# usage: evalworker_wait.sh list core ; list lines: name|file_to_wait_for|args
cd "$(dirname "$0")"
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
export PYTHONPATH=$S/pylib OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
while IFS='|' read -r name waitf args; do
  [ -z "$name" ] && continue
  [ -f "runs/$name.json" ] && continue
  while [ ! -f "$waitf" ]; do sleep 10; done
  sleep 5
  echo "CMD: python3 index_eval.py $args --out runs/$name.json" > runs/$name.log
  s=$(date +%s)
  ( ulimit -t 1200; exec taskset -c $2 python3 index_eval.py $args --out runs/$name.json.tmp ) >> runs/$name.log 2>&1
  rc=$?
  [ $rc -eq 0 ] && mv runs/$name.json.tmp runs/$name.json
  echo "EXIT $rc WALL $(( $(date +%s) - s ))s" >> runs/$name.log
done < "$1"
echo DONE > "$1.done"
