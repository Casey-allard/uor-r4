#!/usr/bin/env bash
# Rust adapter pair smoke: plain app -> Rust edge ==wire==> Rust gateway -> service.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
BIN="$PROJ/crates/wire-rs/target/release/wire-rs"
WORK="$PROJ/local/wire-rust-adapter"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_RUST_ADAPTER_FAIL: $1" >&2; exit 1; }
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
ECHO=$(poll_file echo.log '^READY ' | awk '{print $2}')
"$BIN" gateway --listen 127.0.0.1:0 --target "$ECHO" > gateway.log 2>&1 &
track $!
GATEWAY=$(poll_file gateway.log '^READY ' | awk '{print $2}')
"$BIN" edge --listen 127.0.0.1:0 --remote "$GATEWAY" > edge.log 2>&1 &
track $!
EDGE=$(poll_file edge.log '^READY ' | awk '{print $2}')
python3 "$PROJ/tools/wire_adapter.py" gateway --listen 127.0.0.1:0 --target "$ECHO" > py-gateway.log 2>&1 &
track $!
PY_GATEWAY=$(poll_file py-gateway.log '^READY ' | awk '{print $2}')
python3 "$PROJ/tools/wire_adapter.py" client --listen 127.0.0.1:0 --remote "$PY_GATEWAY" > py-edge.log 2>&1 &
track $!
PY_EDGE=$(poll_file py-edge.log '^READY ' | awk '{print $2}')
# 1. plain app -> rust edge -> rust gateway -> echo, measured against plain TCP
python3 - "$ECHO" "$EDGE" "$PY_EDGE" <<'PYBENCH' > bench.log 2>&1 || fail 'adapter bench'
import socket, sys

payload = (b'{"kind":"rust-adapter","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 8192

def run(addr, reps=10):
    times = []
    for _ in range(reps):
        import time
        started = time.time()
        host, port = addr.rsplit(":", 1)
        sock = socket.create_connection((host, int(port)))
        sock.sendall(payload)
        sock.shutdown(socket.SHUT_WR)
        got = 0
        while got < len(payload):
            data = sock.recv(262144)
            if not data:
                break
            got += len(data)
        sock.close()
        assert got == len(payload)
        times.append(time.time() - started)
    return times

def stats(values):
    values = sorted(values)
    return values[0], sum(values) / len(values)

plain = stats(run(sys.argv[1]))
edge = stats(run(sys.argv[2]))
python_edge = stats(run(sys.argv[3]))
print("ADAPTER_BENCH plain_min=%.5f plain_mean=%.5f rust_edge_min=%.5f rust_edge_mean=%.5f python_edge_min=%.5f python_edge_mean=%.5f" %
      (plain[0], plain[1], edge[0], edge[1], python_edge[0], python_edge[1]))
PYBENCH
# python WireSocket check with tools path passed via environment-free argument
python3 - "$PROJ/tools" "$GATEWAY" <<'PYWIRE' > wire.log 2>&1 || fail 'python wire client -> rust gateway'
import sys
sys.path.insert(0, sys.argv[1])
from wire_socket import WireSocket
payload = (b'{"kind":"python","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
wire = WireSocket.connect(sys.argv[2])
wire.sendall(payload)
wire.shutdown()
received = b""
while len(received) < len(payload):
    data = wire.recv(262144)
    if not data:
        break
    received += data
ratio = wire.stats()["ratio"]
wire.close()
assert received == payload
print("WIRE_CLIENT_OK bytes=%d ratio=%.4f" % (len(payload), ratio))
PYWIRE
grep -q '^WIRE_CLIENT_OK' wire.log || fail 'wire client bytes'
# 3. unmodified binary through the interposer and the Rust adapter pair (macOS)
if [ "$(uname)" = "Darwin" ]; then
  cc -w -dynamiclib -o libwireinterposer.dylib "$PROJ/tools/wire_interposer.c" -lz || fail 'interposer build'
  codesign -f -s - libwireinterposer.dylib >/dev/null 2>&1 || fail 'codesign'
  cc -w -o wire_probe "$PROJ/tools/wire_interposer_probe.c" || fail 'probe build'
  FAKE=127.0.0.1:65083
  DYLD_INSERT_LIBRARIES="$WORK/libwireinterposer.dylib" WIRE_ROUTES="$FAKE=$GATEWAY" \
    WIRE_STATS_FILE="$WORK/probe.stats" ./wire_probe "$FAKE" text 1048576 > probe.log 2>&1 || fail 'probe through rust adapter'
  grep -q 'PROBE_OK' probe.log || fail 'probe bytes'
fi
BENCH=$(grep -m1 '^ADAPTER_BENCH' bench.log)
WIRE=$(grep -m1 '^WIRE_CLIENT_OK' wire.log)
echo "WIRE_RUST_ADAPTER_PASS $BENCH $WIRE"
