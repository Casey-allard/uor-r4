#!/usr/bin/env bash
# Post-epoch measurement chain for a chat arm. Runs everything the goal
# requires to be measured, in order, so the result is read immediately when the
# epoch lands instead of being reconstructed by hand hours later.
#
#   post-epoch-eval.sh <arm-root> <label>
#
# Steps: export the sealed model to the integer artifact, run the derived
# development panel, run the disjoint held-out panel (the only instrument on
# this lane that has caught a template artefact), then score both against the
# pinned reference arms.
set -euo pipefail

ROOT="${1:?usage: post-epoch-eval.sh <arm-root> <label>}"
LABEL="${2:?usage: post-epoch-eval.sh <arm-root> <label>}"
REPORTS="$HOME/uor-r4-worktrees/stack-prose-reports"
# Resolve the root: ~, then a bare name under the reports root, so callers can
# pass just the arm name the way score-chat-arm.py accepts it.
ROOT="${ROOT/#\~/$HOME}"
case "$ROOT" in
  /*) ;;
  *) ROOT="$REPORTS/$ROOT" ;;
esac
STACK="$HOME/.cache/uor-r4-opencode-target/release/examples/geometric-stack"
TOK="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
CAL="$HOME/uor-r4-local/chat-v0-20260925/prepared/train/tokens.u16"
# The derived 38-request development panel (the retained panel died with the
# X10 Pro). Memory rows state their fact in the FIRST turn; the second is a
# distractor. Any rubric that does not know this scores the arm wrongly.
PANEL="$REPORTS/chat-v0-phase-a-1/panel-derived-requests.json"
HELDOUT="$REPORTS/heldout-panel-40/requests.json"
SCORE="$HOME/uor-r4-worktrees/chat-v0-phase-a/scripts/score-chat-arm.py"

for f in "$STACK" "$TOK" "$CAL" "$PANEL"; do
  [ -r "$f" ] || { echo "MISSING input: $f" >&2; exit 1; }
done

echo "=== $LABEL ==="
echo "root: $ROOT"

# 1. the sealed report must exist before anything else is trusted
if [ ! -f "$ROOT/dialogue/report.json" ] && [ ! -f "$ROOT/report.json" ]; then
  echo "REFUSING: no sealed report in $ROOT -- the run did not finish its fit" >&2
  exit 1
fi

# 2. integer export (the serving artifact the reply arms read)
MODEL="$ROOT/dialogue/model"
[ -d "$MODEL" ] || MODEL="$ROOT/model"
echo "--- export ($MODEL)"
rm -rf "$ROOT/integer"
RAYON_NUM_THREADS=8 "$STACK" export model="$MODEL" out="$ROOT/integer" \
  calibration="$CAL" calibration_windows=64 calibration_time=256 damp=0.01 2>&1 | tail -2
LUT="$ROOT/integer/model.lut"
[ -r "$LUT" ] || { echo "REFUSING: export produced no model.lut" >&2; exit 1; }
echo "    lut $(wc -c < "$LUT") bytes"

# 3. derived development panel -> memory + distinct
echo "--- derived panel"
rm -rf "$ROOT/replies-1"
RAYON_NUM_THREADS=2 "$STACK" lut-chat artifact="$LUT" tokenizer="$TOK" \
  requests="$PANEL" max_new_tokens=32 out="$ROOT/replies-1" 2>&1 | tail -1

# 4. disjoint held-out panel -> the template-artefact control. Scored even when
#    the derived panel looks good, because a templated arm scores 10/10 there.
if [ -r "$HELDOUT" ]; then
  echo "--- disjoint held-out (40 rows over relations absent from training)"
  rm -rf "$REPORTS/value-faith-20261002/heldout40-$LABEL"
  RAYON_NUM_THREADS=2 "$STACK" lut-chat artifact="$LUT" tokenizer="$TOK" \
    requests="$HELDOUT" max_new_tokens=32 \
    out="$REPORTS/value-faith-20261002/heldout40-$LABEL" 2>&1 | tail -1
fi

# 5. score everything against the pinned references
echo
python3 "$SCORE" "$ROOT"

if [ -d "$REPORTS/value-faith-20261002/heldout40-$LABEL" ] && [ -r "$REPORTS/heldout-panel-40/expected.json" ]; then
  python3 - "$REPORTS/value-faith-20261002/heldout40-$LABEL/chat.json" \
            "$REPORTS/heldout-panel-40/expected.json" <<'PY'
import json, re, sys
chat, expected = sys.argv[1], sys.argv[2]
want = json.load(open(expected))
payload = json.load(open(chat))
rows = payload.get("record", payload).get("rows", [])
replies = {r["id"]: (r.get("turns") or [{}])[-1].get("reply", "") for r in rows}
def has(text, value):
    return re.search(r"\b" + re.escape(value) + r"\b", text.lower()) is not None
hits = sum(1 for k, v in want.items() if has(replies.get(k, ""), v))
print(f"  DISJOINT held-out : {hits}/{len(want)}")
print("    reference: parent 0/40, synthetic-alone fit 1/40, mixed-store fit 0/40")
print("    a high derived-panel score with a low disjoint score is TEMPLATE")
print("    MEMORISATION, not capability -- report it as such.")
PY
fi

echo
echo "NOTE: label this arm geometry-on-principle, not a demonstrated advantage."
echo "Matched measurement: 1.9981 geometric vs 2.0111 transformer (delta -0.0130,"
echo "inside the +-0.021 seed spread) = parity."
