"""Train the small byte-level Llama on WikiText-2 raw bytes (token = byte + 3).

Resumable: RUN/ckpt.npz holds params, AdamW moments, step and meta; batches
are a pure function of (seed, step), so a resumed run replays the same data.
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
from model import CFG, init_params, nll, evenly_spaced, windows  # noqa: E402

DATA = "/home/user/uor-r4/research/ai-research/ai-router/router-research/data/lm_proxy/raw/wikitext2"


def log(run, row):
    row["wall"] = round(time.time(), 1)
    line = json.dumps(row)
    print(line, flush=True)
    with open(os.path.join(run, "log.jsonl"), "a") as f:
        f.write(line + "\n")


def save(path, params, m, v, step, meta):
    d = {}
    for k in params:
        d["p/" + k] = np.asarray(params[k])
        d["m/" + k] = np.asarray(m[k])
        d["v/" + k] = np.asarray(v[k])
    d["step"] = np.int64(step)
    d["meta"] = np.frombuffer(json.dumps(meta).encode(), dtype=np.uint8)
    tmp = path[:-4] + ".tmp.npz"
    np.savez(tmp, **d)
    os.replace(tmp, path)


def load(path):
    z = np.load(path)
    params = {k[2:]: z[k] for k in z.files if k.startswith("p/")}
    m = {k[2:]: z[k] for k in z.files if k.startswith("m/")}
    v = {k[2:]: z[k] for k in z.files if k.startswith("v/")}
    meta = json.loads(bytes(z["meta"]).decode())
    return params, m, v, int(z["step"]), meta


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", required=True)
    ap.add_argument("--steps", type=int, required=True)
    ap.add_argument("--batch", type=int, default=16)
    ap.add_argument("--lr", type=float, default=2e-3)
    ap.add_argument("--min_lr_frac", type=float, default=0.1)
    ap.add_argument("--warmup", type=int, default=100)
    ap.add_argument("--wd", type=float, default=0.1)
    ap.add_argument("--clip", type=float, default=1.0)
    ap.add_argument("--beta2", type=float, default=0.95)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--eval_every", type=int, default=200)
    ap.add_argument("--ckpt_every_s", type=float, default=150.0)
    ap.add_argument("--max_seconds", type=float, default=0.0, help="stop this invocation after this much training wall time")
    a = ap.parse_args()
    os.makedirs(a.run, exist_ok=True)
    T = CFG["time"]
    config = {k: getattr(a, k) for k in ("steps", "batch", "lr", "min_lr_frac", "warmup", "wd", "clip", "beta2", "seed")}

    train = np.fromfile(os.path.join(DATA, "train.txt"), dtype=np.uint8).astype(np.int32) + 3
    valid = np.fromfile(os.path.join(DATA, "valid.txt"), dtype=np.uint8).astype(np.int32) + 3
    mon_ids, mon_tgt = windows(valid, evenly_spaced(len(valid), T, 32), T)

    ckpt = os.path.join(a.run, "ckpt.npz")
    if os.path.exists(ckpt):
        params, m, v, step, meta = load(ckpt)
        if meta["config"] != config:
            raise SystemExit(f"config differs from checkpoint: {meta['config']} vs {config}")
        train_seconds = meta["train_seconds"]
        log(a.run, {"event": "resume", "step": step, "train_seconds": train_seconds})
    else:
        params = init_params(a.seed)
        m = {k: np.zeros_like(x) for k, x in params.items()}
        v = {k: np.zeros_like(x) for k, x in params.items()}
        step, train_seconds = 0, 0.0
        n = sum(x.size for x in params.values())
        log(a.run, {"event": "start", "params": int(n), "config": config, "cfg": CFG,
                    "jax": jax.__version__, "devices": str(jax.devices())})
    params = {k: jnp.asarray(x) for k, x in params.items()}
    m = {k: jnp.asarray(x) for k, x in m.items()}
    v = {k: jnp.asarray(x) for k, x in v.items()}
    decay = {k: (x.ndim == 2) for k, x in params.items()}
    b1, b2, eps = 0.9, a.beta2, 1e-8

    def step_fn(params, m, v, ids, tgt, t, lr):
        loss, grads = jax.value_and_grad(nll)(params, ids, tgt, True)
        gnorm = jnp.sqrt(sum(jnp.sum(g * g) for g in grads.values()))
        scale = jnp.minimum(1.0, a.clip / (gnorm + 1e-6))
        bc1 = 1.0 - b1 ** t
        bc2 = 1.0 - b2 ** t
        np_, nm, nv = {}, {}, {}
        for k in params:
            g = grads[k] * scale
            nm[k] = b1 * m[k] + (1.0 - b1) * g
            nv[k] = b2 * v[k] + (1.0 - b2) * g * g
            u = (nm[k] / bc1) / (jnp.sqrt(nv[k] / bc2) + eps)
            p = params[k]
            if decay[k]:
                p = p - lr * a.wd * p
            np_[k] = p - lr * u
        return np_, nm, nv, loss, gnorm

    step_jit = jax.jit(step_fn, donate_argnums=(0, 1, 2))
    eval_jit = jax.jit(lambda p, i, t: nll(p, i, t, False))

    def lr_at(s):
        if s < a.warmup:
            return a.lr * (s + 1) / a.warmup
        prog = min(1.0, (s - a.warmup) / max(1, a.steps - a.warmup))
        return a.lr * (a.min_lr_frac + (1 - a.min_lr_frac) * 0.5 * (1 + math.cos(math.pi * prog)))

    def batch(s):
        rng = np.random.default_rng([a.seed, s])
        starts = rng.integers(0, len(train) - T - 1, size=a.batch)
        return windows(train, starts, T)

    def monitor():
        vals = [float(eval_jit(params, jnp.asarray(mon_ids[i:i + 16]), jnp.asarray(mon_tgt[i:i + 16])))
                for i in range(0, len(mon_ids), 16)]
        return float(np.mean(vals))

    session_start = time.time()
    last_ckpt = time.time()
    seg_start, seg_steps, loss_acc = time.time(), 0, []
    while step < a.steps:
        ids, tgt = batch(step)
        lr = lr_at(step)
        params, m, v, loss, gnorm = step_jit(params, m, v, jnp.asarray(ids), jnp.asarray(tgt),
                                             jnp.float32(step + 1), jnp.float32(lr))
        loss = float(loss)
        if not math.isfinite(loss):
            raise SystemExit(f"non-finite loss at step {step}")
        step += 1
        seg_steps += 1
        loss_acc.append(loss)
        now = time.time()
        if step % 20 == 0 or step == a.steps:
            dt = now - seg_start
            train_seconds += dt
            log(a.run, {"step": step, "loss": round(float(np.mean(loss_acc)), 4), "gnorm": round(float(gnorm), 3),
                        "lr": round(lr, 7), "tok_per_s": round(seg_steps * a.batch * T / dt, 1),
                        "train_seconds": round(train_seconds, 1)})
            seg_start, seg_steps, loss_acc = time.time(), 0, []
        if step % a.eval_every == 0 or step == a.steps:
            t0 = time.time()
            val = monitor()
            log(a.run, {"step": step, "val_nll_32win_f32": round(val, 5), "val_bpb_32win": round(val / math.log(2), 4),
                        "eval_seconds": round(time.time() - t0, 1)})
            seg_start = time.time()
        stop = a.max_seconds and (time.time() - session_start) > a.max_seconds
        if time.time() - last_ckpt > a.ckpt_every_s or step == a.steps or stop:
            t0 = time.time()
            save(ckpt, params, m, v, step, {"config": config, "train_seconds": train_seconds, "cfg": CFG})
            last_ckpt = time.time()
            log(a.run, {"event": "checkpoint", "step": step, "save_seconds": round(last_ckpt - t0, 1)})
            seg_start = time.time()
        if stop:
            log(a.run, {"event": "time_cap", "step": step})
            break
    if step >= a.steps:
        final = {k: np.asarray(x) for k, x in params.items()}
        np.savez(os.path.join(a.run, "final_params_f32.npz"), **final)
        log(a.run, {"event": "done", "step": step, "train_seconds": round(train_seconds, 1)})


if __name__ == "__main__":
    main()
