#!/usr/bin/env bash
# Copy-fidelity / phrase-lock-in measurement chain (surface-form breadth).
#
#   scripts/copyfid-run.sh [ref|corpus|mix|train|all]
#
# The experiment: two arms at matched budget that differ ONLY in how many
# surface forms per relation the synthetic memory corpus exposes.
#
#   F0  forms=1  the canonical phrase per relation (the existing wide recipe)
#   F1  forms=8  the canonical phrase plus seven paraphrase question and
#                statement forms per relation
#
# Both arms are trained from the same init, on the same chat-v0 store plus a
# synthetic store of the same token count, with the same steps/lr/seeds, and
# both are scored on the same panel. The panel's `seen` conditions use the
# canonical forms; its `unseen` conditions use forms 8..12, which no arm can
# draw from the generator (`forms` is clamped to 1..=7) and which therefore are
# not in any training store by construction.
set -uo pipefail

REPORTS="${REPORTS:-$HOME/uor-r4-worktrees/reports}"
TOK="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
CHATV0="$HOME/uor-r4-local/chat-v0-20260925/prepared"
INIT="$REPORTS/chat-dose-continue-1/dialogue/model"
GEN="${GEN:-$HOME/.cache/uor-r4-curriculum/release/synthetic-memory-corpus}"
MIX="${MIX:-$HOME/.cache/uor-r4-d19-eval/release/mix-chat-corpus}"
STACK="${STACK:-$HOME/.local/share/uor-r4/bin/geometric-stack-9a80e47a}"
THREADS="${THREADS:-2}"
F0_ROWS="${F0_ROWS:-30000}"       # the existing wide recipe, seed 17
F1_ROWS="${F1_ROWS:-21000}"       # scaled so the synthetic token count matches F0
export RAYON_NUM_THREADS="$THREADS"

# --- F0 store + the shared panel -------------------------------------------------
# forms=1 with rows=30000 seed=17 must reproduce wide-memory-corpus2 token for
# token (sha256 9bae7202... / ed15cb21...); that identity is what makes the
# already-trained wide arm a matched F0.
corpus() {
  "$GEN" out="$REPORTS/copyfid-f0-corpus" tokenizer="$TOK" rows="$F0_ROWS" seed=17 forms=1 \
    panel_out="$REPORTS/copyfid-panel-forms"
  "$GEN" out="$REPORTS/copyfid-f1-corpus" tokenizer="$TOK" rows="$F1_ROWS" seed=17 forms=8
}

# --- the two training stores -----------------------------------------------------
mix() {
  "$MIX" out="$REPORTS/copyfid-mixed-f0" tokenizer="$TOK" \
    inputs="$CHATV0/train,$REPORTS/copyfid-f0-corpus" labels=chat-v0,wide-memory
  "$MIX" out="$REPORTS/copyfid-mixed-f1" tokenizer="$TOK" \
    inputs="$CHATV0/train,$REPORTS/copyfid-f1-corpus" labels=chat-v0,wide-memory
}

# --- F1 training (F0 already exists as wide-bind-1 under these exact settings) ----
train() {
  "$STACK" dialogue-train out="$REPORTS/copyfid-f1-arm" tokenizer="$TOK" \
    train_tokens="$REPORTS/copyfid-mixed-f1/tokens.u16" \
    train_mask="$REPORTS/copyfid-mixed-f1/response_mask.u8" \
    train_manifest="$REPORTS/copyfid-mixed-f1/manifest.json" \
    dev_tokens="$CHATV0/heldout/tokens.u16" dev_mask="$CHATV0/heldout/response_mask.u8" \
    dev_manifest="$CHATV0/heldout/manifest.json" \
    init="$INIT" policy=full_prefix steps=2500 batch=16 lr=0.0002 warmup=50 min_lr=0.1 \
    weight_decay=0.1 clip=1.0 eval_every=250 data_seed=20260929 dev_seed=20260930 \
    dev_per_source=32 checkpoint_every=250 max_new_tokens=32
}

case "${1:-all}" in
  corpus) corpus ;;
  mix) mix ;;
  train) train ;;
  ref)
    # byte-identity check of the fixed-form path against the pre-change
    # generator (the wide generator built before this change).
    "$GEN" out="$REPORTS/copyfid-ref-f0" tokenizer="$TOK" rows="$F0_ROWS" seed=17 forms=1
    ;;
  all) corpus && mix && train ;;
  *) echo "usage: $0 [ref|corpus|mix|train|all]" >&2; exit 2 ;;
esac
