#!/usr/bin/env python3
"""Regression: a route's wire counter sums every completed connection.

A WireGateway bucket is persistent per route.  The bug fixed here assigned
bucket["wire"] = wire.stats()["wire"] at connection completion, replacing the
route aggregate with the latest connection while plain_in/plain_out kept
accumulating, so the documented ratio understated traffic on reused routes.
"""
import pathlib
import socket
import sys
import threading
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_socket import WireGateway, WireSocket


def echo_server():
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


def roundtrip(addr, route, payload):
    wire = WireSocket.connect(addr, route=route)
    wire.sendall(payload)
    wire.shutdown()
    received = b""
    while True:
        chunk = wire.recv(65536)
        if not chunk:
            break
        received += chunk
    wire.close()
    if received != payload:
        raise SystemExit("route %s payload mismatch" % route)
    return wire.stats()["wire"]


def wait_for(predicate, timeout=5.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        if predicate():
            return True
        time.sleep(0.01)
    return False


def main():
    listener, target = echo_server()
    gateway = WireGateway.bind("127.0.0.1:0", {"metrics": target})
    threading.Thread(target=gateway.serve_forever, daemon=True).start()
    hub = gateway.address()
    try:
        payload_a = (b"first-connection-" + b"abcdef0123456789" * 32 + b"\n") * 192
        payload_b = (b"second-connection-" + b"abcdef0123456789" * 32 + b"\n") * 576
        wire_a = roundtrip(hub, "metrics", payload_a)
        if not wait_for(lambda: gateway.snapshot()["routes"]["metrics"]["wire"] == wire_a):
            raise SystemExit("first connection did not finish: " + repr(gateway.snapshot()))
        wire_b = roundtrip(hub, "metrics", payload_b)
        if wire_a == wire_b:
            raise SystemExit("connections need distinct wire counts to prove aggregation")
        # The bucket changes from wire_a only when the second handler completes and
        # records its own count, so this is a completion signal rather than a sleep.
        if not wait_for(lambda: gateway.snapshot()["routes"]["metrics"]["wire"] != wire_a):
            raise SystemExit("second connection did not finish: " + repr(gateway.snapshot()))

        snapshot = gateway.snapshot()
        bucket = snapshot["routes"]["metrics"]
        expected_wire = wire_a + wire_b
        expected_plain = 2 * (len(payload_a) + len(payload_b))  # each echo direction
        expected_ratio = round(expected_wire / expected_plain, 4)
        last_ratio = round(wire_b / expected_plain, 4)
        if bucket["connections"] != 2:
            raise SystemExit("route connection count: " + repr(snapshot))
        if bucket["plain_in"] + bucket["plain_out"] != expected_plain:
            raise SystemExit("route plain aggregate: " + repr(snapshot))
        if bucket["wire"] != expected_wire:
            raise SystemExit("route wire is not the sum of both connections: " + repr(snapshot))
        if bucket["ratio"] != expected_ratio:
            raise SystemExit("route ratio is not the aggregate ratio: " + repr(snapshot))
        if last_ratio == expected_ratio:
            raise SystemExit("test payloads cannot distinguish aggregate from last connection")
        print("WIRE_SOCKET_STATS_PASS connections=2 wire_a=%d wire_b=%d wire_total=%d plain=%d "
              "ratio=%.4f last_only_ratio=%.4f"
              % (wire_a, wire_b, bucket["wire"], expected_plain, bucket["ratio"], last_ratio))
    finally:
        gateway.stop()
        listener.close()


if __name__ == "__main__":
    main()
