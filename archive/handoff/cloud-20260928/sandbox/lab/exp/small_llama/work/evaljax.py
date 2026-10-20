"""f32 JAX evaluation of an exported checkpoint directory (reads the BF16
model.safetensors back, so it evaluates exactly what the Rust tool loads).

usage: evaljax.py MODEL_DIR TOKENS.u16 [--full] [--windows 4] [--f32 PARAMS.npz] [--json OUT]
- probe windows: kappa-conversion's evenly_spaced(len, 256, windows), batch 1
  per window, mean over windows of the per-window mean NLL (nats/token).
- --full: all non-overlapping 256-token windows (starts 0, 256, ...), mean NLL
  over every predicted position, reported also as bits per byte.
"""
import argparse
import json
import math
import os
import sys
import time

import numpy as np
import jax
import jax.numpy as jnp

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model import CFG, nll, evenly_spaced, windows, bf16_to_f32, expected_shapes  # noqa: E402
from export import read_safetensors, load_params  # noqa: E402


def load_dir(model_dir):
    raw = read_safetensors(os.path.join(model_dir, "model.safetensors"))
    want = expected_shapes()
    assert set(raw) == set(want)
    out = {}
    for name, (dt, shape, data) in raw.items():
        assert dt == "BF16" and shape == want[name]
        out[name] = bf16_to_f32(np.frombuffer(data, dtype="<u2")).reshape(shape)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("model_dir")
    ap.add_argument("tokens")
    ap.add_argument("--windows", type=int, default=4)
    ap.add_argument("--full", action="store_true")
    ap.add_argument("--f32", default=None, help="also evaluate these unrounded f32 params")
    ap.add_argument("--json", default=None)
    a = ap.parse_args()
    T = CFG["time"]
    tokens = np.fromfile(a.tokens, dtype="<u2").astype(np.int32)
    ev = jax.jit(lambda p, i, t: nll(p, i, t, False))
    sets = {"bf16_export": load_dir(a.model_dir)}
    if a.f32:
        sets["f32_master"] = load_params(a.f32)
    report = {"tokens_file": a.tokens, "tokens": int(len(tokens)), "time": T}
    starts = evenly_spaced(len(tokens), T, a.windows)
    report["probe_window_starts"] = starts
    for label, params in sets.items():
        p = {k: jnp.asarray(v) for k, v in params.items()}
        per = []
        for s in starts:
            ids, tgt = windows(tokens, [s], T)
            per.append(float(ev(p, jnp.asarray(ids), jnp.asarray(tgt))))
        report[label] = {"probe_windows_nll": float(np.mean(per)), "per_window_nll": per}
        if a.full:
            t0 = time.time()
            n_win = (len(tokens) - 1) // T
            fs = np.arange(n_win) * T
            tot, cnt = 0.0, 0
            for i in range(0, n_win, 16):
                ids, tgt = windows(tokens, fs[i:i + 16], T)
                if len(ids) < 16:  # keep one compiled shape: pad, then drop the padded rows
                    k = len(ids)
                    for j in range(k):
                        m = float(ev(p, jnp.asarray(ids[j:j + 1]), jnp.asarray(tgt[j:j + 1])))
                        tot += m * T
                        cnt += T
                    continue
                m = float(ev(p, jnp.asarray(ids), jnp.asarray(tgt)))
                tot += m * ids.size
                cnt += ids.size
            full = tot / cnt
            report[label]["full"] = {"windows": int(n_win), "positions": int(cnt), "nll": full,
                                     "bits_per_byte": full / math.log(2), "seconds": round(time.time() - t0, 1)}
        print(label, json.dumps(report[label]), flush=True)
    if a.json:
        with open(a.json, "w") as f:
            json.dump(report, f, indent=2)


if __name__ == "__main__":
    main()
