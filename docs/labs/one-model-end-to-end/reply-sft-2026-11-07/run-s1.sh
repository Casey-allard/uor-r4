#!/usr/bin/env bash
# Launch the reviewed seed-1 reply-SFT job on one DeepSeek-owned RTX 5090.
#
# UOR_SFT_PIN must be the exact merge commit that contains this script and the
# reviewed trainer/diagnostic dependencies.
set -euo pipefail

PIN=${UOR_SFT_PIN:?UOR_SFT_PIN must name the reviewed 40-hex merge commit}
case "$PIN" in
  *[!0-9a-f]*|'') echo "UOR_SFT_PIN is not lowercase 40-hex" >&2; exit 2 ;;
esac
[ "${#PIN}" -eq 40 ] || { echo "UOR_SFT_PIN is not 40 hex characters" >&2; exit 2; }

ROOT=$(git rev-parse --show-toplevel)
POD_TOOL="$ROOT/scripts/pod/uor-pod"
STORE="$ROOT/scripts/pod/uor-store"
JOB="$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"
RUN_ID=$(date -u +%Y%m%dT%H%M%SZ)-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')
SESSION="deepseek-sft-$RUN_ID"
RUN="/workspace/uor-r4/runs/deepseek-sft-s1-$RUN_ID"
OUT_STORE="results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-$RUN_ID"
CONTROLLER="$ROOT/local/deepseek/one-model-sft-2026-11-07/$RUN_ID"
MAXSEC=${UOR_SFT_MAXSEC:-7200}

case "$MAXSEC" in ''|*[!0-9]*) echo "UOR_SFT_MAXSEC must be an integer" >&2; exit 2 ;; esac
[ "$MAXSEC" -le 21600 ] || { echo "UOR_SFT_MAXSEC may not exceed six hours" >&2; exit 2; }
[ -x "$POD_TOOL" ] || { echo "missing $POD_TOOL" >&2; exit 2; }
[ -x "$STORE" ] || { echo "missing $STORE" >&2; exit 2; }
[ -f "$JOB" ] || { echo "missing $JOB" >&2; exit 2; }
git cat-file -e "$PIN^{commit}" || { echo "local repository does not contain $PIN" >&2; exit 2; }

install -d "$CONTROLLER"
JOB_SHA256=$(sha256sum "$JOB" | cut -d' ' -f1)
printf '%s  %s\n' "$JOB_SHA256" "$JOB" > "$CONTROLLER/job.sha256"
{
  printf 'export UOR_SFT_PIN=%q\n' "$PIN"
  printf 'export UOR_SFT_RUN=%q\n' "$RUN"
  printf 'export UOR_SFT_OUT_STORE=%q\n' "$OUT_STORE"
  printf 'export UOR_SFT_SESSION=%q\n' "$SESSION"
} > "$CONTROLLER/controller.env"

if listings=$("$STORE" ls "$OUT_STORE" 2>/dev/null) && [ -n "$listings" ]; then
  echo "refusing nonempty store root $OUT_STORE" >&2
  exit 1
fi

"$POD_TOOL" status | tee "$CONTROLLER/status-before.txt"
export UOR_POD_MAX_RATE=${UOR_POD_MAX_RATE:-1.15}

POD=
RESOLVED=0
STATUS=FAILED
cleanup() {
  local rc=$?
  trap - EXIT INT TERM
  printf 'status=%s\n' "$STATUS" > "$CONTROLLER/status.txt"
  if [ -n "$POD" ] && [ "$RESOLVED" -ne 1 ]; then
    RESOLVED=1
    if [ "$STATUS" = SEALED ]; then
      "$POD_TOOL" release "$POD" --lab deepseek --session "$SESSION" || rc=1
      "$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" \
        --confirm-archived || rc=1
    else
      echo "run did not prove a sealed readback; archiving the pod before teardown" >&2
      "$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" --to-store || rc=1
    fi
  fi
  exit "$rc"
}
trap cleanup EXIT INT TERM

created=$("$POD_TOOL" up --lab deepseek --session "$SESSION" --gpu 5090 \
  --count 1 --hours 2 --ref "$PIN" \
  --purpose "reply-masked SFT from b64L seed 1" 2>&1 |
  tee "$CONTROLLER/up.log")
POD=$(printf '%s\n' "$created" |
  awk '$1 == "Pod" && $3 == "created;" { print $2; exit }')
[ -n "$POD" ] || { echo "could not parse the pod id" >&2; exit 1; }
printf '%s\n' "$POD" > "$CONTROLLER/pod"

"$POD_TOOL" ssh "$POD" 'cat > /workspace/one-model-reply-sft-job.sh' < "$JOB"
"$POD_TOOL" ssh "$POD" \
  "printf '%s  /workspace/one-model-reply-sft-job.sh\n' '$JOB_SHA256' | sha256sum -c -" |
  tee "$CONTROLLER/job-upload.sha256"

started=$(date +%s)
"$POD_TOOL" run "$POD" --lab deepseek --session "$SESSION" --gpu 0 -- \
  env UOR_SFT_PIN="$PIN" UOR_SFT_RUN="$RUN" UOR_SFT_OUT_STORE="$OUT_STORE" \
  bash /workspace/one-model-reply-sft-job.sh 1 \
  > "$CONTROLLER/run.out" 2>&1 &
WRAPPER_PID=$!
last_renew=$started
failures=0

while kill -0 "$WRAPPER_PID" 2>/dev/null; do
  now=$(date +%s)
  if [ $((now - started)) -gt "$MAXSEC" ]; then
    echo "watcher time limit reached; the healthy job is not cancelled" |
      tee -a "$CONTROLLER/watch.log"
    started=$now
  fi
  if [ $((now - last_renew)) -ge 1500 ]; then
    if ! "$POD_TOOL" renew "$POD" --lab deepseek --session "$SESSION" --hours 2 \
      >> "$CONTROLLER/watch.log" 2>&1; then
      echo "could not renew $POD" >&2
      exit 1
    fi
    last_renew=$now
  fi
  if "$POD_TOOL" ssh "$POD" "test ! -f '$RUN/EXIT'" >/dev/null 2>&1; then
    failures=0
  else
    failures=$((failures + 1))
    if [ "$failures" -ge 20 ]; then
      if ! "$POD_TOOL" status 2>/dev/null | grep -Eq "^$POD .* RUNNING "; then
        echo "$POD disappeared before writing EXIT" >&2
        exit 1
      fi
      failures=0
    fi
  fi
  sleep 20
done

wait "$WRAPPER_PID"
cat "$CONTROLLER/run.out"
mkdir -p "$CONTROLLER/readback"
"$STORE" get "$OUT_STORE/READBACK" "$CONTROLLER/readback" >/dev/null
READBACK_FILE="$CONTROLLER/readback/$OUT_STORE/READBACK"
[ -f "$READBACK_FILE" ] || { echo "no READBACK file was returned" >&2; exit 1; }
[ "$(cat "$READBACK_FILE")" = verified ] || {
  echo "store readback is $(cat "$READBACK_FILE"), not verified" >&2
  exit 1
}
"$STORE" get "$OUT_STORE/EXIT" "$CONTROLLER/readback" >/dev/null
[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ] || {
  echo "sealed EXIT is not zero" >&2
  exit 1
}
STATUS=SEALED
echo "sealed and read back $OUT_STORE"
