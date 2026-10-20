#!/usr/bin/env bash
# Rust/Python interop smoke for the wire framing (both directions + adapter gateway).
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
CRATE="$PROJ/crates/wire-rs"
BIN="$CRATE/target/release/wire-rs"
WORK="$PROJ/local/wire-rust"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_RUST_INTEROP_FAIL: $1" >&2; exit 1; }
PIDS=""
cleanup() { for pid in $PIDS; do kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; done; }
trap cleanup EXIT
track() { PIDS="$PIDS $1"; }
poll_file() {
  local file=$1 pattern=$2 value=''
  for _ in $(seq 1 600); do
    value=$(grep -m1 "$pattern" "$file" 2>/dev/null)
    [ -n "$value" ] && break
    sleep 0.1
  done
  [ -n "$value" ] || fail "no '$pattern' in $file"
  echo "$value"
}
( cd "$CRATE" && ~/.cargo/bin/cargo build --release --offline ) > build.log 2>&1 || fail 'cargo build'
# 1. Rust server <- Python client (text)
"$BIN" echo-server --bind 127.0.0.1:0 > rust-server-text.log 2>&1 &
track $!
RUST_ADDR=$(poll_file rust-server-text.log '^READY ' | awk '{print $2}')
python3 - "$PROJ/tools" "$RUST_ADDR" <<'PYCLIENT' > py-client-text.log 2>&1 || fail 'python client -> rust server'
import sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
payload = (b'{"kind":"python-to-rust","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
wire = WireSocket.connect(sys.argv[2])
wire.sendall(payload)
wire.shutdown()
received = b""
while len(received) < len(payload):
    chunk = wire.recv(65536)
    if not chunk:
        break
    received += chunk
wire.close()
assert received == payload
print("PY_CLIENT_OK bytes=%d ratio=%.4f" % (len(payload), wire.stats()["ratio"]))
PYCLIENT
poll_file rust-server-text.log '^RUST_SERVER_OK' > /dev/null
poll_file py-client-text.log '^PY_CLIENT_OK' > /dev/null
# 2. Rust client -> Python listener (text + random)
python3 "$PROJ/tools/wire_echo_server.py" > py-server.log 2>&1 &
track $!
PY_ADDR=$(poll_file py-server.log '^READY ' | awk '{print $2}')
"$BIN" echo-client --connect "$PY_ADDR" --bytes 524288 --pattern text > rust-client-text.log 2>&1 || fail 'rust client -> python listener'
poll_file rust-client-text.log '^RUST_CLIENT_OK' > /dev/null
poll_file py-server.log '^PY_SERVER_OK' > /dev/null
python3 "$PROJ/tools/wire_echo_server.py" > py-server-random.log 2>&1 &
track $!
PY_ADDR2=$(poll_file py-server-random.log '^READY ' | awk '{print $2}')
"$BIN" echo-client --connect "$PY_ADDR2" --bytes 98304 --pattern random > rust-client-random.log 2>&1 || fail 'rust client random -> python listener'
poll_file rust-client-random.log '^RUST_CLIENT_OK' > /dev/null
poll_file py-server-random.log '^PY_SERVER_OK' > /dev/null
# 3. Rust client -> adapter gateway (proxy pair server side)
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
track $!
ECHO_ADDR=$(poll_file echo.log '^READY ' | awk '{print $2}')
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO_ADDR" > gateway.log 2>&1 &
track $!
GW_ADDR=$(poll_file gateway.log '^READY ' | awk '{print $2}')
"$BIN" echo-client --connect "$GW_ADDR" --bytes 524288 --pattern text > rust-client-gateway.log 2>&1 || fail 'rust client -> adapter gateway'
poll_file rust-client-gateway.log '^RUST_CLIENT_OK' > /dev/null
# 4. Python client random -> Rust server (raw fallback)
"$BIN" echo-server --bind 127.0.0.1:0 > rust-server-random.log 2>&1 &
track $!
RUST_ADDR2=$(poll_file rust-server-random.log '^READY ' | awk '{print $2}')
python3 - "$PROJ/tools" "$RUST_ADDR2" <<'PYCLIENT2' > py-client-random.log 2>&1 || fail 'python client random -> rust server'
import random, sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
payload = random.Random(20261017).randbytes(96 * 1024)
wire = WireSocket.connect(sys.argv[2])
wire.sendall(payload)
wire.shutdown()
received = b""
while len(received) < len(payload):
    chunk = wire.recv(65536)
    if not chunk:
        break
    received += chunk
wire.close()
assert received == payload
print("PY_CLIENT_OK bytes=%d ratio=%.4f" % (len(payload), wire.stats()["ratio"]))
PYCLIENT2
poll_file rust-server-random.log '^RUST_SERVER_OK' > /dev/null
poll_file py-client-random.log '^PY_CLIENT_OK' > /dev/null
RUST_TEXT_RATIO=$(grep -o 'ratio=[0-9.]*' rust-client-text.log | head -1 | cut -d= -f2)
PY_TEXT_RATIO=$(grep -o 'ratio=[0-9.]*' py-client-text.log | head -1 | cut -d= -f2)
RUST_RANDOM_RATIO=$(grep -o 'ratio=[0-9.]*' rust-client-random.log | head -1 | cut -d= -f2)
PY_RANDOM_RATIO=$(grep -o 'ratio=[0-9.]*' py-client-random.log | head -1 | cut -d= -f2)
RUST_GATEWAY_RATIO=$(grep -o 'ratio=[0-9.]*' rust-client-gateway.log | head -1 | cut -d= -f2)
echo "WIRE_RUST_INTEROP_PASS rust_text_ratio=$RUST_TEXT_RATIO py_text_ratio=$PY_TEXT_RATIO rust_random_ratio=$RUST_RANDOM_RATIO py_random_ratio=$PY_RANDOM_RATIO rust_gateway_ratio=$RUST_GATEWAY_RATIO"
