#!/usr/bin/env python3
"""synth-fixtures.py — generate the synthetic GGUF / ONNX conformance
fixtures (and their `.kappa-label` attestations) reproducibly from a
textual spec, so the binaries need not be committed by hand.

Writes into crates/uor-addr/tests/fixtures/{gguf,onnx}/ and computes each
`.kappa-label` via canonical-gguf.py / canonical-onnx.py (the spec
attestation tools), guaranteeing the committed labels match the
canonical-form specification.

Usage:
    python3 tools/synth-fixtures.py            # regenerate all fixtures
"""
import importlib.util
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
GGUF_DIR = os.path.join(ROOT, "crates/uor-addr/tests/fixtures/gguf")
ONNX_DIR = os.path.join(ROOT, "crates/uor-addr/tests/fixtures/onnx")


def _load(name, fname):
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, fname))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


cgguf = _load("canonical_gguf", "canonical-gguf.py")
connx = _load("canonical_onnx", "canonical-onnx.py")


# ── GGUF builder ──
def align_up(o, a):
    r = o % a
    return o if r == 0 else o + (a - r)


def gguf(kvs, tensors, alignment=32):
    """kvs: list of (key bytes, type_tag, value-payload bytes).
    tensors: list of (name bytes, dims list, ggml_type, data bytes)."""
    body = struct.pack("<IIQQ", 0x46554747, 3, len(tensors), len(kvs))
    for key, t, val in kvs:
        body += struct.pack("<Q", len(key)) + key + struct.pack("<I", t) + val
    offs, cur = [], 0
    for (_, _, _, data) in tensors:
        offs.append(cur)
        cur = align_up(cur + len(data), alignment)
    for (name, dims, gt, _), off in zip(tensors, offs):
        body += struct.pack("<Q", len(name)) + name + struct.pack("<I", len(dims))
        body += b"".join(struct.pack("<Q", d) for d in dims)
        body += struct.pack("<I", gt) + struct.pack("<Q", off)
    data_start = align_up(len(body), alignment)
    buf = bytearray(body) + b"\x00" * (data_start - len(body))
    for (_, _, _, data), off in zip(tensors, offs):
        at = data_start + off
        if len(buf) < at + len(data):
            buf += b"\x00" * (at + len(data) - len(buf))
        buf[at:at + len(data)] = data
    return bytes(buf)


def s_str(x):
    return struct.pack("<Q", len(x)) + x


def f32_data(vals):
    return struct.pack("<%df" % len(vals), *vals)


# ── ONNX builder ──
def varint(v):
    o = bytearray()
    while True:
        b = v & 0x7F
        v >>= 7
        o.append(b | 0x80 if v else b)
        if not v:
            break
    return bytes(o)


def lf(f, b):
    return varint((f << 3) | 2) + varint(len(b)) + b


def vf(f, v):
    return varint((f << 3) | 0) + varint(v)


def onnx_node(name, op, ins, outs):
    n = b"".join(lf(1, i.encode()) for i in ins)
    n += b"".join(lf(2, o.encode()) for o in outs)
    return n + lf(3, name.encode()) + lf(4, op.encode())


def onnx_init_raw(name, vals):
    return vf(2, 1) + vf(1, len(vals)) + lf(8, name.encode()) + lf(9, f32_data(vals))


def onnx_value_info_typed(name):
    """ValueInfoProto with a TypeProto tensor_type{FLOAT, shape{dim 2}}."""
    dim = vf(1, 2)                       # Dimension.dim_value
    shape = lf(1, dim)                   # TensorShapeProto.dim
    tensor = vf(1, 1) + lf(2, shape)     # Tensor.elem_type=FLOAT, shape
    typ = lf(1, tensor)                  # TypeProto.tensor_type
    return lf(1, name.encode()) + lf(2, typ)


def onnx_init_external(name):
    """A FLOAT initializer whose data lives in an external file
    (data_location=EXTERNAL); the κ-label binds the reference."""
    def sse(k, v):
        return lf(1, k.encode()) + lf(2, v.encode())
    ext = b"".join(lf(13, sse(k, v)) for k, v in
                   [("location", "weights.bin"), ("offset", "0"), ("length", "8")])
    return vf(2, 1) + vf(1, 2) + lf(8, name.encode()) + ext + vf(14, 1)


def write(path, data, label):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "wb").write(data)
    open(os.path.splitext(path)[0] + ".kappa-label", "w").write(label)
    print(f"wrote {os.path.relpath(path, ROOT)} ({len(data)} B) → {label}")


def main():
    # GGUF: small two-tensor model.
    g = gguf(
        kvs=[(b"general.architecture", 8, s_str(b"llama")),
             (b"general.name", 8, s_str(b"tiny")),
             (b"general.alignment", 4, struct.pack("<I", 32))],
        tensors=[(b"token_embd.weight", [4], 0, f32_data([1.0, 2.0, 3.0, 4.0])),
                 (b"output.weight", [2], 0, f32_data([5.0, 6.0]))],
    )
    write(os.path.join(GGUF_DIR, "synthetic-f32.gguf"), g, cgguf.kappa_label(g))

    # GGUF: empty-metadata boundary case (0 KVs, 0 tensors).
    g0 = gguf(kvs=[], tensors=[])
    write(os.path.join(GGUF_DIR, "empty-metadata.gguf"), g0, cgguf.kappa_label(g0))

    # GGUF: non-default alignment.
    g256 = gguf(
        kvs=[(b"general.alignment", 4, struct.pack("<I", 256))],
        tensors=[(b"w", [8], 0, f32_data([float(i) for i in range(8)]))],
        alignment=256,
    )
    write(os.path.join(GGUF_DIR, "aligned-256.gguf"), g256, cgguf.kappa_label(g256))

    # ONNX: x -Relu-> y ; (y,w) -Add-> z, nodes emitted out of topo order.
    a = onnx_node("a", "Relu", ["x"], ["y"])
    b = onnx_node("b", "Add", ["y", "w"], ["z"])
    graph = (lf(1, b) + lf(1, a) + lf(2, b"g") + lf(5, onnx_init_raw("w", [1.0, 2.0]))
             + lf(11, lf(1, b"x")) + lf(12, lf(1, b"z")))
    m = vf(1, 13) + lf(8, vf(2, 17)) + lf(7, graph)
    write(os.path.join(ONNX_DIR, "synthetic.onnx"), m, connx.kappa_label(m))

    # ONNX: a value_info carrying a TypeProto + an external-data
    # initializer (exercises canonical_proto_digest + external binding).
    graph_t = (lf(1, a) + lf(1, b) + lf(2, b"g") + lf(5, onnx_init_external("w"))
               + lf(11, onnx_value_info_typed("x")) + lf(12, lf(1, b"z")))
    mt = vf(1, 13) + lf(8, vf(2, 17)) + lf(7, graph_t)
    write(os.path.join(ONNX_DIR, "synthetic-typed.onnx"), mt, connx.kappa_label(mt))
    return 0


if __name__ == "__main__":
    sys.exit(main())
