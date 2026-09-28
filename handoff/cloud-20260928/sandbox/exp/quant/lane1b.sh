#!/bin/bash
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/quant/
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export PYTHONPATH=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/pylib:.
while pgrep -f "serving.py outlier" >/dev/null; do sleep 3; done
echo "EXIT serving outlier (prev)"
timeout 1500 python3 sweep.py outlier "D4-rht,E8-rht,D4-none,E8-none" C > run_outlier_C.out 2>&1; echo "EXIT outlier C $?"
for ds in mvt3 outlier t3 aniso; do timeout 900 python3 radial_test.py $ds > radial_$ds.out 2>&1; echo "EXIT radial $ds $?"; done
for ds in mvt3 t3 aniso; do timeout 900 python3 serving.py $ds > serving_$ds.out 2>&1; echo "EXIT serving $ds $?"; done
timeout 1500 python3 sweep.py mvt3 "D4-rht,E8-rht" C > run_mvt3_C.out 2>&1; echo "EXIT mvt3 C $?"
echo "LANE1 DONE"
