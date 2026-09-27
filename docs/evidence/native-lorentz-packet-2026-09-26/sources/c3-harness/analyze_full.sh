#!/bin/bash
# Key radius / NoRead / copy gate per token for each saved code model, then radius vs brace depth.
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3
cd $S && mkdir -p analysis
for run in "$@"; do
  model=runs/full/$run/model
  [ -d $model ] || { echo "no model for $run"; continue; }
  case $run in
    *dotflat*) env RAYON_NUM_THREADS=1 UOR_ABLATE_READ=dotflat ./bin_keys_abl $model data/code/valid.u16 800 analysis/$run ;;
    *) env RAYON_NUM_THREADS=1 ./bin_keys $model data/code/valid.u16 800 analysis/$run ;;
  esac
done
PYTHONPATH=$S/../pylib python3 radius_depth.py data/code $(for r in "$@"; do echo analysis/$r; done)
