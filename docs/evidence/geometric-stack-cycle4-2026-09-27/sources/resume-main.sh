#!/usr/bin/env bash
# Resume only the cycle-4 main comparison after a container restart (the follow-on stages of pipeline-post.sh are
# held by the director plan until the comparison's reading is published). Idempotent: an arm whose latest root holds
# report.json is done; otherwise it resumes from the newest root that holds a checkpoint, into a new *_rN root.
set -uo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B4=$S/c4/bin/geometric-stack-b87acd63-v3
D=$S/c3/data/code
REG=$S/c4/data/registry.u16
M=$S/c4/runs/main
LOG=$S/c5/chain.log
latest() { local base=$1 n=0 last=""; [ -d "$base" ] && last=$base; while [ -d "${base}_r$((n+1))" ]; do n=$((n+1)); last="${base}_r$n"; done; echo "$last"; }
next_root() { local base=$1; [ -d "$base" ] || { echo "$base"; return; }; local n=1; while [ -d "${base}_r$n" ]; do n=$((n+1)); done; echo "${base}_r$n"; }
latest_ckpt() { local base=$1 n=1 best=""; [ -d "$base/checkpoint" ] && best=$base; while [ -d "${base}_r$n" ]; do [ -d "${base}_r$n/checkpoint" ] && best="${base}_r$n"; n=$((n+1)); done; echo "$best"; }
main=(train=$D/train.u16,$REG train_weights=1,1 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt seed=1
  steps=7324 batch=16 warmup=200 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=50 sample_tokens=128)
arm() {
  local base=$M/$1; shift
  local last; last=$(latest "$base")
  if [ -n "$last" ] && [ -f "$last/report.json" ]; then echo "done $base ($last)" >> $LOG; return 0; fi
  local p; for p in $(pgrep -x geometric-stack); do grep -qs "out=$base" /proc/$p/cmdline && return 0; done
  local ckpt root resume=()
  ckpt=$(latest_ckpt "$base"); root=$(next_root "$base")
  [ -n "$ckpt" ] && resume=(resume="$ckpt/checkpoint")
  echo "resume-main launch $root ${resume[*]:-fresh} $(date -u +%FT%TZ)" >> $LOG
  RAYON_NUM_THREADS=2 $B4 train "${main[@]}" "$@" out="$root" ${resume[@]+"${resume[@]}"} > "$root.log" 2>&1
  echo "resume-main exit $? $root $(date -u +%FT%TZ)" >> $LOG
}
arm transformer_s1 arch=transformer lr=0.002 &
arm geometric_s1 arch=geometric pattern=rrarra read=lorentz rotation=true lr=0.004 &
wait
