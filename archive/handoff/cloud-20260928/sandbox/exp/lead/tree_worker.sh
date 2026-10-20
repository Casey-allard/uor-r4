#!/bin/bash
cd "$(dirname "$0")"
core=$1
n=0
while IFS= read -r line; do
  n=$((n+1)); [ -z "$line" ] && continue
  if mkdir "tree/claim_$n" 2>/dev/null; then
    out=$(echo "$line" | sed -E 's/.*--out ([^ ]+).*/\1/'); log="${out%.json}.log"
    PYTHONPATH=../../leadlib taskset -c $core python3 $line > "$log" 2>&1
  fi
done < tree_jobs.txt
echo DONE > tree/worker_$core.done
