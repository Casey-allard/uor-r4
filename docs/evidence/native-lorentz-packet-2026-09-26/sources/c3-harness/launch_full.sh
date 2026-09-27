#!/bin/bash
# Full comparison wave launcher: code Dot/Lorentz (Dot-matched init, repo code), 2,000 steps, models saved.
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3
cd $S
run() { # data geometry seed
  local name=$1_$2_s$3
  RAYON_NUM_THREADS=1 nohup ./bin_example train=data/$1/train.u16 valid=data/$1/valid.u16 lens=data/$1/lens.u16 \
    out=runs/full/$name geometry=$2 seed=$3 width=128 context=128 batch=16 steps=2000 \
    eval_every=250 eval_windows=32 final_windows=512 save_model=true > runs/full/$name.log 2>&1 &
}
for spec in "$@"; do run $spec; done
