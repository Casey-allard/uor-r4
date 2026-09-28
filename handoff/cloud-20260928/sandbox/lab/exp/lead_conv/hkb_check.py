import sys, math, pickle, numpy as np, jax, jax.numpy as jnp
import conv as C
teacher = jax.tree_util.tree_map(jnp.asarray, pickle.load(open(C.LEAD + "/full_attn_s0_params.pkl", "rb")))
train = np.frombuffer(open(C.WIKI + "train.txt", "rb").read(), dtype=np.uint8)
x = jnp.asarray(np.stack([train[i:i + 128] for i in (1000, 50000)]).astype(np.int32))
for li, (q, k) in enumerate(C.qk_rows(teacher, x)):
    s = np.asarray(q @ jnp.swapaxes(k, 1, 2)) / math.sqrt(128)
    m = np.tril(np.ones((128, 128), bool)); out = []
    for var in ("hkb", "hpol"):
        for logs in (-6.0, -4.0, -3.0, -2.0):
            f = np.asarray(C.scores(q, k, var, {"logb": jnp.array(0.0), "logs": jnp.array(logs)}, 128))
            kl = []
            for r in range(1, 128):
                ps = jax.nn.softmax(s[:, r, m[r]], -1); pf = jax.nn.log_softmax(f[:, r, m[r]], -1)
                kl.append(float(jnp.mean(jnp.sum(ps * (jnp.log(ps + 1e-30) - pf), -1))))
            out.append(f"{var}@{logs:+.0f}:{np.mean(kl):.1e}")
    print(f"layer {li} mean row KL(teacher||variant):", " ".join(out))
