#!/usr/bin/env python3
"""Multiplexing smoke: N concurrent logical streams over one wire connection, exact per stream,
window flow control, and overhead versus separate single-stream connections."""
import pathlib
import random
import subprocess
import sys
import threading
import time
import socket

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_mux import WireMuxGateway, connect_mux
from wire_socket import WireSocket

STREAMS = 8
TEXT = (b'{"kind":"mux","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096   # 128 KiB
RANDOM = random.Random(20261017).randbytes(96 * 1024)


def wait_ready(log_path, timeout=20.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            text = pathlib.Path(log_path).read_text()
        except OSError:
            text = ""
        for line in text.splitlines():
            if line.startswith("READY "):
                return line.split()[1]
        time.sleep(0.05)
    raise SystemExit("no READY in " + str(log_path))


def start(command, log_path):
    handle = open(log_path, "w")
    return subprocess.Popen(command, stdout=handle, stderr=subprocess.STDOUT), handle


def mux_run(gateway_addr):
    payloads = []
    for index in range(STREAMS):
        payloads.append(TEXT if index % 2 == 0 else RANDOM)
    results = [None] * STREAMS
    mux = connect_mux(gateway_addr, route="echo")
    streams = [mux.open_stream() for _ in range(STREAMS)]

    def run(index):
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

    started = time.time()
    threads = [threading.Thread(target=run, args=(index,)) for index in range(STREAMS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    elapsed = time.time() - started
    wire = sum(stream.wire_out + stream.wire_in for stream in streams)
    windows = sum(stream.window_updates for stream in streams)
    mux.close()
    assert all(results), results
    total_plain = sum(len(payload) for payload in payloads)
    return elapsed, wire, windows, total_plain


def separate_run(adapter_addr):
    payloads = [TEXT if index % 2 == 0 else RANDOM for index in range(STREAMS)]
    wires = [0] * STREAMS

    def run(index):
        payload = payloads[index]
        wire = WireSocket.connect(adapter_addr)
        wire.sendall(payload)
        wire.shutdown()
        received = b""
        while len(received) < len(payload):
            chunk = wire.recv(65536)
            if not chunk:
                break
            received += chunk
        wire.close()
        wires[index] = wire.stats()["wire"]
        assert received == payload

    started = time.time()
    threads = [threading.Thread(target=run, args=(index,)) for index in range(STREAMS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    elapsed = time.time() - started
    return elapsed, sum(wires), sum(len(payload) for payload in payloads)


def main():
    work = PROJ / "local" / "wire-mux-smoke"
    work.mkdir(parents=True, exist_ok=True)
    procs = []
    try:
        echo, echo_log = start([sys.executable, str(PROJ / "tools" / "tcp_echo.py"), "--listen", "127.0.0.1:0"], work / "echo.log")
        procs.append(echo)
        echo_addr = wait_ready(work / "echo.log")
        gateway = WireMuxGateway.bind("127.0.0.1:0", {"echo": echo_addr})
        threading.Thread(target=gateway.serve_forever, daemon=True).start()
        runs = []
        windows = 0
        for _ in range(2):
            mux_secs, mux_wire, windows, mux_plain = mux_run(gateway.address())
            runs.append((mux_secs, mux_wire, mux_plain))
        snapshot = gateway.snapshot()
        assert snapshot["routes"]["echo"]["streams"] == STREAMS * 2, snapshot
        gateway.stop()
        adapter, adapter_log = start(
            [sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "gateway",
             "--listen", "127.0.0.1:0", "--target", echo_addr], work / "adapter.log")
        procs.append(adapter)
        adapter_addr = wait_ready(work / "adapter.log")
        sep_runs = []
        for _ in range(2):
            sep_secs, sep_wire, sep_plain = separate_run(adapter_addr)
            sep_runs.append((sep_secs, sep_wire, sep_plain))
        def best(runs):
            return min(runs)
        mux_best = best(runs)
        sep_best = best(sep_runs)
        print("WIRE_MUX_PASS streams=%d runs=2 mux_seconds=%.4f,%.4f separate_seconds=%.4f,%.4f "
              "mux_wire_best=%d separate_wire_best=%d mux_ratio=%.4f separate_ratio=%.4f window_updates=%d"
              % (STREAMS, runs[0][0], runs[1][0], sep_runs[0][0], sep_runs[1][0],
                 mux_best[1], sep_best[1], mux_best[1] / mux_best[2], sep_best[1] / sep_best[2], windows))
    finally:
        for proc in procs:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    main()
