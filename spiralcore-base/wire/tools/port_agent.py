#!/usr/bin/env python3
"""Foreign-runtime UOROBJ01 agent: pull, verify and fleet audit without Rust.

Implements the peer client flow from PROTOCOL.md using hashlib/socket/struct only, and keeps a local
record index (JSON: id hex -> record hex) so warm-store closure checks and pipelined fleet audits work
exactly like the reference client.
"""
import hashlib
import json
import pathlib
import socket
import struct
import sys

def node_id(scale, payload):
    return hashlib.blake2b(bytes([scale]) + payload, digest_size=12).digest()

def decode_record(rec):
    scale = rec[0]
    payload = rec[13:]
    if len(payload) != (32 if scale == 0 else 36):
        raise AssertionError("record length")
    if node_id(scale, payload) != rec[1:13]:
        raise AssertionError("record id")
    return rec[1:13], scale, payload

def recv_exact(sock, size):
    data = b""
    while len(data) < size:
        chunk = sock.recv(size - len(data))
        if not chunk:
            raise AssertionError("eof")
        data += chunk
    return data


def is_ipv4(text):
    parts = text.split(".")
    if len(parts) != 4:
        return False
    for part in parts:
        if not part.isdigit() or not 0 <= int(part) <= 255:
            return False
    return True


def socks5_connect(proxy, target):
    proxy_host, proxy_port = proxy.rsplit(":", 1)
    sock = socket.create_connection((proxy_host, int(proxy_port)), timeout=30)
    sock.sendall(b"\x05\x01\x00")
    if recv_exact(sock, 2) != b"\x05\x00":
        raise AssertionError("socks5 greeting")
    host, port = target.rsplit(":", 1)
    # socks5h semantics: hand the name to the proxy (an IP literal is sent as its textual name too,
    # because the egress policy matches on the allow-list entry, not on a resolved socket address).
    encoded = host.encode()
    address = b"\x03" + bytes([len(encoded)]) + encoded
    sock.sendall(b"\x05\x01\x00" + address + struct.pack(">H", int(port)))
    reply = recv_exact(sock, 4)
    if reply[0] != 5 or reply[1] != 0:
        raise AssertionError("socks5 connect refused code=%d" % reply[1])
    atyp = reply[3]
    if atyp == 1:
        recv_exact(sock, 4)
    elif atyp == 3:
        length = recv_exact(sock, 1)[0]
        recv_exact(sock, length)
    elif atyp == 4:
        recv_exact(sock, 16)
    recv_exact(sock, 2)
    return sock


class Session:
    def __init__(self, addr, proxy=None):
        if proxy:
            self.sock = socks5_connect(proxy, addr)
        else:
            host, port = addr.rsplit(":", 1)
            self.sock = socket.create_connection((host, int(port)))
        self.wire = 0
    def send(self, kind, body=b""):
        frame = bytes([kind]) + struct.pack(">I", len(body)) + body
        self.sock.sendall(frame)
        self.wire += len(frame)
    def recv(self):
        head = b""
        while len(head) < 5:
            chunk = self.sock.recv(5 - len(head))
            if not chunk:
                raise AssertionError("eof")
            head += chunk
        size = struct.unpack(">I", head[1:5])[0]
        body = b""
        while len(body) < size:
            chunk = self.sock.recv(size - len(body))
            if not chunk:
                raise AssertionError("eof body")
            body += chunk
        self.wire += 5 + len(body)
        return head[0], body
    def close(self):
        self.sock.close()

def parse_manifest(body):
    if body[:8] != b"UOROBJ01":
        raise AssertionError("manifest magic")
    length = struct.unpack("<Q", body[8:16])[0]
    sha = body[16:48]
    count = body[48]
    roots = []
    for index in range(count):
        base = 49 + 13 * index
        roots.append((body[base:base + 12], body[base + 12]))
    return length, sha, roots

def request_manifest(session, selector):
    session.send(20, selector)
    kind, body = session.recv()
    if kind == 25:
        return None, None
    if kind != 21:
        raise AssertionError("expected manifest")
    return parse_manifest(body), body

def load_store(path):
    path = pathlib.Path(path)
    if not path.exists():
        return {}
    raw = json.loads(path.read_text())
    return {bytes.fromhex(key): bytes.fromhex(value) for key, value in raw["records"].items()}

def save_store(path, records):
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {"records": {key.hex(): value.hex() for key, value in records.items()}}
    path.write_text(json.dumps(payload))

def frontier_missing(records, roots):
    missing = []
    def walk(nid):
        rec = records.get(nid)
        if rec is None:
            missing.append(nid)
            return
        _, scale, payload = decode_record(rec)
        if scale > 0:
            for offset in range(0, 36, 12):
                walk(payload[offset:offset + 12])
    for root in roots:
        walk(root[0])
    return missing

def assemble(records, roots):
    data = bytearray()
    def emit(nid):
        _, scale, payload = decode_record(records[nid])
        if scale == 0:
            data.extend(payload[:32])
        else:
            for offset in range(0, 36, 12):
                emit(payload[offset:offset + 12])
    for root in roots:
        emit(root[0])
    return bytes(data)

def pull(addr, selector, store_path, out_path, expect_sha=None, expect_bytes=None, proxy=None):
    records = load_store(store_path)
    session = Session(addr, proxy)
    manifest, _ = request_manifest(session, selector)
    if manifest is None:
        raise SystemExit("unknown selector")
    length, sha, roots = manifest
    if expect_bytes is not None and length != expect_bytes:
        print("pin mismatch: length %d != %d" % (length, expect_bytes), file=sys.stderr, flush=True)
        raise SystemExit(2)
    if expect_sha is not None and sha != expect_sha:
        print("pin mismatch: sha256 mismatch", file=sys.stderr, flush=True)
        raise SystemExit(2)
    while True:
        missing = frontier_missing(records, roots)
        if not missing:
            break
        for start in range(0, len(missing), 4096):
            batch = missing[start:start + 4096]
            session.send(22, struct.pack(">H", len(batch)) + b"".join(batch))
            kind, response = session.recv()
            if kind != 23:
                raise AssertionError("expected nodes")
            count = struct.unpack(">H", response[:2])[0]
            if count != len(batch):
                raise AssertionError("correlation")
            offset = 2
            for nid in batch:
                size = 45 if response[offset] == 0 else 49
                record = response[offset:offset + size]
                offset += size
                rid, _, _ = decode_record(record)
                if rid != nid:
                    raise AssertionError("unexpected record")
                records[rid] = record
            if offset != len(response):
                raise AssertionError("trailing bytes")
    data = assemble(records, roots)[:length]
    if hashlib.sha256(data).digest() != sha:
        raise AssertionError("digest mismatch")
    save_store(store_path, records)
    pathlib.Path(out_path).write_bytes(data)
    session.send(24)
    session.close()
    print("PORT_PULL_OK records=%d bytes=%d wire=%d sha=%s" % (len(records), len(data), session.wire, hashlib.sha256(data).hexdigest()[:16]))

def audit(addr, store_path, targets_path, as_json, proxy=None):
    selectors = []
    for number, line in enumerate(pathlib.Path(targets_path).read_text().splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if len(line) != 64:
            raise SystemExit("targets line %d is not 64 hex characters" % number)
        selectors.append(bytes.fromhex(line))
    records = load_store(store_path)
    session = Session(addr, proxy)
    outcomes = []
    for start in range(0, len(selectors), 64):
        window = selectors[start:start + 64]
        for selector in window:
            session.send(20, selector)
        for selector in window:
            kind, body = session.recv()
            if kind == 25:
                outcomes.append({"status": "unknown", "missing": None, "selector": selector.hex()})
                continue
            if kind != 21:
                raise AssertionError("expected manifest")
            length, sha, roots = parse_manifest(body)
            missing = len(frontier_missing(records, roots))
            outcomes.append({"status": "verified" if missing == 0 else "stale", "missing": missing, "selector": selector.hex()})
    session.send(24)
    session.close()
    verified = sum(1 for item in outcomes if item["status"] == "verified")
    stale = sum(1 for item in outcomes if item["status"] == "stale")
    unknown = sum(1 for item in outcomes if item["status"] == "unknown")
    if as_json:
        print(json.dumps({"checks": len(outcomes), "verified": verified, "stale": stale, "unknown": unknown, "wire_bytes": session.wire, "results": outcomes}))
    else:
        print("PORT_AUDIT checks=%d verified=%d stale=%d unknown=%d wire=%d" % (len(outcomes), verified, stale, unknown, session.wire))
    raise SystemExit(2 if unknown else (3 if stale else 0))

def usage():
    raise SystemExit("usage: port_agent.py [--proxy HOST:PORT] pull ADDR SELECTOR_HEX STORE OUT [--expect-sha256 HEX] [--expect-bytes N] | audit ADDR STORE TARGETS [--json]")

def main(argv):
    argv = list(argv)
    proxy = None
    if "--proxy" in argv:
        position = argv.index("--proxy")
        if position + 1 >= len(argv):
            usage()
        proxy = argv[position + 1]
        del argv[position:position + 2]
    if len(argv) < 2:
        usage()
    if argv[1] == "pull" and len(argv) >= 6:
        expect_sha = None
        expect_bytes = None
        rest = argv[6:]
        index = 0
        while index < len(rest):
            if rest[index] == "--expect-sha256" and index + 1 < len(rest):
                expect_sha = bytes.fromhex(rest[index + 1])
                index += 2
            elif rest[index] == "--expect-bytes" and index + 1 < len(rest):
                expect_bytes = int(rest[index + 1])
                index += 2
            else:
                usage()
        pull(argv[2], bytes.fromhex(argv[3]), argv[4], argv[5], expect_sha, expect_bytes, proxy)
    elif argv[1] == "audit" and len(argv) == 5:
        audit(argv[2], argv[3], argv[4], False, proxy)
    elif argv[1] == "audit" and len(argv) == 6 and argv[5] == "--json":
        audit(argv[2], argv[3], argv[4], True, proxy)
    else:
        usage()

if __name__ == "__main__":
    main(sys.argv)
