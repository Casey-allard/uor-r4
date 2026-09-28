#!/usr/bin/env bash
# D1, pre-registered in #1453 §4: is the quaternion transport in the main-line `rrarra` core load-bearing?
# Arms: quaternion transport (rotation=true) against identity transport (rotation=false), both at MLP 749
# (stack_mlp=749), Lorentz reads, seeds 1 and 2, 1,000 updates on the repository code split, cycle-4 settings.
# One seed's pair runs at a time, two threads per arm. Idempotent: an arm whose latest root holds report.json is
# done; otherwise it resumes from the newest root that holds a checkpoint, in a new *_rN root (at most 3 launches
# per arm per invocation, so a crash cannot loop).
set -uo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=${D1_BIN:?set D1_BIN to the D1 executable}
D=$S/c3/data/code
R=$S/d1/runs
LOG=$S/d1/chain.log
latest() { local base=$1 n=0 last=""; [ -d "$base" ] && last=$base; while [ -d "${base}_r$((n+1))" ]; do n=$((n+1)); last="${base}_r$n"; done; echo "$last"; }
next_root() { local base=$1; [ -d "$base" ] || { echo "$base"; return; }; local n=1; while [ -d "${base}_r$n" ]; do n=$((n+1)); done; echo "${base}_r$n"; }
latest_ckpt() { local base=$1 n=1 best=""; [ -d "$base/checkpoint" ] && best=$base; while [ -d "${base}_r$n" ]; do [ -d "${base}_r$n/checkpoint" ] && best="${base}_r$n"; n=$((n+1)); done; echo "$best"; }
common=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt arch=geometric pattern=rrarra
  read=lorentz stack_mlp=749 lr=0.004 steps=1000 batch=16 warmup=100 eval_every=250 eval_windows=64 final_windows=512
  checkpoint_every=50 sample_tokens=96)
arm() {
  local base=$R/$1; shift
  local launch
  for launch in 1 2 3; do
    local last; last=$(latest "$base")
    if [ -n "$last" ] && [ -f "$last/report.json" ]; then echo "done $base ($last)" >> $LOG; return 0; fi
    local p; for p in $(pgrep -x geometric-stack); do grep -qs "out=$base" /proc/$p/cmdline && return 0; done
    local ckpt root resume=()
    ckpt=$(latest_ckpt "$base"); root=$(next_root "$base")
    [ -n "$ckpt" ] && resume=(resume="$ckpt/checkpoint")
    echo "d1 launch $root ${resume[*]:-fresh} $(date -u +%FT%TZ)" >> $LOG
    RAYON_NUM_THREADS=2 "$B" train "${common[@]}" "$@" out="$root" ${resume[@]+"${resume[@]}"} > "$root.log" 2>&1
    echo "d1 exit $? $root $(date -u +%FT%TZ)" >> $LOG
  done
  echo "d1 gave up on $base after 3 launches $(date -u +%FT%TZ)" >> $LOG
}
for seed in 1 2; do
  arm rot_s$seed rotation=true seed=$seed &
  arm id_s$seed rotation=false seed=$seed &
  wait
done
echo "d1 all done $(date -u +%FT%TZ)" >> $LOG
