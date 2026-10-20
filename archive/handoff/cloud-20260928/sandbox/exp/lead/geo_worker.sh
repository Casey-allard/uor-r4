#!/bin/bash
cd "$(dirname "$0")"
core=$1
while read -r kind dk steps; do
  [ -z "$kind" ] && continue
  tag="${kind}_dk${dk}"
  if mkdir "geo/claim_$tag" 2>/dev/null; then
    PYTHONPATH=../../leadlib taskset -c $core python3 geoattn.py --kind $kind --dk $dk --steps $steps --out geo/$tag.json > geo/$tag.log 2>&1
  fi
done < geo_jobs.txt
echo DONE > geo/worker_$core.done
