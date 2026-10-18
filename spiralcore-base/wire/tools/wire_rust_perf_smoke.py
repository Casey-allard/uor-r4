#!/usr/bin/env python3
"""Consolidated native performance harness: same-run comparison of plain TCP and every native path."""
import json
import pathlib
import socket
import subprocess
import sys
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
BIN = PROJ / "crates" / "wire-rs" / "target" / "release" / "wire-rs"
WORK = PROJ / "local" / "wire-rust-perf"
PYTHON_TOOLS = str(PROJ / "tools")
RUNS = 7


def wait_ready(path, timeout=30.0):
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


def stop(procs):
    for proc in procs:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


def parse_seconds(output, marker):
    for token in output.split():
        if token.startswith("seconds="):
            return float(token.split("=")[1])
    raise SystemExit("no seconds in " + marker + ": " + output)


def summarize(values):
    ordered = sorted(values)
    return round(ordered[0], 5), round(sum(ordered) / len(ordered), 5), round(ordered[-1], 5)


def plain_client_process(addr, size):
    run = subprocess.run([str(BIN), "echo-client", "--connect", addr, "--bytes", str(size),
                          "--pattern", "text", "--plain"], capture_output=True, text=True, timeout=120)
    if run.returncode != 0 or "PLAIN_CLIENT_OK" not in run.stdout:
        raise SystemExit("plain client failed: " + run.stdout + run.stderr)
    return parse_seconds(run.stdout, "plain")


def wire_client_process(addr, size, route=""):
    command = [str(BIN), "echo-client", "--connect", addr, "--bytes", str(size), "--pattern", "text"]
    if route:
        command += ["--route", route]
    run = subprocess.run(command, capture_output=True, text=True, timeout=120)
    if run.returncode != 0 or "RUST_CLIENT_OK" not in run.stdout:
        raise SystemExit("wire client failed: " + run.stdout + run.stderr)
    return parse_seconds(run.stdout, "wire")


def plain_roundtrip_local(addr, size, reps):
    host, port = addr.rsplit(":", 1)
    sock = socket.create_connection((host, int(port)))
    payload = b"x" * size
    started = time.time()
    for _ in range(reps):
        sock.sendall(payload)
        got = 0
        while got < size:
            data = sock.recv(65536)
            if not data:
                break
            got += len(data)
        assert got == size
    elapsed = time.time() - started
    sock.close()
    return elapsed / reps


def wire_roundtrip_local(addr, size, reps, route=""):
    sys.path.insert(0, PYTHON_TOOLS)
    from wire_socket import WireSocket
    wire = WireSocket.connect(addr, route=route)
    payload = b"x" * size
    started = time.time()
    for _ in range(reps):
        wire.sendall(payload)
        got = 0
        while got < size:
            data = wire.recv(65536)
            if not data:
                break
            got += len(data)
        assert got == size
    elapsed = time.time() - started
    wire.close()
    return elapsed / reps


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    procs = []
    try:
        echo, _ = start([sys.executable, str(PROJ / "tools" / "tcp_echo.py"), "--listen", "127.0.0.1:0"],
                        WORK / "echo.log")
        procs.append(echo)
        echo_addr = wait_ready(WORK / "echo.log")

        gateway, _ = start([str(BIN), "gateway", "--listen", "127.0.0.1:0", "--target", echo_addr],
                           WORK / "gateway.log")
        procs.append(gateway)
        gateway_addr = wait_ready(WORK / "gateway.log")

        edge, _ = start([str(BIN), "edge", "--listen", "127.0.0.1:0", "--remote", gateway_addr],
                        WORK / "edge.log")
        procs.append(edge)
        edge_addr = wait_ready(WORK / "edge.log")

        hub, _ = start([str(BIN), "hub", "--listen", "127.0.0.1:0", "--route", "echo=" + echo_addr],
                       WORK / "hub.log")
        procs.append(hub)
        hub_addr = wait_ready(WORK / "hub.log")

        mux_gateway, _ = start([str(BIN), "mux-gateway", "--listen", "127.0.0.1:0",
                                "--route", "echo=" + echo_addr], WORK / "mux.log")
        procs.append(mux_gateway)
        mux_addr = wait_ready(WORK / "mux.log")

        python_gateway, _ = start([sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "gateway",
                                   "--listen", "127.0.0.1:0", "--target", echo_addr], WORK / "pgw.log")
        procs.append(python_gateway)
        py_gateway_addr = wait_ready(WORK / "pgw.log")
        python_edge, _ = start([sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "client",
                                "--listen", "127.0.0.1:0", "--remote", py_gateway_addr], WORK / "pedge.log")
        procs.append(python_edge)
        py_edge_addr = wait_ready(WORK / "pedge.log")

        size = 1310720
        plain = [plain_client_process(echo_addr, size) for _ in range(RUNS)]
        wire = [wire_client_process(gateway_addr, size) for _ in range(RUNS)]
        rust_adapter = [plain_roundtrip_local(edge_addr, 65536, 10) for _ in range(RUNS)]
        python_adapter = [plain_roundtrip_local(py_edge_addr, 65536, 10) for _ in range(RUNS)]
        rust_hub = [wire_client_process(hub_addr, 262144, route="echo") for _ in range(RUNS)]
        mux_runs = []
        for _ in range(3):
            run = subprocess.run([str(BIN), "mux-client", "--connect", mux_addr, "--route", "echo",
                                  "--streams", "8", "--bytes", "131072", "--pattern", "text"],
                                 capture_output=True, text=True, timeout=120)
            if "MUX_CLIENT_OK" not in run.stdout:
                raise SystemExit("mux client failed: " + run.stdout + run.stderr)
            mux_runs.append(parse_seconds(run.stdout, "mux"))

        plain_small = plain_roundtrip_local(echo_addr, 1024, 200)
        wire_small = wire_roundtrip_local(gateway_addr, 1024, 200)
        adapter_small = plain_roundtrip_local(edge_addr, 1024, 200)

        report = {
            "plain_peer_1.25MiB": summarize(plain),
            "wire_peer_1.25MiB": summarize(wire),
            "rust_adapter_64KiB_10x": summarize(rust_adapter),
            "python_adapter_64KiB_10x": summarize(python_adapter),
            "rust_hub_256KiB": summarize(rust_hub),
            "rust_mux_8x128KiB": summarize(mux_runs),
            "smallmsg_plain_us": round(plain_small * 1e6, 1),
            "smallmsg_wire_us": round(wire_small * 1e6, 1),
            "smallmsg_rust_adapter_us": round(adapter_small * 1e6, 1),
        }
        print(json.dumps(report, indent=2))
        print("WIRE_RUST_PERF_PASS runs=%d" % RUNS)
    finally:
        stop(procs)


if __name__ == "__main__":
    main()
