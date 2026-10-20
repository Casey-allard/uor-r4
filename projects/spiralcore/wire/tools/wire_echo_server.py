#!/usr/bin/env python3
"""One-connection wire echo server (Python side of the Rust interop smoke)."""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from wire_socket import WireListener


def main():
    listener = WireListener.bind("127.0.0.1:0")
    host, port = listener.getsockname()[:2]
    print("READY %s:%d" % (host, port), flush=True)
    conn = listener.accept()
    route = conn.route
    total = 0
    while True:
        data = conn.recv(262144)
        if not data:
            break
        conn.sendall(data)
        total += len(data)
    conn.shutdown()
    stats = conn.stats()
    conn.close()
    listener.close()
    print("PY_SERVER_OK route=%s bytes=%d ratio=%.4f" % (route, total, stats["ratio"]))


if __name__ == "__main__":
    main()
