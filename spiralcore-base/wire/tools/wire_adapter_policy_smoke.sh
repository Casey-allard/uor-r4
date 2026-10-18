#!/usr/bin/env bash
# Per-route policy smoke: same hub, two routes with different gateway compression levels.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
mkdir -p "$PROJ/local"
WORK=$(mktemp -d "$PROJ/local/wire-policy.XXXXXX")
ALL_PIDS=""
cleanup() { for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; rm -rf "$WORK"; }
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_POLICY_FAIL: $1" >&2; exit 1; }
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
roundtrip() {
  python3 - "$1" "$2" "$3" <<'PYSEND'
import socket, sys
addr = sys.argv[1].rsplit(':', 1)
sock = socket.create_connection((addr[0], int(addr[1])), timeout=120)
sock.sendall(open(sys.argv[2], 'rb').read())
sock.shutdown(socket.SHUT_WR)
chunks = []
while True:
    data = sock.recv(65536)
    if not data:
        break
    chunks.append(data)
open(sys.argv[3], 'wb').write(b''.join(chunks))
PYSEND
}
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > e1.log 2>&1 &
E1_PID=$!
track "$E1_PID"
E1=$(poll e1.log)
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > e2.log 2>&1 &
E2_PID=$!
track "$E2_PID"
E2=$(poll e2.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 \
  --route "fast=$E1" --route "best=$E2" --route-level fast=1 --route-level best=9 \
  --stats-file "$WORK/hub.json" > hub.log 2>&1 &
HUB_PID=$!
track "$HUB_PID"
HUB=$(poll hub.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$HUB" --route fast > fast.log 2>&1 &
FAST_PID=$!
track "$FAST_PID"
FAST=$(poll fast.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$HUB" --route best > best.log 2>&1 &
BEST_PID=$!
track "$BEST_PID"
BEST=$(poll best.log)
python3 - <<'PY'
import pathlib
pathlib.Path('text.bin').write_bytes((b'{"kind":"delta","route":"uor","payload":"' + b'abcdef0123456789' * 8 + b'"}\n') * 8192)
PY
roundtrip "$FAST" text.bin fast.out
cmp -s text.bin fast.out || fail 'fast route mismatch'
roundtrip "$BEST" text.bin best.out
cmp -s text.bin best.out || fail 'best route mismatch'
kill -TERM "$FAST_PID" 2>/dev/null || true
kill -TERM "$BEST_PID" 2>/dev/null || true
kill -TERM "$HUB_PID" 2>/dev/null || true
sleep 0.5
python3 - "$WORK/hub.json" <<'PYSTATS'
import json, sys
d = json.load(open(sys.argv[1]))
routes = d['routes']
fast = routes['fast']['ratio']
best = routes['best']['ratio']
print('fast=%.4f best=%.4f' % (fast, best))
assert best < fast, (fast, best)
PYSTATS
echo "WIRE_ADAPTER_POLICY_PASS"
