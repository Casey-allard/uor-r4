#!/usr/bin/env python3
"""TLS transport smoke for the socket layer: peer TLS, TLS routed gateway, wrong-transport fail-closed."""
import pathlib
import random
import subprocess
import sys
import threading
import time

PROJ = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJ / "tools"))
from wire_socket import WireGateway, WireListener, WireSocket

WORK = PROJ / "local" / "wire-tls-socket"
CERTS = WORK / "certs"


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


def roundtrip_tls(addr, payload, route="", ca=None):
    wire = WireSocket.connect(addr, route=route, tls=True, ca=ca if ca else str(CERTS / "ca.pem"))
    wire.sendall(payload)
    wire.shutdown()
    received = b""
    while len(received) < len(payload):
        chunk = wire.recv(65536)
        if not chunk:
            break
        received += chunk
    ratio = wire.stats()["ratio"]
    wire.close()
    return received, ratio


def tls_echo_once(listener):
    conn = listener.accept()
    total = 0
    while True:
        data = conn.recv(65536)
        if not data:
            break
        conn.sendall(data)
        total += len(data)
    conn.shutdown()
    conn.close()
    return total


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    subprocess.run(["bash", str(PROJ / "tools" / "make_tls_certs.sh"), str(CERTS)], check=True,
                   stdout=subprocess.DEVNULL)
    text_payload = (b'{"kind":"tls","payload":"' + b"abcdef0123456789" * 8 + b'"}\n') * 4096
    random_payload = random.Random(20261017).randbytes(96 * 1024)
    cert = str(CERTS / "server.pem")
    key = str(CERTS / "server.key")
    # 1. peer TLS
    listener = WireListener.bind("127.0.0.1:0", tls_cert=cert, tls_key=key)
    host, port = listener.getsockname()[:2]
    server_thread = threading.Thread(target=tls_echo_once, args=(listener,), daemon=True)
    server_thread.start()
    received, peer_ratio = roundtrip_tls("%s:%d" % (host, port), text_payload)
    assert received == text_payload
    random_thread_listener = WireListener.bind("127.0.0.1:0", tls_cert=cert, tls_key=key)
    other = threading.Thread(target=tls_echo_once, args=(random_thread_listener,), daemon=True)
    other.start()
    received, peer_random_ratio = roundtrip_tls("%s:%d" % random_thread_listener.getsockname()[:2], random_payload)
    assert received == random_payload
    # 2. TLS routed gateway in front of a plain echo service
    echo, echo_log = start([sys.executable, str(PROJ / "tools" / "tcp_echo.py"), "--listen", "127.0.0.1:0"],
                           WORK / "echo.log")
    try:
        echo_addr = wait_ready(WORK / "echo.log")
        gateway = WireGateway.bind("127.0.0.1:0", {"echo": echo_addr}, tls_cert=cert, tls_key=key)
        threading.Thread(target=gateway.serve_forever, daemon=True).start()
        received, gateway_ratio = roundtrip_tls(gateway.address(), text_payload, route="echo")
        assert received == text_payload
        gateway.stop()
    finally:
        echo.terminate()
        try:
            echo.wait(timeout=5)
        except subprocess.TimeoutExpired:
            echo.kill()
    # 3. wrong transport must fail closed
    plain_listener = WireListener.bind("127.0.0.1:0", tls_cert=cert, tls_key=key)

    def close_once():
        conn, _ = plain_listener.sock.accept()
        conn.close()

    threading.Thread(target=close_once, daemon=True).start()
    failed = False
    try:
        WireSocket.connect("%s:%d" % plain_listener.getsockname()[:2])  # plain client to TLS listener
    except OSError:
        failed = True
    plain_listener.close()
    tls_to_plain = WireListener.bind("127.0.0.1:0")

    def raw_close_once():
        conn, _ = tls_to_plain.sock.accept()
        conn.close()

    threading.Thread(target=raw_close_once, daemon=True).start()
    failed2 = False
    try:
        WireSocket.connect("%s:%d" % tls_to_plain.getsockname()[:2], tls=True, ca=str(CERTS / "ca.pem"))
    except OSError:
        failed2 = True
    tls_to_plain.close()
    if not (failed and failed2):
        raise SystemExit("wrong-transport cases did not fail closed: %s %s" % (failed, failed2))
    print("WIRE_SOCKET_TLS_PASS peer_ratio=%.4f peer_random_ratio=%.4f gateway_ratio=%.4f wrong_transport_failed=true"
          % (peer_ratio, peer_random_ratio, gateway_ratio))


if __name__ == "__main__":
    main()
