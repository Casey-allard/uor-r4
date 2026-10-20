import os, time
import numpy as np
import jax, jax.numpy as jnp
print(jax.__version__, jax.devices())
for dt in [jnp.float32, jnp.bfloat16]:
    a = jnp.asarray(np.random.randn(4096, 256), dtype=dt)
    b = jnp.asarray(np.random.randn(256, 640), dtype=dt)
    f = jax.jit(lambda a, b: (a @ b).astype(jnp.float32) if dt == jnp.bfloat16 else a @ b)
    f(a, b).block_until_ready()
    n = 20
    t = time.perf_counter()
    for _ in range(n):
        f(a, b).block_until_ready()
    el = (time.perf_counter() - t) / n
    fl = 2 * 4096 * 256 * 640
    print(dt.__name__ if hasattr(dt,'__name__') else dt, f"{el*1e3:.2f} ms  {fl/el/1e9:.1f} GFLOP/s")
    # also preferred_element_type f32 with bf16 inputs
    if dt == jnp.bfloat16:
        g = jax.jit(lambda a, b: jnp.dot(a, b, preferred_element_type=jnp.float32))
        g(a, b).block_until_ready()
        t = time.perf_counter()
        for _ in range(n):
            g(a, b).block_until_ready()
        el = (time.perf_counter() - t) / n
        print("bf16->f32 acc", f"{el*1e3:.2f} ms  {fl/el/1e9:.1f} GFLOP/s")
