#!/usr/bin/env python3
"""canonical-onnx.py — the executable form of the ONNX realization's
canonical-form specification.

Reads an ONNX `ModelProto`, applies the canonicalization rules
implemented by `crates/uor-addr/src/onnx/value.rs`, and emits the
κ-label (SHA-256 of the canonical flat skeleton). Byte-identical to
`uor_addr::onnx::address`. Stdlib-only (a minimal protobuf reader is
inlined) so it runs without the `onnx` Python package.

ADR-060: the canonical form is the full flat skeleton emitted inline
(ir_version, opset imports, the graph laid out node-by-node in
Kahn-topological order with subgraphs recursed inline, then model
metadata), with variable-length leaves replaced by their streamed
SHA-256 digest. There is no two-level commitment and no count / width
ceiling — only a subgraph-nesting stack-safety bound.

Usage:
    python3 canonical-onnx.py MODEL.onnx
"""
import hashlib
import struct
import sys

IR_VERSION_MAX = 13  # onnx.proto Version::IR_VERSION (latest); accept 1..=MAX
OPSET_VERSION_MIN = 1
SUBGRAPH_DEPTH_MAX = 64


def sha256(b):
    return hashlib.sha256(b).digest()


class WireError(ValueError):
    pass


def read_varint(buf, pos):
    # Mirror the Rust MessageReader: cap at 10 bytes, error on truncation.
    result = shift = 0
    start = pos
    while True:
        if pos >= len(buf):
            raise WireError("truncated varint")
        if pos - start >= 10:
            raise WireError("varint overflow")
        b = buf[pos]
        result |= (b & 0x7F) << shift
        pos += 1
        if not (b & 0x80):
            return result, pos
        shift += 7


def fields(body):
    """Yield (number, wire_type, value) for each field, with the same
    strict acceptance criteria as the Rust `MessageReader` (zero field
    number, unknown wire type, truncation, and out-of-range lengths all
    raise) so the recurse-or-fallback decision in `canonical_proto_digest`
    matches the Rust realization exactly. value is int for varint, raw
    little-endian bytes for fixed32/64, bytes for length-delimited."""
    pos = 0
    while pos < len(body):
        tag, pos = read_varint(body, pos)
        num, wt = tag >> 3, tag & 7
        if num == 0:
            raise WireError("zero field number")
        if wt == 0:
            v, pos = read_varint(body, pos)
            yield num, wt, v
        elif wt == 1:
            if pos + 8 > len(body):
                raise WireError("truncated fixed64")
            yield num, wt, body[pos:pos + 8]
            pos += 8
        elif wt == 2:
            ln, pos = read_varint(body, pos)
            if pos + ln > len(body):
                raise WireError("length out of range")
            yield num, wt, body[pos:pos + ln]
            pos += ln
        elif wt == 5:
            if pos + 4 > len(body):
                raise WireError("truncated fixed32")
            yield num, wt, body[pos:pos + 4]
            pos += 4
        else:
            raise WireError(f"bad wire type {wt}")


def canonical_proto_digest(body, depth=0):
    """Field-order-canonical digest of an opaque protobuf message —
    mirrors `canonical_proto_digest` in crates/uor-addr/src/onnx/value.rs.
    Fields are folded in ascending field-number order (stable), recursing
    into length-delimited fields with a raw-bytes fallback on parse
    failure."""
    fs = list(fields(body))            # raises on malformed (caller may fall back)
    fs.sort(key=lambda f: f[0])        # Python sort is stable
    h = hashlib.sha256()
    for num, wt, val in fs:
        h.update(struct.pack("<Q", num))
        h.update(bytes([wt]))
        if wt == 0:
            h.update(struct.pack("<Q", val))
        elif wt in (1, 5):
            h.update(val)              # raw LE bytes == u64/u32 to_le_bytes
        else:
            if depth < 32 and len(val) > 0:
                try:
                    sub = canonical_proto_digest(val, depth + 1)
                except WireError:
                    sub = sha256(val)
            else:
                sub = sha256(val)
            h.update(sub)
    return h.digest()


def first(body, n):
    for num, _, v in fields(body):
        if num == n:
            return v
    return None


def first_bytes(body, n):
    v = first(body, n)
    return v if isinstance(v, (bytes, bytearray)) else b""


def first_varint(body, n, default=0):
    v = first(body, n)
    return v if isinstance(v, int) else default


def each(body, n):
    for num, _, v in fields(body):
        if num == n:
            yield v


# ── tensor data digest ──

# typed-field (#5/#7/#11) varint widths per data_type id.
INT32_WIDTH = {6: 4, 5: 2, 4: 2, 10: 2, 16: 2, 3: 1, 2: 1, 9: 1,
               17: 1, 18: 1, 19: 1, 20: 1, 22: 1, 21: 1, 23: 1}


def fold_packed_varints_i64(h, body, n):
    for v in each(body, n):
        if isinstance(v, (bytes, bytearray)):
            pos = 0
            while pos < len(v):
                val, pos = read_varint(v, pos)
                h.update(struct.pack("<q", val if val < (1 << 63) else val - (1 << 64)))
        elif isinstance(v, int):
            h.update(struct.pack("<q", v if v < (1 << 63) else v - (1 << 64)))


def fold_typed_varints(h, body, n, width):
    for v in each(body, n):
        if isinstance(v, (bytes, bytearray)):
            pos = 0
            while pos < len(v):
                val, pos = read_varint(v, pos)
                h.update(struct.pack("<Q", val)[:width])
        elif isinstance(v, int):
            h.update(struct.pack("<Q", v)[:width])


def fold_fixed_payload(h, body, n):
    for v in each(body, n):
        if isinstance(v, (bytes, bytearray)):
            h.update(v)


def tensor_data_digest(t, dtype):
    # External data (data_location #14 == EXTERNAL): bind the external
    # reference (#13, sorted by key) under a domain tag — mirrors the Rust
    # no_std core, which cannot dereference sibling files.
    if first_varint(t, 14) == 1:
        h = hashlib.sha256()
        h.update(b"onnx:external-data:v1")
        sub = bytearray()
        emit_string_string(sub, t, 13)   # u32(count) || sorted key/value digests
        h.update(bytes(sub))
        return h.digest()
    raw = first(t, 9)
    if isinstance(raw, (bytes, bytearray)) and len(raw) > 0:
        return sha256(raw)
    h = hashlib.sha256()
    if dtype == 1:           # FLOAT
        fold_fixed_payload(h, t, 4)
    elif dtype in (11, 15):  # DOUBLE, COMPLEX128
        fold_fixed_payload(h, t, 10)
    elif dtype == 14:        # COMPLEX64
        fold_fixed_payload(h, t, 4)
    elif dtype == 7:         # INT64
        fold_typed_varints(h, t, 7, 8)
    elif dtype == 13:        # UINT64
        fold_typed_varints(h, t, 11, 8)
    elif dtype == 12:        # UINT32
        fold_typed_varints(h, t, 11, 4)
    elif dtype == 8:         # STRING
        for s in each(t, 6):
            h.update(sha256(s))
    else:                    # int32_data-backed
        fold_typed_varints(h, t, 5, INT32_WIDTH.get(dtype, 4))
    return h.digest()


def count_dims(t):
    n = 0
    for v in each(t, 1):
        if isinstance(v, (bytes, bytearray)):
            pos = 0
            while pos < len(v):
                _, pos = read_varint(v, pos)
                n += 1
        else:
            n += 1
    return n


def emit_packed_varints_i64(out, body, n):
    """Append each varint of a packed/unpacked repeated field as 8-byte LE
    (the canonical dims / INTS layout — `(val as i64).to_le_bytes()`)."""
    for v in each(body, n):
        if isinstance(v, (bytes, bytearray)):
            pos = 0
            while pos < len(v):
                val, pos = read_varint(v, pos)
                out += struct.pack("<Q", val & 0xFFFFFFFFFFFFFFFF)
        elif isinstance(v, int):
            out += struct.pack("<Q", v & 0xFFFFFFFFFFFFFFFF)


def emit_tensor(out, t):
    dtype = first_varint(t, 2)
    if not (1 <= dtype <= 23):
        raise ValueError(f"unknown dtype {dtype}")
    out += sha256(first_bytes(t, 8))         # name
    out += struct.pack("<i", dtype)
    out += struct.pack("<I", count_dims(t))  # rank
    emit_packed_varints_i64(out, t, 1)       # dims
    out += tensor_data_digest(t, dtype)      # 32-byte leaf digest


def emit_attribute_value(out, a, atype, depth):
    if atype == 1:           # FLOAT (fixed32)
        v = first(a, 2)
        if isinstance(v, (bytes, bytearray)):
            out += v
    elif atype == 2:         # INT
        out += struct.pack("<q", first_varint(a, 3))
    elif atype == 3:         # STRING
        out += sha256(first_bytes(a, 4))
    elif atype == 4:         # TENSOR (inline)
        emit_tensor(out, first_bytes(a, 5))
    elif atype == 5:         # GRAPH (recurse inline)
        emit_canonical_graph(out, first_bytes(a, 6), depth + 1)
    elif atype == 6:         # FLOATS
        for v in each(a, 7):
            out += sha256(v) if isinstance(v, (bytes, bytearray)) else v
    elif atype == 7:         # INTS
        emit_packed_varints_i64(out, a, 8)
    elif atype == 8:         # STRINGS
        for s in each(a, 9):
            out += sha256(s)
    elif atype == 9:         # TENSORS (inline)
        for tb in each(a, 10):
            emit_tensor(out, tb)
    elif atype == 10:        # GRAPHS (recurse inline)
        for g in each(a, 11):
            emit_canonical_graph(out, g, depth + 1)
    elif atype == 11:        # SPARSE_TENSOR
        out += canonical_proto_digest(first_bytes(a, 22))
    elif atype == 12:        # SPARSE_TENSORS
        for s in each(a, 23):
            out += canonical_proto_digest(s)
    elif atype == 13:        # TYPE_PROTO
        out += canonical_proto_digest(first_bytes(a, 14))
    elif atype == 14:        # TYPE_PROTOS
        for s in each(a, 15):
            out += canonical_proto_digest(s)


def emit_attributes(out, node, depth):
    attrs = list(each(node, 5))
    attrs.sort(key=lambda a: first_bytes(a, 1))
    out += struct.pack("<I", len(attrs))
    for a in attrs:
        out += sha256(first_bytes(a, 1))
        atype = first_varint(a, 20)
        out += struct.pack("<i", atype)
        emit_attribute_value(out, a, atype, depth)


def emit_node(out, node, depth):
    out += sha256(first_bytes(node, 3))   # name
    out += sha256(first_bytes(node, 4))   # op_type
    out += sha256(first_bytes(node, 7))   # domain
    out += sha256(first_bytes(node, 8))   # overload
    ins = list(each(node, 1))
    out += struct.pack("<I", len(ins))
    for i in ins:
        out += sha256(i)
    outs = list(each(node, 2))
    out += struct.pack("<I", len(outs))
    for o in outs:
        out += sha256(o)
    emit_attributes(out, node, depth)


def topo_order(nodes):
    producers = {}
    for idx, n in enumerate(nodes):
        for o in each(n, 2):
            producers.setdefault(bytes(o), idx)
    emitted = [False] * len(nodes)
    order = []
    for _ in range(len(nodes)):
        best = None
        for cand, n in enumerate(nodes):
            if emitted[cand]:
                continue
            ready = True
            for i in each(n, 1):
                p = producers.get(bytes(i))
                if p is not None and not emitted[p]:
                    ready = False
                    break
            if not ready:
                continue
            key = (first_bytes(n, 3), first_bytes(n, 4), first_bytes(n, 7))
            if best is None or key < best[1]:
                best = (cand, key)
        if best is None:
            raise ValueError("graph cycle")
        emitted[best[0]] = True
        order.append(best[0])
    return [nodes[i] for i in order]


def emit_string_string(out, body, n):
    # u32(count) || for each (sorted by key): sha256(key) || sha256(value)
    entries = list(each(body, n))
    entries.sort(key=lambda e: first_bytes(e, 1))
    out += struct.pack("<I", len(entries))
    for e in entries:
        out += sha256(first_bytes(e, 1))
        out += sha256(first_bytes(e, 2))


def emit_value_info(out, graph, n):
    # u32(count) || for each (sorted by name): sha256(name) || proto-digest(TypeProto)
    vis = list(each(graph, n))
    vis.sort(key=lambda v: first_bytes(v, 1))
    out += struct.pack("<I", len(vis))
    for v in vis:
        out += sha256(first_bytes(v, 1))
        out += canonical_proto_digest(first_bytes(v, 2))


def emit_canonical_graph(out, graph, depth):
    if depth > SUBGRAPH_DEPTH_MAX:
        raise ValueError("subgraph nesting too deep")
    out += sha256(first_bytes(graph, 2))              # name
    nodes = list(each(graph, 1))
    out += struct.pack("<I", len(nodes))              # node count
    for n in topo_order(nodes):                       # nodes (topo)
        emit_node(out, n, depth)
    # initializers (#5), sorted by name, inline.
    inits = list(each(graph, 5))
    inits.sort(key=lambda t: first_bytes(t, 8))
    out += struct.pack("<I", len(inits))
    for t in inits:
        emit_tensor(out, t)
    # graph IO: inputs (#11), outputs (#12), value_info (#13).
    emit_value_info(out, graph, 11)
    emit_value_info(out, graph, 12)
    emit_value_info(out, graph, 13)


def emit_opsets(out, model):
    # opset imports sorted by (domain, version); no count prefix.
    entries = list(each(model, 8))
    ok_min = any(first_bytes(e, 1) == b"" and first_varint(e, 2) >= OPSET_VERSION_MIN
                 for e in entries)
    if entries and not ok_min:
        raise ValueError("opset below minimum version")
    entries.sort(key=lambda e: (first_bytes(e, 1), first_varint(e, 2)))
    for e in entries:
        out += sha256(first_bytes(e, 1))
        out += struct.pack("<q", first_varint(e, 2))


def emit_model_meta(out, model):
    out += sha256(first_bytes(model, 2))              # producer_name
    out += sha256(first_bytes(model, 3))              # producer_version
    out += sha256(first_bytes(model, 4))              # domain
    out += struct.pack("<q", first_varint(model, 5))  # model_version
    emit_string_string(out, model, 14)                # metadata_props


def commitment(model):
    # ADR-060: the full flat skeleton (no two-level commitment).
    ir = first_varint(model, 1)
    if ir < 1 or ir > IR_VERSION_MAX:
        raise ValueError(f"unsupported IR version {ir}")
    graph = first_bytes(model, 7)
    if not graph:
        raise ValueError("missing graph")
    out = bytearray()
    out += struct.pack("<q", ir)
    emit_opsets(out, model)
    emit_canonical_graph(out, graph, 0)
    emit_model_meta(out, model)
    return bytes(out)


def kappa_label(model):
    return "sha256:" + sha256(commitment(model)).hex()


def main(argv):
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    print(kappa_label(open(argv[1], "rb").read()))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
