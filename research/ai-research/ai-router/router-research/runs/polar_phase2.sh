#!/usr/bin/env bash
set -euo pipefail
mkdir -p results/raw
TS=$(date +"%Y%m%d_%H%M%S")
LOG="results/raw/polar_phase2_${TS}.log"
echo "Writing log to $LOG"
PYTHONHASHSEED=0 python hyperbolic_router_so8.py --mode anis --K 8 --delta_r 3.0 --seed 0 \
  --learn_so8 0 --learn_scale 1 --scale_mode radial --radial_bins 10 \
  --chart_beta 50.0 --extra_budget 96 --max_slots_per_bucket 8 \
  --sector_mode phase2 --phase_dims 0,1 --time_pressure_lambda 0.0 \
  --run_tag polar_phase2 | tee "$LOG"
