"""Export f32 params (npz with HF names, or a train ckpt.npz) to a Hugging Face
Llama directory: config.json + model.safetensors (BF16, round-to-nearest-even).
tokenizer.json is written separately by tok.py.

usage: export.py PARAMS.npz OUT_DIR
"""
import json
import os
import struct
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model import CFG, expected_shapes, bf16_round  # noqa: E402


def load_params(path):
    z = np.load(path)
    if any(k.startswith("p/") for k in z.files):
        return {k[2:]: z[k] for k in z.files if k.startswith("p/")}
    return {k: z[k] for k in z.files}


def hf_config(c=CFG):
    return {
        "architectures": ["LlamaForCausalLM"],
        "model_type": "llama",
        "attention_bias": False,
        "attention_dropout": 0.0,
        "bos_token_id": 0,
        "eos_token_id": 0,
        "head_dim": c["head_dim"],
        "hidden_act": "silu",
        "hidden_size": c["width"],
        "initializer_range": 0.02,
        "intermediate_size": c["ffn"],
        "max_position_embeddings": c["time"],
        "mlp_bias": False,
        "num_attention_heads": c["heads"],
        "num_hidden_layers": c["layers"],
        "num_key_value_heads": c["kv_heads"],
        "pretraining_tp": 1,
        "rms_norm_eps": c["rms_eps"],
        "rope_interleaved": False,
        "rope_scaling": None,
        "rope_theta": c["rope_theta"],
        "tie_word_embeddings": True,
        "torch_dtype": "bfloat16",
        "use_cache": True,
        "vocab_size": c["vocab"],
    }


def write_safetensors(path, arrays):
    """arrays: name -> (dtype string, shape, little-endian bytes). Names sorted,
    data contiguous in that order, header space-padded to 8 bytes."""
    names = sorted(arrays)
    header = {"__metadata__": {"format": "pt"}}
    offset = 0
    for n in names:
        dt, shape, data = arrays[n]
        header[n] = {"dtype": dt, "shape": list(shape), "data_offsets": [offset, offset + len(data)]}
        offset += len(data)
    hb = json.dumps(header, separators=(",", ":")).encode("utf-8")
    hb += b" " * ((8 - len(hb) % 8) % 8)
    tmp = path + ".tmp"
    with open(tmp, "wb") as f:
        f.write(struct.pack("<Q", len(hb)))
        f.write(hb)
        for n in names:
            f.write(arrays[n][2])
    os.replace(tmp, path)


def read_safetensors(path):
    """Independent reader: name -> (dtype, shape, raw bytes); checks coverage."""
    buf = open(path, "rb").read()
    (n,) = struct.unpack("<Q", buf[:8])
    header = json.loads(buf[8:8 + n].decode("utf-8"))
    header.pop("__metadata__", None)
    data = buf[8 + n:]
    spans = sorted((v["data_offsets"][0], v["data_offsets"][1]) for v in header.values())
    pos = 0
    for s, e in spans:
        assert s == pos and e >= s, "non-contiguous offsets"
        pos = e
    assert pos == len(data), "file not fully covered"
    return {k: (v["dtype"], tuple(v["shape"]), data[v["data_offsets"][0]:v["data_offsets"][1]])
            for k, v in header.items()}


def main():
    src, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    params = load_params(src)
    want = expected_shapes()
    assert set(params) == set(want), (set(params) ^ set(want))
    arrays = {}
    for name, shape in want.items():
        a = params[name]
        assert a.shape == shape and a.dtype == np.float32, (name, a.shape, a.dtype)
        assert np.all(np.isfinite(a)), name
        arrays[name] = ("BF16", shape, bf16_round(a).astype("<u2").tobytes())
    write_safetensors(os.path.join(out, "model.safetensors"), arrays)
    with open(os.path.join(out, "config.json"), "w") as f:
        json.dump(hf_config(), f, indent=2)
        f.write("\n")
    back = read_safetensors(os.path.join(out, "model.safetensors"))
    assert set(back) == set(want)
    for name in want:
        assert back[name][0] == "BF16" and back[name][1] == want[name] and back[name][2] == arrays[name][2]
    n = sum(int(np.prod(s)) for s in want.values())
    print(f"exported {len(want)} tensors, {n} parameters -> {out}")


if __name__ == "__main__":
    main()
