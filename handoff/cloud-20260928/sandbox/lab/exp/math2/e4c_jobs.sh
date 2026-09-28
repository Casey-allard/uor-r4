#!/bin/bash
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/math2
export OMP_NUM_THREADS=1 XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1" PYTHONPATH=$S/leadlib:$S/pylib
run() { [ -f e4c/$1.json ] || timeout 1500 python3 $S/exp/lead/geoattn.py --task tree --tree_file $S/exp/lead/data/code_tree.npz ${@:2} --out e4c/$1.json > e4c/$1.log 2>&1; }
run code_dot_dk64 --kind dot --dk 64 --D 48 --steps 1500
run code_hyp_dk64 --kind hyp --dk 64 --D 48 --steps 1500
run code_dot_dk64_long --kind dot --dk 64 --D 48 --steps 4500
echo done > e4c/ALL_DONE
