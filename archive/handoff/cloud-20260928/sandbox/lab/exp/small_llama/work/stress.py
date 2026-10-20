import numpy as np
from model import expected_shapes
rng = np.random.default_rng(7)
p = {}
for name, shape in expected_shapes().items():
    if len(shape) == 1:
        p[name] = (1.0 + 0.2 * rng.standard_normal(shape)).astype(np.float32)
    else:
        s = 0.5 if 'embed' in name else 0.15 if ('q_proj' in name or 'k_proj' in name) else 0.08
        p[name] = (s * rng.standard_normal(shape)).astype(np.float32)
np.savez('stress_params.npz', **p)
