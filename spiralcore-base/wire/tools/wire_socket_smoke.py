#!/usr/bin/env python3
"""Smoke the wire-socket protocol: peer-to-peer, and interop with the adapter in both directions."""
import pathlib
import random
import socket
import subprocess
import sys
import threading
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
import wire_socket
from wire_socket import WireListener, WireSocket

ADAPTER = PROJ / "tools" / "wire_adapter.py"
ECHO = PROJ / "tools" / "tcp_echo.py"


def read_all(sock, size=None):
    chunks = []
    total = 0
    while True:
        data = sock.recv(65536)
        if not data:
            break
        chunks.append(data)
        total += len(data)
        if size is not None and total >= size:
            break
    return b"".join(chunks)


def wire_echo(listener, stop):
    def serve():
        try:
            while not stop.is_set():
                conn = listener.accept()
                def handle(sock):
                    try:
                        while True:
                            data = sock.recv(65536)
                            if not data:
                                break
                            sock.sendall(data)
                    finally:
                        sock.close()
                threading.Thread(target=handle, args=(conn,), daemon=True).start()
        except OSError:
            pass
    threading.Thread(target=serve, daemon=True).start()


def plain_echo(listener):
    def serve():
        while True:
            try:
                conn, _ = listener.accept()
            except OSError:
                return
            def handle(sock):
                try:
                    while True:
                        data = sock.recv(65536)
                        if not data:
                            break
                        sock.sendall(data)
                finally:
                    sock.close()
            threading.Thread(target=handle, args=(conn,), daemon=True).start()
    threading.Thread(target=serve, daemon=True).start()


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
    proc = subprocess.Popen(command, stdout=handle, stderr=subprocess.STDOUT)
    return proc


def main():
    work = pathlib.Path(__file__).resolve().parent.parent / "local" / "wire-socket-smoke"
    work.mkdir(parents=True, exist_ok=True)
    procs = []
    try:
        # 1. peer-to-peer socket layer
        stop = threading.Event()
        listener = WireListener.bind("127.0.0.1:0")
        wire_echo(listener, stop)
        addr = "%s:%d" % listener.getsockname()[:2]
        text_payload = (b'{"kind":"socket","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 8192
        client = WireSocket.connect(addr)
        client.sendall(text_payload)
        client.shutdown()
        echoed = read_all(client, len(text_payload))
        client.close()
        assert echoed == text_payload, "peer-to-peer text mismatch"
        peer_ratio = client.stats()["ratio"]
        rng = random.Random(20261017)
        random_payload = rng.randbytes(96 * 1024)
        client = WireSocket.connect(addr)
        client.sendall(random_payload)
        client.shutdown()
        echoed = read_all(client, len(random_payload))
        client.close()
        assert echoed == random_payload, "peer-to-peer random mismatch"
        peer_random_ratio = client.stats()["ratio"]

        # 2. socket layer against the existing adapter gateway
        echo_proc = start([sys.executable, str(ECHO), "--listen", "127.0.0.1:0"], work / "echo.log")
        procs.append(echo_proc)
        echo_addr = wait_ready(work / "echo.log")
        gateway = start([sys.executable, str(ADAPTER), "gateway", "--listen", "127.0.0.1:0",
                         "--target", echo_addr], work / "gateway.log")
        procs.append(gateway)
        gateway_addr = wait_ready(work / "gateway.log")
        client = WireSocket.connect(gateway_addr)
        client.sendall(text_payload)
        client.shutdown()
        echoed = read_all(client, len(text_payload))
        client.close()
        assert echoed == text_payload, "adapter-gateway interop mismatch"
        gateway_ratio = client.stats()["ratio"]

        # 3. adapter edge in front of a WireListener service (reverse interop)
        listener2 = WireListener.bind("127.0.0.1:0")
        wire_echo(listener2, stop)
        service_addr = "%s:%d" % listener2.getsockname()[:2]
        edge = start([sys.executable, str(ADAPTER), "client", "--listen", "127.0.0.1:0",
                      "--remote", service_addr], work / "edge.log")
        procs.append(edge)
        edge_addr = wait_ready(work / "edge.log")
        host, port = edge_addr.rsplit(":", 1)
        plain = socket.create_connection((host, int(port)), timeout=30)
        plain.sendall(text_payload)
        plain.shutdown(socket.SHUT_WR)
        echoed = read_all(plain, len(text_payload))
        plain.close()
        assert echoed == text_payload, "reverse interop mismatch"
        stop.set()
        listener.close()
        listener2.close()
        print("WIRE_SOCKET_PASS peer_ratio=%.4f peer_random_ratio=%.4f gateway_ratio=%.4f reverse=ok"
              % (peer_ratio, peer_random_ratio, gateway_ratio))
    finally:
        for proc in procs:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    main()
