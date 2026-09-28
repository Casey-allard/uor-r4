import numpy as np, jax, jax.numpy as jnp, math
import model as M
from evaljax import load_dir
p = {k: jnp.asarray(v) for k, v in load_dir('stress_ckpt').items()}
tok = np.fromfile('../valid.u16', dtype='<u2').astype(np.int32)
starts = M.evenly_spaced(len(tok), 256, 4)
def run(label):
    f = jax.jit(lambda p, i, t: M.nll(p, i, t, False))
    v = np.mean([float(f(p, *map(jnp.asarray, M.windows(tok, [s], 256)))) for s in starts])
    print(f"{label:28s} {v:.6f}")
run("correct")
orig_rope = M.rope
def rope_interleaved(x, cos, sin):
    x1, x2 = x[..., 0::2], x[..., 1::2]
    return jnp.stack([x1 * cos - x2 * sin, x2 * cos + x1 * sin], axis=-1).reshape(x.shape)
M.rope = rope_interleaved; run("interleaved RoPE"); M.rope = orig_rope
M.CFG["rope_theta"] = 10000.0; run("rope_theta 1e4"); M.CFG["rope_theta"] = 100000.0
# GQA order h % KV: permute q heads so the reshape (KV, G) pairs head h with kv h % KV
orig_einsum = M.EXACT["qk"]; orig_pv = M.EXACT["pv"]
def perm(q):  # q: (B, KV, G, T, D) from heads [0..H) -> reorder so group k holds heads {k, k+KV}
    B, KV, G, T, D = q.shape
    return q.reshape(B, KV * G, T, D)[:, jnp.array([0, 2, 1, 3])].reshape(B, KV, G, T, D)
M.EXACT["qk"] = lambda q, k: orig_einsum(perm(q), k)
M.EXACT["pv"] = lambda pr, v: perm(orig_pv(pr, v))
run("GQA order h % kv_heads"); M.EXACT["qk"] = orig_einsum; M.EXACT["pv"] = orig_pv
run("correct (restored)")
