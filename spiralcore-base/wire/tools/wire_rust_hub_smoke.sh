#!/usr/bin/env bash
# Rust routed hub smoke: native hub serving Python clients and an unmodified binary via the interposer.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
BIN="$PROJ/crates/wire-rs/target/release/wire-rs"
WORK="$PROJ/local/wire-rust-hub"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_RUST_HUB_FAIL: $1" >&2; exit 1; }
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
( cd "$PROJ/crates/wire-rs" && ~/.cargo/bin/cargo build --release --offline ) > build.log 2>&1 || fail 'cargo build'
python3 "$PROJ/tools/tcp_echo.py" --listen 127.0.0.1:0 > echo.log 2>&1 &
track $!
ECHO_ADDR=$(poll_file echo.log '^READY ' | awk '{print $2}')
"$BIN" hub --listen 127.0.0.1:0 --route "echo=$ECHO_ADDR" > rust-hub.log 2>&1 &
track $!
HUB_ADDR=$(poll_file rust-hub.log '^READY ' | awk '{print $2}')
# 1. Python WireSocket clients (text + random)
python3 - "$PROJ/tools" "$HUB_ADDR" <<'PYCLIENT' > client.log 2>&1 || fail 'python client -> rust hub'
import random, sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
text = (b'{"kind":"rust-hub","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 8192
for payload in (text, random.Random(20261017).randbytes(96 * 1024)):
    wire = WireSocket.connect(sys.argv[2], route="echo")
    wire.sendall(payload)
    wire.shutdown()
    received = b""
    while len(received) < len(payload):
        chunk = wire.recv(262144)
        if not chunk:
            break
        received += chunk
    wire.close()
    assert received == payload
print("PY_CLIENTS_OK")
PYCLIENT
poll_file client.log '^PY_CLIENTS_OK' > /dev/null
# 2. unknown route fails closed
python3 - "$PROJ/tools" "$HUB_ADDR" <<'PYUNKNOWN' > unknown.log 2>&1 || fail 'unknown route'
import sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
wire = WireSocket.connect(sys.argv[2], route="missing")
wire.sendall(b"x")
assert wire.recv(65536) == b""
wire.close()
print("UNKNOWN_FAILED_CLOSED")
PYUNKNOWN
poll_file unknown.log '^UNKNOWN_FAILED_CLOSED' > /dev/null
# 3. unmodified binary through the interposer, routed by the Rust hub (macOS)
if [ "$(uname)" = "Darwin" ]; then
  cc -w -dynamiclib -o libwireinterposer.dylib "$PROJ/tools/wire_interposer.c" -lz || fail 'interposer build'
  codesign -f -s - libwireinterposer.dylib >/dev/null 2>&1 || fail 'codesign'
  cc -w -o wire_probe "$PROJ/tools/wire_interposer_probe.c" || fail 'probe build'
  FAKE=127.0.0.1:65082
  DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$HUB_ADDR#echo" \
    WIRE_STATS_FILE="$WORK/probe.stats" ./wire_probe "$FAKE" text 1048576 > probe.log 2>&1 || fail 'interposed probe through rust hub'
  grep -q 'PROBE_OK' probe.log || fail 'interposed probe bytes'
fi
# 4. compare Rust hub vs Python hub on the same route (3 runs each)
python3 "$PROJ/tools/wire_hub.py" --listen 127.0.0.1:0 --route "echo=$ECHO_ADDR" > py-hub.log 2>&1 &
track $!
PY_HUB=$(poll_file py-hub.log '^READY ' | awk '{print $2}')
python3 - "$PROJ/tools" "$HUB_ADDR" "$PY_HUB" <<'PYBENCH' > hub-bench.log 2>&1 || fail 'hub benchmark'
import sys, time
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
payload = (b'{"kind":"hub-bench","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 8192
def run(addr):
    started = time.time()
    wire = WireSocket.connect(addr, route="echo")
    wire.sendall(payload)
    wire.shutdown()
    received = 0
    while received < len(payload):
        data = wire.recv(262144)
        if not data:
            break
        received += len(data)
    ratio = wire.stats()["ratio"]
    wire.close()
    assert received == len(payload)
    return time.time() - started, ratio
def stats(values):
    values = sorted(values)
    return values[0], sum(values) / len(values)
rust = [run(sys.argv[2]) for _ in range(3)]
python_ = [run(sys.argv[3]) for _ in range(3)]
r_min, r_mean = stats([v[0] for v in rust])
p_min, p_mean = stats([v[0] for v in python_])
print("HUB_BENCH rust_min=%.4f rust_mean=%.4f python_min=%.4f python_mean=%.4f rust_ratio=%.4f python_ratio=%.4f"
      % (r_min, r_mean, p_min, p_mean, rust[0][1], python_[0][1]))
PYBENCH
poll_file hub-bench.log '^HUB_BENCH' > /dev/null
RUST_CLIENT_TIMES=""
for _ in 1 2 3; do
  LINE=$("$BIN" echo-client --connect "$HUB_ADDR" --route echo --bytes 1310720 --pattern text) || fail 'rust client through hub'
  echo "$LINE" | grep -q RUST_CLIENT_OK || fail 'rust client through hub bytes'
  SECONDS_VALUE=$(echo "$LINE" | sed -n 's/.*seconds=\([0-9.]*\).*/\1/p')
  RUST_CLIENT_TIMES="$RUST_CLIENT_TIMES $SECONDS_VALUE"
done
BENCH=$(grep -m1 '^HUB_BENCH' hub-bench.log)
echo "WIRE_RUST_HUB_PASS rust_client_seconds:$RUST_CLIENT_TIMES $BENCH"
