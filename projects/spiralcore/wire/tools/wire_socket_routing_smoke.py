#!/usr/bin/env python3
"""Routing smoke: one WireGateway, two named routes to distinct services, fail-closed unknown route."""
import pathlib
import socket
import sys
import threading
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_socket import WireGateway, WireSocket


def labeled_server(label):
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(16)

    def serve():
        while True:
            try:
                conn, _ = listener.accept()
            except OSError:
                return
            def handle(sock):
                try:
                    sock.sendall(label.encode() + b"\n")
                    while True:
                        data = sock.recv(65536)
                        if not data:
                            break
                        sock.sendall(data)
                except OSError:
                    pass
                finally:
                    try:
                        sock.shutdown(socket.SHUT_WR)
                    except OSError:
                        pass
                    sock.close()
            threading.Thread(target=handle, args=(conn,), daemon=True).start()

    threading.Thread(target=serve, daemon=True).start()
    return listener, "%s:%d" % listener.getsockname()[:2]


def read_line(wire):
    data = b""
    while not data.endswith(b"\n"):
        chunk = wire.recv(1)
        if not chunk:
            break
        data += chunk
    return data


def roundtrip(addr, route, expected_label, payload):
    wire = WireSocket.connect(addr, route=route)
    label = read_line(wire)
    if label != expected_label.encode() + b"\n":
        raise SystemExit("route %s reached %r" % (route, label))
    wire.sendall(payload)
    wire.shutdown()
    received = b""
    while len(received) < len(payload):
        chunk = wire.recv(65536)
        if not chunk:
            break
        received += chunk
    wire.close()
    if received != payload:
        raise SystemExit("route %s payload mismatch" % route)
    return wire.stats()["ratio"]


def main():
    listener_a, addr_a = labeled_server("MODEL")
    listener_b, addr_b = labeled_server("STORE")
    gateway = WireGateway.bind("127.0.0.1:0", {"model": addr_a, "store": addr_b})
    threading.Thread(target=gateway.serve_forever, daemon=True).start()
    hub = gateway.address()
    payload = (b'{"kind":"route","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
    model_ratio = roundtrip(hub, "model", "MODEL", payload)
    store_ratio = roundtrip(hub, "store", "STORE", payload)
    # unknown route: handshake completes, then the hub closes without a target
    probe = WireSocket.connect(hub, route="missing")
    probe.sendall(b"x")
    got = probe.recv(65536)
    probe.close()
    if got != b"":
        raise SystemExit("unknown route unexpectedly served data")
    snapshot = gateway.snapshot()
    routes = snapshot["routes"]
    if routes["model"]["connections"] != 1 or routes["store"]["connections"] != 1:
        raise SystemExit("route connection stats: " + repr(routes))
    if snapshot["unknown_requests"] != 1:
        raise SystemExit("unknown route counter: " + repr(snapshot))
    gateway.stop()
    listener_a.close()
    listener_b.close()
    print("WIRE_SOCKET_ROUTING_PASS model_ratio=%.4f store_ratio=%.4f unknown_failed=true model_wire=%d store_wire=%d"
          % (model_ratio, store_ratio, routes["model"]["wire"], routes["store"]["wire"]))


if __name__ == "__main__":
    main()
