#!/usr/bin/env python3
"""#607: independent numpy reference over the REAL openai-community/gpt2
124M weights, for the presence-gated executor canary.

Reads the pinned snapshot at .uor-models/sources/gpt2-124m (never downloads),
runs a second GPT-2 implementation for a few fixed prompts, and writes
crates/uor-r4-model-source/tests/fixtures/gpt2-real/golden.json with the
final hidden state, top-10, and argmax per prompt. The Rust oracle must
reproduce these when the snapshot is present.
"""
import json
import os
import struct

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "..", ".uor-models", "sources", "gpt2-124m")
OUT = os.path.join(HERE, "..", "crates", "uor-r4-model-source", "tests", "fixtures", "gpt2-real")

if not os.path.exists(os.path.join(SRC, "model.safetensors")):
    raise SystemExit(f"real gpt2 snapshot absent at {SRC}; nothing to do")
os.makedirs(OUT, exist_ok=True)

cfg = json.load(open(os.path.join(SRC, "config.json")))
D, NH, NL = cfg["n_embd"], cfg["n_head"], cfg["n_layer"]
NPOS, V, INNER = cfg["n_positions"], cfg["vocab_size"], 4 * cfg["n_embd"]
HS = D // NH
EPS = np.float32(cfg["layer_norm_epsilon"])

# ---- load safetensors (F32) as a name -> ndarray map ----
path = os.path.join(SRC, "model.safetensors")
with open(path, "rb") as f:
    n = struct.unpack("<Q", f.read(8))[0]
    header = json.loads(f.read(n))
    blob = f.read()
T = {}
for name, meta in header.items():
    if name == "__metadata__":
        continue
    assert meta["dtype"] == "F32", (name, meta["dtype"])
    a, b = meta["data_offsets"]
    T[name] = np.frombuffer(blob[a:b], dtype="<f4").reshape(meta["shape"]).astype(np.float32)


def layernorm(x, w, b):
    x = x.astype(np.float32)
    mean = np.float32(x.mean())
    var = np.float32(((x - mean) ** 2).mean())
    return ((x - mean) / np.sqrt(var + EPS) * w + b).astype(np.float32)


def gelu_new(x):
    x = x.astype(np.float32)
    c = np.float32(np.sqrt(2.0 / np.pi))
    return (np.float32(0.5) * x * (np.float32(1.0) + np.tanh(c * (x + np.float32(0.044715) * x ** 3)))).astype(np.float32)


def forward(tokens):
    kc = [np.zeros((len(tokens), D), np.float32) for _ in range(NL)]
    vc = [np.zeros((len(tokens), D), np.float32) for _ in range(NL)]
    scale = np.float32(1.0 / np.sqrt(HS))
    hidden = logits = None
    for pos, tok in enumerate(tokens):
        x = (T["wte.weight"][tok] + T["wpe.weight"][pos]).astype(np.float32)
        for l in range(NL):
            n1 = layernorm(x, T[f"h.{l}.ln_1.weight"], T[f"h.{l}.ln_1.bias"])
            qkv = (n1 @ T[f"h.{l}.attn.c_attn.weight"] + T[f"h.{l}.attn.c_attn.bias"]).astype(np.float32)
            q, k, v = qkv[:D], qkv[D:2 * D], qkv[2 * D:]
            kc[l][pos], vc[l][pos] = k, v
            ao = np.zeros(D, np.float32)
            for h in range(NH):
                qh = q[h * HS:(h + 1) * HS]
                sc = np.array([np.float32(np.dot(qh, kc[l][t][h * HS:(h + 1) * HS])) * scale for t in range(pos + 1)], np.float32)
                e = np.exp(sc - sc.max()).astype(np.float32)
                wgt = (e / e.sum()).astype(np.float32)
                acc = np.zeros(HS, np.float32)
                for t in range(pos + 1):
                    acc += wgt[t] * vc[l][t][h * HS:(h + 1) * HS]
                ao[h * HS:(h + 1) * HS] = acc
            x = (x + (ao @ T[f"h.{l}.attn.c_proj.weight"] + T[f"h.{l}.attn.c_proj.bias"]).astype(np.float32)).astype(np.float32)
            n2 = layernorm(x, T[f"h.{l}.ln_2.weight"], T[f"h.{l}.ln_2.bias"])
            hmid = gelu_new((n2 @ T[f"h.{l}.mlp.c_fc.weight"] + T[f"h.{l}.mlp.c_fc.bias"]).astype(np.float32))
            x = (x + (hmid @ T[f"h.{l}.mlp.c_proj.weight"] + T[f"h.{l}.mlp.c_proj.bias"]).astype(np.float32)).astype(np.float32)
        hf = layernorm(x, T["ln_f.weight"], T["ln_f.bias"])
        hidden = hf
        logits = (hf @ T["wte.weight"].T).astype(np.float32)
    return hidden, logits


def top_k(logits, k):
    m = logits.max()
    p = np.exp((logits - m).astype(np.float32))
    p = (p / p.sum()).astype(np.float32)
    order = sorted(range(len(p)), key=lambda i: (-float(p[i]), i))
    return [[int(i), float(p[i])] for i in order[:k]]


prompts = [[464, 3290, 373, 257], [15496, 995], [50256, 464, 968, 1971, 318]]
cases = []
for toks in prompts:
    hidden, logits = forward(toks)
    cases.append({
        "tokens": toks,
        "hidden": [float(v) for v in hidden],
        "top_k": top_k(logits, 10),
        "argmax": int(np.argmax(logits)),
    })

json.dump({"cases": cases}, open(os.path.join(OUT, "golden.json"), "w"))
print("wrote", os.path.normpath(os.path.join(OUT, "golden.json")))
for c in cases:
    print("tokens", c["tokens"], "argmax", c["argmax"], "top1", c["top_k"][0])
