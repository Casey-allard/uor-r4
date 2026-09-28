#!/bin/bash
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/math2
export OMP_NUM_THREADS=1 XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1" PYTHONPATH=$S/leadlib:$S/pylib
run() { [ -f e4/$1.json ] || timeout 900 python3 e4_crossover.py ${@:2} --out e4/$1.json > e4/$1.log 2>&1; }
run d5_dot_dk64 --kind dot --dk 64 --depth 5 --D 48 --evalD 48,96,192
run d5_hyp_dk64 --kind hyp --dk 64 --depth 5 --D 48 --evalD 48,96,192
run d5_dot_dk32 --kind dot --dk 32 --depth 5 --D 48 --evalD 48,96,192
run d5_dot_dk16 --kind dot --dk 16 --depth 5 --D 48 --evalD 48,96,192
run d5_dot_dk8 --kind dot --dk 8 --depth 5 --D 48 --evalD 48,96,192
run d6_dot_dk64 --kind dot --dk 64 --depth 6 --D 96 --evalD 96,384,768
run d6_hyp_dk64 --kind hyp --dk 64 --depth 6 --D 96 --evalD 96,384,768
run d6_hyp_dk16 --kind hyp --dk 16 --depth 6 --D 96 --evalD 96,384,768
run d6_dot_dk16 --kind dot --dk 16 --depth 6 --D 96 --evalD 96,384,768
run d5_hyp_dk32 --kind hyp --dk 32 --depth 5 --D 48 --evalD 48,96,192
run d5_hyp_dk16 --kind hyp --dk 16 --depth 5 --D 48 --evalD 48,96,192
echo done > e4/ALL_DONE
