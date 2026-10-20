# Scratch: a tiny SmolLM2-shaped Llama checkpoint (bf16 safetensors) and a structured u16 token stream.
import json, struct, numpy as np
V, D, L, H, KV, F = 300, 64, 2, 4, 2, 128
hd = D // H
rng = np.random.default_rng(0)
cfg = {"architectures": ["LlamaForCausalLM"], "hidden_act": "silu", "hidden_size": D, "intermediate_size": F,
       "num_attention_heads": H, "num_hidden_layers": L, "num_key_value_heads": KV, "rms_norm_eps": 1e-5,
       "rope_theta": 100000, "tie_word_embeddings": True, "vocab_size": V, "attention_bias": False,
       "mlp_bias": False, "rope_interleaved": False, "rope_scaling": None, "model_type": "llama"}
t = {"model.embed_tokens.weight": rng.normal(0, 0.5, (V, D)), "model.norm.weight": 1 + 0.1 * rng.normal(size=D)}
for l in range(L):
    p = f"model.layers.{l}"
    t[f"{p}.input_layernorm.weight"] = 1 + 0.1 * rng.normal(size=D)
    t[f"{p}.post_attention_layernorm.weight"] = 1 + 0.1 * rng.normal(size=D)
    t[f"{p}.self_attn.q_proj.weight"] = rng.normal(0, 1.5 / np.sqrt(D), (H * hd, D))
    t[f"{p}.self_attn.k_proj.weight"] = rng.normal(0, 1.5 / np.sqrt(D), (KV * hd, D))
    t[f"{p}.self_attn.v_proj.weight"] = rng.normal(0, 1 / np.sqrt(D), (KV * hd, D))
    t[f"{p}.self_attn.o_proj.weight"] = rng.normal(0, 0.5 / np.sqrt(D), (D, H * hd))
    t[f"{p}.mlp.gate_proj.weight"] = rng.normal(0, 1 / np.sqrt(D), (F, D))
    t[f"{p}.mlp.up_proj.weight"] = rng.normal(0, 1 / np.sqrt(D), (F, D))
    t[f"{p}.mlp.down_proj.weight"] = rng.normal(0, 0.5 / np.sqrt(F), (D, F))
header, blobs, off = {}, [], 0
for name in sorted(t):
    a = np.ascontiguousarray(t[name], dtype=np.float32)
    bf = (a.view(np.uint32) >> 16).astype(np.uint16).tobytes()   # truncate to bfloat16
    header[name] = {"dtype": "BF16", "shape": list(a.shape), "data_offsets": [off, off + len(bf)]}
    blobs.append(bf); off += len(bf)
h = json.dumps(header).encode()
h += b" " * ((8 - len(h) % 8) % 8)
open("tiny_llama/model.safetensors", "wb").write(struct.pack("<Q", len(h)) + h + b"".join(blobs))
json.dump(cfg, open("tiny_llama/config.json", "w"))
# structured stream: a noisy copy task (tokens repeat with period 17) so a model can learn from context
n = 60000
base = rng.integers(3, V, 17)
seq = np.array([base[i % 17] if rng.random() > 0.2 else rng.integers(3, V) for i in range(n)], dtype=np.uint16)
seq[:50000].tofile("tiny_train.u16"); seq[50000:].tofile("tiny_valid.u16")
print("params", sum(v.size for v in t.values()))
