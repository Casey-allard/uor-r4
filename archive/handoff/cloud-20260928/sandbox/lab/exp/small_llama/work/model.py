"""Scratch JAX Llama matching uor_r4_training::kappa_llama::KappaLlama::forward
with ScoreKind::Dot (log_beta = 0). Parameters use Hugging Face names and
[out, in] layouts, so export is a rename-free dump.

Two numerics:
- mixed=False: plain f32 everywhere (evaluation / agreement with the Rust probe).
- mixed=True: every matmul takes bf16 inputs and accumulates in f32 (AMX on
  this CPU), forward and backward; norms, RoPE, softmax, residuals and the
  loss stay f32; master weights and AdamW state are f32 (training only).
"""
import math

import numpy as np
import jax
import jax.numpy as jnp

F32, BF16 = jnp.float32, jnp.bfloat16

CFG = dict(vocab=264, width=256, layers=6, heads=4, kv_heads=2, head_dim=64,
           ffn=640, rope_theta=100000.0, rms_eps=1e-5, time=256)


def expected_shapes(c=CFG):
    """Mirror of LlamaShape::expected_tensors (tied embeddings)."""
    W, H, KV, D, F, V = c["width"], c["heads"], c["kv_heads"], c["head_dim"], c["ffn"], c["vocab"]
    s = {"model.embed_tokens.weight": (V, W), "model.norm.weight": (W,)}
    for l in range(c["layers"]):
        p = f"model.layers.{l}"
        s[f"{p}.input_layernorm.weight"] = (W,)
        s[f"{p}.post_attention_layernorm.weight"] = (W,)
        s[f"{p}.self_attn.q_proj.weight"] = (H * D, W)
        s[f"{p}.self_attn.k_proj.weight"] = (KV * D, W)
        s[f"{p}.self_attn.v_proj.weight"] = (KV * D, W)
        s[f"{p}.self_attn.o_proj.weight"] = (W, H * D)
        s[f"{p}.mlp.gate_proj.weight"] = (F, W)
        s[f"{p}.mlp.up_proj.weight"] = (F, W)
        s[f"{p}.mlp.down_proj.weight"] = (W, F)
    return s


def init_params(seed, c=CFG):
    rng = np.random.default_rng(seed)
    std = 0.02
    resid = std / math.sqrt(2 * c["layers"])
    out = {}
    for name, shape in sorted(expected_shapes(c).items()):
        if len(shape) == 1:
            out[name] = np.ones(shape, np.float32)
        else:
            s = resid if (name.endswith("o_proj.weight") or name.endswith("down_proj.weight")) else std
            out[name] = (rng.standard_normal(shape) * s).astype(np.float32)
    return out


def rope_tables(theta, head_dim, time):
    """Exactly kappa_llama::rope_tables: angles in f64, cos/sin cast to f32."""
    half = head_dim // 2
    pair = np.arange(half, dtype=np.float64)
    inverse = 1.0 / np.power(theta, (2.0 * pair) / head_dim)
    angle = np.arange(time, dtype=np.float64)[:, None] * inverse[None, :]
    return np.cos(angle).astype(np.float32), np.sin(angle).astype(np.float32)


def _mixed_einsum(spec):
    ins, out = spec.split("->")
    a_s, b_s = ins.split(",")
    da_spec, db_spec = f"{out},{b_s}->{a_s}", f"{out},{a_s}->{b_s}"

    @jax.custom_vjp
    def f(a, b):
        return jnp.einsum(spec, a.astype(BF16), b.astype(BF16), preferred_element_type=F32)

    def fwd(a, b):
        ab, bb = a.astype(BF16), b.astype(BF16)
        return jnp.einsum(spec, ab, bb, preferred_element_type=F32), (ab, bb)

    def bwd(res, g):
        ab, bb = res
        gb = g.astype(BF16)
        return (jnp.einsum(da_spec, gb, bb, preferred_element_type=F32),
                jnp.einsum(db_spec, gb, ab, preferred_element_type=F32))

    f.defvjp(fwd, bwd)
    return f


def _f32_einsum(spec):
    return lambda a, b: jnp.einsum(spec, a, b, precision=jax.lax.Precision.HIGHEST,
                                   preferred_element_type=F32)


SPECS = {"lin": "btw,ow->bto", "qk": "bkgtd,bksd->bkgts", "pv": "bkgts,bksd->bkgtd"}
MIXED = {k: _mixed_einsum(v) for k, v in SPECS.items()}
EXACT = {k: _f32_einsum(v) for k, v in SPECS.items()}


def rms_norm(x, w, eps):
    # kappa: input / sqrt(mean(x^2) + eps) * weight
    return x / jnp.sqrt(jnp.mean(x * x, axis=-1, keepdims=True) + eps) * w


def rope(x, cos, sin):
    """Half-split RoPE on (..., time, head_dim): HF rotate_half layout."""
    half = x.shape[-1] // 2
    x1, x2 = x[..., :half], x[..., half:]
    return jnp.concatenate([x1 * cos - x2 * sin, x2 * cos + x1 * sin], axis=-1)


def forward(params, ids, mixed, c=CFG):
    """Causal logits (B, T, V) for ids (B, T)."""
    e = MIXED if mixed else EXACT
    B, T = ids.shape
    H, KV, D, W = c["heads"], c["kv_heads"], c["head_dim"], c["width"]
    G = H // KV
    eps = c["rms_eps"]
    cos_np, sin_np = rope_tables(c["rope_theta"], D, T)
    cos, sin = jnp.asarray(cos_np), jnp.asarray(sin_np)          # (T, D/2)
    causal = jnp.asarray(np.tril(np.ones((T, T), dtype=bool)))  # keep column <= row
    scale = 1.0 / math.sqrt(D)
    emb = params["model.embed_tokens.weight"]
    x = jnp.take(emb, ids, axis=0)
    for l in range(c["layers"]):
        p = f"model.layers.{l}"
        h = rms_norm(x, params[f"{p}.input_layernorm.weight"], eps)
        q = e["lin"](h, params[f"{p}.self_attn.q_proj.weight"]).reshape(B, T, H, D).transpose(0, 2, 1, 3)
        k = e["lin"](h, params[f"{p}.self_attn.k_proj.weight"]).reshape(B, T, KV, D).transpose(0, 2, 1, 3)
        v = e["lin"](h, params[f"{p}.self_attn.v_proj.weight"]).reshape(B, T, KV, D).transpose(0, 2, 1, 3)
        q = rope(q, cos, sin)
        k = rope(k, cos, sin)
        # query head h = kv * G + g reads kv head h // G (kappa's repeat order).
        q = q.reshape(B, KV, G, T, D)
        s = e["qk"](q, k) * scale
        s = jnp.where(causal, s, -jnp.inf)
        prob = jax.nn.softmax(s, axis=-1)
        o = e["pv"](prob, v).reshape(B, H, T, D).transpose(0, 2, 1, 3).reshape(B, T, H * D)
        x = x + e["lin"](o, params[f"{p}.self_attn.o_proj.weight"])
        h = rms_norm(x, params[f"{p}.post_attention_layernorm.weight"], eps)
        gate = e["lin"](h, params[f"{p}.mlp.gate_proj.weight"])
        up = e["lin"](h, params[f"{p}.mlp.up_proj.weight"])
        x = x + e["lin"](jax.nn.silu(gate) * up, params[f"{p}.mlp.down_proj.weight"])
    h = rms_norm(x, params["model.norm.weight"], eps)
    return e["lin"](h, emb)


def nll(params, ids, targets, mixed):
    """Mean next-token NLL in nats over all (B, T) positions."""
    logits = forward(params, ids, mixed)
    logp = jax.nn.log_softmax(logits, axis=-1)
    return -jnp.mean(jnp.take_along_axis(logp, targets[..., None], axis=-1))


def bf16_round(a):
    """Round-to-nearest-even f32 -> bf16 bits (uint16); finite inputs only."""
    u = np.ascontiguousarray(a, dtype=np.float32).view(np.uint32).astype(np.uint64)
    u = u + 0x7FFF + ((u >> 16) & 1)
    return (u >> 16).astype(np.uint16)


def bf16_to_f32(bits):
    return (bits.astype(np.uint32) << 16).view(np.float32)


def evenly_spaced(length, time, count):
    """kappa-conversion's probe window starts."""
    span = length - time - 1
    return [i * span // count for i in range(count)]


def windows(tokens, starts, time):
    idx = np.asarray(starts)[:, None] + np.arange(time + 1)[None, :]
    w = tokens[idx]
    return w[:, :-1].astype(np.int32), w[:, 1:].astype(np.int32)
