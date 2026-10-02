#!/usr/bin/env bash
# Teach the geometric stack to answer on the prepared literal-role dialogue corpus (chat-v0), serve it in integers
# under owner decision D10, and chat with it, on an Apple-silicon Mac. The stack learns responses with the retained
# dialogue study's episodes (contract, eligibility, response-uniform sampler and source-stratified development
# panel), so its development NLL is on the same responses as that study's. See the `dialogue-train` and `lut-chat`
# modes of crates/uor-r4-training/examples/geometric-stack.rs.
#
# Usage, from the repository root, with the prepared corpus's split files:
#   TRAIN_TOKENS=.../train/tokens TRAIN_MASK=.../train/mask TRAIN_MANIFEST=.../train/manifest.json \
#   DEV_TOKENS=.../heldout/tokens DEV_MASK=.../heldout/mask DEV_MANIFEST=.../heldout/manifest.json \
#   TOKENIZER=.../tokenizer.json [REQUESTS=requests.json] [INIT=ROOT/model] \
#   scripts/geometric-stack-chat-m1.sh [OUT_PARENT]
#
# The token stores are UORT files, the masks one byte per token and the manifests the split manifests whose
# `files` list each source's label and token count. TOKENIZER is the corpus's tokenizer.json (the retained
# 4,096-token one, with <|bos|>, <|eos|> and <|unk|>). REQUESTS is a request panel in the study's format
# ([{"id", "category", "user_turns": [...]}]). OUT_PARENT (default reports/geometric-stack-chat-<timestamp>) must
# not exist; every stage claims and seals its own report root inside it.
#
# Stages:
#   1. build the geometric-stack example (with Accelerate on macOS)
#   2. unless LM_STEPS=0: train LM_STEPS updates of BATCH x 256 tokens as a language model on the whole
#      training store (every token, windows across documents), starting from INIT when one is given
#   3. response learning from that model (or from INIT when LM_STEPS=0): DIALOGUE_STEPS updates of
#      DIALOGUE_BATCH episodes, loss on
#      response and EOS targets only, the development panel scored every EVAL_EVERY updates; with REQUESTS, the float
#      model's greedy replies
#   4. export the result with GPTQ calibrated on the training store (4-bit table weight maps, no floating point)
#   5. with REQUESTS: the integer engine's greedy replies to the same panel
#   6. summary.txt: development NLL before and after, and the replies
#
# Knobs: LM_STEPS (2000), BATCH (16), LR_LM (0.004), PATTERN (rrarra), READ (lorentz), ROTATION (true), SEED (1),
# DIALOGUE_STEPS (1024), DIALOGUE_BATCH (16), LR_DIALOGUE (0.001), POLICY (full_prefix), DATA_SEED (1), DEV_SEED
# (1) and DEV_PER_SOURCE (32) — pass the study's development seed to score its exact panel — EVAL_EVERY (128),
# MAX_NEW_TOKENS (32, the retained study's cap; at most 128, and every request's history must fit 256 positions
# with each reply at the cap), THREADS (8), FEATURES (cpu-accelerate on macOS, none elsewhere). A request panel
# that does not fit is refused before any training.
#
# Shared machine: check #973 for other labs' active fits before launching, and lower THREADS beside one.
#
# Cost: unmeasured on the M1. The lab sandbox trained this 7.2M-parameter stack at about 880 tokens per second on
# four x86 threads, so LM_STEPS=2000 (8.2M tokens) would take about 2.6 hours there; response learning is shorter,
# because each batch keeps only the positions up to its longest episode. Try LM_STEPS=100 DIALOGUE_STEPS=64 first
# to time both stages. Send OUT_PARENT back for the record.
#
# Chat afterwards (one user message per line; /reset starts over; a full 256-position context starts over):
#   target/release/examples/geometric-stack lut-chat artifact=OUT_PARENT/integer/model.lut tokenizer=TOKENIZER \
#     out=OUT_PARENT/chat-1 [temperature=0.7 top_k=40]
set -euo pipefail

OUT=${1:-reports/geometric-stack-chat-$(date +%Y%m%d-%H%M%S)}
for name in TRAIN_TOKENS TRAIN_MASK TRAIN_MANIFEST DEV_TOKENS DEV_MASK DEV_MANIFEST TOKENIZER; do
  [ -n "${!name:-}" ] || { echo "set $name (see the header of $0)" >&2; exit 1; }
  [ -f "${!name}" ] || { echo "missing $name=${!name}" >&2; exit 1; }
done
REQUESTS=${REQUESTS:-}
INIT=${INIT:-}
LM_STEPS=${LM_STEPS:-2000}
BATCH=${BATCH:-16}
LR_LM=${LR_LM:-0.004}
PATTERN=${PATTERN:-rrarra}
READ=${READ:-lorentz}
ROTATION=${ROTATION:-true}
SEED=${SEED:-1}
DIALOGUE_STEPS=${DIALOGUE_STEPS:-1024}
DIALOGUE_BATCH=${DIALOGUE_BATCH:-16}
LR_DIALOGUE=${LR_DIALOGUE:-0.001}
POLICY=${POLICY:-full_prefix}
DATA_SEED=${DATA_SEED:-1}
DEV_SEED=${DEV_SEED:-1}
DEV_PER_SOURCE=${DEV_PER_SOURCE:-32}
EVAL_EVERY=${EVAL_EVERY:-128}
MAX_NEW_TOKENS=${MAX_NEW_TOKENS:-32}
THREADS=${THREADS:-8}
if [ -z "${FEATURES+x}" ]; then
  if [ "$(uname -s)" = Darwin ]; then FEATURES=cpu-accelerate; else FEATURES=; fi
fi
TARGET=${CARGO_TARGET_DIR:-target}
STACK=$TARGET/release/examples/geometric-stack
[ -n "$REQUESTS" ] && { [ -f "$REQUESTS" ] || { echo "missing REQUESTS=$REQUESTS" >&2; exit 1; }; }
[ -n "$INIT" ] && { [ -f "$INIT/model.safetensors" ] || { echo "INIT=$INIT has no model.safetensors" >&2; exit 1; }; }
[ -e "$OUT" ] && { echo "$OUT exists; choose a new output parent" >&2; exit 1; }
mkdir -p "$OUT"
git rev-parse HEAD > "$OUT/source-revision.txt"
uname -a > "$OUT/machine.txt"
sysctl -n machdep.cpu.brand_string >> "$OUT/machine.txt" 2>/dev/null || true

features=()
[ -n "$FEATURES" ] && features=(--features "$FEATURES")
cargo build --release ${features[@]+"${features[@]}"} -p uor-r4-training --example geometric-stack
export RAYON_NUM_THREADS=$THREADS

# The language-model phase runs whenever LM_STEPS is nonzero, *including* when
# INIT= is set: an initialised base plus a chat-v0 LM phase is a distinct arm
# from either alone (it is the arm that combines a prose base's short-range
# fluency with the target domain's response distribution). LM_STEPS=0 with
# INIT= remains "response learning from INIT", and INIT unset remains
# "train a new stack for LM_STEPS".
if [ "$LM_STEPS" != 0 ]; then
  lm_init=()
  [ -n "$INIT" ] && lm_init=(init="$INIT")
  "$STACK" train train="$TRAIN_TOKENS" valid="$DEV_TOKENS" tokenizer="$TOKENIZER" out="$OUT/lm" arch=geometric \
    pattern="$PATTERN" read="$READ" rotation="$ROTATION" seed="$SEED" steps="$LM_STEPS" batch="$BATCH" \
    lr="$LR_LM" warmup=$((LM_STEPS / 20 + 1)) eval_every=250 eval_windows=64 final_windows=256 \
    checkpoint_every=250 sample_tokens=0 ${lm_init[@]+"${lm_init[@]}"} > "$OUT/lm.log" 2>&1
  INIT=$OUT/lm/model
fi

init=()
[ -n "$INIT" ] && init=(init="$INIT")
[ -z "$INIT" ] && init=(arch=geometric pattern="$PATTERN" read="$READ" rotation="$ROTATION" seed="$SEED")
requests=()
[ -n "$REQUESTS" ] && requests=(requests="$REQUESTS")
"$STACK" dialogue-train out="$OUT/dialogue" tokenizer="$TOKENIZER" \
  train_tokens="$TRAIN_TOKENS" train_mask="$TRAIN_MASK" train_manifest="$TRAIN_MANIFEST" \
  dev_tokens="$DEV_TOKENS" dev_mask="$DEV_MASK" dev_manifest="$DEV_MANIFEST" "${init[@]}" \
  policy="$POLICY" data_seed="$DATA_SEED" steps="$DIALOGUE_STEPS" batch="$DIALOGUE_BATCH" lr="$LR_DIALOGUE" \
  eval_every="$EVAL_EVERY" dev_seed="$DEV_SEED" dev_per_source="$DEV_PER_SOURCE" checkpoint_every="$EVAL_EVERY" \
  max_new_tokens="$MAX_NEW_TOKENS" ${requests[@]+"${requests[@]}"} > "$OUT/dialogue.log" 2>&1

"$STACK" export model="$OUT/dialogue/model" calibration="$TRAIN_TOKENS" out="$OUT/integer"
if [ -n "$REQUESTS" ]; then
  RAYON_NUM_THREADS=1 "$STACK" lut-chat artifact="$OUT/integer/model.lut" tokenizer="$TOKENIZER" \
    requests="$REQUESTS" max_new_tokens="$MAX_NEW_TOKENS" out="$OUT/integer-replies"
fi

python3 - "$OUT" <<'PY' | tee "$OUT/summary.txt"
import json, os, sys
out = sys.argv[1]
report = json.load(open(f"{out}/dialogue/report.json"))
first = report["initial_development"] or {}
last = report["final_development"]
print("development response NLL %s -> %.4f; first four targets %s -> %.4f (%d responses)" % (
    "%.4f" % first["response_mean_nll"] if first else "(resumed)", last["response_mean_nll"],
    "%.4f" % first["first_four_response_targets_mean_nll"] if first else "(resumed)",
    last["first_four_response_targets_mean_nll"], last["selected_responses"]))
for source in last["per_source"]:
    print("  %-28s %s" % (source["label"], source["response_mean_nll"]))
integer = f"{out}/integer-replies/chat.json"
integer_rows = {row["id"]: row for row in json.load(open(integer))["record"]["rows"]} if os.path.exists(integer) else {}
for row in (report.get("replies") or {}).get("rows", []):
    for turn in row["turns"]:
        print("\n[%s] user: %s" % (row["id"], turn["user"]))
        print("  float:   %s" % turn["reply"].replace("\n", " "))
        match = integer_rows.get(row["id"])
        if match:
            print("  integer: %s" % match["turns"][turn["turn"] - 1]["reply"].replace("\n", " "))
PY
echo "done: lm/, dialogue/, integer/ and summary.txt are in $OUT"
echo "chat: $STACK lut-chat artifact=$OUT/integer/model.lut tokenizer=$TOKENIZER out=$OUT/chat-1"
