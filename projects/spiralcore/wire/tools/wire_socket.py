#!/usr/bin/env python3
"""UOR wire socket: the adapter's tunnel framing as a socket-layer protocol.

Two deployment shapes share one wire protocol:

  proxy pair   app --plain--> edge adapter ==compressed==> gateway adapter --plain--> service
  socket layer app uses WireSocket/WireListener directly (no proxy process) and speaks the same
               handshake + hybrid-compressed frames over its own TCP connection.

Handshake: MAGIC, u8 route-name length, route name, MAGIC reply.
Stream:    u32 header (bit 31 = raw frame) + payload; zero-length frame = end of stream.
           First chunk of each direction picks raw (incompressible, no expansion) or streaming zlib.
Compatible with tools/wire_adapter.py (a WireSocket can talk straight to an adapter gateway).
"""
import select
import socket
import ssl
import struct
import threading
import zlib

MAGIC = b"UORADP01"
FRAME_HEADER = struct.Struct(">I")
RAW_FLAG = 1 << 31
CHUNK = 65536
LEVEL = 6


def _recv_exact(sock, size):
    data = b""
    while len(data) < size:
        chunk = sock.recv(size - len(data))
        if not chunk:
            return None
        data += chunk
    return data


class WireSocket:
    def __init__(self, sock, route="", tls=False):
        self.sock = sock
        self.route = route
        self.tls = tls
        self.plain_in = 0
        self.plain_out = 0
        self.wire_in = 0
        self.wire_out = 0
        self._send_mode = None
        self._compressor = None
        self._recv_mode = None
        self._decompressor = None
        self._send_buf = bytearray()
        self._recv_buf = b""
        self._eof = False
        self._eof_sent = False

    @classmethod
    def connect(cls, addr, route="", timeout=30.0, tls=False, ca=None, server_name="localhost"):
        host, port = addr.rsplit(":", 1)
        sock = socket.create_connection((host, int(port)), timeout=timeout)
        if tls:
            context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH, cafile=ca)
            context.check_hostname = True
            context.verify_mode = ssl.CERT_REQUIRED
            sock = context.wrap_socket(sock, server_hostname=server_name)
        encoded = route.encode()
        sock.sendall(MAGIC + bytes([len(encoded)]) + encoded)
        if _recv_exact(sock, len(MAGIC)) != MAGIC:
            sock.close()
            raise OSError("wire handshake failed")
        sock.settimeout(None)
        return cls(sock, route, tls)

    def _choose_send_mode(self, data):
        # Probe a bounded sample: compressing a whole multi-megabyte incompressible chunk just to
        # discover it does not compress is pure CPU waste.
        sample = data[:65536] if len(data) > 65536 else data
        probe = zlib.compress(sample, LEVEL)
        if len(sample) >= 4096 and len(probe) >= len(sample):
            self._send_mode = "raw"
        else:
            self._send_mode = "compressed"
            self._compressor = zlib.compressobj(LEVEL)

    def sendall(self, data):
        if self._eof_sent:
            raise OSError("send after shutdown")
        if not data:
            return
        if self._send_mode is None:
            self._choose_send_mode(data)
        if self._send_mode == "raw":
            header = RAW_FLAG | len(data)
            frame = data
        else:
            frame = self._compressor.compress(data) + self._compressor.flush(zlib.Z_SYNC_FLUSH)
            header = len(frame)
        self.sock.sendall(FRAME_HEADER.pack(header) + frame)
        self.plain_in += len(data)
        self.wire_out += FRAME_HEADER.size + len(frame)

    def _read_frame(self):
        header = _recv_exact(self.sock, FRAME_HEADER.size)
        if header is None:
            self._eof = True
            return
        word = FRAME_HEADER.unpack(header)[0]
        raw = bool(word & RAW_FLAG)
        size = word & ~RAW_FLAG
        self.wire_in += FRAME_HEADER.size + size
        if size == 0:
            self._eof = True
            return
        blob = _recv_exact(self.sock, size)
        if blob is None:
            self._eof = True
            return
        if self._recv_mode is None:
            self._recv_mode = raw
        if self._recv_mode:
            data = blob
        else:
            if self._decompressor is None:
                self._decompressor = zlib.decompressobj()
            data = self._decompressor.decompress(blob)
        self.plain_out += len(data)
        self._recv_buf += data

    def recv(self, size=CHUNK):
        while not self._recv_buf and not self._eof:
            self._read_frame()
        if not self._recv_buf:
            return b""
        data = self._recv_buf[:size]
        self._recv_buf = self._recv_buf[size:]
        return data

    def shutdown(self, how=socket.SHUT_WR):
        if how in (socket.SHUT_WR, socket.SHUT_RDWR) and not self._eof_sent:
            self.sock.sendall(FRAME_HEADER.pack(0))
            self._eof_sent = True
            if not self.tls:
                # SSLSocket.shutdown tears the TLS session down (the peer loses the response direction);
                # the zero frame above is the wire-level end-of-stream signal for TLS peers.
                try:
                    self.sock.shutdown(socket.SHUT_WR)
                except OSError:
                    pass
        if how in (socket.SHUT_RD, socket.SHUT_RDWR):
            self._eof = True

    def close(self):
        try:
            self.shutdown(socket.SHUT_WR)
        except OSError:
            pass
        self.sock.close()

    def stats(self):
        plain = self.plain_in + self.plain_out
        wire = self.wire_in + self.wire_out
        return {"plain": plain, "wire": wire, "ratio": round(wire / plain, 4) if plain else 0.0}


class WireListener:
    def __init__(self, sock, ssl_context=None):
        self.sock = sock
        self.ssl_context = ssl_context

    @classmethod
    def bind(cls, addr, tls_cert=None, tls_key=None):
        host, port = addr.rsplit(":", 1)
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind((host, int(port)))
        sock.listen(64)
        context = None
        if tls_cert and tls_key:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.load_cert_chain(tls_cert, tls_key)
        return cls(sock, context)

    def getsockname(self):
        return self.sock.getsockname()

    def accept(self):
        conn, _ = self.sock.accept()
        tls = self.ssl_context is not None
        if tls:
            conn = self.ssl_context.wrap_socket(conn, server_side=True)
        header = _recv_exact(conn, len(MAGIC))
        if header != MAGIC:
            conn.close()
            raise OSError("bad wire magic")
        size = _recv_exact(conn, 1)
        if size is None:
            conn.close()
            raise OSError("bad wire route header")
        route = ""
        length = size[0]
        if length:
            raw = _recv_exact(conn, length)
            if raw is None:
                conn.close()
                raise OSError("bad wire route")
            route = raw.decode("utf-8", "replace")
        conn.sendall(MAGIC)
        return WireSocket(conn, route, tls)

    def close(self):
        self.sock.close()


class WireGateway:
    """Route-based socket-layer hub: handshake route names map to local TCP targets."""

    def __init__(self, listener, routes, stats_path=None):
        self.listener = listener
        self.routes = dict(routes)
        self.stats_path = stats_path
        self.lock = threading.Lock()
        self._buckets = {}
        self.unknown_requests = 0
        self._stop = threading.Event()

    @classmethod
    def bind(cls, addr, routes, stats_path=None, tls_cert=None, tls_key=None):
        return cls(WireListener.bind(addr, tls_cert, tls_key), routes, stats_path)

    def address(self):
        host, port = self.listener.getsockname()[:2]
        return "%s:%d" % (host, port)

    def _bucket(self, name):
        with self.lock:
            if name not in self._buckets:
                self._buckets[name] = {"connections": 0, "plain_in": 0, "plain_out": 0, "wire": 0}
            return self._buckets[name]

    def snapshot(self):
        with self.lock:
            routes = {name: dict(value) for name, value in self._buckets.items()}
            unknown = self.unknown_requests
        for value in routes.values():
            plain = value["plain_in"] + value["plain_out"]
            value["ratio"] = round(value["wire"] / plain, 4) if plain else 0.0
        return {"routes": routes, "unknown_requests": unknown}

    def write_stats(self):
        if not self.stats_path:
            return
        path = self.stats_path
        tmp = path + ".tmp"
        with open(tmp, "w") as handle:
            import json
            json.dump(self.snapshot(), handle, sort_keys=True)
        import os
        os.replace(tmp, path)

    def serve_forever(self):
        while not self._stop.is_set():
            try:
                wire = self.listener.accept()
            except OSError:
                break
            threading.Thread(target=self._handle, args=(wire,), daemon=True).start()

    def stop(self):
        self._stop.set()
        try:
            self.listener.close()
        except OSError:
            pass
        self.write_stats()

    def _handle(self, wire):
        name = wire.route or ""
        bucket = self._bucket(name)
        with self.lock:
            bucket["connections"] += 1
        target = self.routes.get(name)
        if target is None:
            with self.lock:
                self.unknown_requests += 1
            try:
                wire.shutdown(socket.SHUT_WR)
            except OSError:
                pass
            try:
                wire.close()
            except OSError:
                pass
            self.write_stats()
            return
        try:
            host, port = target.rsplit(":", 1)
            plain = socket.create_connection((host, int(port)), timeout=30)
            plain.settimeout(None)
        except OSError:
            try:
                wire.close()
            except OSError:
                pass
            self.write_stats()
            return
        if wire.tls:
            _relay_tls(wire, plain)
            return
        def client_to_target():
            try:
                while True:
                    data = wire.recv(65536)
                    if not data:
                        try:
                            plain.shutdown(socket.SHUT_WR)
                        except OSError:
                            pass
                        break
                    plain.sendall(data)
                    with self.lock:
                        bucket["plain_in"] += len(data)
            except OSError:
                pass
        def target_to_client():
            try:
                while True:
                    data = plain.recv(262144)
                    if not data:
                        try:
                            wire.shutdown(socket.SHUT_WR)
                        except OSError:
                            pass
                        break
                    wire.sendall(data)
                    with self.lock:
                        bucket["plain_out"] += len(data)
            except OSError:
                pass
        first = threading.Thread(target=client_to_target, daemon=True)
        second = threading.Thread(target=target_to_client, daemon=True)
        first.start()
        second.start()
        first.join()
        second.join()
        with self.lock:
            bucket["wire"] += wire.stats()["wire"]
        try:
            plain.close()
        except OSError:
            pass
        try:
            wire.close()
        except OSError:
            pass
        self.write_stats()


def _relay_tls(wire, plain):
    """Single-threaded relay: the SSLSocket is only ever touched by this thread.

    Zero-length frames carry end-of-stream in both directions; no SSLSocket.shutdown is used.
    """
    tls = wire.sock
    tls.setblocking(False)
    plain.setblocking(False)
    to_tls = b""
    to_plain = b""
    tls_buf = b""
    client_eof = False
    target_eof = False
    sent_zero = False
    plain_shut = False
    send_mode = None
    compressor = None
    recv_mode = None
    decompressor = None
    while True:
        if client_eof and target_eof and not to_tls and not to_plain and sent_zero:
            break
        rlist = []
        wlist = []
        if not client_eof and len(to_plain) < 1024 * 1024:
            rlist.append(tls)
        if not target_eof and len(to_tls) < 1024 * 1024:
            rlist.append(plain)
        if to_tls:
            wlist.append(tls)
        if to_plain:
            wlist.append(plain)
        try:
            readable, writable, _ = select.select(rlist, wlist, [], 1.0)
        except OSError:
            break
        if tls in readable:
            try:
                data = tls.recv(262144)
            except (ssl.SSLWantReadError, ssl.SSLWantWriteError, BlockingIOError):
                data = None
            except OSError:
                data = b""
            if data == b"":
                client_eof = True
            elif data:
                tls_buf += data
                while len(tls_buf) >= 4:
                    word = struct.unpack(">I", tls_buf[:4])[0]
                    raw = bool(word & 0x80000000)
                    size = word & 0x7FFFFFFF
                    if len(tls_buf) < 4 + size:
                        break
                    payload = tls_buf[4:4 + size]
                    tls_buf = tls_buf[4 + size:]
                    if size == 0:
                        client_eof = True
                        break
                    if recv_mode is None:
                        recv_mode = "raw" if raw else "compressed"
                        if recv_mode == "compressed":
                            decompressor = zlib.decompressobj()
                    data2 = payload if recv_mode == "raw" else decompressor.decompress(payload)
                    to_plain += data2
        if plain in readable and not target_eof:
            try:
                data = plain.recv(262144)
            except BlockingIOError:
                data = None
            except OSError:
                data = b""
            if data == b"":
                target_eof = True
            elif data:
                if send_mode is None:
                    probe = zlib.compress(data, 6)
                    if len(data) >= 4096 and len(probe) >= len(data):
                        send_mode = "raw"
                    else:
                        send_mode = "compressed"
                        compressor = zlib.compressobj(6)
                if send_mode == "raw":
                    to_tls += struct.pack(">I", 0x80000000 | len(data)) + data
                else:
                    payload = compressor.compress(data) + compressor.flush(zlib.Z_SYNC_FLUSH)
                    to_tls += struct.pack(">I", len(payload)) + payload
        if to_plain and plain in writable:
            try:
                sent = plain.send(to_plain)
                to_plain = to_plain[sent:]
            except BlockingIOError:
                pass
            except OSError:
                target_eof = True
        if to_tls and tls in writable:
            try:
                sent = tls.send(to_tls)
                to_tls = to_tls[sent:]
            except (ssl.SSLWantReadError, ssl.SSLWantWriteError, BlockingIOError):
                pass
            except OSError:
                client_eof = True
        if client_eof and not to_plain and not plain_shut:
            try:
                plain.shutdown(socket.SHUT_WR)
            except OSError:
                pass
            plain_shut = True
        if target_eof and not to_tls and not sent_zero:
            to_tls += struct.pack(">I", 0)
            sent_zero = True
    try:
        plain.close()
    except OSError:
        pass
    try:
        wire.close()
    except OSError:
        pass
