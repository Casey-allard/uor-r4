#!/usr/bin/env python3
"""Overhead benchmark: plain TCP, proxy pair, socket layer, single mux stream, process interposer."""
import json
import os
import pathlib
import socket
import subprocess
import sys
import threading
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_socket import WireSocket
from wire_mux import WireMuxGateway, connect_mux

RUNS = 3
PAYLOAD = (b'{"kind":"bench","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
WORK = PROJ / "local" / "wire-bench"
GATEWAY = ""


def wait_ready(path, timeout=20.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            text = pathlib.Path(path).read_text()
        except OSError:
            text = ""
        for line in text.splitlines():
            if line.startswith("READY "):
                return line.split()[1]
        time.sleep(0.05)
    raise SystemExit("no READY in " + str(path))


def start(command, log):
    handle = open(log, "w")
    return subprocess.Popen(command, stdout=handle, stderr=subprocess.STDOUT), handle


def read_all(sock, size):
    chunks = []
    total = 0
    while total < size:
        data = sock.recv(65536)
        if not data:
            break
        chunks.append(data)
        total += len(data)
    return b"".join(chunks)


def plain_roundtrip(addr):
    host, port = addr.rsplit(":", 1)
    started = time.time()
    sock = socket.create_connection((host, int(port)))
    sock.sendall(PAYLOAD)
    sock.shutdown(socket.SHUT_WR)
    received = read_all(sock, len(PAYLOAD))
    sock.close()
    assert received == PAYLOAD
    return time.time() - started, 1.0


def wire_roundtrip(addr):
    started = time.time()
    wire = WireSocket.connect(addr)
    wire.sendall(PAYLOAD)
    wire.shutdown()
    received = b""
    while len(received) < len(PAYLOAD):
        chunk = wire.recv(65536)
        if not chunk:
            break
        received += chunk
    ratio = wire.stats()["ratio"]
    wire.close()
    assert received == PAYLOAD
    return time.time() - started, ratio


def mux_roundtrip(addr):
    started = time.time()
    mux = connect_mux(addr, route="echo")
    stream = mux.open_stream()
    stream.sendall(PAYLOAD)
    stream.shutdown()
    received = b""
    while len(received) < len(PAYLOAD):
        chunk = stream.recv(65536)
        if not chunk:
            break
        received += chunk
    ratio = (stream.wire_out + stream.wire_in) / (len(PAYLOAD) * 2)
    stream.close()
    mux.close()
    assert received == PAYLOAD
    return time.time() - started, ratio


def interposer_roundtrip(dylib, probe):
    fake = "127.0.0.1:65079"
    ratio_path = WORK / "interposer.stats"
    env = dict(os.environ)
    env.update({
        "DYLD_INSERT_LIBRARIES": str(dylib),
        "WIRE_ROUTES": fake + "=" + GATEWAY,
        "WIRE_STATS_FILE": str(ratio_path),
    })
    started = time.time()
    result = subprocess.run([str(probe), fake, "text", str(len(PAYLOAD))], env=env,
                            capture_output=True, text=True, timeout=120)
    elapsed = time.time() - started
    if result.returncode != 0 or "PROBE_OK" not in result.stdout:
        raise SystemExit("interposer probe failed: " + result.stdout + result.stderr)
    text = ratio_path.read_text().strip()
    prefix = "WIRE_INTERPOSER_STATS "
    if text.startswith(prefix):
        text = text[len(prefix):]
    ratio = json.loads(text)["ratio"]
    return elapsed, ratio


def summarize(samples):
    values = [item[0] for item in samples]
    return {
        "min": round(min(values), 4),
        "mean": round(sum(values) / len(values), 4),
        "max": round(max(values), 4),
        "ratio": samples[0][1],
    }


def main():
    global GATEWAY
    WORK.mkdir(parents=True, exist_ok=True)
    procs = []
    try:
        echo, echo_log = start([sys.executable, str(PROJ / "tools" / "tcp_echo.py"), "--listen", "127.0.0.1:0"], WORK / "echo.log")
        procs.append(echo)
        echo_addr = wait_ready(WORK / "echo.log")
        gateway, gateway_log = start([sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "gateway",
                                      "--listen", "127.0.0.1:0", "--target", echo_addr], WORK / "gateway.log")
        procs.append(gateway)
        GATEWAY = wait_ready(WORK / "gateway.log")
        edge, edge_log = start([sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "client",
                                "--listen", "127.0.0.1:0", "--remote", GATEWAY], WORK / "edge.log")
        procs.append(edge)
        edge_addr = wait_ready(WORK / "edge.log")
        mux_gateway = WireMuxGateway.bind("127.0.0.1:0", {"echo": echo_addr})
        threading.Thread(target=mux_gateway.serve_forever, daemon=True).start()

        dylib = WORK / "libwireinterposer.dylib"
        probe = WORK / "wire_probe"
        if sys.platform == "darwin":
            subprocess.run(["cc", "-dynamiclib", "-o", str(dylib), str(PROJ / "tools" / "wire_interposer.c"), "-lz"], check=True)
            subprocess.run(["codesign", "-f", "-s", "-", str(dylib)], check=True)
            subprocess.run(["cc", "-o", str(probe), str(PROJ / "tools" / "wire_interposer_probe.c")], check=True)

        modes = {}
        modes["plain"] = [plain_roundtrip(echo_addr) for _ in range(RUNS)]
        modes["proxy-pair"] = [plain_roundtrip(edge_addr) for _ in range(RUNS)]
        modes["socket-layer"] = [wire_roundtrip(GATEWAY) for _ in range(RUNS)]
        modes["mux-single-stream"] = [mux_roundtrip(mux_gateway.address()) for _ in range(RUNS)]
        if sys.platform == "darwin":
            modes["interposer"] = [interposer_roundtrip(dylib, probe) for _ in range(RUNS)]
        mux_gateway.stop()
        report = {name: summarize(samples) for name, samples in modes.items()}
        print(json.dumps(report, indent=2))
        print("WIRE_OVERHEAD_BENCH_PASS payload=%d runs=%d" % (len(PAYLOAD), RUNS))
    finally:
        for proc in procs:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    main()
