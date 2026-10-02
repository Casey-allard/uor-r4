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
#   idle>=N   -- measured idle CPU, tiered. NOT load average: observed here at
#                load 9.26 with 81% idle, because load counts I/O-wait tasks as
#                demand, so a load gate blocks a machine with free cores.
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

# Poll counter persisted across restarts. Without this, restarting the monitor
# resets the counter to 1 and the tier progression can never advance -- so under
# sustained load the run would wait forever, which is the same defect as a guard
# that can never fire, arrived at from a different direction. Observed: five
# restarts in a row each began at poll 1.
POLL_FILE="$HOME/.local/share/uor-r4/locks/waiter-poll-count"
i=$(cat "$POLL_FILE" 2>/dev/null || echo 0)
case "$i" in ''|*[!0-9]*) i=0 ;; esac
log "resuming at poll $i (persisted)"
for _ in $(seq 1 "$MAX_POLLS"); do
  i=$((i + 1))
  echo "$i" > "$POLL_FILE" 2>/dev/null || true
  others=$(pgrep -f "geometric-stack" | wc -l | tr -d ' ')

  # `others` counts *processes*, but what cost me 9x was *resource contention*
  # (0.87 cores; 4.7 h -> 43.4 h). Those differ. Observed here: a peer job held
  # 0.14 cores for 40 minutes with no evaluation -- a process competing with
  # nothing -- yet `others=0` blocked on it. An all-or-nothing trigger can
  # therefore wait out a job that will never finish, however safe the machine
  # becomes.
  #
  # So the trigger is resource-based, in two tiers:
  #   tier 1 (always)   others=0 exactly -- the normal case
  #   tier 2 (after LONG_WAIT_POLLS)  allow lingering processes, but only on a
  #                     genuinely quiet machine: a LOWER idle bar is not
  #                     enough, so tier 3 also raises the memory floor, buying
  #                     back with measured headroom what it trades away
  # Tiers are keyed on MEASURED IDLE, not load average. Observed on this host:
  # load 9.26 with 81% idle, because load counts uninterruptible (I/O-wait)
  # tasks as demand. A load gate would therefore block a launch onto a machine
  # with ~6.5 free cores -- the same mistake as gating on swap occupancy.
  # Idle is the quantity the decision actually depends on: my run uses 8
  # threads and needs cores.
  if [ "$i" -le "${STRICT_POLLS:-48}" ]; then
    peer_required=0
    idle_floor="${MIN_IDLE_PCT:-50}"
    avail_floor="${MIN_AVAIL_MB:-2100}"
  elif [ "$i" -le "${LONG_WAIT_POLLS:-720}" ]; then
    peer_required=0
    idle_floor="${RELAXED_IDLE_PCT:-30}"
    avail_floor="${MIN_AVAIL_MB:-2100}"
  else
    peer_required=1
    idle_floor="${LONG_WAIT_IDLE_PCT:-15}"
    avail_floor="${LONG_WAIT_AVAIL_MB:-3000}"
  fi
  if [ "$peer_required" = "0" ]; then
    owner_ok=$(python3 -c "print(1 if $others == 0 else 0)")
  else
    owner_ok=1
  fi

  # Idle CPU, measured directly. Load average is a proxy, and a poor one for
  # this decision: what matters is whether EIGHT cores are available for a
  # job whose whole point is throughput. Observed here with a peer at 168% and
  # a build at 190%: load 7.4 but only 33% idle, i.e. ~2.7 free cores against a
  # need for 8. Launching into that oversubscribes and slows every peer, which
  # is the exact harm the sequencing exists to avoid -- so require idle.
  # Measured idle CPU, from `top`'s interval sample. iostat was tried first and
  # is unusable here: on this host `iostat -c 2` reported sys=214.6% and an
  # implied idle of -230.9%, so its columns are not the percentages they appear
  # to be. `top -l 2 -n 0` is stable across repeated runs (40-53% idle observed
  # back to back) and its failure mode is a missing line, which is detectable.
  idle_pct=$(top -l 2 -n 0 2>/dev/null | grep "CPU usage" | tail -1 \
    | sed -E 's/.*, *([0-9.]+)% idle.*/\1/' | tail -1)
  case "$idle_pct" in
    ''|*[!0-9.]*) idle_pct=-1 ;;   # any parse failure blocks, never permits
  esac

  idle_ok=$(python3 -c "print(1 if float('$idle_pct') >= $idle_floor else 0)" 2>/dev/null || echo 0)

  # Memory guard. Two numbers, because they measure different things.
  #
  # Reclaimable memory (free + inactive + speculative + purgeable) is what a new
  # 1.4 GB process can actually take. Swap free is what the kernel can fall back
  # on when it must evict. Both matter, and neither alone is the right test:
  # swap occupancy is sticky -- pages stay swapped with no pressure to reclaim
  # them -- so a swap-only test can refuse to launch on an idle machine, while a
  # reclaimable-only test would launch into a box whose swap is exhausted and
  # whose compressor is already holding gigabytes.
  #
  # Measured on this machine during the queue: reclaimable 2168 MB against a
  # 1400 MB need (1.5x, tight), swap free 762 MB, wired 8977 MB, compressor
  # 4245 MB. This is not about my own throughput: when memory runs out the
  # kernel kills the largest task present, which here is a peer lab's
  # multi-hour run.
  read -r avail_mb swap_free_mb <<EOF2
$(python3 -c "
import re, subprocess
ps = int(subprocess.run(['sysctl','-n','hw.pagesize'],capture_output=True,text=True).stdout.strip())
vm = subprocess.run(['vm_stat'],capture_output=True,text=True).stdout
def pg(n):
    m = re.search(rf'{n}:\\s+(\\d+)', vm)
    return int(m.group(1)) if m else 0
avail = (pg('Pages free')+pg('Pages inactive')+pg('Pages speculative')+pg('Pages purgeable'))*ps//(1024*1024)
out = subprocess.run(['sysctl','-n','vm.swapusage'],capture_output=True,text=True).stdout
m = re.search(r'free = ([0-9.]+)M', out)
print(avail, int(float(m.group(1))) if m else 0)
" 2>/dev/null || echo "0 0")
EOF2
  roomy=$(python3 -c "
ok = $avail_mb >= $avail_floor and $swap_free_mb >= ${MIN_SWAP_FREE_MB:-512}
print(1 if ok else 0)" 2>/dev/null || echo 0)

  # Heartbeat every ~30 min so the wait is observable and a stall cannot be
  # mistaken for patience: logging only state changes is indistinguishable from
  # a hung monitor.
  if [ $((i % 12)) = 1 ]; then
    log "waiting: others=$others idle=${idle_pct}% (floor $idle_floor) avail=${avail_mb}MB (floor $avail_floor) swap_free=${swap_free_mb}MB | tier poll=$i peer_ok=$owner_ok"
  fi

  if [ "$owner_ok" = "1" ] && [ "$idle_ok" = "1" ] && [ "$roomy" = "1" ]; then
    log "machine free (others=$others idle=${idle_pct}% avail=${avail_mb}MB swap_free=${swap_free_mb}MB); claiming slot"
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
