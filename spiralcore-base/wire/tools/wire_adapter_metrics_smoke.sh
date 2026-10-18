#!/usr/bin/env bash
# Live-metrics smoke: the stats file must update while the adapter is still running.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
mkdir -p "$PROJ/local"
WORK=$(mktemp -d "$PROJ/local/wire-metrics.XXXXXX")
ALL_PIDS=""
cleanup() { for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; rm -rf "$WORK"; }
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_METRICS_FAIL: $1" >&2; exit 1; }
track() { ALL_PIDS="$ALL_PIDS $1"; }
poll() {
  local log=$1 value=''
  for _ in $(seq 1 600); do
    value=$(grep -m1 '^READY ' "$log" 2>/dev/null | awk '{print $2}')
    [ -n "$value" ] && break
    sleep 0.1
  done
  [ -n "$value" ] || fail "no READY in $log"
  echo "$value"
}
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
ECHO_PID=$!
track "$ECHO_PID"
ECHO=$(poll echo.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" --stats-file "$WORK/gw.json" --stats-interval 0.5 > gw.log 2>&1 &
GW_PID=$!
track "$GW_PID"
GATEWAY=$(poll gw.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY" --stats-file "$WORK/cli.json" --stats-interval 0.5 > cli.log 2>&1 &
CLI_PID=$!
track "$CLI_PID"
CLIENT=$(poll cli.log)
python3 - "$CLIENT" <<'PYSEND'
import socket, sys
addr = sys.argv[1].rsplit(':', 1)
sock = socket.create_connection((addr[0], int(addr[1])), timeout=60)
payload = b'{"kind":"metrics","payload":"' + b'abcdef0123456789' * 8 + b'"}\n'
payload = payload * 4096
sock.sendall(payload)
sock.shutdown(socket.SHUT_WR)
total = 0
while True:
    data = sock.recv(65536)
    if not data:
        break
    total += len(data)
assert total == len(payload), (total, len(payload))
PYSEND
# The adapters are still running; the periodic writer must have produced a readable snapshot.
for _ in $(seq 1 50); do
  if [ -f "$WORK/cli.json" ] && grep -q '"connections"' "$WORK/cli.json" 2>/dev/null; then
    break
  fi
  sleep 0.2
done
[ -f "$WORK/cli.json" ] || fail 'no live stats file'
python3 - "$WORK/cli.json" <<'PYSTATS'
import json, sys
d = json.load(open(sys.argv[1]))
assert d['connections'] >= 1, d
assert d['plain_in'] + d['plain_out'] > 0, d
print('live connections=%d plain=%d ratio=%.4f' % (d['connections'], d['plain_in'] + d['plain_out'], d['ratio']))
PYSTATS
echo "WIRE_ADAPTER_METRICS_PASS"
