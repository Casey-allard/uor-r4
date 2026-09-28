#!/bin/bash
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/exp/quant/
until grep -q "EXIT gauss B" lane2.log; do sleep 5; done
for ds in t3 mvt3 outlier aniso; do ./run_ds.sh $ds A; ./run_ds.sh $ds B; done
echo "LANE2 DONE"
