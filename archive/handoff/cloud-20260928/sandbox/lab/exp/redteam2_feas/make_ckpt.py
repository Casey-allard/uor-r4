# Synthetic SmolLM2-shaped Llama checkpoints (bf16 safetensors), reduced depth, for memory measurement only.
import json, struct, sys, os, numpy as np
def make(out, L, V=49152, D=576, H=9, KV=3, F=1536, qk_scale=1.5, seed=0):
    hd = D // H
    rng = np.random.default_rng(seed)
    os.makedirs(out, exist_ok=True)
    cfg = {"architectures": ["LlamaForCausalLM"], "hidden_act": "silu", "hidden_size": D, "intermediate_size": F,
           "num_attention_heads": H, "num_hidden_layers": L, "num_key_value_heads": KV, "rms_norm_eps": 1e-5,
           "rope_theta": 100000, "tie_word_embeddings": True, "vocab_size": V, "attention_bias": False,
           "mlp_bias": False, "rope_interleaved": False, "rope_scaling": None, "model_type": "llama"}
    t = {"model.embed_tokens.weight": (V, D, 0.05), "model.norm.weight": None}
    for l in range(L):
        p = f"model.layers.{l}"
        t[f"{p}.input_layernorm.weight"] = None
        t[f"{p}.post_attention_layernorm.weight"] = None
        t[f"{p}.self_attn.q_proj.weight"] = (H * hd, D, qk_scale / np.sqrt(D))
        t[f"{p}.self_attn.k_proj.weight"] = (KV * hd, D, qk_scale / np.sqrt(D))
        t[f"{p}.self_attn.v_proj.weight"] = (KV * hd, D, 1 / np.sqrt(D))
        t[f"{p}.self_attn.o_proj.weight"] = (D, H * hd, 0.5 / np.sqrt(D))
        t[f"{p}.mlp.gate_proj.weight"] = (F, D, 1 / np.sqrt(D))
        t[f"{p}.mlp.up_proj.weight"] = (F, D, 1 / np.sqrt(D))
        t[f"{p}.mlp.down_proj.weight"] = (D, F, 0.5 / np.sqrt(F))
    header, blobs, off = {}, [], 0
    for name in sorted(t):
        spec = t[name]
        a = (np.ones(D) if spec is None else rng.normal(0, spec[2], spec[:2])).astype(np.float32)
        bf = (a.view(np.uint32) >> 16).astype(np.uint16).tobytes()
        header[name] = {"dtype": "BF16", "shape": list(a.shape), "data_offsets": [off, off + len(bf)]}
        blobs.append(bf); off += len(bf)
    h = json.dumps(header).encode(); h += b" " * ((8 - len(h) % 8) % 8)
    with open(f"{out}/model.safetensors", "wb") as f:
        f.write(struct.pack("<Q", len(h)) + h)
        for b in blobs: f.write(b)
    json.dump(cfg, open(f"{out}/config.json", "w"))
if __name__ == "__main__":
    make(sys.argv[1], int(sys.argv[2]))
    rng = np.random.default_rng(1)
    if not os.path.exists("tok_train.u16"):
        rng.integers(3, 49152, 40000).astype(np.uint16).tofile("tok_train.u16")
        rng.integers(3, 49152, 20000).astype(np.uint16).tofile("tok_valid.u16")
