#!/bin/bash
# usage: run_ds.sh <dataset> <group>   (group A|B|C)
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/quant/
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export PYTHONPATH=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/pylib:.
case "$2" in
  A) F="TQ-qr,TQ-rht,SCchan,P2-rht,P2-none,PQ-rht,PQ-none,TQprod-qr,SC-pow2,SC-opt";;
  B) F="GS4-rht-vec,GS4-none-vec,GS4-rht-abs,GS4-none-abs,GS4rnd-rht-vec,GS4skm-rht-vec,VQ4km-rht";;
  C) F="D4-rht,E8-rht,D4-none,E8-none";;
esac
timeout 1500 python3 sweep.py "$1" "$F" "$2" > "run_$1_$2.out" 2>&1
echo "EXIT $1 $2 $?"
