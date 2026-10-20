#!/bin/bash
# Usage: worker.sh CORE  -- claims queued specs with mkdir (atomic) and runs them pinned to CORE
cd "$(dirname "$0")"
core=$1
for spec in "quatint 0" "quat 1" "diag 1"; do
  set -- $spec
  if mkdir "claim_${1}_s$2" 2>/dev/null; then
    PYTHONPATH=../../leadlib taskset -c $core python3 textlm.py --kind $1 --steps 1000 --seed $2 --out full_${1}_s$2.json > full_${1}_s$2.log 2>&1
  fi
done
echo DONE > worker_$core.done
