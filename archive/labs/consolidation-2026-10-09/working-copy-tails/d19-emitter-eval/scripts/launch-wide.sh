#!/usr/bin/env bash
# Can binding GENERALISE to relations never trained?
#
# Confound this fixes: my earlier synthetic arm trained on 10 relations and was
# scored on those same 10 (10/10) and on 10 others (0/40). It never trained on a
# WIDE relation set, so "does binding generalise to unseen relations" was never
# asked. This trains on 76 relations disjoint from the panel's 10, then tests on
# the panel. If binding is a general operation, unseen relations should improve;
# if it is per-phrase template completion, they should stay at 0/40.
set -uo pipefail
P="$HOME/uor-r4-local/chat-v0-20260925/prepared"
T="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
MX="${MIX:-$HOME/uor-r4-worktrees/reports/mixed-wide-2}"
INIT="$HOME/uor-r4-worktrees/reports/chat-dose-continue-1/dialogue/model"
O="${OUT:-$HOME/uor-r4-worktrees/reports/wide-bind-1}"
B="${STACK_BIN:-$HOME/.local/share/uor-r4/bin/geometric-stack-9a80e47a}"
export RAYON_NUM_THREADS="${THREADS:-2}"
"$B" dialogue-train out="$O" tokenizer="$T" \
  train_tokens="$MX/tokens.u16" train_mask="$MX/response_mask.u8" train_manifest="$MX/manifest.json" \
  dev_tokens="$P/heldout/tokens.u16" dev_mask="$P/heldout/response_mask.u8" dev_manifest="$P/heldout/manifest.json" \
  init="$INIT" policy=full_prefix data_seed=20260929 steps="${STEPS:-2500}" batch=16 \
  lr="${LR:-0.0002}" eval_every=250 dev_seed=20260930 dev_per_source=32 checkpoint_every=250 \
  max_new_tokens=32 max_seconds="${MAX_SECONDS:-10800}" > "$O.train.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] train done rc=$?"
