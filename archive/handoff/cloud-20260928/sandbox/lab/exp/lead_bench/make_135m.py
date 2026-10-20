# Scratch: random checkpoint with SmolLM2-135M's exact shape (throughput only; the weights mean nothing).
import json, struct, numpy as np
V, D, L, H, KV, F = 49152, 576, 30, 9, 3, 1536
hd = D // H
rng = np.random.default_rng(0)
cfg = {"architectures": ["LlamaForCausalLM"], "hidden_act": "silu", "hidden_size": D, "intermediate_size": F,
       "num_attention_heads": H, "num_hidden_layers": L, "num_key_value_heads": KV, "rms_norm_eps": 1e-5,
       "rope_theta": 100000, "tie_word_embeddings": True, "vocab_size": V, "attention_bias": False,
       "mlp_bias": False, "rope_interleaved": False, "rope_scaling": None}
shapes = {"model.embed_tokens.weight": (V, D), "model.norm.weight": (D,)}
for l in range(L):
    p = f"model.layers.{l}"
    shapes.update({f"{p}.input_layernorm.weight": (D,), f"{p}.post_attention_layernorm.weight": (D,),
                   f"{p}.self_attn.q_proj.weight": (H * hd, D), f"{p}.self_attn.k_proj.weight": (KV * hd, D),
                   f"{p}.self_attn.v_proj.weight": (KV * hd, D), f"{p}.self_attn.o_proj.weight": (D, H * hd),
                   f"{p}.mlp.gate_proj.weight": (F, D), f"{p}.mlp.up_proj.weight": (F, D), f"{p}.mlp.down_proj.weight": (D, F)})
header, off = {}, 0
names = sorted(shapes)
for n in names:
    size = int(np.prod(shapes[n])) * 2
    header[n] = {"dtype": "BF16", "shape": list(shapes[n]), "data_offsets": [off, off + size]}
    off += size
h = json.dumps(header).encode(); h += b" " * ((8 - len(h) % 8) % 8)
with open("ckpt135/model.safetensors", "wb") as f:
    f.write(struct.pack("<Q", len(h)) + h)
    for n in names:
        if n.endswith("norm.weight"):
            a = np.ones(shapes[n], dtype=np.float32)
        else:
            a = rng.standard_normal(shapes[n], dtype=np.float32) * 0.04
        f.write((a.view(np.uint32) >> 16).astype(np.uint16).tobytes())
json.dump(cfg, open("ckpt135/config.json", "w"))
print("bytes", off)
