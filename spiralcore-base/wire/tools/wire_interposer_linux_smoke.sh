#!/usr/bin/env bash
# Linux LD_PRELOAD interposer smoke: runs locally (loopback only) with gcc + zlib available.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
WORK=$(printenv WIRE_SMOKE_WORK 2>/dev/null || true)
[ -n "$WORK" ] || WORK="$SCRIPT_DIR/../local/wire-interposer-linux"
rm -rf "$WORK"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_INTERPOSER_LINUX_FAIL: $1" >&2; exit 1; }
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
connections_of() {
  python3 - "$1" <<'PY'
import json, sys
text = open(sys.argv[1]).read().strip()
prefix = "WIRE_INTERPOSER_STATS "
if text.startswith(prefix):
    text = text[len(prefix):]
print(json.loads(text)["connections"])
PY
}
cc -shared -fPIC -o libwireinterposer.so "$SCRIPT_DIR/wire_interposer.c" -ldl -lz || fail 'linux interposer build'
cc -o wire_probe "$SCRIPT_DIR/wire_interposer_probe.c" || fail 'probe build'
python3 "$SCRIPT_DIR/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
ECHO_PID=$!
ECHO=$(poll echo.log)
python3 "$SCRIPT_DIR/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" > gw.log 2>&1 &
GW_PID=$!
GATEWAY=$(poll gw.log)
FAKE=127.0.0.1:65077
LD_PRELOAD="$WORK/libwireinterposer.so" WIRE_ROUTES="$FAKE=$GATEWAY" WIRE_STATS_FILE="$WORK/text.stats" ./wire_probe "$FAKE" text 1048576 > text.log 2>&1 || fail 'linux interposed text'
grep -q 'PROBE_OK' text.log || fail 'linux text payload'
TEXT_RATIO=$(ratio_of "$WORK/text.stats")
python3 -c "import sys; sys.exit(0 if float('$TEXT_RATIO') < 0.5 else 1)" || fail "linux text ratio $TEXT_RATIO"
LD_PRELOAD="$WORK/libwireinterposer.so" WIRE_ROUTES="$FAKE=$GATEWAY" WIRE_STATS_FILE="$WORK/random.stats" ./wire_probe "$FAKE" random 98304 > random.log 2>&1 || fail 'linux interposed random'
grep -q 'PROBE_OK' random.log || fail 'linux random payload'
RANDOM_RATIO=$(ratio_of "$WORK/random.stats")
python3 -c "import sys; v=float('$RANDOM_RATIO'); sys.exit(0 if 0.9 < v < 1.2 else 1)" || fail "linux random ratio $RANDOM_RATIO"
LD_PRELOAD="$WORK/libwireinterposer.so" WIRE_ROUTES="$FAKE=$GATEWAY" WIRE_STATS_FILE="$WORK/pass.stats" ./wire_probe "$ECHO" text 65536 > pass.log 2>&1 || fail 'linux passthrough'
grep -q 'PROBE_OK' pass.log || fail 'linux passthrough payload'
PASSTHROUGH_CONNS=$(connections_of "$WORK/pass.stats")
[ "$PASSTHROUGH_CONNS" = "0" ] || fail "linux passthrough connections $PASSTHROUGH_CONNS"
cat > py_client.py <<'PYCLIENT'
import socket
payload = (b'{"kind":"python","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 2048
sock = socket.create_connection(("127.0.0.1", 65077))
sock.sendall(payload)
sock.shutdown(socket.SHUT_WR)
chunks = []
while True:
    data = sock.recv(65536)
    if not data:
        break
    chunks.append(data)
assert b"".join(chunks) == payload
print("PY_CLIENT_OK bytes=%d" % len(payload))
PYCLIENT
LD_PRELOAD="$WORK/libwireinterposer.so" WIRE_ROUTES="$FAKE=$GATEWAY" WIRE_STATS_FILE="$WORK/python.stats" python3 py_client.py > py.log 2>&1 || fail 'linux python client'
grep -q 'PY_CLIENT_OK' py.log || fail 'linux python payload'
cc -o wire_fork_test "$SCRIPT_DIR/wire_interposer_fork_test.c" || fail 'linux fork test build'
LD_PRELOAD="$WORK/libwireinterposer.so" WIRE_ROUTES="$FAKE=$GATEWAY" WIRE_STATS_FILE="$WORK/fork.stats" \
  ./wire_fork_test "$FAKE" 32768 > fork_test.log 2>&1 || fail 'linux fork-safety round trip'
grep -q 'FORK_TEST_OK.*interrupt_ok=true reused_fd_ok=true' fork_test.log || fail 'linux fork-safety bytes/mechanism'
cc -shared -fPIC -DWIRE_UNSAFE_CLOSE_SHUTDOWN -o libwireinterposer_unsafe.so "$SCRIPT_DIR/wire_interposer.c" -ldl -lz || fail 'linux unsafe build'
LD_PRELOAD="$WORK/libwireinterposer_unsafe.so" WIRE_ROUTES="$FAKE=$GATEWAY" ./wire_fork_test "$FAKE" 32768 > fork_unsafe.log 2>&1
UNSAFE_RC=$?
if [ "$UNSAFE_RC" -ne 7 ] || ! grep -q 'post-fork round trip failed' fork_unsafe.log; then
  fail "linux negative control did not fail for the expected reason (rc=$UNSAFE_RC)"
fi
NEGATIVE_CONTROL=true
FORK_OK=true
PYTHON_RATIO=$(ratio_of "$WORK/python.stats")
echo "WIRE_INTERPOSER_LINUX_PASS text_ratio=$TEXT_RATIO random_ratio=$RANDOM_RATIO python_ratio=$PYTHON_RATIO passthrough_connections=$PASSTHROUGH_CONNS fork_ok=$FORK_OK negative_control=$NEGATIVE_CONTROL"
kill $ECHO_PID $GW_PID 2>/dev/null || true
