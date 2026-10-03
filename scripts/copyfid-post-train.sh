#!/usr/bin/env bash
# Post-fit measurement chain for the surface-form-breadth experiment.
#
#   scripts/copyfid-post-train.sh
#
# Waits for the F1 fit to seal, exports it to the integer artifact, then scores
# F0 (the existing wide-relation arm, whose synthetic store is byte-identical to
# what `forms=1` produces here), F1 and the untrained-for-this-purpose dose
# control on:
#   * the 464-row held-out-form panel (7 conditions, run as chunks because
#     `lut-chat` refuses a panel above 128 requests), and
#   * the three held-out-RELATION panels, so relation breadth and form breadth
#     are read from the same artifact.
set -euo pipefail

R="${REPORTS:-$HOME/uor-r4-worktrees/reports}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STACK="${STACK:-$HOME/.local/share/uor-r4/bin/geometric-stack-9a80e47a}"
TOK="$HOME/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json"
PREP="$HOME/uor-r4-local/chat-v0-20260925/prepared"
ARM="$R/copyfid-f1-arm"

echo "[$(date -u +%H:%M:%SZ)] waiting for $ARM/report.json"
while [ ! -f "$ARM/report.json" ]; do sleep 30; done
echo "[$(date -u +%H:%M:%SZ)] fit sealed"

echo "[$(date -u +%H:%M:%SZ)] export"
[ -e "$ARM/integer" ] && { echo "refusing: $ARM/integer exists" >&2; exit 1; }
"$STACK" export model="$ARM/model" out="$ARM/integer" \
  calibration="$PREP/train/tokens.u16" calibration_windows=64 calibration_time=256 damp=0.01

echo "[$(date -u +%H:%M:%SZ)] held-out-form panel"
"$HERE/copyfid-eval-arm.sh" f0-wide "$R/wide-bind-1/integer/model.lut"
"$HERE/copyfid-eval-arm.sh" f1-forms "$ARM/integer/model.lut"
"$HERE/copyfid-eval-arm.sh" dose-init "$R/chat-dose-continue-1/integer/model.lut"

echo "[$(date -u +%H:%M:%SZ)] held-out-relation panels (F1)"
for panel in heldout-panel-40 heldout-panel-s5 heldout-panel-s11; do
  out="$R/copyfid-eval-f1-$panel"
  [ -e "$out" ] && { echo "refusing: $out exists" >&2; exit 1; }
  "$STACK" lut-chat artifact="$ARM/integer/model.lut" tokenizer="$TOK" \
    requests="$R/$panel/requests.json" max_new_tokens=32 out="$out" threads=1 >/dev/null
done

echo "[$(date -u +%H:%M:%SZ)] scoring"
F0="$R/copyfid-eval-f0-wide-forms4-chunk-00/chat.json"
for chunk in 01 02 03; do
  F0="$F0+$R/copyfid-eval-f0-wide-forms4-chunk-$chunk/chat.json"
done
F1="$R/copyfid-eval-f1-forms-forms4-chunk-00/chat.json"
for chunk in 01 02 03; do
  F1="$F1+$R/copyfid-eval-f1-forms-forms4-chunk-$chunk/chat.json"
done
DOSE="$R/copyfid-eval-dose-init-forms4-chunk-00/chat.json"
for chunk in 01 02 03; do
  DOSE="$DOSE+$R/copyfid-eval-dose-init-forms4-chunk-$chunk/chat.json"
done
python3 "$HERE/copyfid-score.py" --panel "$R/copyfid-panel-forms4" \
  --values "$R/copyfid-panel-forms4/manifest.json" \
  --arm "F0-wide-bind-1=$F0" --arm "F1-forms=$F1" --arm "dose-init=$DOSE" \
  --pair "F0-wide-bind-1,F1-forms" \
  --out "$R/copyfid-score-forms4-final.json" > "$R/copyfid-score-forms4-final.md"

for panel in 40 s5 s11; do
  python3 "$HERE/copyfid-score.py" --panel "$R/heldout-panel-$panel" \
    --arm "F1=$R/copyfid-eval-f1-heldout-panel-$panel/chat.json" \
    --out "$R/copyfid-score-f1-heldout-$panel.json" >> "$R/copyfid-score-forms4-final.md"
done
echo "[$(date -u +%H:%M:%SZ)] DONE"
