#!/usr/bin/env bash
# Cycle 4 pipeline after the 08:22Z container reboot; idempotent, so it can be relaunched after any restart.
# For each planned run: a final report means done; otherwise resume the latest attempt's checkpoint into a
# new root <name>_rN (report roots are never reused); with no checkpoint, start fresh in a new root.
# Stages, in order: seed-2 Lorentz/Dot pair, reads-only Dot(s1)/Lorentz(s2) pair, main comparison pair.
# 08:45Z: checkpoint_every 250 -> 50 after a second reboot (~08:40Z) killed the seed-2 pair before its first checkpoint;
# checkpoint cadence is outside the resume lineage and does not change the computation (the sampler state is saved).
set -uo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
B=$S/c4/bin/geometric-stack-b87acd63
D=$S/c3/data/code
R=$S/c4/data/registry.u16
LOG=$S/c4/chain.log
exec 9>$S/c4/pipeline.lock
flock -n 9 || { echo "pipeline already running"; exit 0; }
echo "pipeline start $(date -u +%FT%TZ)" >> $LOG

# latest attempt directory for a run name: NAME, NAME_r1, NAME_r2, ...
latest() { local base=$1 n=0 last=""; [ -d "$base" ] && last=$base; while [ -d "${base}_r$((n+1))" ]; do n=$((n+1)); last="${base}_r$n"; done; echo "$last"; }
next_root() { local base=$1; [ -d "$base" ] || { echo "$base"; return; }; local n=1; while [ -d "${base}_r$n" ]; do n=$((n+1)); done; echo "${base}_r$n"; }

# run NAME THREADS ARGS...: completes or resumes one run in the foreground
run() {
  local base=$1 threads=$2; shift 2
  local last; last=$(latest "$base")
  if [ -n "$last" ] && [ -f "$last/report.json" ]; then echo "done $base ($last)" >> $LOG; return 0; fi
  local root; root=$(next_root "$base")
  local resume=()
  if [ -n "$last" ] && [ -d "$last/checkpoint" ]; then resume=(resume="$last/checkpoint"); fi
  echo "launch $root ${resume[*]:-fresh} $(date -u +%FT%TZ)" >> $LOG
  RAYON_NUM_THREADS=$threads "$B" train "$@" out="$root" ${resume[@]+"${resume[@]}"} > "$root.log" 2>&1
  local status=$?
  echo "exit $status $root $(date -u +%FT%TZ)" >> $LOG
  return $status
}

code=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt batch=16 lr=0.004 warmup=100
  steps=1000 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=50 sample_tokens=96 arch=geometric)

A=$S/c4/runs/ablation-seed2
run $A/lorentz 2 "${code[@]}" seed=2 pattern=rrarra read=lorentz rotation=true &
run $A/dot 2 "${code[@]}" seed=2 pattern=rrarra read=dot rotation=true &
wait

A=$S/c4/runs/ablation-readsonly
mkdir -p $A
run $A/dot_s1 2 "${code[@]}" seed=1 pattern=aaaaaa read=dot rotation=false &
run $A/lorentz_s2 2 "${code[@]}" seed=2 pattern=aaaaaa read=lorentz rotation=false &
wait

M=$S/c4/runs/main
mkdir -p $M
main=(train=$D/train.u16,$R train_weights=1,1 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt seed=1
  steps=7324 batch=16 warmup=200 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=50 sample_tokens=128)
run $M/transformer_s1 2 "${main[@]}" arch=transformer lr=0.002 &
run $M/geometric_s1 2 "${main[@]}" arch=geometric pattern=rrarra read=lorentz rotation=true lr=0.004 &
wait
echo "pipeline done $(date -u +%FT%TZ)" >> $LOG
