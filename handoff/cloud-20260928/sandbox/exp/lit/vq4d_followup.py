"""Bounded follow-up to vq4d.py (the full run exceeded the 10-minute budget and was stopped):
(1) exact-fp16-radius + polytope direction cost; (2) 4-D k-means at 3 bits/dim with a reduced
budget (150k train, 10 Lloyd iterations) -> an UPPER bound on the best 4-D VQ MSE at that rate."""
import time
import numpy as np
import vq4d as V

rng = np.random.default_rng(777)
te4 = rng.standard_normal((200_000, 4))
for name, cb in (("24-cell", V.cell24()), ("600-cell", V.cell600())):
    u = te4 / np.linalg.norm(te4, axis=1, keepdims=True)
    cosmax = V.max_dot(u, cb)
    mse = np.mean(np.sum(te4 ** 2, axis=1) * 2 * (1 - cosmax)) / 4
    print(f"exact fp16 radius + {name} direction: MSE/dim {mse:.5f} at {(np.log2(len(cb)) + 16) / 4:.3f} bits/dim "
          f"(direction-only {np.log2(len(cb)) / 4:.3f} bits/dim)", flush=True)

t0 = time.time()
tr = rng.standard_normal((150_000, 4))
te = rng.standard_normal((150_000, 4))
c = V.kmeans(tr, 4096, iters=10, seed=3)
rec = c[V.assign(te, c)]
print(f"4-D k-means 4096 codewords (150k train, 10 iters; upper bound): test MSE/dim "
      f"{np.mean(np.sum((te - rec) ** 2, axis=1)) / 4:.5f}  [{time.time() - t0:.0f}s]", flush=True)
