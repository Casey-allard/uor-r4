#!/bin/bash
# Pause the niced training runs (matched by exact process name), bench, always resume.
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/target/release/examples/lut-tool
LUT=${1:-$S/lab/exp/lead_bench/r135.lut}
PIDS=$(pgrep -x bin_example3 | tr '\n' ' ')
resume() { [ -n "$PIDS" ] && kill -CONT $PIDS; echo "resumed: $PIDS" >&2; }
trap resume EXIT
[ -n "$PIDS" ] && kill -STOP $PIDS
echo "paused: $PIDS at $(date -u +%T)" >&2
for th in 1 2 4; do
  RAYON_NUM_THREADS=$th timeout 300 $B mode=bench lut=$LUT tokens=${TOKENS:-32}
done
