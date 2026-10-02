#!/usr/bin/env bash
# Wait for a genuinely free machine, then claim the slot and launch the
# full-dose chat run. Never touches another lab's process.
#
# Why this exists rather than just launching: this machine has 8 cores and the
# project runs several labs on it. Measured, running a second trainer alongside
# the first gave the newer process 0.87 cores and turned a 4.7 h projection into
# 43.4 h. Waiting is ~9x cheaper than contending.
#
# The two conditions are deliberately both required:
#   others=0  -- no other geometric-stack process at all, regardless of load,
#                because a peer job that has not yet spun up its threads is
#                still a job that will contend;
#   load<4    -- the machine is otherwise quiet. Verified attainable: the app
#                harness's own floor is about load 1.5, so this is reachable and
#                not an impossible bar. `test-monitor-logic.sh` unit-tests both
#                boundaries.
#
# The launcher is resolved next to this script, so a copy of the pair works from
# anywhere -- earlier revisions hard-coded /tmp, which the OS may clear.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LAUNCHER="${LAUNCHER:-$HERE/launch-fulldose.sh}"
REPORTS="${REPORTS:-$HOME/uor-r4-worktrees/stack-prose-reports}"
SLOT="$HOME/.local/share/uor-r4/locks/model-slot.json"
LOG="$REPORTS/sequence-wait.log"
RUNLOG="$REPORTS/dose-ladder.log"
POLL_SECONDS="${POLL_SECONDS:-150}"
MAX_POLLS="${MAX_POLLS:-2300}"        # ~96 h at the default interval
LOAD_LIMIT="${LOAD_LIMIT:-4.0}"

log(){ echo "[$(date -u +%H:%M:%SZ)] $*" >> "$LOG"; }

if [ ! -x "$LAUNCHER" ]; then
  log "FATAL: launcher not executable: $LAUNCHER"
  exit 1
fi

log "monitor started; waiting for a free machine (launcher=$LAUNCHER)"

for i in $(seq 1 "$MAX_POLLS"); do
  others=$(pgrep -f "geometric-stack" | wc -l | tr -d ' ')
  load=$(uptime | sed -E 's/.*averages: ([0-9.]+).*/\1/')
  quiet=$(python3 -c "print(1 if float('$load') < $LOAD_LIMIT else 0)" 2>/dev/null || echo 0)

  # Heartbeat every ~30 min so the wait is observable and a stall cannot be
  # mistaken for patience: logging only state changes is indistinguishable from
  # a hung monitor.
  if [ $((i % 12)) = 1 ]; then
    log "waiting: others=$others load=$load (need others=0 and load<$LOAD_LIMIT)"
  fi

  if [ "$others" = "0" ] && [ "$quiet" = "1" ]; then
    log "machine free (others=0, load=$load); claiming slot"
    if (set -o noclobber; cat > "$SLOT" <<SLOTEOF
{"lab":"deepseek","card":"full-dose chat-v0 LM + response #1512","pid":$$,"started_utc":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","expected_end_utc":"$(date -u -v+12H +%Y-%m-%dT%H:%M:%SZ)","threads":8}
SLOTEOF
    ) 2>/dev/null; then
      log "slot claimed; launching $LAUNCHER"
      nohup "$LAUNCHER" > "$RUNLOG" 2>&1 &
      log "launched pid $!"
      exit 0
    else
      log "slot already taken; retrying"
    fi
  fi
  sleep "$POLL_SECONDS"
done

log "monitor expired after $MAX_POLLS polls without a free machine"
