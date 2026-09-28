#!/bin/bash
# Main run (resumable: re-running this script continues from run1/ckpt.npz).
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
W=$S/lab/exp/small_llama/work
export XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1"
export OMP_NUM_THREADS=1 PYTHONPATH=$S/leadlib:$S/pylib
# XLA:CPU mmaps/munmaps a 735 MB temp arena per step from a secondary glibc
# arena; one brk arena that is never trimmed keeps it resident (+38% tok/s).
export MALLOC_ARENA_MAX=1 MALLOC_MMAP_MAX_=0 MALLOC_TRIM_THRESHOLD_=68719476736 MALLOC_TOP_PAD_=268435456
cd $W
exec taskset -c ${CORE:-3} nice python3 train.py --run run1 --steps 2300 --batch 16 --lr 2e-3 \
  --min_lr_frac 0.1 --warmup 100 --wd 0.1 --clip 1.0 --beta2 0.95 --seed 1 \
  --eval_every 200 --ckpt_every_s 150 --max_seconds 2700
