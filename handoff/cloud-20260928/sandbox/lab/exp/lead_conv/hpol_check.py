# check: hpol scores approach the dot scores (row-centred) as eps -> 0, on the teacher's real q, k
import sys, math, pickle, numpy as np, jax, jax.numpy as jnp
sys.argv = ["x", "--var", "hpol", "--out", "/dev/null"]
import conv as C
teacher = jax.tree_util.tree_map(jnp.asarray, pickle.load(open(C.LEAD + "/full_attn_s0_params.pkl", "rb")))
train = np.frombuffer(open(C.WIKI + "train.txt", "rb").read(), dtype=np.uint8)
x = jnp.asarray(np.stack([train[i:i + 128] for i in (1000, 50000, 90000, 200000)]).astype(np.int32))
for li, (q, k) in enumerate(C.qk_rows(teacher, x)):
    d = q.shape[-1]
    s = np.asarray(q @ jnp.swapaxes(k, 1, 2)) / math.sqrt(d)
    qn = float(jnp.sqrt(jnp.mean(jnp.sum(q * q, -1)))); kn = float(jnp.sqrt(jnp.mean(jnp.sum(k * k, -1))))
    out = []
    for logs in (-6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 0.0):
        f = np.asarray(C.scores(q, k, "hpol", {"logb": jnp.array(0.0), "logs": jnp.array(logs)}, d))
        m = np.tril(np.ones((128, 128), bool))
        err = []
        for r in range(128):
            sc = s[:, r, m[r]] - s[:, r, m[r]].mean(-1, keepdims=True)
            fc = f[:, r, m[r]] - f[:, r, m[r]].mean(-1, keepdims=True)
            err.append(np.abs(sc - fc).max())
        out.append(f"logs={logs:+.0f}: max|dlogit|={max(err):.2e}")
    print(f"layer {li} |q|rms={qn:.1f} |k|rms={kn:.1f}", "; ".join(out))
