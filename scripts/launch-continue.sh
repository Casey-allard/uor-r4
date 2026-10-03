#!/usr/bin/env bash
# Continue the WINNING ARM's chat-v0 LM to a higher dose, then re-run the
# response phase -- to produce the dose-response the objective asks for.
#
# Why this shape rather than a from-scratch full epoch:
#  * the objective's question is whether the arm is UNDERTRAINED. The winning arm
#    saw 9.2M tokens (11.1% of the store, 1.28 tokens/parameter). Continuing from
#    its own weights tests exactly that, and reuses the 57 min of Phase A already
#    spent instead of redoing it;
#  * a from-scratch run would also REMOVE the prose init, confounding "more dose"
#    with "lost the base that produced the winning arm";
#  * starting from the trained weights means every step is additional dose, so
#    the whole budget buys the thing being measured.
#
# init= loads the weights ONLY: Adam moments restart at zero and the LR jumps
# to its warmup target. Measured on the first attempt at lr=0.002, a converged
# model got WORSE over 1,000 further steps (2.304152 -> 2.437755, +0.1336). That
# is a restart artefact, not dose, and it would have been reported as evidence
# against more dose if the starting value had not been checked.
#
# So the continuation runs at lr=0.0002 -- an order of magnitude below the
# from-scratch rate -- keeping each step small enough that the additional dose
# dominates the restart perturbation. Stated plainly: this is a CONTINUATION,
# not an exact resume, and the LR is chosen for that reason.
set -uo pipefail
P="$HOME/uor-r4-local/chat-v0-20260925/prepared"
T="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
W="${INIT_LM:-$HOME/uor-r4-worktrees/reports/chat-prose-init-phase-a-1/lm/model}"
O="${OUT:-$HOME/uor-r4-worktrees/reports/chat-dose-continue-1}"
# 6 threads, not 8: peers are active and 8 would oversubscribe the machine,
# slowing them and me. Slower wall clock, but it coexists rather than contends.
export RAYON_NUM_THREADS="${THREADS:-6}"
# The build cache this launcher originally pointed at was removed by a storage
# cleanup, so it now uses the surviving binary built from the same source
# revision the winning arm ran on (1b39690f, binary 9a80e47a) -- which is the
# right thing anyway, since a continuation must not mix revisions.
B="${STACK_BIN:-$HOME/.local/share/uor-r4/bin/geometric-stack-9a80e47a}"
[ -x "$B" ] || { echo "FATAL: no usable binary at $B"; exit 1; }
STEPS="${STEPS:-6000}"
mkdir -p "$O"

# Additional chat-v0 dose on top of the winning arm's 2,237 steps.
# 6,000 steps x 16 x 256 = 24.6M more tokens -> 33.8M total = 41% of the store,
# 4.7 tokens/parameter, on top of the 1.28 the arm already had.
"$B" train train="$P/train/tokens.u16" valid="$P/heldout/tokens.u16" tokenizer="$T" out="$O/lm" \
  arch=geometric pattern=rrarra read=lorentz rotation=true seed=1 init="$W" \
  steps="$STEPS" batch=16 lr="${LR:-0.0002}" warmup="${WARMUP:-100}" eval_every=1000 eval_windows=64 final_windows=256 \
  checkpoint_every=1000 sample_tokens=0 max_seconds="${LM_MAX_SECONDS:-28800}" > "$O/lm.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] LM continuation done rc=$?"

# Response phase from the higher-dose LM, same 1,024 steps as the winning arm so
# the comparison isolates the LM dose and nothing else.
"$B" dialogue-train out="$O/dialogue" tokenizer="$T" \
  train_tokens="$P/train/tokens.u16" train_mask="$P/train/response_mask.u8" train_manifest="$P/train/manifest.json" \
  dev_tokens="$P/heldout/tokens.u16" dev_mask="$P/heldout/response_mask.u8" dev_manifest="$P/heldout/manifest.json" \
  init="$O/lm/model" policy=full_prefix data_seed=20260929 steps=1024 batch=16 lr=0.001 \
  eval_every=128 dev_seed=20260930 dev_per_source=32 checkpoint_every=128 max_new_tokens=32 \
  max_seconds="${DIALOGUE_MAX_SECONDS:-7200}" > "$O/dialogue.log" 2>&1
echo "[$(date -u +%H:%M:%SZ)] response phase done rc=$?"

SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -x "$SELF/post-epoch-eval.sh" ]; then
  "$SELF/post-epoch-eval.sh" "$O" dose-continue >> "$O.eval.log" 2>&1 \
    && echo "[$(date -u +%H:%M:%SZ)] measured -> $O.eval.log" \
    || echo "[$(date -u +%H:%M:%SZ)] measurement FAILED (see $O.eval.log)"
fi
rm -f "$HOME/.local/share/uor-r4/locks/model-slot.json" 2>/dev/null && echo "slot released"
echo "CONTINUATION CHAIN COMPLETE"
