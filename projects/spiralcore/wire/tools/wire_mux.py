#!/usr/bin/env python3
"""Opt-in multiplexing for the wire protocol: several logical streams over one TCP connection.

Wire format (separate handshake so single-stream peers fail closed on it):
  handshake   MAGIC_MUX (UORADM01), u8 route length, route name, MAGIC_MUX reply
  frame       u16 stream_id, u8 type, u16 length, payload
  types       1 OPEN (empty payload), 2 DATA compressed, 3 DATA raw, 4 FIN (empty),
              5 WINDOW (u32 credit), 6 RESET
Per logical stream: independent compression state, a 64 KiB send window advanced by WINDOW frames,
and FIN for half-close. One writer lock serializes frames on the shared connection.
"""
import socket
import struct
import threading
import zlib

MUX_MAGIC = b"UORADM01"
HEADER = struct.Struct(">HBH")
OPEN = 1
DATA_COMPRESSED = 2
DATA_RAW = 3
FIN = 4
WINDOW = 5
RESET = 6
INITIAL_WINDOW = 64 * 1024
LEVEL = 6
CHUNK = 65535


def recv_exact(sock, size):
    data = b""
    while len(data) < size:
        chunk = sock.recv(size - len(data))
        if not chunk:
            return None
        data += chunk
    return data


def split(addr):
    host, port = addr.rsplit(":", 1)
    return host, int(port)


class _Writer:
    def __init__(self, sock):
        self.sock = sock
        self.lock = threading.Lock()

    def frame(self, stream_id, kind, payload=b""):
        if len(payload) > 65535:
            raise ValueError("mux frame too large")
        data = HEADER.pack(stream_id, kind, len(payload)) + payload
        with self.lock:
            self.sock.sendall(data)
        return HEADER.size + len(payload)


class MuxStream:
    def __init__(self, connection, stream_id, route=""):
        self.connection = connection
        self.stream_id = stream_id
        self.route = route
        self.cond = threading.Condition()
        self.recv_buf = bytearray()
        self.eof = False
        self.error = None
        self.credit = INITIAL_WINDOW
        self.send_mode = None
        self.compressor = None
        self.recv_mode = None
        self.decompressor = None
        self.fin_sent = False
        self.consumed = 0
        self.plain_in = 0
        self.plain_out = 0
        self.wire_out = 0
        self.wire_in = 0
        self.window_updates = 0

    # sender side -----------------------------------------------------------------
    def sendall(self, data):
        offset = 0
        while offset < len(data):
            chunk = data[offset:offset + CHUNK]
            with self.cond:
                while self.credit <= 0 and self.error is None:
                    self.cond.wait()
                if self.error is not None:
                    raise OSError("stream reset")
                allowed = min(len(chunk), self.credit)
                self.credit -= allowed
                piece = chunk[:allowed]
            if self.send_mode is None:
                probe = zlib.compress(piece, LEVEL)
                if len(piece) >= 4096 and len(probe) >= len(piece):
                    self.send_mode = "raw"
                else:
                    self.send_mode = "compressed"
                    self.compressor = zlib.compressobj(LEVEL)
            if self.send_mode == "raw":
                payload = piece
                kind = DATA_RAW
            else:
                payload = self.compressor.compress(piece) + self.compressor.flush(zlib.Z_SYNC_FLUSH)
                kind = DATA_COMPRESSED
            self.wire_out += self.connection.writer.frame(self.stream_id, kind, payload)
            self.plain_in += len(piece)
            offset += allowed

    def shutdown(self):
        with self.cond:
            if self.fin_sent:
                return
            self.fin_sent = True
        self.wire_out += self.connection.writer.frame(self.stream_id, FIN)

    def close(self):
        try:
            self.shutdown()
        finally:
            self.connection.remove(self.stream_id)

    # receiver side ---------------------------------------------------------------
    def recv(self, size=CHUNK):
        with self.cond:
            while not self.recv_buf and not self.eof and self.error is None:
                self.cond.wait()
            if self.error is not None:
                raise OSError("stream reset")
            if not self.recv_buf:
                return b""
            data = bytes(self.recv_buf[:size])
            del self.recv_buf[:len(data)]
            self.consumed += len(data)
            self.plain_out += len(data)
            window = self.consumed
            self.consumed = 0
        if window > 0:
            self.window_updates += 1
            self.wire_in += self.connection.writer.frame(self.stream_id, WINDOW, struct.pack(">I", window))
        return data

    def _on_frame(self, kind, payload):
        with self.cond:
            self.wire_in += HEADER.size + len(payload)
            if kind in (DATA_COMPRESSED, DATA_RAW):
                if self.recv_mode is None:
                    self.recv_mode = kind == DATA_RAW
                if self.recv_mode:
                    data = payload
                else:
                    if self.decompressor is None:
                        self.decompressor = zlib.decompressobj()
                    data = self.decompressor.decompress(payload)
                self.recv_buf += data
            elif kind == FIN:
                self.eof = True
            elif kind == WINDOW:
                self.credit += struct.unpack(">I", payload)[0]
            elif kind == RESET:
                self.error = OSError("stream reset by peer")
            self.cond.notify_all()


class MuxConnection:
    def __init__(self, sock):
        self.sock = sock
        self.writer = _Writer(sock)
        self.lock = threading.Lock()
        self.streams = {}
        self.next_id = 1
        self.closed = False
        self.reader = threading.Thread(target=self._reader_loop, daemon=True)
        self.reader.start()

    def open_stream(self, route=None):
        route_name = route or ""
        with self.lock:
            stream_id = self.next_id
            self.next_id += 1
            stream = MuxStream(self, stream_id, route_name)
            self.streams[stream_id] = stream
        encoded = route_name.encode()[:255]
        self.writer.frame(stream_id, OPEN, encoded)
        return stream

    def remove(self, stream_id):
        with self.lock:
            self.streams.pop(stream_id, None)

    def _reader_loop(self):
        try:
            while True:
                header = recv_exact(self.sock, HEADER.size)
                if header is None:
                    break
                stream_id, kind, size = HEADER.unpack(header)
                payload = b""
                if size:
                    payload = recv_exact(self.sock, size)
                    if payload is None:
                        break
                with self.lock:
                    stream = self.streams.get(stream_id)
                if stream is not None:
                    stream._on_frame(kind, payload)
        except OSError:
            pass
        finally:
            with self.lock:
                streams = list(self.streams.values())
            for stream in streams:
                with stream.cond:
                    stream.eof = True
                    stream.cond.notify_all()

    def close(self):
        self.closed = True
        try:
            self.sock.close()
        except OSError:
            pass


def connect_mux(addr, route=""):
    sock = socket.create_connection(split(addr), timeout=30)
    encoded = route.encode()
    sock.sendall(MUX_MAGIC + bytes([len(encoded)]) + encoded)
    if recv_exact(sock, len(MUX_MAGIC)) != MUX_MAGIC:
        sock.close()
        raise OSError("mux handshake failed")
    sock.settimeout(None)
    return MuxConnection(sock)


class _ServerStream:
    def __init__(self, connection, stream_id, target):
        self.connection = connection
        self.stream_id = stream_id
        self.target = target
        self.cond = threading.Condition()
        self.credit = INITIAL_WINDOW
        self.send_mode = None
        self.compressor = None
        self.recv_mode = None
        self.decompressor = None
        self.eof = False
        self.fin_sent = False
        self.closed = False
        self.plain = 0
        self.wire = 0
        self.window_updates = 0
        self.input_done = False
        self.output_done = False
        self.thread = threading.Thread(target=self._target_reader, daemon=True)

    def start(self):
        self.thread.start()

    def _target_reader(self):
        try:
            while True:
                data = self.target.recv(65536)
                if not data:
                    break
                offset = 0
                while offset < len(data):
                    chunk = data[offset:offset + CHUNK]
                    with self.cond:
                        while self.credit <= 0 and not self.closed:
                            self.cond.wait()
                        if self.closed:
                            return
                        allowed = min(len(chunk), self.credit)
                        self.credit -= allowed
                        piece = chunk[:allowed]
                    if self.send_mode is None:
                        probe = zlib.compress(piece, LEVEL)
                        if len(piece) >= 4096 and len(probe) >= len(piece):
                            self.send_mode = "raw"
                        else:
                            self.send_mode = "compressed"
                            self.compressor = zlib.compressobj(LEVEL)
                    if self.send_mode == "raw":
                        payload = piece
                        kind = DATA_RAW
                    else:
                        payload = self.compressor.compress(piece) + self.compressor.flush(zlib.Z_SYNC_FLUSH)
                        kind = DATA_COMPRESSED
                    self.wire += self.connection.writer.frame(self.stream_id, kind, payload)
                    self.plain += len(piece)
                    offset += allowed
        except OSError:
            pass
        finally:
            try:
                self.connection.writer.frame(self.stream_id, FIN)
            except OSError:
                pass
            self.output_done = True
            self.connection.finish_stream_if_done(self)

    def on_frame(self, kind, payload):
        if kind in (DATA_COMPRESSED, DATA_RAW):
            if self.recv_mode is None:
                self.recv_mode = kind == DATA_RAW
            if self.recv_mode:
                data = payload
            else:
                if self.decompressor is None:
                    self.decompressor = zlib.decompressobj()
                data = self.decompressor.decompress(payload)
            total = 0
            while total < len(data):
                written = self.target.send(data[total:])
                if written <= 0:
                    break
                total += written
            if total:
                self.connection.writer.frame(self.stream_id, WINDOW, struct.pack(">I", total))
                self.window_updates += 1
        elif kind == FIN:
            self.input_done = True
            try:
                self.target.shutdown(socket.SHUT_WR)
            except OSError:
                pass
            self.connection.finish_stream_if_done(self)
        elif kind == WINDOW:
            with self.cond:
                self.credit += struct.unpack(">I", payload)[0]
                self.cond.notify_all()
        elif kind == RESET:
            self.close()

    def close(self):
        with self.cond:
            self.closed = True
            self.cond.notify_all()
        try:
            self.target.close()
        except OSError:
            pass
        self.connection.finish_stream(self.stream_id)


class WireMuxGateway:
    """Hub for muxed connections: handshake route selects the target for all streams on that connection."""

    def __init__(self, listener, routes, stats_path=None):
        self.listener = listener
        self.routes = dict(routes)
        self.stats_path = stats_path
        self.lock = threading.Lock()
        self.routes_stats = {}
        self.unknown = 0
        self._stop = threading.Event()

    @classmethod
    def bind(cls, addr, routes, stats_path=None):
        host, port = split(addr)
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind((host, port))
        sock.listen(64)
        return cls(sock, routes, stats_path)

    def address(self):
        host, port = self.listener.getsockname()[:2]
        return "%s:%d" % (host, port)

    def serve_forever(self):
        while not self._stop.is_set():
            try:
                sock, _ = self.listener.accept()
            except OSError:
                break
            threading.Thread(target=self._handle, args=(sock,), daemon=True).start()

    def stop(self):
        self._stop.set()
        try:
            self.listener.close()
        except OSError:
            pass
        self.write_stats()

    def _route_bucket(self, name):
        with self.lock:
            if name not in self.routes_stats:
                self.routes_stats[name] = {"connections": 0, "streams": 0, "plain": 0, "wire": 0}
            return self.routes_stats[name]

    def snapshot(self):
        with self.lock:
            routes = {name: dict(value) for name, value in self.routes_stats.items()}
            unknown = self.unknown
        return {"routes": routes, "unknown_requests": unknown}

    def write_stats(self):
        if not self.stats_path:
            return
        import json
        import os
        tmp = self.stats_path + ".tmp"
        with open(tmp, "w") as handle:
            json.dump(self.snapshot(), handle, sort_keys=True)
        os.replace(tmp, self.stats_path)

    def _handle(self, sock):
        try:
            magic = recv_exact(sock, len(MUX_MAGIC))
            if magic != MUX_MAGIC:
                sock.close()
                return
            header = recv_exact(sock, 1)
            if header is None:
                sock.close()
                return
            length = header[0]
            route = ""
            if length:
                raw = recv_exact(sock, length)
                if raw is None:
                    sock.close()
                    return
                route = raw.decode("utf-8", "replace")
            sock.sendall(MUX_MAGIC)
            target = self.routes.get(route) if route else None
            bucket = self._route_bucket(route)
            with self.lock:
                bucket["connections"] += 1
            if route and target is None:
                with self.lock:
                    self.unknown += 1
                sock.close()
                self.write_stats()
                return
            connection = _ServerConnection(sock, target, self.routes, bucket, self)
            connection.run()
        except OSError:
            try:
                sock.close()
            except OSError:
                pass

    def _stream_finished(self):
        self.write_stats()


class _ServerConnection:
    def __init__(self, sock, default_target, routes, bucket, gateway):
        self.sock = sock
        self.writer = _Writer(sock)
        self.default_target = default_target
        self.routes = routes
        self.bucket = bucket
        self.gateway = gateway
        self.streams = {}
        self.lock = threading.Lock()

    def finish_stream(self, stream_id):
        with self.lock:
            self.streams.pop(stream_id, None)

    def finish_stream_if_done(self, stream):
        with self.lock:
            if stream.input_done and stream.output_done:
                self.streams.pop(stream.stream_id, None)

    def run(self):
        try:
            while True:
                header = recv_exact(self.sock, HEADER.size)
                if header is None:
                    break
                stream_id, kind, size = HEADER.unpack(header)
                payload = b""
                if size:
                    payload = recv_exact(self.sock, size)
                    if payload is None:
                        break
                if kind == OPEN:
                    with self.lock:
                        self.bucket["streams"] += 1
                    requested = payload.decode("utf-8", "replace") if payload else ""
                    if requested:
                        target_addr = self.routes.get(requested)
                        if target_addr is None:
                            with self.lock:
                                self.gateway.unknown += 1
                            self.writer.frame(stream_id, RESET)
                            continue
                    else:
                        requested = ""
                        target_addr = self.default_target
                        if target_addr is None:
                            self.writer.frame(stream_id, RESET)
                            continue
                    try:
                        target = socket.create_connection(split(target_addr), timeout=30)
                        target.settimeout(None)
                    except OSError:
                        self.writer.frame(stream_id, RESET)
                        continue
                    stream = _ServerStream(self, stream_id, target)
                    with self.lock:
                        self.streams[stream_id] = stream
                    stream.start()
                else:
                    with self.lock:
                        stream = self.streams.get(stream_id)
                    if stream is not None:
                        stream.on_frame(kind, payload)
                        self.bucket["plain"] += len(payload)
                        self.bucket["wire"] += len(payload)
        except OSError:
            pass
        finally:
            with self.lock:
                streams = list(self.streams.values())
                self.streams = {}
            for stream in streams:
                stream.close()
            try:
                self.sock.close()
            except OSError:
                pass
            self.gateway._stream_finished()
