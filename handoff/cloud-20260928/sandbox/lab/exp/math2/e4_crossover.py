"""math2 E4: width x candidate-count crossover for deepest-stored-ancestor retrieval (dot vs hyperbolic).

Adapted from the lead's cycle-1 script (scratch/exp/lead/geoattn_v4.py, tree task), stripped to the tree task.
Question: at the D8 read width (64) and the cycle-3 window (<=128 candidates), is there any representational headroom
for hyperbolic keys over dot-product keys on hierarchical retrieval?  And at what candidate count does it reappear?

Model (unchanged from cycle 1): q = Wq E[leaf], k_j = Wk E[stored node], value = E[value token]; tied readout.
Scores: dot q.k/sqrt(dk);  hyp -beta*arcosh(q0 k0 - q.k).
Reports raw accuracy and accuracy on NON-ROOT queries (target ancestor is not the always-stored root).
"""
import argparse, json, math, time
import numpy as np
import jax, jax.numpy as jnp


def build_tree(branch, depth):
    parent, level, frontier = [-1], [0], [0]
    for dpt in range(1, depth + 1):
        new = []
        for p_ in frontier:
            for _ in range(branch):
                parent.append(p_); level.append(dpt); new.append(len(parent) - 1)
        frontier = new
    parent, level = np.array(parent), np.array(level)
    anc = np.full((len(parent), depth + 1), -1)
    for n in range(len(parent)):
        m = n
        while m >= 0:
            anc[n, level[m]] = m; m = parent[m]
    return parent, level, anc


def make_tree_batch(rng, tree, B, D, Q, V):
    parent, level, anc = tree
    N = len(parent); internal = np.where(level < level.max())[0]; leaves = np.where(level == level.max())[0]
    toks = np.empty((B, 2 * D + Q), np.int32); tgt = np.empty((B, Q), np.int32); nonroot = np.empty((B, Q), bool)
    for b in range(B):
        stored = np.concatenate([[0], rng.choice(internal[1:], size=D - 1, replace=False)])
        rng.shuffle(stored)
        vals = rng.integers(0, V, size=D)
        pos = {n: i for i, n in enumerate(stored)}
        qs = rng.choice(leaves, size=Q, replace=False)
        toks[b, 0:2 * D:2] = stored; toks[b, 1:2 * D:2] = vals + N; toks[b, 2 * D:] = qs
        for j, leaf in enumerate(qs):
            best = next(pos[m] for m in anc[leaf][::-1] if m >= 0 and m in pos)
            tgt[b, j] = vals[best]; nonroot[b, j] = stored[best] != 0
    return toks, tgt, nonroot


def init(key, kind, vocab, d, dk):
    ks = iter(jax.random.split(key, 8))
    lin = lambda i, o: jax.random.normal(next(ks), (i, o)) / math.sqrt(i)
    return {"E": jax.random.normal(next(ks), (vocab, d)) / math.sqrt(d), "Wq": lin(d, dk), "Wk": lin(d, dk),
            "logout": jnp.array(math.log(10.0 * d)), "logb": jnp.array(0.0)}


def score(p, q, k, kind):
    dk = q.shape[-1]
    kt = jnp.swapaxes(k, -1, -2)
    if kind == "dot":
        return q @ kt / math.sqrt(dk)
    b = jnp.exp(p["logb"])
    q0 = jnp.sqrt(1.0 + jnp.sum(q * q, -1, keepdims=True))
    k0 = jnp.sqrt(1.0 + jnp.sum(k * k, -1, keepdims=True))
    z = q0 * jnp.swapaxes(k0, -1, -2) - q @ kt
    return -b * jnp.arccosh(jnp.maximum(z, 1.0 + 1e-5))


def forward(p, toks, kind, K, V, Q):
    L = toks.shape[1]
    pad = K + V
    prev = jnp.concatenate([jnp.full((toks.shape[0], 1), pad, toks.dtype), toks[:, :-1]], 1)
    x, s = p["E"][toks], p["E"][prev]           # cycle-1 wiring: key_j = Wk E[tok_{j-1}], value_j = E[tok_j]
    q = x[:, -Q:] @ p["Wq"]; k = s @ p["Wk"]
    sc = score(p, q, k, kind)
    causal = (jnp.arange(L)[None, :] < (L - Q + jnp.arange(Q))[:, None])[None]
    sc = jnp.where(causal, sc, -1e30)
    w = jax.nn.softmax(sc, -1)
    o = w @ x
    Evals = p["E"][K:K + V]
    return jnp.exp(p["logout"]) * (o @ Evals.T) / x.shape[-1]


def loss_fn(p, toks, tgt, kind, K, V, Q):
    logits = forward(p, toks, kind, K, V, Q)
    return jnp.mean(jax.nn.logsumexp(logits, -1) - jnp.take_along_axis(logits, tgt[..., None], -1)[..., 0])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kind", required=True, choices=["dot", "hyp"])
    ap.add_argument("--dk", type=int, default=64)
    ap.add_argument("--d", type=int, default=128)
    ap.add_argument("--branch", type=int, default=4)
    ap.add_argument("--depth", type=int, default=5)
    ap.add_argument("--D", type=int, default=48)
    ap.add_argument("--evalD", default="48,96,192")
    ap.add_argument("--Q", type=int, default=32)
    ap.add_argument("--V", type=int, default=512)
    ap.add_argument("--B", type=int, default=64)
    ap.add_argument("--steps", type=int, default=2000)
    ap.add_argument("--lr", type=float, default=3e-3)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    tree = build_tree(a.branch, a.depth)
    K = len(tree[0]); vocab = K + a.V + 1
    rng = np.random.default_rng(a.seed)
    p = init(jax.random.PRNGKey(a.seed), a.kind, vocab, a.d, a.dk)
    m = jax.tree_util.tree_map(jnp.zeros_like, p); v2 = jax.tree_util.tree_map(jnp.zeros_like, p)

    @jax.jit
    def step(p, m, v2, toks, tgt, lr, t):
        l, gr = jax.value_and_grad(loss_fn)(p, toks, tgt, a.kind, K, a.V, a.Q)
        gn = jnp.sqrt(sum(jnp.sum(x * x) for x in jax.tree_util.tree_leaves(gr)))
        sc = jnp.minimum(1.0, 1.0 / (gn + 1e-6))
        gr = jax.tree_util.tree_map(lambda x: x * sc, gr)
        m = jax.tree_util.tree_map(lambda a_, g: 0.9 * a_ + 0.1 * g, m, gr)
        v2 = jax.tree_util.tree_map(lambda a_, g: 0.99 * a_ + 0.01 * g * g, v2, gr)
        p = jax.tree_util.tree_map(lambda p_, mm, vv: p_ - lr * (mm / (1 - 0.9 ** t)) / (jnp.sqrt(vv / (1 - 0.99 ** t)) + 1e-8), p, m, v2)
        return p, m, v2, l

    t0 = time.time(); log = []
    for it in range(1, a.steps + 1):
        toks, tgt, _ = make_tree_batch(rng, tree, a.B, a.D, a.Q, a.V)
        lr = a.lr * min(1.0, it / 100) * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * it / a.steps)))
        p, m, v2, l = step(p, m, v2, jnp.asarray(toks), jnp.asarray(tgt), lr, it)
        if it % 500 == 0 or it == a.steps:
            log.append({"step": it, "loss": float(l)}); print(a.kind, a.dk, it, round(float(l), 4), flush=True)
    train_s = time.time() - t0
    fwd = jax.jit(forward, static_argnames=("kind", "K", "V", "Q"))
    ev = {}
    erng = np.random.default_rng(12345)
    for D in [int(x) for x in a.evalD.split(",")]:
        acc, nr_acc, nr_frac = [], [], []
        for _ in range(8):
            toks, tgt, nonroot = make_tree_batch(erng, tree, 32, D, a.Q, a.V)
            pred = np.asarray(jnp.argmax(fwd(p, jnp.asarray(toks), kind=a.kind, K=K, V=a.V, Q=a.Q), -1))
            ok = pred == tgt
            acc.append(ok.mean()); nr_acc.append(ok[nonroot].mean() if nonroot.any() else np.nan); nr_frac.append(nonroot.mean())
        ev[f"D{D}"] = {"acc": float(np.mean(acc)), "nonroot_acc": float(np.nanmean(nr_acc)),
                       "always_root_baseline": float(1 - np.mean(nr_frac)), "candidates": 2 * D}
    res = {"kind": a.kind, "dk": a.dk, "d": a.d, "branch": a.branch, "depth": a.depth, "nodes": K, "D_train": a.D,
           "steps": a.steps, "seed": a.seed, "train_s": train_s, "beta": float(jnp.exp(p["logb"])), "eval": ev, "log": log}
    json.dump(res, open(a.out, "w"), indent=1)
    print(json.dumps({k: v for k, v in res.items() if k != "log"}))


if __name__ == "__main__":
    main()
