#!/usr/bin/env bash
# Keep a copy of each main-comparison arm's newest checkpoint: a completed run deletes its own checkpoint, so the
# last resumable state (step 7,300) would otherwise be lost. Copies only complete checkpoints (state.json present,
# copied into a staging directory, then renamed); stops when both final reports exist.
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
M=$S/c4/runs/main
K=$S/c4/ckpt-keep
mkdir -p $K
while :; do
  for a in transformer_s1 geometric_s1; do
    src=$M/${a}_r4/checkpoint
    [ -f $src/state.json ] || continue
    step=$(python3 -c "import json; print(json.load(open('$src/state.json'))['step'])" 2>/dev/null) || continue
    have=$(python3 -c "import json; print(json.load(open('$K/$a/state.json'))['step'])" 2>/dev/null || echo -1)
    if [ "$step" != "$have" ]; then
      rm -rf $K/$a.staging && cp -r $src $K/$a.staging 2>/dev/null && \
      [ "$(python3 -c "import json; print(json.load(open('$K/$a.staging/state.json'))['step'])" 2>/dev/null)" = "$step" ] && \
      rm -rf $K/$a && mv $K/$a.staging $K/$a && echo "$(date -u +%FT%TZ) kept $a step $step" >> $K/log
    fi
  done
  [ -f $M/transformer_s1_r4/report.json ] && [ -f $M/geometric_s1_r4/report.json ] && break
  sleep 20
done
echo "$(date -u +%FT%TZ) done" >> $K/log
