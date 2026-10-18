#!/usr/bin/env bash
# Wire-adapter smoke/measurement: compress a pair of TCP endpoints (echo + objpeer) and report ratios.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(cd "$PROJ/../.." && pwd)
if [ -z "$(printenv OBJPEER)" ]; then
  OBJPEER="$REPO_ROOT/target/release/objpeer"
else
  OBJPEER=$(printenv OBJPEER)
fi
[ -x "$OBJPEER" ] || { echo "missing objpeer: $OBJPEER (build: cargo build --release -p uor-r4-object-store --bin objpeer)" >&2; exit 2; }
mkdir -p "$PROJ/local"
WORK=$(mktemp -d "$PROJ/local/wire-adapter.XXXXXX")
ALL_PIDS=""
cleanup() {
  for pid in $ALL_PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done
  rm -rf "$WORK"
}
trap cleanup EXIT
cd "$WORK"
fail() { echo "WIRE_ADAPTER_FAIL: $1" >&2; exit 1; }
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
ratio_of() { python3 -c "import json; d=json.load(open('$1')); print(d['ratio'])" 2>/dev/null || echo 0; }

python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
ECHO_PID=$!
track "$ECHO_PID"
ECHO=$(poll echo.log)
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" --stats-file "$WORK/gateway.stats.json" > gateway.log 2>&1 &
GW_PID=$!
track "$GW_PID"
GATEWAY=$(poll gateway.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY" --stats-file "$WORK/client.stats.json" > client.log 2>&1 &
CLIENT_PID=$!
track "$CLIENT_PID"
CLIENT=$(poll client.log)
python3 - "$WORK" <<'PYMAKE'
import pathlib, sys
root = pathlib.Path(sys.argv[1])
line = b'{"kind":"delta","route":"uor","payload":"' + b'abcdef0123456789' * 8 + b'"}\n'
(root / 'text.bin').write_bytes(line * 8192)
PYMAKE
roundtrip "$CLIENT" text.bin text.out
cmp -s text.bin text.out || fail 'text roundtrip mismatch'
kill -TERM "$CLIENT_PID" 2>/dev/null || true
sleep 0.5
TEXT_RATIO=$(ratio_of "$WORK/client.stats.json")
python3 -c "import sys; sys.exit(0 if float('$TEXT_RATIO') < 0.5 else 1)" || fail "text ratio $TEXT_RATIO"

python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY" --stats-file "$WORK/client2.stats.json" > client2.log 2>&1 &
CLIENT2_PID=$!
track "$CLIENT2_PID"
CLIENT=$(poll client2.log)
python3 - "$WORK" <<'PYMAKE2'
import pathlib, random, sys
root = pathlib.Path(sys.argv[1])
rng = random.Random(7)
(root / 'random.bin').write_bytes(rng.randbytes(96 * 1024))
PYMAKE2
roundtrip "$CLIENT" random.bin random.out
cmp -s random.bin random.out || fail 'random roundtrip mismatch'
kill -TERM "$CLIENT2_PID" 2>/dev/null || true
sleep 0.5
RANDOM_RATIO=$(ratio_of "$WORK/client2.stats.json")
echo "text_ratio=$TEXT_RATIO random_ratio=$RANDOM_RATIO" >&2

python3 - "$WORK" <<'PYMAKE3'
import pathlib, random, sys
root = pathlib.Path(sys.argv[1])
rng = random.Random(20261017)
root.joinpath('artifact.bin').write_bytes(bytes((i * 7 + i // 251) % 251 for i in range(2 * 1024 * 1024)))
PYMAKE3
"$OBJPEER" serve --store server.store --object artifact.bin --sessions 1 --bind 127.0.0.1:0 > server.log 2>&1 &
SERVER_PID=$!
track "$SERVER_PID"
SERVER=$(poll server.log)
kill -TERM "$GW_PID" 2>/dev/null || true
sleep 0.5
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$SERVER" --stats-file "$WORK/gw2.stats.json" > gw2.log 2>&1 &
GW2_PID=$!
track "$GW2_PID"
GATEWAY2=$(poll gw2.log)
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$GATEWAY2" --stats-file "$WORK/cli2.stats.json" > cli2.log 2>&1 &
CLI2_PID=$!
track "$CLI2_PID"
CLIENT2=$(poll cli2.log)
SHA=$(shasum -a 256 artifact.bin | awk '{print $1}')
"$OBJPEER" pull --store client.store --addr "$CLIENT2" --select-sha256 "$SHA" --out pulled.bin > pull.log 2>&1 || fail 'objpeer pull through adapter'
cmp -s pulled.bin artifact.bin || fail 'objpeer pull mismatch'
kill -TERM "$CLI2_PID" 2>/dev/null || true
kill -TERM "$GW2_PID" 2>/dev/null || true
sleep 0.5
OBJ_RATIO=$(ratio_of "$WORK/cli2.stats.json")
echo "objpeer_ratio=$OBJ_RATIO" >&2
echo "WIRE_ADAPTER_PASS text_ratio=$TEXT_RATIO random_ratio=$RANDOM_RATIO objpeer_ratio=$OBJ_RATIO"
