#!/usr/bin/env bash
# The lab's queue after the container restart of 2026-09-27 (~17:00Z), which rolled the sandbox back to its 08:24Z
# state and moved it to a Cascade Lake host. Replaces c5/pipeline2.sh, pipeline2-gptq.sh, pipeline3.sh and
# pipeline4.sh, which the rollback lost; their settings are kept. In order:
#   0. the cycle-4 main comparison (c4/launch_main.sh, relaunched 17:08Z): waits while it runs, and resumes it from its
#      last checkpoint if it stopped unfinished (after a restart, rerun this script alone);
#   A. integer serving of the main pair with the cycle-5 build (main 72538ffb plus the memory layer, x86-64-v4):
#      round-to-nearest and GPTQ exports (GPTQ calibrated on 64 windows of the repository training split, damp 0.01),
#      each scored on the 512 final windows beside the float model and its grid reference; integer continuations of
#      both (3 prompts of 64 tokens, 96 new, temperature 0.8, top-k 40, seed 1);
#   B. cycle 5 (docs/integration/memory-layers-cycle5-2026-09-27.md, amended before any arm ran): the reads-only
#      Lorentz base re-run and the four memory arms with the cycle-5 build, two at a time with two threads each; the
#      reads-only Dot seed 2 (cycle-4 executable b87acd63, cycle 4 §7) shares the last pair;
#   C. the reads-only Lorentz stack at the main comparison's full exposure (b87acd63, four threads; cycle 4 §6).
# Idempotent: a sealed report or manifest means done; a checkpoint resumes into a new *_rN root.
set -uo pipefail
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
# Both executables are built on this host for x86-64-v3 (AVX2, FMA): the pinned host-tuned build of b87acd63 traps
# on this CPU (AVX512-VBMI and later instructions), so b87acd63 is rebuilt from source.
B4=$S/c4/bin/geometric-stack-b87acd63-v3
B5=$S/c5/bin/geometric-stack-97c8b6ea-v3
D=$S/c3/data/code
REG=$S/c4/data/registry.u16
LOG=$S/c5/chain.log
mkdir -p $S/c5
exec 8>$S/c5/pipeline-post.lock
flock -n 8 || { echo "pipeline-post already running"; exit 0; }
latest() { local base=$1 n=0 last=""; [ -d "$base" ] && last=$base; while [ -d "${base}_r$((n+1))" ]; do n=$((n+1)); last="${base}_r$n"; done; echo "$last"; }
next_root() { local base=$1; [ -d "$base" ] || { echo "$base"; return; }; local n=1; while [ -d "${base}_r$n" ]; do n=$((n+1)); done; echo "${base}_r$n"; }
done_run() { local last; last=$(latest "$1"); [ -n "$last" ] && [ -f "$last/report.json" ]; }
# The newest attempt root that holds a checkpoint (an attempt stopped before its first checkpoint leaves none).
latest_ckpt() { local base=$1 n=1 best=""; [ -d "$base/checkpoint" ] && best=$base; while [ -d "${base}_r$n" ]; do [ -d "${base}_r$n/checkpoint" ] && best="${base}_r$n"; n=$((n+1)); done; echo "$best"; }
# run BASE BINARY THREADS ARGS...: train into BASE (or its next *_rN root), resuming the latest checkpoint.
run() {
  local base=$1 bin=$2 threads=$3; shift 3
  if done_run "$base"; then echo "done $base ($(latest "$base"))" >> $LOG; return 0; fi
  local ckpt root resume=()
  ckpt=$(latest_ckpt "$base"); root=$(next_root "$base")
  if [ -n "$ckpt" ]; then resume=(resume="$ckpt/checkpoint"); fi
  echo "launch $root ${resume[*]:-fresh} $(date -u +%FT%TZ) binary $(basename $bin)" >> $LOG
  RAYON_NUM_THREADS=$threads "$bin" train "$@" out="$root" ${resume[@]+"${resume[@]}"} > "$root.log" 2>&1
  echo "exit $? $root $(date -u +%FT%TZ)" >> $LOG
}
echo "pipeline-post start $(date -u +%FT%TZ); b87acd63-v3 $(sha256sum $B4 | cut -d' ' -f1); 97c8b6ea-v3 $(sha256sum $B5 | cut -d' ' -f1)" >> $LOG

# 0. The main comparison.
M=$S/c4/runs/main
main=(train=$D/train.u16,$REG train_weights=1,1 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt seed=1
  steps=7324 batch=16 warmup=200 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=50 sample_tokens=128)
launches=0
until done_run $M/transformer_s1 && done_run $M/geometric_s1; do
  if pgrep -f "$(basename $B4) train .*out=$M/" > /dev/null; then sleep 60; continue; fi
  launches=$((launches + 1))
  if [ $launches -gt 3 ]; then echo "main comparison unfinished after 3 launches; stopping $(date -u +%FT%TZ)" >> $LOG; exit 1; fi
  echo "main comparison not running and unfinished; launch $launches $(date -u +%FT%TZ)" >> $LOG
  run $M/transformer_s1 $B4 2 "${main[@]}" arch=transformer lr=0.002 &
  run $M/geometric_s1 $B4 2 "${main[@]}" arch=geometric pattern=rrarra read=lorentz rotation=true lr=0.004 &
  wait
done
echo "main comparison done $(date -u +%FT%TZ)" >> $LOG

# A. Integer serving of the main pair.
[ -x $B5 ] || { echo "missing $B5; stopping before stage A $(date -u +%FT%TZ)" >> $LOG; exit 1; }
echo "stage A start $(date -u +%FT%TZ); cycle-5 build $(sha256sum $B5 | cut -d' ' -f1)" >> $LOG
L=$S/c5/lut-main
mkdir -p $L
sealed() { local l; l=$(latest "$1"); [ -n "$l" ] && [ -f "$l/manifest.json" ]; }
integer() {
  local name=$1 model
  model=$(latest $M/$name)
  sealed $L/export-$name || RAYON_NUM_THREADS=1 $B5 export model=$model/model out=$(next_root $L/export-$name)
  sealed $L/gptq-export-$name || RAYON_NUM_THREADS=1 $B5 export model=$model/model calibration=$D/train.u16 \
    calibration_windows=64 damp=0.01 out=$(next_root $L/gptq-export-$name)
  local kind artifact
  for kind in export gptq-export; do
    artifact=$(latest $L/$kind-$name)/model.lut
    sealed $L/eval512-$kind-$name || RAYON_NUM_THREADS=1 $B5 lut-evaluate artifact=$artifact valid=$D/valid.u16 \
      lens=$D/lens.u16 model=$model/model windows=512 threads=1 reference=true out=$(next_root $L/eval512-$kind-$name)
    sealed $L/samples-$kind-$name || RAYON_NUM_THREADS=1 $B5 lut-sample artifact=$artifact valid=$D/valid.u16 \
      merges=$D/merges.txt prompts=3 prompt_tokens=64 sample_tokens=96 temperature=0.8 top_k=40 seed=1 threads=1 \
      out=$(next_root $L/samples-$kind-$name)
  done
  echo "integer done $name $(date -u +%FT%TZ)" >> $LOG
}
integer transformer_s1 > $L/transformer_s1.out 2>&1 &
integer geometric_s1 > $L/geometric_s1.out 2>&1 &
wait

# B. Cycle 5, and the reads-only Dot seed 2.
C5=$S/c5/runs
mkdir -p $C5
c5=(train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 merges=$D/merges.txt batch=16 lr=0.004 warmup=100
  steps=1000 eval_every=250 eval_windows=64 final_windows=512 checkpoint_every=50 sample_tokens=96 arch=geometric
  seed=1 pattern=aaaaaa read=lorentz rotation=false)
mem=(memory_layers=3 memory_heads=4 memory_top_k=32)
run $C5/base $B5 2 "${c5[@]}" &
run $C5/memory_dot $B5 2 "${c5[@]}" "${mem[@]}" memory_score=dot &
wait
run $C5/memory_lorentz $B5 2 "${c5[@]}" "${mem[@]}" memory_score=lorentz &
run $C5/memory_h4 $B5 2 "${c5[@]}" "${mem[@]}" memory_codebook=h4 &
wait
run $C5/memory_e8 $B5 2 "${c5[@]}" "${mem[@]}" memory_codebook=e8 &
run $S/c4/runs/ablation-readsonly/dot_s2 $B4 2 train=$D/train.u16 valid=$D/valid.u16 lens=$D/lens.u16 \
  merges=$D/merges.txt batch=16 lr=0.004 warmup=100 steps=1000 eval_every=250 eval_windows=64 final_windows=512 \
  checkpoint_every=50 sample_tokens=96 arch=geometric seed=2 pattern=aaaaaa read=dot rotation=false &
wait

# C. The reads-only Lorentz stack at full exposure.
run $M/readsonly_s1 $B4 4 "${main[@]}" arch=geometric pattern=aaaaaa \
  read=lorentz rotation=false lr=0.004
echo "pipeline-post done $(date -u +%FT%TZ)" >> $LOG
