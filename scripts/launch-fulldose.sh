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
I="${REPORTS:-$HOME/uor-r4-worktrees/reports}/main-10000/geometric-s1/model"
O="${REPORTS:-$HOME/uor-r4-worktrees/reports}/chat-fulldose-1"
# Resolve the training binary. An earlier revision hardcoded
# CARGO_TARGET_DIR="$HOME/.cache/uor-r4-opencode-target", which a storage cleanup
# DELETED, so every invocation died with "No such file or directory" for the
# binary -- a failure that reads as a missing file, not as a stale build path.
# Resolution order, most-explicit first:
#   1. STACK_BIN               -- an explicit binary, e.g. a pinned revision
#   2. CARGO_TARGET_DIR        -- honour it when the caller sets it
#   3. a prebuilt shared binary -- what the other lanes actually run
#   4. build from source       -- last resort, offline, into a private cache
resolve_stack_bin() {
  if [ -n "${STACK_BIN:-}" ] && [ -x "${STACK_BIN}" ]; then
    printf '%s\n' "$STACK_BIN"; return 0
  fi
  if [ -n "${CARGO_TARGET_DIR:-}" ] \
     && [ -x "$CARGO_TARGET_DIR/release/examples/geometric-stack" ]; then
    printf '%s\n' "$CARGO_TARGET_DIR/release/examples/geometric-stack"; return 0
  fi
  local shared
  for shared in "$HOME"/.local/share/uor-r4/bin/geometric-stack-*; do
    [ -x "$shared" ] || continue
    printf '%s\n' "$shared"; return 0
  done
  local cache="$HOME/.cache/uor-r4-stack-bin"
  export CARGO_TARGET_DIR="$cache"
  ( cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)" \
    && ~/.cargo/bin/cargo build --release -j 2 --features cpu-accelerate \
         -p uor-r4-training --example geometric-stack ) >&2 || return 1
  printf '%s\n' "$cache/release/examples/geometric-stack"
}
B="$(resolve_stack_bin)" || { echo "FATAL: no usable geometric-stack binary"; exit 1; }
[ -x "$B" ] || { echo "FATAL: resolved binary is not executable: $B"; exit 1; }
echo "[$(date -u +%H:%M:%SZ)] binary: $B"
mkdir -p "$O"

# Phase A: prose init + a FULL pass over chat-v0 (20,149 updates = 82.5M tokens).
# 8 eval points give the dose-response directly.
"$B" train train="$P/train/tokens.u16" valid="$P/heldout/tokens.u16" tokenizer="$T" out="$O/lm" \
  arch=geometric pattern=rrarra read=lorentz rotation=true seed=1 init="$I" \
  steps=20149 batch=16 lr=0.004 warmup=1008 eval_every=2518 eval_windows=64 final_windows=256 \
  checkpoint_every=2518 sample_tokens=0 max_seconds="${LM_MAX_SECONDS:-43200}" > "$O/lm.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] LM done rc=$?"

# Phase B: 8x the previous response dose, from the full-dose LM.
"$B" dialogue-train out="$O/dialogue" tokenizer="$T" \
  train_tokens="$P/train/tokens.u16" train_mask="$P/train/response_mask.u8" train_manifest="$P/train/manifest.json" \
  dev_tokens="$P/heldout/tokens.u16" dev_mask="$P/heldout/response_mask.u8" dev_manifest="$P/heldout/manifest.json" \
  init="$O/lm/model" policy=full_prefix data_seed=20260929 steps=8192 batch=16 lr=0.001 \
  eval_every=512 dev_seed=20260930 dev_per_source=32 checkpoint_every=512 max_new_tokens=32 \
  max_seconds="${DIALOGUE_MAX_SECONDS:-10800}" > "$O/dialogue.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] response phase done rc=$?"
echo "FULLDOSE COMPLETE"

# Measure, without waiting for a human. The objective is not "the fit ran" but
# "the fit was measured and reported", and the measurement chain already exists
# as post-epoch-eval.sh -- so it runs here rather than depending on someone
# noticing the fit finished.
#
# This is deliberately non-fatal: if the fit died, post-epoch-eval refuses (it
# requires a sealed report) and its refusal is the useful output. A failure here
# must not lose the fact that the fit itself completed.
echo "[$(date -u +%H:%M:%SZ)] measuring"
SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -x "$SELF/post-epoch-eval.sh" ]; then
  "$SELF/post-epoch-eval.sh" "$O" fulldose >> "$O.eval.log" 2>&1 \
    && echo "[$(date -u +%H:%M:%SZ)] measurement done -> $O.eval.log" \
    || echo "[$(date -u +%H:%M:%SZ)] measurement FAILED (see $O.eval.log)"
else
  echo "[$(date -u +%H:%M:%SZ)] post-epoch-eval.sh not found beside the launcher"
fi

# Release the slot so the next lab is not blocked by a finished run.
rm -f "$HOME/.local/share/uor-r4/locks/model-slot.json" 2>/dev/null \
  && echo "[$(date -u +%H:%M:%SZ)] slot released"
echo "FULLDOSE CHAIN COMPLETE"
