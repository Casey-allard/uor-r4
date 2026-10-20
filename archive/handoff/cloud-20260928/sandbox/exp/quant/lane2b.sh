#!/bin/bash
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/quant/
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export PYTHONPATH=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/pylib:.
while pgrep -f "sweep.py mvt3 TQ-qr" >/dev/null; do sleep 3; done
echo "EXIT mvt3 A (prev)"
./run_ds.sh mvt3 B
for ds in outlier aniso; do ./run_ds.sh $ds A; ./run_ds.sh $ds B; done
timeout 1500 python3 sweep.py aniso "D4-rht,E8-rht,D4-none,E8-none" C > run_aniso_C.out 2>&1; echo "EXIT aniso C $?"
timeout 1500 python3 sweep.py t3 "D4-rht,E8-rht" C > run_t3_C.out 2>&1; echo "EXIT t3 C $?"
echo "LANE2 DONE"
