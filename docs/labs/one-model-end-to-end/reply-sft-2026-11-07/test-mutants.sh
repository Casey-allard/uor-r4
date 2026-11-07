#!/usr/bin/env bash
# Prove the static gates catch changes to the registered stop and launch rules.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
JOB="$HERE/job.sh"
LAUNCHER="$HERE/run-s1.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
caught=0
attempted=0

"$HERE/gate.sh" >/dev/null
zero_pin=$(printf '0%.0s' {1..40})
if UOR_SFT_PIN="$zero_pin" bash "$JOB" 2 >"$TMP/seed-two.out" 2>&1; then
  echo "unmodified job accepted seed 2" >&2
  exit 1
fi
grep -Fq 'seed 2 is conditional and is not launched by this script' "$TMP/seed-two.out" || {
  echo "unmodified job refused seed 2 for the wrong reason" >&2
  exit 1
}

mutate() {
  local name=$1 file=$2 search=$3 replacement=$4
  attempted=$((attempted + 1))
  local target="$TMP/$name"
  cp "$file" "$target"
  SEARCH=$search REPLACE=$replacement perl -0pi -e \
    's/\Q$ENV{SEARCH}\E/$ENV{REPLACE}/g' "$target"
  cmp -s "$file" "$target" && {
    echo "mutation $name did not change its file" >&2
    exit 1
  }
  if "$HERE/gate.sh" \
    "$([ "$file" = "$JOB" ] && printf '%s' "$target" || printf '%s' "$JOB")" \
    "$([ "$file" = "$LAUNCHER" ] && printf '%s' "$target" || printf '%s' "$LAUNCHER")" \
    >/dev/null 2>&1; then
    echo "mutation $name escaped" >&2
    exit 1
  fi
  caught=$((caught + 1))
}

mutate_first() {
  local name=$1 file=$2 search=$3 replacement=$4
  attempted=$((attempted + 1))
  local target="$TMP/$name"
  cp "$file" "$target"
  SEARCH=$search REPLACE=$replacement perl -0pi -e \
    's/\Q$ENV{SEARCH}\E/$ENV{REPLACE}/' "$target"
  cmp -s "$file" "$target" && {
    echo "mutation $name did not change its file" >&2
    exit 1
  }
  if "$HERE/gate.sh" \
    "$([ "$file" = "$JOB" ] && printf '%s' "$target" || printf '%s' "$JOB")" \
    "$([ "$file" = "$LAUNCHER" ] && printf '%s' "$target" || printf '%s' "$LAUNCHER")" \
    >/dev/null 2>&1; then
    echo "mutation $name escaped" >&2
    exit 1
  fi
  caught=$((caught + 1))
}

mutate wrong-stop-step "$JOB" '[ "$step" -eq 512 ]' '[ "$step" -eq 768 ]'
mutate disabled-stop "$JOB" '[ "$eos_p50" -ge 7 ]; then' '[ "$eos_p50" -ge 7 ] && false; then'
mutate weak-eos-p50 "$JOB" '[ "$eos_p50" -ge 7 ]' '[ "$eos_p50" -ge 0 ]'
mutate weak-guard "$JOB" '[ "$guard" -lt 93 ]' '[ "$guard" -lt 0 ]'
mutate weak-base-guard "$JOB" '[ "$base_guard" -ge 93 ]' '[ "$base_guard" -ge 0 ]'
mutate no-freeze "$JOB" 'kill -STOP "$TRAIN_PID"' 'kill -CONT "$TRAIN_PID"'
mutate no-termination-cont "$JOB" 'kill -CONT "$TRAIN_PID" 2>/dev/null || true' ''
mutate wrong-eval "$JOB" '--eval-every 256' '--eval-every 2560'
mutate wrong-seed "$JOB" '--seed 1 \' '--seed 2 \'
mutate seed-two-enabled "$JOB" '2) echo "seed 2 is conditional and is not launched by this script" >&2; exit 2 ;;' '2) ;;'
mutate zero-replies "$JOB" '--max-replies 1 --reply-tokens 1' \
  '--max-replies 0 --reply-tokens 1'
mutate_first checkpoint-zero-replies "$JOB" \
  '--max-evaluation 512 --max-replies 1 --reply-tokens 1' \
  '--max-evaluation 512 --max-replies 0 --reply-tokens 1'
mutate wrong-panel-target "$JOB" \
  'PANEL_BIN="$RUN/panel-target/release/served-reply-panel"' \
  'PANEL_BIN="$REPO/docs/labs/one-model-end-to-end/instrument/target/release/served-reply-panel"'
mutate mixed-hash "$JOB" \
  3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  0000000000000000000000000000000000000000000000000000000000000000
mutate forge-hash "$JOB" \
  'got=$(sha256sum "$path" | cut -d'"'"' '"'"' -f1)' \
  'got=$want'
mutate no-readback "$JOB" 'READBACK_STATUS=verified' 'READBACK_STATUS=unknown'
mutate directory-readback "$JOB" \
  '[ -d "$sealed" ] && (cd "$sealed" && sha256sum -c SHA256SUMS)' \
  '[ -d "$sealed" ]'
mutate two-gpus "$LAUNCHER" '--count 1 --hours 2' '--count 2 --hours 2'
mutate no-renew "$LAUNCHER" 'if ! "$POD_TOOL" renew "$POD"' 'if ! true'
mutate launcher-readback-true "$LAUNCHER" \
  '[ "$(cat "$READBACK_FILE")" = verified ]' 'true'
mutate launcher-exit-true "$LAUNCHER" \
  '[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ]' 'true'

[ "$caught" -eq "$attempted" ]
echo "$caught of $attempted mutations caught"
