#!/usr/bin/env bash
# Step 4 (barrier assessment 2026-10-05, #820): the world=v2c high-MQAR corpus
# and its world=v2 control, built exactly as Step 1's (/root/run-step1.sh) so a
# v2 vs v2c fine-tune A/B can run later.
#
#   bash scripts/pod/step4-corpus.sh
#
# world=v2c is M-world v2 whose rehearsing MQAR replies read "So {k} is {v}."
# instead of "{K} is {v}.", so a rehearsed key keeps its asserted casing and
# its token tuple occurs in context (Step 0b, #1709: every rehearsal in the D19
# draw was recased). Everything else is Step 1's:
#
#   m-world corpus world=W protocol=2 conversations=40000 seed=12 recall=off
#     mqar_share=0.6 copy_share=0.1 relation_share=0.2 other_share=0.1
#   mix-chat-corpus: that M-world train split x4 + chat-v0-p2/train
#
# The world=v2 corpus is rebuilt by this binary as the control; a world=v2 draw
# is byte-identical to every earlier one (test the_v2_seed_9101_draw_is_unchanged),
# so its tokens.u16 must equal Step 1's $D/ft/mw-hi/train/tokens.u16 when that
# exists (the script compares and prints MATCH/DIFFER). The v2c stream is paired
# with v2's draw for draw until the first episode whose one-piece-longer reply
# changes a context fit; after that the two are independent draws of the same
# mix (corpus.json records episodes and tokens per kind for each).
#
# CPU only. Every report root is claimed exclusively and sealed: pick a new
# OUT_ROOT for a retry, never reuse one. Expected wall time on the pod:
# ~8 min to build the two binaries (if not cached), ~2 min per 40,000-
# conversation M-world corpus (400 took ~1 s on an M1 at 2 threads), ~1 min
# per mix: about 15 min in all.
set -euo pipefail

REPO="${REPO:-/root/uor-r4}"           # a checkout of this PR's head
D="${D:-/root/data}"                    # Step 1's data root
TOKENIZER="${TOKENIZER:-$D/tokenizer.json}"
CHAT="${CHAT:-$D/chat-v0-p2/train}"     # Step 1's chat-v0 protocol-2 train split
OUT_ROOT="${OUT_ROOT:-$D/ft/step4}"
TARGET="${CARGO_TARGET_DIR:-/root/target-step4}"
CARGO="${CARGO:-$HOME/.cargo/bin/cargo}"
WORLDS="${WORLDS:-v2c v2}"
export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-8}"

sha256sum "$TOKENIZER"
cd "$REPO"
echo "head $(git rev-parse HEAD)"
CARGO_TARGET_DIR="$TARGET" "$CARGO" build --release -j 8 -p uor-r4-training \
  --example m-world --bin mix-chat-corpus
MW="$TARGET/release/examples/m-world"
MIX="$TARGET/release/mix-chat-corpus"
sha256sum "$MW" "$MIX"
mkdir -p "$OUT_ROOT"

for world in $WORLDS; do
  mw="$OUT_ROOT/mw-hi-$world"
  hi="$OUT_ROOT/hi-$world"
  start=$(date +%s)
  "$MW" corpus "world=$world" "out=$mw" "tokenizer=$TOKENIZER" protocol=2 \
    conversations=40000 seed=12 recall=off \
    mqar_share=0.6 copy_share=0.1 relation_share=0.2 other_share=0.1 \
    > "$mw.log" 2>&1
  echo "corpus world=$world out=$mw seconds=$(( $(date +%s) - start ))"
  python3 - "$mw/corpus.json" <<'PY'
import json, sys
c = json.load(open(sys.argv[1]))
m = c["composition"]["m_world"]
print(json.dumps({
    "world": c["world"],
    "world_digest": c["world_digest"],
    "tokens": m["tokens"],
    "response_tokens": m["response_tokens"],
    "episodes_per_kind": {k: v["episodes"] for k, v in m["episodes_per_kind"].items()},
}))
PY
  sha256sum "$mw/train/tokens.u16" "$mw/train/response_mask.u8"
  if [ "$world" = v2 ] && [ -f "$D/ft/mw-hi/train/tokens.u16" ]; then
    if cmp -s "$mw/train/tokens.u16" "$D/ft/mw-hi/train/tokens.u16"; then
      echo "v2 control MATCHES Step 1's mw-hi"
    else
      echo "v2 control DIFFERS from Step 1's mw-hi"
    fi
  fi
  start=$(date +%s)
  t="$mw/train"
  "$MIX" "out=$hi" "tokenizer=$TOKENIZER" \
    "inputs=$t,$t,$t,$t,$CHAT" \
    "labels=mw-hi-$world-1,mw-hi-$world-2,mw-hi-$world-3,mw-hi-$world-4,chat-v0-p2" \
    > "$hi.log" 2>&1
  echo "mix world=$world out=$hi seconds=$(( $(date +%s) - start ))"
  sha256sum "$hi/tokens.u16" "$hi/response_mask.u8" "$hi/manifest.json"
done
echo "STEP4 CORPUS DONE"
