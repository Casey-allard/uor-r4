#!/usr/bin/env bash
# Idle-timeout smoke: idle tunnels close; active ones still round-trip.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
mkdir -p "$PROJ/local"
WORK=$(mktemp -d "$PROJ/local/wire-idle.XXXXXX")
ALL_PIDS=""
cleanup() { for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; rm -rf "$WORK"; }
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_IDLE_FAIL: $1" >&2; exit 1; }
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
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" --idle-timeout 1 > gw.log 2>&1 &
GW_PID=$!
track "$GW_PID"
GATEWAY=$(poll gw.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY" --idle-timeout 1 > cli.log 2>&1 &
CLI_PID=$!
track "$CLI_PID"
CLIENT=$(poll cli.log)
python3 - "$CLIENT" <<'PYIDLE'
import socket, sys, time
addr = sys.argv[1].rsplit(':', 1)
sock = socket.create_connection((addr[0], int(addr[1])), timeout=10)
started = time.time()
sock.settimeout(10)
data = sock.recv(1)
elapsed = time.time() - started
assert data == b'', data
assert 0.5 <= elapsed <= 5.0, elapsed
print('idle_closed_after=%.2fs' % elapsed)
PYIDLE
python3 - "$CLIENT" <<'PYACTIVE'
import socket, sys
addr = sys.argv[1].rsplit(':', 1)
sock = socket.create_connection((addr[0], int(addr[1])), timeout=60)
payload = b'active tunnel ' * 4096
sock.sendall(payload)
sock.shutdown(socket.SHUT_WR)
chunks = []
while True:
    data = sock.recv(65536)
    if not data:
        break
    chunks.append(data)
assert b''.join(chunks) == payload
print('active_roundtrip_ok bytes=%d' % len(payload))
PYACTIVE
echo "WIRE_ADAPTER_IDLE_PASS"
