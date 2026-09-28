#!/bin/bash
# Post-training: export -> Rust probe -> JAX eval -> Rust sample -> JAX continuation.
set -e
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
W=$S/lab/exp/small_llama; B=$S/target/release/examples/kappa-conversion
export XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1"
export OMP_NUM_THREADS=1 RAYON_NUM_THREADS=1 PYTHONPATH=$S/leadlib:$S/pylib
export MALLOC_ARENA_MAX=1 MALLOC_MMAP_MAX_=0 MALLOC_TRIM_THRESHOLD_=68719476736
C="taskset -c ${CORE:-3} nice"
cd $W/work
$C python3 export.py run1/final_params_f32.npz $W/ckpt
sha256sum $W/ckpt/*
echo "=== probe"; $C $B mode=probe model=$W/ckpt tokens=$W/valid.u16 out=$W/probe1 time=256 windows=4 t_grid=1e-8,1,3
python3 -c "import json; r=json.load(open('$W/probe1/probe.json')); print('RUST dot_nll', repr(r['dot_nll']), r['window_starts'], 'seconds', r['seconds'])"
echo "=== jax eval"; $C python3 evaljax.py $W/ckpt $W/valid.u16 --windows 4 --full --f32 run1/final_params_f32.npz --json $W/work/eval_final.json
echo "=== sample (chat template)"; $C $B mode=sample model=$W/ckpt prompt="The history of"
echo "=== sample 200 tokens"; $C $B mode=sample model=$W/ckpt prompt="The history of" tokens=200
echo "=== jax plain continuation"; $C python3 gen.py $W/ckpt "The history of" 300
