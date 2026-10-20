#!/bin/bash
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/quant/
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export PYTHONPATH=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/pylib:.
until grep -q "LANE1 DONE" lane1.log; do sleep 5; done
F="TQ-qr-c,P2-rht-c,GS4-rht-vec-c,GS4-rht-abs-c,VQ4km-rht-c,D4-rht-c,E8-rht-c"
for ds in outlier aniso; do timeout 900 python3 serving.py $ds center > serving_${ds}_c.out 2>&1; echo "EXIT serving-c $ds $?"; done
for ds in outlier aniso; do timeout 1500 python3 sweep.py $ds "$F" D > run_${ds}_D.out 2>&1; echo "EXIT $ds D $?"; done
echo "LANE3 DONE"
