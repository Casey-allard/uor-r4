#!/usr/bin/env bash
# api-watch.sh STATE_DIR CAP_CENTS OUT_DIR... (claude-400, amendment 6)
#
# The spend watcher for the API segments. It runs on the laptop beside `generate --api openai`.
# Every M2_API_POLL seconds (default 60) it reads the provider's balance with `donor-corpus
# balance` (the key is read by the client from M2_API_KEY_FILE, never by this script) and counts
# spend as the sum of every fall in the balance between readings, so a top-up never hides spend.
# The total and the last reading are kept in STATE_DIR, so a restarted watcher keeps counting.
#
# - Spend at or above CAP_CENTS: writes STOP into every OUT_DIR and an alert to STATE_DIR/ALERTS.
# - A segment that ended `refused`: an alert (once per segment line).
# - A balance it cannot read: an alert at 5 failed readings in a row, and STOP at 10 (fail closed).
# - A balance that rises: an alert.
# It exits when STATE_DIR/WATCH-END exists. It never kills a process.
set -u
[ $# -ge 3 ] || { echo "usage: api-watch.sh STATE_DIR CAP_CENTS OUT_DIR..."; exit 2; }
STATE=$1; CAP=$2; shift 2
case $CAP in *[!0-9]*|'') echo "api-watch: CAP_CENTS must be a whole number of cents, got $CAP"; exit 2 ;; esac
BIN=${M2_API_BIN:?set M2_API_BIN to the donor-corpus binary}
KEY=${M2_API_KEY_FILE:?set M2_API_KEY_FILE to the key file path}
SERVER=${M2_API_SERVER:-https://api.deepseek.com}
POLL=${M2_API_POLL:-60}
mkdir -p "$STATE"
LOG=$STATE/api-watch.log
now() { date -u +%Y-%m-%dT%H:%M:%SZ; }
alert() { echo "$(now) $*" | tee -a "$STATE/ALERTS" >> "$LOG"; }

# The USD total balance in cents, or nothing when it cannot be read.
balance_cents() {
  local v
  v=$("$BIN" balance --server "$SERVER" --api-key-file "$KEY" 2>/dev/null \
    | jq -r '.balance_infos[] | select(.currency == "USD") | .total_balance' 2>/dev/null) || return 0
  case $v in
    *,*) return 0 ;;
    *.[0-9][0-9]) v=${v%.*}${v##*.} ;;
    *.[0-9]) v=${v%.*}${v##*.}0 ;;
    *[!0-9]*|'') return 0 ;;
    *) v=${v}00 ;;
  esac
  echo $((10#$v))
}

stop_all() {
  local d
  for d in "$@"; do
    [ -e "$d/STOP" ] && continue
    if touch "$d/STOP"; then alert "$WHY: STOP written to $d"; else alert "$WHY: could NOT write STOP to $d"; fi
  done
}

for d in "$@"; do [ -d "$d" ] || { echo "api-watch: OUT_DIR $d is not a directory"; exit 2; }; done

fails=0
while [ ! -e "$STATE/WATCH-END" ]; do
  b=$(balance_cents)
  if [ -z "$b" ]; then
    fails=$((fails + 1))
    [ "$fails" -ne 5 ] || alert "balance unreadable 5 times in a row"
    # Fail closed: spend cannot be counted, so the runs stop.
    if [ "$fails" -ge 10 ]; then WHY="balance unreadable $fails times in a row"; stop_all "$@"; fi
  else
    fails=0
    [ -s "$STATE/start-balance-cents" ] || echo "$b" > "$STATE/start-balance-cents"
    [ -s "$STATE/last-balance-cents" ] || echo "$b" > "$STATE/last-balance-cents"
    [ -s "$STATE/spent-cents" ] || echo 0 > "$STATE/spent-cents"
    last=$(cat "$STATE/last-balance-cents"); spent=$(cat "$STATE/spent-cents")
    # Spend is the sum of every fall in the balance, so a top-up never hides spend.
    if [ "$b" -lt "$last" ]; then
      spent=$((spent + last - b))
    elif [ "$b" -gt "$last" ]; then
      alert "balance rose from $last to $b cents (a top-up?); spend so far is kept"
    fi
    echo "$spent" > "$STATE/spent-cents"; echo "$b" > "$STATE/last-balance-cents"
    echo "$(now) balance_cents=$b spent_cents=$spent cap_cents=$CAP" >> "$LOG"
    if [ "$spent" -ge "$CAP" ]; then WHY="spent_cents=$spent >= cap $CAP"; stop_all "$@"; fi
  fi
  for d in "$@"; do
    [ -s "$d/segments.jsonl" ] || continue
    n=$(grep -c '"stopped_by":"refused"' "$d/segments.jsonl")
    seen=$(cat "$STATE/refused-seen-$(echo "$d" | tr / _)" 2>/dev/null || echo 0)
    if [ "$n" -gt "$seen" ]; then
      alert "a segment in $d ended refused ($n in all)"
      echo "$n" > "$STATE/refused-seen-$(echo "$d" | tr / _)"
    fi
  done
  sleep "$POLL"
done
echo "$(now) WATCH-END found; exiting" >> "$LOG"
