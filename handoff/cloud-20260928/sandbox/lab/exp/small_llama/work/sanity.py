import numpy as np, jax, jax.numpy as jnp, ml_dtypes, time
from model import *
# 1. RNE bf16 vs ml_dtypes, including ties and tiny values
rng = np.random.default_rng(0)
x = np.concatenate([rng.standard_normal(200000).astype(np.float32) * 10.0 ** rng.integers(-6, 3, 200000),
                    np.array([1.0 + 2**-8, 1.0 + 3 * 2**-8, -1.0 - 2**-8, 0.0, -0.0, 1e-40], np.float32)])
ours = bf16_round(x)
ref = np.asarray(x, dtype=ml_dtypes.bfloat16).view(np.uint16)
print("bf16 RNE matches ml_dtypes:", bool(np.array_equal(ours, ref)))
# 2. mixed vs exact loss and grads on a small random model/batch
p = {k: jnp.asarray(v) for k, v in init_params(3).items()}
ids = jnp.asarray(rng.integers(3, 259, (2, 64)), jnp.int32); tgt = jnp.asarray(rng.integers(3, 259, (2, 64)), jnp.int32)
le, ge = jax.value_and_grad(nll)(p, ids, tgt, False)
lm, gm = jax.value_and_grad(nll)(p, ids, tgt, True)
print("loss exact %.6f mixed %.6f (ln 264 = %.6f)" % (le, lm, np.log(264)))
worst = 1.0
for k in ge:
    a, b = np.asarray(ge[k]).ravel(), np.asarray(gm[k]).ravel()
    cos = float(a @ b / (np.linalg.norm(a) * np.linalg.norm(b) + 1e-30))
    worst = min(worst, cos)
print("min cosine(grad exact, grad mixed) over tensors: %.5f" % worst)
# 3. causality: changing a future token must not change earlier logits
lg1 = forward(p, ids, False); ids2 = ids.at[:, 40].set(7); lg2 = forward(p, ids2, False)
print("causal max diff before pos 40:", float(jnp.max(jnp.abs(lg1[:, :40] - lg2[:, :40]))), " at/after:", float(jnp.max(jnp.abs(lg1[:, 40:] - lg2[:, 40:]))))
