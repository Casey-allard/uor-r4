#!/usr/bin/env python3
"""Loopback throughput benchmark: direct TCP echo vs the paired wire adapter at several zlib levels."""
import argparse
import json
import pathlib
import random
import signal
import socket
import subprocess
import sys
import tempfile
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent


def start(command, log_path):
    handle = open(log_path, "w")
    return subprocess.Popen(command, stdout=handle, stderr=subprocess.STDOUT), handle


def wait_ready(log_path, timeout=30.0):
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
    raise SystemExit("no READY in %s" % log_path)


def stop(proc):
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


def roundtrip(addr, payload, timeout=180):
    import threading
    host, port = addr.rsplit(":", 1)
    started = time.time()
    sock = socket.create_connection((host, int(port)), timeout=timeout)
    received = {"bytes": 0}

    def read_loop():
        while True:
            data = sock.recv(1 << 20)
            if not data:
                break
            received["bytes"] += len(data)

    reader = threading.Thread(target=read_loop, daemon=True)
    reader.start()
    sock.sendall(payload)
    sock.shutdown(socket.SHUT_WR)
    reader.join()
    sock.close()
    elapsed = time.time() - started
    if received["bytes"] != len(payload):
        raise SystemExit("echo mismatch: %d / %d" % (received["bytes"], len(payload)))
    return elapsed


def make_payload(pattern, size):
    if pattern == "random":
        return random.Random(20261017).randbytes(size)
    line = b'{"kind":"delta","route":"uor","payload":"' + b"abcdef0123456789" * 8 + b'"}\n'
    return (line * (size // len(line) + 1))[:size]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--size", type=int, default=8 * 1024 * 1024)
    parser.add_argument("--levels", default="1,6,9")
    args = parser.parse_args()
    work = pathlib.Path(tempfile.mkdtemp(prefix="uor-wire-bench."))
    procs = []
    try:
        echo, echo_log = start([sys.executable, str(PROJ / "tools" / "tcp_echo.py"), "--listen", "127.0.0.1:0"], work / "echo.log")
        procs.append(echo)
        echo_addr = wait_ready(work / "echo.log")
        rows = []
        for pattern in ("text", "random"):
            payload = make_payload(pattern, args.size)
            elapsed = roundtrip(echo_addr, payload)
            rows.append({"mode": "direct", "pattern": pattern, "level": None, "wall": round(elapsed, 3),
                         "mibs": round(len(payload) / elapsed / (1024 * 1024), 1), "ratio": 1.0})
            for level in [int(value) for value in args.levels.split(",")]:
                gateway, gateway_log = start(
                    [sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "gateway", "--listen", "127.0.0.1:0",
                     "--target", echo_addr, "--level", str(level), "--stats-file", str(work / "gw.json")],
                    work / ("gw-%d.log" % level))
                procs.append(gateway)
                gateway_addr = wait_ready(work / ("gw-%d.log" % level))
                client, client_log = start(
                    [sys.executable, str(PROJ / "tools" / "wire_adapter.py"), "client", "--listen", "127.0.0.1:0",
                     "--remote", gateway_addr, "--level", str(level), "--stats-file", str(work / ("cli-%d.json" % level))],
                    work / ("cli-%d.log" % level))
                procs.append(client)
                client_addr = wait_ready(work / ("cli-%d.log" % level))
                elapsed = roundtrip(client_addr, payload)
                stop(client)
                stop(gateway)
                procs = [p for p in procs if p is not client and p is not gateway]
                client_log.close()
                gateway_log.close()
                stats_path = work / ("cli-%d.json" % level)
                ratio = 0.0
                if stats_path.exists():
                    try:
                        ratio = json.loads(stats_path.read_text())["ratio"]
                    except (OSError, ValueError):
                        ratio = 0.0
                rows.append({"mode": "adapter", "pattern": pattern, "level": level, "wall": round(elapsed, 3),
                             "mibs": round(len(payload) / elapsed / (1024 * 1024), 1), "ratio": ratio})
        print(json.dumps(rows))
        header = "mode    pattern  level  wall_s  MiB/s  ratio"
        print(header)
        for row in rows:
            print("%-7s %-8s %-6s %-7s %-6s %s" % (row["mode"], row["pattern"], row["level"] if row["level"] is not None else "-",
                                                   row["wall"], row["mibs"], row["ratio"]))
        print("ADAPTER_BENCH_PASS size=%d" % args.size)
    finally:
        for proc in procs:
            stop(proc)
        for handle in []:
            pass
        import shutil
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
