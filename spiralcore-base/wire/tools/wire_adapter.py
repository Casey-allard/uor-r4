#!/usr/bin/env python3
"""Paired wire adapter: compress arbitrary TCP traffic between two endpoints.

client mode: listen for plaintext locally, tunnel compressed frames to a gateway, optionally naming a
route. gateway mode: accept compressed tunnels and forward plaintext to a local target (a default target
and/or a named route table). hub is the gateway with --route NAME=ADDR entries.

Both sides stay plaintext to their local peers; only the tunnel is compressed. No TLS is added (run TLS
underneath if needed: the tunnel is just bytes on the wire)."""
import argparse
import json
import socket
import struct
import sys
import threading
import zlib

MAGIC = b'UORADP01'
FRAME_HEADER = struct.Struct('>I')


class Stats:
    def __init__(self):
        self.lock = threading.Lock()
        self.data = {'connections': 0, 'plain_in': 0, 'plain_out': 0, 'wire_in': 0, 'wire_out': 0}
        self.routes = {}

    def add(self, route=None, **fields):
        with self.lock:
            for key, value in fields.items():
                self.data[key] = self.data.get(key, 0) + value
            if route is not None:
                bucket = self.routes.setdefault(route, {'connections': 0, 'plain': 0, 'wire': 0})
                if 'connections' in fields:
                    bucket['connections'] += fields['connections']
                bucket['plain'] += fields.get('plain_in', 0) + fields.get('plain_out', 0)
                bucket['wire'] += fields.get('wire_in', 0) + fields.get('wire_out', 0)

    def snapshot(self, path, mode):
        with self.lock:
            data = dict(self.data)
            routes = {name: dict(value) for name, value in self.routes.items()}
        plain = data['plain_in'] + data['plain_out']
        wire = data['wire_in'] + data['wire_out']
        result = dict(data)
        result['mode'] = mode
        result['ratio'] = round(wire / plain, 4) if plain else 0.0
        for name, value in routes.items():
            value['ratio'] = round(value['wire'] / value['plain'], 4) if value['plain'] else 0.0
        result['routes'] = routes
        if path:
            tmp = path + '.tmp'
            with open(tmp, 'w') as handle:
                json.dump(result, handle)
            import os
            os.replace(tmp, path)
        return result


def recv_exact(sock, size):
    chunks = []
    remaining = size
    while remaining > 0:
        chunk = sock.recv(remaining)
        if not chunk:
            return None
        chunks.append(chunk)
        remaining -= len(chunk)
    return b''.join(chunks)


RAW_FLAG = 1 << 31


def send_stream(src, dst, level, stats, route):
    compressor = None
    raw_mode = False
    decided = False
    try:
        while True:
            data = src.recv(262144)
            if not data:
                break
            if not decided:
                # Probe the first chunk: incompressible streams go raw for the whole connection
                # (no expansion), compressible streams use a streaming compressor (best ratio).
                probe = zlib.compress(data, level)
                if len(data) >= 4096 and len(probe) >= len(data):
                    raw_mode = True
                else:
                    compressor = zlib.compressobj(level)
                decided = True
            if raw_mode:
                frame = data
                header = RAW_FLAG | len(data)
            else:
                payload = compressor.compress(data) + compressor.flush(zlib.Z_SYNC_FLUSH)
                frame = payload
                header = len(payload)
            dst.sendall(FRAME_HEADER.pack(header) + frame)
            stats.add(route=route, plain_in=len(data), wire_out=FRAME_HEADER.size + len(frame))
        dst.sendall(FRAME_HEADER.pack(0))
    except OSError:
        pass
    try:
        dst.shutdown(socket.SHUT_WR)
    except OSError:
        pass


def recv_stream(src, dst, stats, route):
    decompressor = None
    raw_mode = None
    try:
        while True:
            header = recv_exact(src, FRAME_HEADER.size)
            if header is None:
                break
            word = FRAME_HEADER.unpack(header)[0]
            raw = bool(word & RAW_FLAG)
            size = word & ~RAW_FLAG
            if size == 0:
                break
            blob = recv_exact(src, size)
            if blob is None:
                break
            if raw_mode is None:
                raw_mode = raw
            if raw_mode:
                data = blob
            else:
                if decompressor is None:
                    decompressor = zlib.decompressobj()
                data = decompressor.decompress(blob)
            if data:
                dst.sendall(data)
            stats.add(route=route, plain_out=len(data), wire_in=FRAME_HEADER.size + size)
    except (OSError, zlib.error):
        pass
    try:
        dst.shutdown(socket.SHUT_WR)
    except OSError:
        pass


def relay(plain, tunnel, level, stats, route):
    first = threading.Thread(target=send_stream, args=(plain, tunnel, level, stats, route), daemon=True)
    second = threading.Thread(target=recv_stream, args=(tunnel, plain, stats, route), daemon=True)
    first.start()
    second.start()
    first.join()
    second.join()
    for sock in (plain, tunnel):
        try:
            sock.close()
        except OSError:
            pass


def listen(addr):
    host, port = addr.rsplit(':', 1)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((host, int(port)))
    sock.listen(64)
    return sock


def split(addr):
    host, port = addr.rsplit(':', 1)
    return host, int(port)


def read_route(sock):
    header = recv_exact(sock, 1)
    if header is None:
        raise OSError('missing route header')
    size = header[0]
    if size == 0:
        return ''
    route = recv_exact(sock, size)
    if route is None:
        raise OSError('missing route name')
    return route.decode('utf-8', 'replace')


def handshake(sock):
    if recv_exact(sock, len(MAGIC)) != MAGIC:
        raise OSError('bad adapter magic')


def run_client(args, stats):
    listener = listen(args.listen)
    host, port = listener.getsockname()[:2]
    print('READY %s:%d' % (host, port), flush=True)
    remote = split(args.remote)
    route = (args.route or '').encode('utf-8')[:255]
    while True:
        plain, _ = listener.accept()
        try:
            tunnel = socket.create_connection(remote, timeout=args.connect_timeout)
            tunnel.settimeout(None)
            tunnel.sendall(MAGIC + bytes([len(route)]) + route)
            handshake(tunnel)
            if args.idle_timeout > 0:
                plain.settimeout(args.idle_timeout)
                tunnel.settimeout(args.idle_timeout)
        except OSError as error:
            print('client adapter: tunnel failed: %r' % (error,), file=sys.stderr, flush=True)
            try:
                plain.close()
            except OSError:
                pass
            continue
        stats.add(route=args.route or '', connections=1)
        threading.Thread(target=relay, args=(plain, tunnel, args.level, stats, args.route or ''),
                         daemon=True).start()


def run_gateway(args, stats):
    listener = listen(args.listen)
    host, port = listener.getsockname()[:2]
    print('READY %s:%d' % (host, port), flush=True)
    routes = {}
    for entry in args.route or []:
        if '=' not in entry:
            raise SystemExit('gateway --route must be NAME=ADDR, got ' + entry)
        name, target = entry.split('=', 1)
        routes[name] = target
    route_levels = {}
    for entry in args.route_level or []:
        if '=' not in entry:
            raise SystemExit('gateway --route-level must be NAME=N, got ' + entry)
        name, level = entry.split('=', 1)
        try:
            value = int(level)
        except ValueError:
            raise SystemExit('bad --route-level for ' + name)
        if value < 0 or value > 9:
            raise SystemExit('--route-level must be 0..9')
        route_levels[name] = value
    for name in route_levels:
        if name not in routes:
            raise SystemExit('--route-level names an unknown route: ' + name)
    default = args.target
    if default is None and not routes:
        raise SystemExit('gateway needs --target or at least one --route')
    while True:
        tunnel, _ = listener.accept()
        try:
            handshake(tunnel)
            requested = read_route(tunnel)
            target = routes.get(requested, default)
            if target is None:
                raise OSError('no route for ' + (requested or '<default>'))
            tunnel.sendall(MAGIC)
            plain = socket.create_connection(split(target))
            if args.idle_timeout > 0:
                plain.settimeout(args.idle_timeout)
                tunnel.settimeout(args.idle_timeout)
        except OSError as error:
            print('gateway: setup failed: %r' % (error,), file=sys.stderr, flush=True)
            try:
                tunnel.close()
            except OSError:
                pass
            continue
        label = requested if requested in routes else ''
        level = route_levels.get(label, args.level)
        stats.add(route=label, connections=1)
        threading.Thread(target=relay, args=(plain, tunnel, level, stats, label), daemon=True).start()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['client', 'gateway'])
    parser.add_argument('--listen', required=True)
    parser.add_argument('--remote')
    parser.add_argument('--target')
    parser.add_argument('--route', action='append')
    parser.add_argument('--route-level', action='append')
    parser.add_argument('--level', type=int, default=6)
    parser.add_argument('--connect-timeout', type=float, default=10.0)
    parser.add_argument('--idle-timeout', type=float, default=0.0)
    parser.add_argument('--stats-file')
    parser.add_argument('--stats-interval', type=float, default=1.0)
    args = parser.parse_args()
    if args.mode == 'client' and not args.remote:
        raise SystemExit('client mode needs --remote HOST:PORT')
    stats = Stats()
    import signal
    def finish(_signum, _frame):
        result = stats.snapshot(args.stats_file, args.mode)
        print('STATS ' + json.dumps(result, sort_keys=True), flush=True)
        raise SystemExit(0)
    signal.signal(signal.SIGTERM, finish)
    signal.signal(signal.SIGINT, finish)
    if args.stats_file and args.stats_interval > 0:
        import time

        def metrics_loop():
            while True:
                time.sleep(args.stats_interval)
                try:
                    stats.snapshot(args.stats_file, args.mode)
                except OSError:
                    pass

        threading.Thread(target=metrics_loop, daemon=True).start()
    if args.mode == 'client':
        if args.route:
            args.route = args.route[0]
        run_client(args, stats)
    else:
        run_gateway(args, stats)


if __name__ == '__main__':
    main()
