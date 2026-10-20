"""State-tracking experiment: geometric (quaternion) vs commutative recurrences on group word problems.

Usage: python3 st.py --task A5 --model quat --seed 0 --lr 1e-2 --steps 3000 --out runs/xxx
All models: token -> transition params (lookup), recurrence over a D=32 real state, then a shared
readout  logits = W2 gelu(W1 rms(h_t) + b1) + b2  at every position.
"""
import argparse, json, os, time
import numpy as np
import jax
import jax.numpy as jnp
from groups import get_task, sample_batch

D = 32          # real state size for every model
H = 64          # readout hidden width (same for every model)
ALPHA = 0.1     # repo QUATERNION_DELTA_SCALE


def hamilton(q, x):
    w, a, b, c = q[..., 0], q[..., 1], q[..., 2], q[..., 3]
    x0, x1, x2, x3 = x[..., 0], x[..., 1], x[..., 2], x[..., 3]
    return jnp.stack([w * x0 - a * x1 - b * x2 - c * x3,
                      w * x1 + a * x0 + b * x3 - c * x2,
                      w * x2 - a * x3 + b * x0 + c * x1,
                      w * x3 + a * x2 - b * x1 + c * x0], -1)


def unit(v, eps=1e-12):
    return v / jnp.sqrt(jnp.sum(v * v, -1, keepdims=True) + eps)


def householder(v, x):
    """(I - 2 v v^T) x for unit v, lane-wise."""
    return x - 2.0 * v * jnp.sum(v * x, -1, keepdims=True)


E0 = jnp.array([1.0, 0.0, 0.0, 0.0])


def init_params(model, V, C, key):
    ks = jax.random.split(key, 12)
    k4 = D // 4
    p = {}
    if model in ("quat", "quat_r", "hh", "hh_r"):
        p["raw"] = jax.random.normal(ks[0], (V, k4, 4))
        p["h0"] = jax.random.normal(ks[1], (k4, 4))
    elif model == "dprod":
        p["raw"] = jax.random.normal(ks[0], (V, k4, 4, 4))
        p["beta"] = jax.random.normal(ks[2], (V, k4, 4))
        p["h0"] = jax.random.normal(ks[1], (k4, 4))
    elif model in ("diag", "diagneg"):
        if model == "diag":
            p["u"] = jax.random.uniform(ks[0], (V, D), minval=0.0, maxval=4.6)   # a in (0.5,0.99)
        else:
            s = jnp.where(jax.random.bernoulli(ks[3], 0.5, (V, D)), 1.0, -1.0)
            p["u"] = s * jax.random.uniform(ks[0], (V, D), minval=0.55, maxval=2.6)  # |a| in (0.5,0.99)
        p["b"] = 0.5 * jax.random.normal(ks[1], (V, D))
        p["h0"] = jax.random.normal(ks[2], (D,))
    elif model in ("cplx_u", "lru"):
        m = D // 2
        p["theta"] = jax.random.uniform(ks[0], (V, m), minval=0.0, maxval=2 * np.pi)
        p["h0"] = jax.random.normal(ks[1], (2, m))
        if model == "lru":
            p["rho"] = jax.random.uniform(ks[2], (V, m), minval=2.0, maxval=5.0)  # |lambda| in (0.88,0.993)
            p["b"] = 0.5 * jax.random.normal(ks[3], (V, 2, m))
    elif model == "gru":
        s = 1.0 / np.sqrt(D)
        p["Wx"] = jax.random.uniform(ks[0], (V, 3 * D), minval=-s, maxval=s)
        p["U"] = jax.random.uniform(ks[1], (D, 3 * D), minval=-s, maxval=s)
        p["bh"] = jnp.zeros((3 * D,))
        p["h0"] = jnp.zeros((D,))
    else:
        raise ValueError(model)
    p["W1"] = jax.random.normal(ks[8], (D, H)) / np.sqrt(D)
    p["b1"] = jnp.zeros((H,))
    p["W2"] = jax.random.normal(ks[9], (H, C)) / np.sqrt(H)
    p["b2"] = jnp.zeros((C,))
    return p


def transition_fn(model, p):
    """Return (h_init[B,...] builder, step(h, x_t) -> h, flatten(h)->[B,D])."""
    k4 = D // 4
    if model in ("quat", "quat_r", "hh", "hh_r"):
        if model == "quat":
            qtab = unit(p["raw"])
        elif model == "quat_r":
            qtab = unit(E0 + ALPHA * p["raw"])
        elif model == "hh":
            qtab = unit(p["raw"])
        else:
            qtab = unit(E0 + (ALPHA / np.sqrt(2.0)) * p["raw"])

        if model.startswith("quat"):
            def step(h, x):
                return hamilton(qtab[x], h)
        else:
            def step(h, x):
                refl = h * jnp.array([-1.0, 1.0, 1.0, 1.0])     # H(e0) h
                return householder(qtab[x], refl)               # H(v) H(e0) h  (== v h v)

        def init(B):
            return jnp.broadcast_to(p["h0"], (B, k4, 4))

        return init, step, lambda h: h.reshape(h.shape[0], D)

    if model == "dprod":
        vt = unit(p["raw"])                 # [V,k4,4(reflector),4]
        bt = 2.0 * jax.nn.sigmoid(p["beta"])  # [V,k4,4]

        def step(h, x):
            v = vt[x]
            b = bt[x]
            for j in range(4):
                vj = v[:, :, j, :]
                h = h - b[:, :, j:j + 1] * vj * jnp.sum(vj * h, -1, keepdims=True)
            return h

        def init(B):
            return jnp.broadcast_to(p["h0"], (B, k4, 4))

        return init, step, lambda h: h.reshape(h.shape[0], D)

    if model in ("diag", "diagneg"):
        at = jax.nn.sigmoid(p["u"]) if model == "diag" else jnp.tanh(p["u"])

        def step(h, x):
            return at[x] * h + p["b"][x]

        def init(B):
            return jnp.broadcast_to(p["h0"], (B, D))

        return init, step, lambda h: h

    if model in ("cplx_u", "lru"):
        c, s = jnp.cos(p["theta"]), jnp.sin(p["theta"])
        if model == "lru":
            r = jax.nn.sigmoid(p["rho"])
            c, s = r * c, r * s

        def step(h, x):
            re, im = h[:, 0], h[:, 1]
            cx, sx = c[x], s[x]
            nre = cx * re - sx * im
            nim = sx * re + cx * im
            out = jnp.stack([nre, nim], 1)
            if model == "lru":
                out = out + p["b"][x]
            return out

        def init(B):
            return jnp.broadcast_to(p["h0"], (B, 2, D // 2))

        return init, step, lambda h: h.reshape(h.shape[0], D)

    if model == "gru":
        def step(h, x):
            gx = p["Wx"][x]
            gh = h @ p["U"] + p["bh"]
            z = jax.nn.sigmoid(gx[:, :D] + gh[:, :D])
            r = jax.nn.sigmoid(gx[:, D:2 * D] + gh[:, D:2 * D])
            n = jnp.tanh(gx[:, 2 * D:] + r * gh[:, 2 * D:])
            return (1 - z) * n + z * h

        def init(B):
            return jnp.broadcast_to(p["h0"], (B, D))

        return init, step, lambda h: h

    raise ValueError(model)


def readout(p, hflat):
    h = hflat / jnp.sqrt(jnp.mean(hflat * hflat, -1, keepdims=True) + 1e-6)
    z = jax.nn.gelu(h @ p["W1"] + p["b1"])
    return z @ p["W2"] + p["b2"]


def make_forward(model):
    def forward(p, xs):
        """xs [B,L] -> logits [L,B,C]"""
        init, step, flat = transition_fn(model, p)
        B = xs.shape[0]

        def body(h, x):
            h = step(h, x)
            return h, flat(h)

        _, hs = jax.lax.scan(body, init(B), xs.T)
        return readout(p, hs)
    return forward


def make_train(model, lr, steps, warm=100, clip=1.0):
    forward = make_forward(model)

    def loss_fn(p, xs, ys):
        logits = forward(p, xs)
        lp = jax.nn.log_softmax(logits, -1)
        nll = -jnp.take_along_axis(lp, ys.T[..., None], -1)[..., 0]
        return nll.mean()

    def sched(t):
        warmf = jnp.minimum(1.0, (t + 1.0) / warm)
        cos = 0.1 + 0.9 * 0.5 * (1 + jnp.cos(jnp.pi * jnp.minimum(t / steps, 1.0)))
        return lr * warmf * cos

    @jax.jit
    def train_step(p, m, v, t, xs, ys):
        loss, g = jax.value_and_grad(loss_fn)(p, xs, ys)
        gn = jnp.sqrt(sum(jnp.sum(x * x) for x in jax.tree_util.tree_leaves(g)))
        scale = jnp.minimum(1.0, clip / (gn + 1e-12))
        g = jax.tree_util.tree_map(lambda x: x * scale, g)
        m = jax.tree_util.tree_map(lambda a, b: 0.9 * a + 0.1 * b, m, g)
        v = jax.tree_util.tree_map(lambda a, b: 0.999 * a + 0.001 * b * b, v, g)
        t1 = t + 1.0
        lr_t = sched(t)
        p = jax.tree_util.tree_map(
            lambda w, a, b: w - lr_t * (a / (1 - 0.9 ** t1)) / (jnp.sqrt(b / (1 - 0.999 ** t1)) + 1e-8),
            p, m, v)
        return p, m, v, loss, gn

    @jax.jit
    def eval_batch(p, xs, ys):
        logits = forward(p, xs)
        pred = jnp.argmax(logits, -1).T      # [B,L]
        lp = jax.nn.log_softmax(logits, -1)
        nll = -jnp.take_along_axis(lp, ys.T[..., None], -1)[..., 0]
        return pred, nll.T

    return train_step, eval_batch, loss_fn


def count_params(p):
    return int(sum(np.prod(x.shape) for x in jax.tree_util.tree_leaves(p)))


EVAL_LENS = (32, 64, 128, 256, 512)


def evaluate(eval_batch, p, task, n_seq=512, Lmax=512, seed=12345, chunk=128):
    """One causal pass over length-Lmax sequences. For each L in EVAL_LENS report accuracy / NLL
    over the dyadic band of positions (L/2, L] (band for L=32 is in-distribution), and the exact
    final-position accuracy at L."""
    rng = np.random.default_rng(seed)
    xs, ys = sample_batch(rng, task, n_seq, Lmax)
    preds, nlls = [], []
    for i in range(0, n_seq, chunk):
        pr, nl = eval_batch(p, jnp.asarray(xs[i:i + chunk]), jnp.asarray(ys[i:i + chunk]))
        preds.append(np.asarray(pr))
        nlls.append(np.asarray(nl))
    pred = np.concatenate(preds, 0)
    nll = np.concatenate(nlls, 0)          # [B,L]
    corr = pred == ys
    out = {}
    for L in EVAL_LENS:
        if L > Lmax:
            continue
        band = slice(L // 2, L)
        out[L] = dict(acc_band=float(corr[:, band].mean()), acc_final=float(corr[:, L - 1].mean()),
                      nll_band=float(nll[:, band].mean()))
    return out, (xs, ys, pred)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", required=True)
    ap.add_argument("--model", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--lr", type=float, default=1e-2)
    ap.add_argument("--steps", type=int, default=3000)
    ap.add_argument("--batch", type=int, default=64)
    ap.add_argument("--ltrain", type=int, default=32)
    ap.add_argument("--out", required=True)
    ap.add_argument("--val_only", action="store_true", help="LR sweep: only in-distribution validation")
    ap.add_argument("--save", action="store_true")
    ap.add_argument("--gens", type=int, default=3, help="number of generator tokens (0 = all elements)")
    ap.add_argument("--curriculum", type=int, default=1, help="1: train lengths 2,4,8,16,ltrain")
    a = ap.parse_args()

    t_wall0, t_cpu0 = time.time(), time.process_time()
    task = get_task(a.task, a.gens)
    V, C = len(task["tokens"]), task["n"]
    key = jax.random.PRNGKey(a.seed)
    p = init_params(a.model, V, C, key)
    m = jax.tree_util.tree_map(jnp.zeros_like, p)
    v = jax.tree_util.tree_map(jnp.zeros_like, p)
    train_step, eval_batch, _ = make_train(a.model, a.lr, a.steps)
    rng = np.random.default_rng(1000 + a.seed)
    log = []
    if a.curriculum:
        lens = [l for l in (2, 4, 8, 16) if l < a.ltrain] + [a.ltrain]
        fr = np.array([0.15, 0.15, 0.2, 0.2, 0.3][-len(lens):])
        bounds = np.cumsum(fr / fr.sum()) * a.steps
    for t in range(a.steps):
        Lt = a.ltrain if not a.curriculum else lens[int(np.searchsorted(bounds, t, side="right"))]
        xs, ys = sample_batch(rng, task, a.batch, Lt)
        p, m, v, loss, gn = train_step(p, m, v, jnp.float32(t), jnp.asarray(xs), jnp.asarray(ys))
        if t % 250 == 0 or t == a.steps - 1:
            lf = float(loss)
            log.append((t, lf, float(gn)))
            if not np.isfinite(lf):
                break
    t_train_wall, t_train_cpu = time.time() - t_wall0, time.process_time() - t_cpu0
    if a.val_only:
        res, _ = evaluate(eval_batch, p, task, n_seq=512, Lmax=a.ltrain, seed=777)
    else:
        res, _ = evaluate(eval_batch, p, task)
    rec = dict(task=a.task, gens=a.gens, tokens=task["tokens"].tolist(), curriculum=a.curriculum, model=a.model, seed=a.seed, lr=a.lr, steps=a.steps, batch=a.batch,
               ltrain=a.ltrain, D=D, H=H, params=count_params(p), log=log, eval=res,
               train_wall_s=t_train_wall, train_cpu_s=t_train_cpu,
               total_wall_s=time.time() - t_wall0, total_cpu_s=time.process_time() - t_cpu0)
    os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
    with open(a.out + ".json", "w") as f:
        json.dump(rec, f)
    if a.save:
        np.savez(a.out + ".npz", **{k: np.asarray(x) for k, x in p.items()})
    summ = " ".join(f"L{L}:{r['acc_band']:.3f}" for L, r in res.items())
    print(f"{a.task} {a.model} s{a.seed} lr{a.lr} loss {log[-1][1]:.4f} | {summ} | "
          f"params {rec['params']} cpu {rec['total_cpu_s']:.1f}s", flush=True)


if __name__ == "__main__":
    main()
