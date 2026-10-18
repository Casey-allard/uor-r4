#!/usr/bin/env bash
# Routed-hub CLI smoke: wire_hub.py routes, a WireSocket client, and an unmodified binary through the
# process interposer (macOS DYLD) reaching a route.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
WORK="$PROJ/local/wire-hub"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_HUB_CLI_FAIL: $1" >&2; exit 1; }
ECHO_PID=""
HUB_PID=""
cleanup() { [ -n "$HUB_PID" ] && kill "$HUB_PID" 2>/dev/null || true; [ -n "$ECHO_PID" ] && kill "$ECHO_PID" 2>/dev/null || true; wait 2>/dev/null || true; }
trap cleanup EXIT
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
cc -dynamiclib -o libwireinterposer.dylib "$PROJ/tools/wire_interposer.c" -lz 2>/dev/null || fail 'interposer build'
codesign -f -s - libwireinterposer.dylib >/dev/null 2>&1 || fail 'codesign'
cc -o wire_probe "$PROJ/tools/wire_interposer_probe.c" || fail 'probe build'
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
ECHO_PID=$!
ECHO=$(poll echo.log)
python3 "$PROJ/tools/wire_hub.py" --listen 127.0.0.1:0 --route "echo=$ECHO" --stats-file "$WORK/hub.json" > hub.log 2>&1 &
HUB_PID=$!
HUB=$(poll hub.log)
# 1. WireSocket client through a named route
cat > ws_client.py <<'PYCLIENT'
import socket, sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
payload = (b'{"kind":"hub-client","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
wire = WireSocket.connect(sys.argv[2], route="echo")
wire.sendall(payload)
wire.shutdown()
chunks = []
while len(b"".join(chunks)) < len(payload):
    data = wire.recv(65536)
    if not data:
        break
    chunks.append(data)
wire.close()
assert b"".join(chunks) == payload
print("WS_CLIENT_OK bytes=%d" % len(payload))
PYCLIENT
python3 ws_client.py "$PROJ/tools" "$HUB" > ws.log 2>&1 || fail 'wiresocket client'
grep -q 'WS_CLIENT_OK' ws.log || fail 'wiresocket client payload'
# 2. unmodified stock binary through the interposer, routed by the hub
FAKE=127.0.0.1:65078
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$HUB#echo" \
  WIRE_STATS_FILE="$WORK/interposer.stats" ./wire_probe "$FAKE" text 2097152 > probe.log 2>&1 || fail 'interposed hub probe'
grep -q 'PROBE_OK' probe.log || fail 'interposed hub payload'
# 3. hub stats show both requests on the echo route
python3 - "$WORK/hub.json" <<'PYSTATS'
import json, sys
data = json.load(open(sys.argv[1]))
route = data["routes"]["echo"]
assert route["connections"] == 2, data
assert route["wire"] > 0, data
print("hub_echo_connections=%d wire=%d ratio=%.4f" % (route["connections"], route["wire"], route["ratio"]))
PYSTATS
echo "WIRE_HUB_CLI_PASS"
