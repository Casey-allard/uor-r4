#!/usr/bin/env bash
# Process-interposer smoke: unmodified binaries (C probe + Homebrew Python) get their configured
# TCP connections adapted by DYLD_INSERT_LIBRARIES -> wire gateway, with exact payloads.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
WORK="$PROJ/local/interposer"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_INTERPOSER_FAIL: $1" >&2; exit 1; }
ECHO_PID=""
GW_PID=""
cleanup() { [ -n "$GW_PID" ] && kill "$GW_PID" 2>/dev/null || true; [ -n "$ECHO_PID" ] && kill "$ECHO_PID" 2>/dev/null || true; wait 2>/dev/null || true; }
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
ratio_of() {
  python3 - "$1" <<'PY'
import json, sys
text = open(sys.argv[1]).read().strip()
prefix = "WIRE_INTERPOSER_STATS "
if text.startswith(prefix):
    text = text[len(prefix):]
print("%.4f" % json.loads(text)["ratio"])
PY
}
cc -dynamiclib -o libwireinterposer.dylib "$PROJ/tools/wire_interposer.c" -lz || fail 'dylib build'
codesign -f -s - libwireinterposer.dylib || fail 'dylib codesign'
cc -o wire_probe "$PROJ/tools/wire_interposer_probe.c" || fail 'probe build'
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
ECHO_PID=$!
ECHO=$(poll echo.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" > gateway.log 2>&1 &
GW_PID=$!
GATEWAY=$(poll gateway.log)
FAKE=127.0.0.1:65077
# baseline: direct to echo (no interposer)
./wire_probe "$ECHO" text 2097152 > direct.log 2>&1 || fail 'direct baseline'
grep -q 'PROBE_OK' direct.log || fail 'direct baseline payload'
# interposed: unmodified C binary redirected to the gateway
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  WIRE_STATS_FILE="$WORK/text.stats" ./wire_probe "$FAKE" text 2097152 > text.log 2>&1 || fail 'interposed text'
grep -q 'PROBE_OK' text.log || fail 'interposed text payload'
TEXT_RATIO=$(ratio_of "$WORK/text.stats")
python3 -c "import sys; sys.exit(0 if float('$TEXT_RATIO') < 0.5 else 1)" || fail "text ratio $TEXT_RATIO"
# random payload keeps the raw fallback
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  WIRE_STATS_FILE="$WORK/random.stats" ./wire_probe "$FAKE" random 98304 > random.log 2>&1 || fail 'interposed random'
grep -q 'PROBE_OK' random.log || fail 'interposed random payload'
RANDOM_RATIO=$(ratio_of "$WORK/random.stats")
python3 -c "import sys; v=float('$RANDOM_RATIO'); sys.exit(0 if 0.9 < v < 1.2 else 1)" || fail "random ratio $RANDOM_RATIO"
# pass-through: configured interposer, but a non-matching destination stays untouched
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  WIRE_STATS_FILE="$WORK/passthrough.stats" ./wire_probe "$ECHO" text 65536 > passthrough.log 2>&1 || fail 'passthrough'
grep -q 'PROBE_OK' passthrough.log || fail 'passthrough payload'
PASSTHROUGH_CONNS=$(python3 - "$WORK/passthrough.stats" <<'PY'
import json, sys
line = open(sys.argv[1]).read().strip()
line = line[len("WIRE_INTERPOSER_STATS "):] if line.startswith("WIRE_INTERPOSER_STATS ") else line
print(json.loads(line)["connections"])
PY
)
[ "$PASSTHROUGH_CONNS" = "0" ] || fail "passthrough connections $PASSTHROUGH_CONNS"
# unmodified Homebrew Python client through the interposer
cat > py_client.py <<'PYCLIENT'
import socket, sys
addr = ("127.0.0.1", 65077)
payload = (b'{"kind":"python-client","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
sock = socket.create_connection(addr)  # blocking connect: the interposer handshakes during connect
sock.sendall(payload)
sock.shutdown(socket.SHUT_WR)
chunks = []
while True:
    data = sock.recv(65536)
    if not data:
        break
    chunks.append(data)
received = b"".join(chunks)
assert received == payload, (len(received), len(payload))
print("PY_CLIENT_OK bytes=%d" % len(payload))
PYCLIENT
PYTHON_BIN=$(command -v python3)
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  WIRE_STATS_FILE="$WORK/python.stats" "$PYTHON_BIN" py_client.py > py.log 2>&1 || fail 'python client'
grep -q 'PY_CLIENT_OK' py.log || fail 'python client payload'
cc -o wire_fork_test "$PROJ/tools/wire_interposer_fork_test.c" || fail 'fork test build'
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  WIRE_STATS_FILE="$WORK/fork.stats" ./wire_fork_test "$FAKE" 32768 > fork_test.log 2>&1 || fail 'fork-safety round trip'
grep -q 'FORK_TEST_OK.*interrupt_ok=true reused_fd_ok=true' fork_test.log || fail 'fork-safety bytes/mechanism'
cc -dynamiclib -DWIRE_UNSAFE_CLOSE_SHUTDOWN -o libwireinterposer_unsafe.dylib "$PROJ/tools/wire_interposer.c" -lz || fail 'unsafe interposer build'
codesign -f -s - libwireinterposer_unsafe.dylib >/dev/null 2>&1 || fail 'unsafe codesign'
DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer_unsafe.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
  ./wire_fork_test "$FAKE" 32768 > fork_unsafe.log 2>&1
UNSAFE_RC=$?
if [ "$UNSAFE_RC" -ne 7 ] || ! grep -q 'post-fork round trip failed' fork_unsafe.log; then
  fail "negative control did not fail for the expected reason (rc=$UNSAFE_RC)"
fi
NEGATIVE_CONTROL=true
FORK_OK=true
PYTHON_RATIO=$(ratio_of "$WORK/python.stats")
DIRECT_WALL=$(awk '/PROBE_OK/ {print $4}' direct.log)
TEXT_WALL=$(awk '/PROBE_OK/ {print $4}' text.log)
echo "WIRE_INTERPOSER_PASS text_ratio=$TEXT_RATIO random_ratio=$RANDOM_RATIO python_ratio=$PYTHON_RATIO passthrough_connections=$PASSTHROUGH_CONNS direct_wall=$DIRECT_WALL interposed_wall=$TEXT_WALL fork_ok=$FORK_OK negative_control=$NEGATIVE_CONTROL"
