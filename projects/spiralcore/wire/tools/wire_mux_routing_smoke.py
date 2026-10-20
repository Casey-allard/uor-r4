#!/usr/bin/env python3
"""Per-stream mux routing: one mux connection opens streams to different named targets; unknown -> RESET."""
import pathlib
import sys
import threading

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_mux import WireMuxGateway, connect_mux


def labeled_server(label):
    import socket
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


def read_line(stream):
    data = b""
    while not data.endswith(b"\n"):
        chunk = stream.recv(1)
        if not chunk:
            break
        data += chunk
    return data


def stream_roundtrip(mux, route, expected_label, payload):
    stream = mux.open_stream(route)
    if read_line(stream) != expected_label.encode() + b"\n":
        raise SystemExit("stream route %s reached the wrong target" % route)
    stream.sendall(payload)
    stream.shutdown()
    received = b""
    while len(received) < len(payload):
        chunk = stream.recv(65536)
        if not chunk:
            break
        received += chunk
    ratio = (stream.wire_out + stream.wire_in) / (len(payload) * 2)
    stream.close()
    if received != payload:
        raise SystemExit("stream route %s payload mismatch" % route)
    return ratio


def main():
    listener_a, addr_a = labeled_server("MODEL")
    listener_b, addr_b = labeled_server("STORE")
    gateway = WireMuxGateway.bind("127.0.0.1:0", {"model": addr_a, "store": addr_b})
    threading.Thread(target=gateway.serve_forever, daemon=True).start()
    payload = (b'{"kind":"mux-route","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 2048
    # One connection, handshake route empty: each stream selects its own target.
    mux = connect_mux(gateway.address())
    model_ratio = stream_roundtrip(mux, "model", "MODEL", payload)
    store_ratio = stream_roundtrip(mux, "store", "STORE", payload)
    failed = False
    unknown = mux.open_stream("missing")
    try:
        unknown.sendall(b"x")
        unknown.recv(65536)
    except OSError:
        failed = True
    unknown.close()
    mux.close()
    gateway.stop()
    listener_a.close()
    listener_b.close()
    snapshot = gateway.snapshot()
    if not failed:
        raise SystemExit("unknown mux route did not fail closed")
    if snapshot["unknown_requests"] < 1:
        raise SystemExit("unknown per-stream route not counted: " + repr(snapshot))
    print("WIRE_MUX_ROUTING_PASS model_ratio=%.4f store_ratio=%.4f unknown_failed=true"
          % (model_ratio, store_ratio))


if __name__ == "__main__":
    main()
