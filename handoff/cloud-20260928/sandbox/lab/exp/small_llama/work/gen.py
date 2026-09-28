"""Plain-prompt continuation (no chat template) from an exported dir, f32.

usage: gen.py MODEL_DIR PROMPT [N]
One compiled shape: the context (last <= 256 tokens) is right-padded to 256;
the causal mask makes the padding invisible to the read position.
"""
import os
import sys

import numpy as np
import jax
import jax.numpy as jnp

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model import CFG, forward  # noqa: E402
from evaljax import load_dir  # noqa: E402

T = CFG["time"]
p = {k: jnp.asarray(v) for k, v in load_dir(sys.argv[1]).items()}
prompt = sys.argv[2].encode("utf-8")
n = int(sys.argv[3]) if len(sys.argv) > 3 else 200
f = jax.jit(lambda p, ids, pos: forward(p, ids, False)[0, pos])
for mode in ("greedy", "sampled T=0.8 seed 0"):
    rng = np.random.default_rng(0)
    ids = [b + 3 for b in prompt]
    for _ in range(n):
        ctx = ids[-T:]
        arr = jnp.asarray([ctx + [3] * (T - len(ctx))], jnp.int32)
        logits = np.asarray(f(p, arr, len(ctx) - 1), dtype=np.float64)
        if mode == "greedy":
            nxt = int(np.argmax(logits))
        else:
            z = np.exp((logits - logits.max()) / 0.8)
            nxt = int(rng.choice(len(z), p=z / z.sum()))
        ids.append(nxt)
    out = bytes(i - 3 for i in ids if 3 <= i < 259)
    print(f"--- {mode}:\n{out.decode('utf-8', 'replace')}\n", flush=True)
