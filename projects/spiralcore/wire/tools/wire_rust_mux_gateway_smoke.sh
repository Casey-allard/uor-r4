#!/usr/bin/env bash
# Rust mux gateway smoke: UORADM01 streams over one connection, fail-closed unknown route, native benchmark.
set -u
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
PROJ=$(cd "$SCRIPT_DIR/.." && pwd)
BIN="$PROJ/crates/wire-rs/target/release/wire-rs"
WORK="$PROJ/local/wire-rust-mux"
mkdir -p "$WORK"
cd "$WORK"
fail() { echo "WIRE_RUST_MUX_FAIL: $1" >&2; exit 1; }
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
"$BIN" mux-gateway --listen 127.0.0.1:0 --route "echo=$ECHO" > mux.log 2>&1 &
track $!
MUX_ADDR=$(poll_file mux.log '^READY ' | awk '{print $2}')
python3 - "$PROJ/tools" "$MUX_ADDR" "$ECHO" "$BIN" <<'PYMUX' > client.log 2>&1 || fail 'python mux client -> rust mux gateway'
import random, socket, sys, threading, time
sys.path.insert(0, sys.argv[1])
from wire_mux import WireMuxGateway, connect_mux

rust_addr = sys.argv[2]
echo_addr = sys.argv[3]
text = (b'{"kind":"rust-mux","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
random_payload = random.Random(20261017).randbytes(96 * 1024)
payloads = [text if index % 2 == 0 else random_payload for index in range(8)]

def run(addr):
    started = time.time()
    mux = connect_mux(addr, route="echo")
    streams = [mux.open_stream() for _ in payloads]
    results = [None] * len(payloads)

    def worker(index):
        stream = streams[index]
        payload = payloads[index]
        stream.sendall(payload)
        stream.shutdown()
        received = b""
        while len(received) < len(payload):
            chunk = stream.recv(65536)
            if not chunk:
                break
            received += chunk
        results[index] = received == payload
        stream.close()

    threads = [threading.Thread(target=worker, args=(i,)) for i in range(len(payloads))]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    mux.close()
    assert all(results), results
    return time.time() - started

# unknown handshake route must fail closed (RESET for the opened stream)
failed = False
try:
    mux = connect_mux(rust_addr, route="missing")
    stream = mux.open_stream()
    stream.sendall(b"x")
    stream.recv(65536)
    mux.close()
except OSError:
    failed = True
assert failed, "unknown route did not fail closed"

# same-run benchmark against the Python mux gateway
python_gateway = WireMuxGateway.bind("127.0.0.1:0", {"echo": echo_addr})
threading.Thread(target=python_gateway.serve_forever, daemon=True).start()
rust_times = [run(rust_addr) for _ in range(3)]
python_times = [run(python_gateway.address()) for _ in range(3)]
python_gateway.stop()
print("MUX_BENCH rust_min=%.4f rust_mean=%.4f python_min=%.4f python_mean=%.4f" %
      (min(rust_times), sum(rust_times) / len(rust_times),
       min(python_times), sum(python_times) / len(python_times)))
print("PY_MUX_OK streams=%d" % len(payloads))
# per-stream routes: one mux connection, different named targets per stream
import os, pathlib, subprocess, time

def labeled_server(label):
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(8)

    def serve():
        while True:
            try:
                connection, _ = listener.accept()
            except OSError:
                return
            def handle(sock):
                try:
                    sock.sendall(label.encode() + b"\n")
                    while True:
                        chunk = sock.recv(65536)
                        if not chunk:
                            break
                        sock.sendall(chunk)
                except OSError:
                    pass
                finally:
                    try:
                        sock.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                    sock.close()
            threading.Thread(target=handle, args=(connection,), daemon=True).start()

    threading.Thread(target=serve, daemon=True).start()
    return "%s:%d" % listener.getsockname()[:2]

def read_line(stream):
    data = b""
    while not data.endswith(b"\n"):
        chunk = stream.recv(1)
        if not chunk:
            break
        data += chunk
    return data

model_addr = labeled_server("MODEL")
store_addr = labeled_server("STORE")
routes_log = pathlib.Path("mux-routes.log")
routes_log.write_text("")
routes_proc = subprocess.Popen(
    [sys.argv[4], "mux-gateway", "--listen", "127.0.0.1:0",
     "--route", "model=" + model_addr, "--route", "store=" + store_addr],
    stdout=open(routes_log, "w"), stderr=subprocess.STDOUT)
routes_addr = None
for _ in range(300):
    text = routes_log.read_text()
    for line in text.splitlines():
        if line.startswith("READY "):
            routes_addr = line.split()[1]
            break
    if routes_addr:
        break
    time.sleep(0.05)
assert routes_addr, "rust mux gateway routes did not start"

mux = connect_mux(routes_addr)  # empty handshake route: each stream selects its own target
payload = (b'{"kind":"mux-route","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 1024
for route, label in (("model", b"MODEL"), ("store", b"STORE")):
    stream = mux.open_stream(route)
    assert read_line(stream) == label + b"\n", route
    stream.sendall(payload)
    stream.shutdown()
    received = b""
    while len(received) < len(payload):
        chunk = stream.recv(65536)
        if not chunk:
            break
        received += chunk
    stream.close()
    assert received == payload, route
unknown_failed = False
try:
    stream = mux.open_stream("missing")
    stream.sendall(b"x")
    stream.recv(65536)
except OSError:
    unknown_failed = True
mux.close()
routes_proc.terminate()
try:
    routes_proc.wait(timeout=5)
except subprocess.TimeoutExpired:
    routes_proc.kill()
assert unknown_failed, "unknown per-stream route did not fail closed"
print("PY_MUX_ROUTES_OK")

PYMUX
grep -q '^PY_MUX_OK' client.log || fail 'python mux client bytes'
grep -q '^PY_MUX_ROUTES_OK' client.log || fail 'python mux per-stream routes'
RUST_CLIENT_TIMES=""
for _ in 1 2 3; do
  OUT=$("$BIN" mux-client --connect "$MUX_ADDR" --route echo --streams 8 --bytes 131072 --pattern text) || fail 'rust mux client'
  echo "$OUT" | grep -q MUX_CLIENT_OK || fail 'rust mux client bytes'
  SEC=$(echo "$OUT" | sed -n 's/.*seconds=\([0-9.]*\).*/\1/p')
  RUST_CLIENT_TIMES="$RUST_CLIENT_TIMES $SEC"
done
BENCH=$(grep -m1 '^MUX_BENCH' client.log)
echo "WIRE_RUST_MUX_PASS routes=ok rust_client_seconds:$RUST_CLIENT_TIMES $BENCH"
