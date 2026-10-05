#!/usr/bin/env bash
# Pod-side reaper for the shared UOR-R4 GPU pods (#820; docs/labs/compute.md).
# Installed at /root/uor-reaper.sh and started (nohup) by uor-pod-bootstrap.sh
# only on pods created by `uor-pod up`.
#
#   uor-reaper.sh --loop    every 60 s: when no /root/leases/*.json is unexpired
#                           AND the GPUs have been idle >= UOR_REAPER_IDLE_MIN
#                           (default 20) minutes, delete this pod through the
#                           Runpod API with the pod's own RUNPOD_API_KEY
#   uor-reaper.sh --check   verify that key can read this pod (prints api-ok/FAILED)
#
# Busy = any GPU above 2 % utilisation, any CUDA compute process, or a held
# /root/gpuK.lock (`uor-pod run` jobs, including queued ones).
# The key is read from /proc/1/environ at the moment of use and passed to curl
# on stdin, never in argv, a file or the log. Log: /workspace/uor-r4/pods/reaper.log
set -uo pipefail

IDLE_MIN=${UOR_REAPER_IDLE_MIN:-20}
INTERVAL=${UOR_REAPER_INTERVAL:-60}
LOG=/workspace/uor-r4/pods/reaper.log
SINCE=/root/.uor-idle-since
env1() { tr '\0' '\n' < /proc/1/environ | sed -n "s/^$1=//p" | head -1; }
log() { echo "$(date -u +%FT%TZ) pod=$POD $*" >> "$LOG"; }

# The pod's own RUNPOD_API_KEY is accepted by the GraphQL API (verified
# 2026-10-05) but refused by REST v1 (HTTP 403), so both calls use GraphQL.
gql() {  # gql QUERY -> response body; the Authorization header comes from stdin
  printf 'Authorization: Bearer %s\n' "$(env1 RUNPOD_API_KEY)" |
    curl -sS -H @- -H 'Content-Type: application/json' --max-time 30 \
      -d "{\"query\": \"$1\"}" https://api.runpod.io/graphql
}

# Leases are mirrored verbatim from the laptop, where other tools may have
# written them, so `expires` is parsed as any RFC 3339 / ISO 8601 time with an
# explicit offset (fractional seconds, Z, +HH:MM, +HHMM, +HH) or an epoch
# number, the same rules as uor-pod's `ts`. Anything unparseable (including a
# time without an offset) counts as LIVE: never delete on doubt.
live_lease() {
  python3 - "${UOR_REAPER_LEASES:-/root/leases}" <<'PY'
import calendar, glob, json, re, sys, time
RX = re.compile(r'^\s*(\d{4})-(\d{2})-(\d{2})[Tt ](\d{2}):(\d{2})(?::(\d{2})(?:[.,]\d+)?)?\s*([Zz]|[+-]\d{2}(?::?\d{2})?)\s*$')
def epoch(v):
    if isinstance(v, (int, float)) and not isinstance(v, bool):
        return float(v)
    m = RX.match(v) if isinstance(v, str) else None
    if not m:
        return None
    y, mo, d, h, mi = (int(x) for x in m.groups()[:5])
    s = int(m.group(6) or 0)
    if not (1 <= mo <= 12 and 1 <= d <= 31 and h <= 23 and mi <= 59 and s <= 60):
        return None
    z = m.group(7)
    off = 0
    if z not in ('Z', 'z'):
        z = z.replace(':', '')
        off = (-1 if z[0] == '-' else 1) * (int(z[1:3]) * 3600 + int(z[3:5] or 0) * 60)
    return calendar.timegm((y, mo, d, h, mi, s, 0, 0, 0)) - off
for f in glob.glob(sys.argv[1] + '/*.json'):
    try:
        t = epoch(json.load(open(f))['expires'])
    except Exception:
        t = None
    if t is None or t > time.time():
        sys.exit(0)  # live, or unreadable/unparseable: treat as live
sys.exit(1)
PY
}
if [ "${1:-}" = --live-lease ]; then live_lease; exit; fi  # test hook: exit 0 = a live lease exists
POD=$(env1 RUNPOD_POD_ID)
mkdir -p "$(dirname "$LOG")"

busy() {
  local f
  nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null |
    awk '$1 > 2 { b = 1 } END { exit !b }' && return 0
  [ -n "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader 2>/dev/null)" ] && return 0
  for f in /root/gpu*.lock; do
    [ -e "$f" ] || continue
    flock -n "$f" true || return 0
  done
  return 1
}

case ${1:-} in
  --check)
    [ -n "$POD" ] || { echo "api-FAILED (no RUNPOD_POD_ID)"; exit 1; }
    [ -n "$(env1 RUNPOD_API_KEY)" ] || { echo "api-FAILED (no RUNPOD_API_KEY in the pod environment)"; exit 1; }
    body=$(gql "query { pod(input: {podId: \\\"$POD\\\"}) { id } }" 2>&1)
    case $body in *"\"id\":\"$POD\""*) echo api-ok; exit 0;; esac
    echo "api-FAILED ($(echo "$body" | head -c 200))"; exit 1;;
  --loop) ;;
  *) sed -n '2,17p' "$0"; exit 2;;
esac

log "reaper start (idle limit $IDLE_MIN min)"
date +%s > "$SINCE"   # grace: a fresh pod or restarted reaper waits a full period
while :; do
  if live_lease || busy; then
    date +%s > "$SINCE"
  else
    idle=$(( $(date +%s) - $(cat "$SINCE" 2>/dev/null || date +%s) ))
    if [ "$idle" -ge $(( IDLE_MIN * 60 )) ]; then
      log "no live lease and GPUs idle $(( idle / 60 )) min: deleting this pod"
      body=$(gql "mutation { podTerminate(input: {podId: \\\"$POD\\\"}) }" 2>&1)
      log "podTerminate response: $(echo "$body" | head -c 200)"
      case $body in *'"errors"'*|'') sleep 120;; *) sleep 600;; esac
    fi
  fi
  sleep "$INTERVAL"
done
