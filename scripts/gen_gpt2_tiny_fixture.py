#!/usr/bin/env python3
"""#607: generate a tiny GPT-2 snapshot + an INDEPENDENT numpy-reference
golden file for the executor parity test.

Deterministic (seeded), tiny, and network-free. The numpy forward here is a
second implementation of GPT-2 semantics; the Rust `Gpt2` executor must
reproduce these goldens within tolerance, so parity is measured against a
different implementation rather than against itself.

Outputs into crates/uor-r4-model-source/tests/fixtures/gpt2-tiny/:
  config.json, model.safetensors, golden.json
"""
import json
import os
import struct

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(
    HERE, "..", "crates", "uor-r4-model-source", "tests", "fixtures", "gpt2-tiny"
)
os.makedirs(OUT, exist_ok=True)

rng = np.random.default_rng(607)
D, NH, NL, NPOS, V = 32, 4, 2, 16, 24
INNER = 4 * D
HS = D // NH
EPS = np.float32(1e-5)


def randn(*shape, s=0.02):
    return (rng.standard_normal(shape) * s).astype(np.float32)


def ln_params():
    return (np.float32(1.0) + randn(D, s=0.1)).astype(np.float32), randn(D, s=0.1)


tensors = {}
tensors["wte.weight"] = randn(V, D)
tensors["wpe.weight"] = randn(NPOS, D)
for l in range(NL):
    w, b = ln_params()
    tensors[f"h.{l}.ln_1.weight"], tensors[f"h.{l}.ln_1.bias"] = w, b
    tensors[f"h.{l}.attn.c_attn.weight"] = randn(D, 3 * D)
    tensors[f"h.{l}.attn.c_attn.bias"] = randn(3 * D, s=0.02)
    tensors[f"h.{l}.attn.c_proj.weight"] = randn(D, D)
    tensors[f"h.{l}.attn.c_proj.bias"] = randn(D, s=0.02)
    w, b = ln_params()
    tensors[f"h.{l}.ln_2.weight"], tensors[f"h.{l}.ln_2.bias"] = w, b
    tensors[f"h.{l}.mlp.c_fc.weight"] = randn(D, INNER)
    tensors[f"h.{l}.mlp.c_fc.bias"] = randn(INNER, s=0.02)
    tensors[f"h.{l}.mlp.c_proj.weight"] = randn(INNER, D)
    tensors[f"h.{l}.mlp.c_proj.bias"] = randn(D, s=0.02)
w, b = ln_params()
tensors["ln_f.weight"], tensors["ln_f.bias"] = w, b


def layernorm(x, w, b):
    x = x.astype(np.float32)
    mean = np.float32(x.mean())
    var = np.float32(((x - mean) ** 2).mean())
    return ((x - mean) / np.sqrt(var + EPS) * w + b).astype(np.float32)


def gelu_new(x):
    x = x.astype(np.float32)
    c = np.float32(np.sqrt(2.0 / np.pi))
    return (
        np.float32(0.5)
        * x
        * (np.float32(1.0) + np.tanh(c * (x + np.float32(0.044715) * x ** 3)))
    ).astype(np.float32)


def forward(tokens):
    T = len(tokens)
    kc = [np.zeros((T, D), np.float32) for _ in range(NL)]
    vc = [np.zeros((T, D), np.float32) for _ in range(NL)]
    scale = np.float32(1.0 / np.sqrt(HS))
    per_layer_last, hidden, logits = None, None, None
    for pos, tok in enumerate(tokens):
        x = (tensors["wte.weight"][tok] + tensors["wpe.weight"][pos]).astype(np.float32)
        caps = []
        for l in range(NL):
            n1 = layernorm(x, tensors[f"h.{l}.ln_1.weight"], tensors[f"h.{l}.ln_1.bias"])
            qkv = (n1 @ tensors[f"h.{l}.attn.c_attn.weight"] + tensors[f"h.{l}.attn.c_attn.bias"]).astype(np.float32)
            q, k, v = qkv[:D], qkv[D : 2 * D], qkv[2 * D :]
            kc[l][pos], vc[l][pos] = k, v
            ao = np.zeros(D, np.float32)
            for h in range(NH):
                qh = q[h * HS : (h + 1) * HS]
                scores = np.zeros(pos + 1, np.float32)
                for t in range(pos + 1):
                    kh = kc[l][t][h * HS : (h + 1) * HS]
                    scores[t] = np.float32(np.dot(qh, kh)) * scale
                e = np.exp(scores - scores.max()).astype(np.float32)
                wgt = (e / e.sum()).astype(np.float32)
                acc = np.zeros(HS, np.float32)
                for t in range(pos + 1):
                    acc += wgt[t] * vc[l][t][h * HS : (h + 1) * HS]
                ao[h * HS : (h + 1) * HS] = acc
            proj = (ao @ tensors[f"h.{l}.attn.c_proj.weight"] + tensors[f"h.{l}.attn.c_proj.bias"]).astype(np.float32)
            x = (x + proj).astype(np.float32)
            n2 = layernorm(x, tensors[f"h.{l}.ln_2.weight"], tensors[f"h.{l}.ln_2.bias"])
            hmid = gelu_new((n2 @ tensors[f"h.{l}.mlp.c_fc.weight"] + tensors[f"h.{l}.mlp.c_fc.bias"]).astype(np.float32))
            mout = (hmid @ tensors[f"h.{l}.mlp.c_proj.weight"] + tensors[f"h.{l}.mlp.c_proj.bias"]).astype(np.float32)
            x = (x + mout).astype(np.float32)
            caps.append(x.copy())
        hf = layernorm(x, tensors["ln_f.weight"], tensors["ln_f.bias"])
        lg = (hf @ tensors["wte.weight"].T).astype(np.float32)
        per_layer_last, hidden, logits = caps, hf, lg
    return per_layer_last, hidden, logits


def top_k(logits, k):
    m = logits.max()
    e = np.exp((logits - m).astype(np.float32))
    p = (e / e.sum()).astype(np.float32)
    order = sorted(range(len(p)), key=lambda i: (-float(p[i]), i))
    return [[int(i), float(p[i])] for i in order[:k]]


prompts = [[5, 12, 3, 8], [1], [7, 7, 2]]
cases = []
for toks in prompts:
    per_layer, hidden, logits = forward(toks)
    cases.append(
        {
            "tokens": toks,
            "per_layer": [[float(v) for v in layer] for layer in per_layer],
            "hidden": [float(v) for v in hidden],
            "logits": [float(v) for v in logits],
            "top_k": top_k(logits, 4),
        }
    )

# ---- write config.json ----
config = {
    "model_type": "gpt2",
    "architectures": ["GPT2LMHeadModel"],
    "activation_function": "gelu_new",
    "layer_norm_epsilon": 1e-05,
    "n_embd": D,
    "n_head": NH,
    "n_layer": NL,
    "n_positions": NPOS,
    "n_ctx": NPOS,
    "vocab_size": V,
    "bos_token_id": 0,
    "eos_token_id": 0,
}
with open(os.path.join(OUT, "config.json"), "w") as f:
    json.dump(config, f, indent=2)

# ---- write model.safetensors (F32, contiguous, row-major) ----
order = list(tensors.keys())
header, blob, offset = {}, bytearray(), 0
for name in order:
    arr = np.ascontiguousarray(tensors[name], dtype="<f4")
    data = arr.tobytes()
    header[name] = {
        "dtype": "F32",
        "shape": list(arr.shape),
        "data_offsets": [offset, offset + len(data)],
    }
    blob.extend(data)
    offset += len(data)
header_bytes = json.dumps(header, separators=(",", ":")).encode("utf-8")
with open(os.path.join(OUT, "model.safetensors"), "wb") as f:
    f.write(struct.pack("<Q", len(header_bytes)))
    f.write(header_bytes)
    f.write(bytes(blob))

with open(os.path.join(OUT, "golden.json"), "w") as f:
    json.dump({"config": config, "cases": cases}, f)

print("wrote fixture to", os.path.normpath(OUT))
print("tensors:", len(order), "cases:", len(cases), "vocab:", V, "dim:", D)
