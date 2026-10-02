#!/usr/bin/env bash
# Full-dose chat-v0 run that ISOLATES DOSE.
#
# Two errors this fixes in the previous ladder:
#  1. that ladder trained each stage FROM SCRATCH, so the three stages cost the
#     SUM (8.2-12.4 h) rather than the full epoch (4.7-7.1 h) -- a 1.75x
#     overshoot against an objective whose point was to reduce time;
#  2. from scratch also REMOVES THE PROSE INIT, which conflates "more dose"
#     with "lost the base that produced the winning arm". The winning arm is
#     prose init -> chat-v0 LM -> response phase, so dose must be varied with
#     the init held fixed.
#
# So: one LM run from the prose init, to a full pass over the store, with a
# dense eval cadence (dev NLL every ~2,518 updates) so the DOSE-RESPONSE is
# readable from the log even if the run is stopped early. Then one response
# phase at 8x the previous dose.
set -uo pipefail
P="$HOME/uor-r4-local/chat-v0-20260925/prepared"
T="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
I="$HOME/uor-r4-worktrees/stack-prose-reports/main-10000/geometric-s1/model"
O="$HOME/uor-r4-worktrees/stack-prose-reports/chat-fulldose-1"
export CARGO_TARGET_DIR="$HOME/.cache/uor-r4-opencode-target"
export RAYON_NUM_THREADS=8
B="$CARGO_TARGET_DIR/release/examples/geometric-stack"
mkdir -p "$O"

# Phase A: prose init + a FULL pass over chat-v0 (20,149 updates = 82.5M tokens).
# 8 eval points give the dose-response directly.
"$B" train train="$P/train/tokens.u16" valid="$P/heldout/tokens.u16" tokenizer="$T" out="$O/lm" \
  arch=geometric pattern=rrarra read=lorentz rotation=true seed=1 init="$I" \
  steps=20149 batch=16 lr=0.004 warmup=1008 eval_every=2518 eval_windows=64 final_windows=256 \
  checkpoint_every=2518 sample_tokens=0 > "$O/lm.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] LM done rc=$?"

# Phase B: 8x the previous response dose, from the full-dose LM.
"$B" dialogue-train out="$O/dialogue" tokenizer="$T" \
  train_tokens="$P/train/tokens.u16" train_mask="$P/train/response_mask.u8" train_manifest="$P/train/manifest.json" \
  dev_tokens="$P/heldout/tokens.u16" dev_mask="$P/heldout/response_mask.u8" dev_manifest="$P/heldout/manifest.json" \
  init="$O/lm/model" policy=full_prefix data_seed=20260929 steps=8192 batch=16 lr=0.001 \
  eval_every=512 dev_seed=20260930 dev_per_source=32 checkpoint_every=512 max_new_tokens=32 \
  > "$O/dialogue.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] response phase done rc=$?"
echo "FULLDOSE COMPLETE"
