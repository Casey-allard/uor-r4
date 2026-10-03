#!/usr/bin/env bash
# Score one integer artifact on the held-out-form panel.
#
#   scripts/copyfid-eval-arm.sh LABEL path/to/model.lut
#
# `lut-chat` refuses a panel above 128 requests
#   ("a request panel needs 1 to 128 requests with distinct ids, ...")
# so the panel is run as pre-split chunks and the chunks are scored as a union.
set -euo pipefail

LABEL="${1:?usage: copyfid-eval-arm.sh LABEL path/to/model.lut}"
LUT="${2:?usage: copyfid-eval-arm.sh LABEL path/to/model.lut}"
R="${REPORTS:-$HOME/uor-r4-worktrees/reports}"
STACK="${STACK:-$HOME/.local/share/uor-r4/bin/geometric-stack-9a80e47a}"
TOK="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
PANEL="$R/copyfid-panel-forms4"
CHUNKS="$R/copyfid-panel-forms4-chunks"
THREADS="${THREADS:-1}"

paths=()
for chunk in "$CHUNKS"/chunk-*.json; do
  tag="$(basename "$chunk" .json)"
  out="$R/copyfid-eval-$LABEL-forms4-$tag"
  [ -e "$out" ] && { echo "refusing: $out exists" >&2; exit 1; }
  "$STACK" lut-chat artifact="$LUT" tokenizer="$TOK" requests="$chunk" \
    max_new_tokens=32 out="$out" threads="$THREADS" >/dev/null
  paths+=("$out/chat.json")
done
echo "$LABEL: ${#paths[@]} chunks"
python3 - "$LABEL" "${paths[@]}" <<'PY'
import sys
print("+".join(sys.argv[2:]))
PY
python3 "$(dirname "$0")/copyfid-score.py" --panel "$PANEL" \
  --values "$PANEL/manifest.json" --out "$R/copyfid-score-$LABEL-forms3.json" \
  --arm "$LABEL=$(python3 -c "
import sys
print('+'.join(sys.argv[1:]))" "${paths[@]}")"
