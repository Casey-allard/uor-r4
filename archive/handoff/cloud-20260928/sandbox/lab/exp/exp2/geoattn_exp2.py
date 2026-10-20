"""exp2 copy of the lead's geoattn.py (scratch; NOT a project artifact).  exp2 changes: --save_params (npz of E,Wq,Wk,logb),
--tree_branch/--tree_depth (synthetic tree size), --eval_Ds, --no_route_eval, per-row sampling for large K.

Lead's geometric-attention scratch experiment, v2 (NOT a project artifact; project model code stays in Rust).

Task: multi-query associative recall (MQAR-like).  A sequence holds D key-value pairs, then Q queries
(keys seen earlier); the target at each query is the value paired with that key.

Minimal retrieval head, so that the *scoring geometry* is what is tested:
  query  q_t = Wq E[tok_t]          (at query positions only)
  key    k_j = Wk E[tok_{j-1}]      (each position is keyed by its previous token)
  value  v_j = E[tok_j]
  logits = beta_out * (sum_j w_tj v_j) . E[values]^T      (readout tied to the value embeddings)
Scoring kinds (same parameters everywhere):
  dot   : q.k / sqrt(dk)                                   (transformer softmax attention)
  cos   : beta * cos(q,k)                                   (spherical geodesic; monotone in the angle)
  hyp   : -beta * arcosh(-<q,k>_Lorentz)                    (hyperbolic geodesic on the hyperboloid)
  ham   : beta * sign(q).sign(k) / dk                       (angle via Hamming distance: XOR+popcount)
  e8    : beta * mean_b E8(q_b).E8(k_b)                     (8-D blocks snapped to the 240 E8 roots)
  bind  : no attention.  One superposed state h = sum_j (Wk E[tok_{j-1}]) * (Wv E[tok_j]) (element-wise
          binding, "entangled" pairs in one vector); read by unbinding with Wq E[query]
Post-hoc routing ("locations advertise their data"): chunks of c positions publish a bundle (majority of
sign codes, or mean key); a query ranks bundles, keeps the top r chunks and attends only inside them.
Two-level routing bundles the bundles (recursion).
"""
import argparse, json, math, time
import numpy as np
import jax, jax.numpy as jnp


def e8_roots():
    import itertools
    r = []
    for i, j in itertools.combinations(range(8), 2):
        for si in (1.0, -1.0):
            for sj in (1.0, -1.0):
                v = [0.0] * 8; v[i] = si; v[j] = sj; r.append(v)
    for signs in itertools.product((0.5, -0.5), repeat=8):
        if sum(s < 0 for s in signs) % 2 == 0:
            r.append(list(signs))
    r = np.array(r)
    assert r.shape == (240, 8) and np.allclose((r * r).sum(1), 2.0)
    assert set(np.unique(np.round(r @ r.T, 9))) == {-2.0, -1.0, 0.0, 1.0, 2.0}
    return jnp.asarray(r / np.sqrt(2.0), jnp.float32)


E8 = e8_roots()


def make_batch(rng, B, D, Q, K, V):
    keys = np.stack([rng.choice(K, size=D, replace=False) for _ in range(B)])
    vals = rng.integers(0, V, size=(B, D)) + K
    L = 2 * D + Q
    toks = np.empty((B, L), np.int32)
    toks[:, 0:2 * D:2] = keys
    toks[:, 1:2 * D:2] = vals
    qi = np.argsort(rng.random((B, D)), axis=1)[:, :Q]
    toks[:, 2 * D:] = np.take_along_axis(keys, qi, 1)
    tgt = (np.take_along_axis(vals, qi, 1) - K).astype(np.int32)
    ans = (2 * qi + 1).astype(np.int32)
    return toks, tgt, ans


def build_tree(branch, depth):
    parent, level, frontier = [-1], [0], [0]
    for dpt in range(1, depth + 1):
        new = []
        for p_ in frontier:
            for _ in range(branch):
                parent.append(p_); level.append(dpt); new.append(len(parent) - 1)
        frontier = new
    parent, level = np.array(parent), np.array(level)
    anc = np.full((len(parent), depth + 1), -1)                # anc[n, l] = ancestor of n at level l (or n itself)
    for n in range(len(parent)):
        m = n
        while m >= 0:
            anc[n, level[m]] = m; m = parent[m]
    return parent, level, anc


def tree_from_parent(parent):
    parent = np.asarray(parent)
    N = len(parent)
    level = np.zeros(N, int)
    for n in range(1, N):
        level[n] = level[parent[n]] + 1
    anc = np.full((N, level.max() + 1), -1)
    for n in range(N):
        m = n
        while m >= 0:
            anc[n, level[m]] = m; m = parent[m]
    return parent, level, anc


TREE = build_tree(4, 5)


ANC_BIAS = 0          # exp2: number of training queries per sequence that get one random proper ancestor stored


def make_tree_batch(rng, B, D, Q, V):
    """Stored keys: D internal nodes (root always included) with random values. Queries: leaves.
    Target: the value stored at the query's deepest stored ancestor.
    exp2: with ANC_BIAS > 0 the first ANC_BIAS queries each get one random non-root proper ancestor stored (so large
    trees still give non-root training targets); the rest of the store is uniform over internal nodes."""
    parent, level, anc = TREE
    N = len(parent)
    has_child = np.zeros(N, bool); has_child[parent[parent >= 0]] = True
    internal = np.where(has_child)[0]; leaves = np.where(~has_child)[0]
    toks = np.empty((B, 2 * D + Q), np.int32); tgt = np.empty((B, Q), np.int32); ans = np.empty((B, Q), np.int32)
    for b in range(B):
        qs = rng.choice(leaves, size=Q, replace=False)
        must = []
        for leaf in qs[:ANC_BIAS]:
            a = anc[leaf][1:level[leaf]]
            a = a[a >= 0]
            if len(a):
                must.append(int(rng.choice(a)))
        must = np.unique(np.array(must, dtype=np.int64))[:D - 1]
        pool = internal[(internal != 0) & ~np.isin(internal, must)]
        stored = np.concatenate([[0], must, rng.choice(pool, size=D - 1 - len(must), replace=False)]).astype(np.int64)
        rng.shuffle(stored)
        vals = rng.integers(0, V, size=D)
        pos = {n: i for i, n in enumerate(stored)}
        toks[b, 0:2 * D:2] = stored; toks[b, 1:2 * D:2] = vals + N; toks[b, 2 * D:] = qs
        for j, leaf in enumerate(qs):
            best = next(pos[m] for m in anc[leaf][::-1] if m >= 0 and m in pos)
            tgt[b, j] = vals[best]; ans[b, j] = 2 * best + 1
    return toks, tgt, ans


def sign_ste(x):
    return x + jax.lax.stop_gradient(jnp.where(x >= 0, 1.0, -1.0) - x)


def e8_snap(x):
    xb = x.reshape(x.shape[:-1] + (-1, 8))
    s = E8[jnp.argmax(xb @ E8.T, -1)]
    return (xb + jax.lax.stop_gradient(s - xb)).reshape(x.shape)


def init(key, kind, vocab, d, dk):
    ks = iter(jax.random.split(key, 8))
    lin = lambda i, o: jax.random.normal(next(ks), (i, o)) / math.sqrt(i)
    p = {"E": jax.random.normal(next(ks), (vocab, d)) / math.sqrt(d), "Wq": lin(d, dk), "Wk": lin(d, dk),
         "logout": jnp.array(math.log(10.0 * d)), "logr": jnp.array(math.log(10.0))}
    if kind == "bind":
        p["Wv"] = lin(d, dk); p["Wo"] = lin(dk, d)
    else:
        p["logb"] = jnp.array(math.log(10.0) if kind in ("cos", "ham", "e8") else 0.0)
    return p


RBITS = 0
DBITS = 1


def radius_sign_quantize(k):
    """Owner's quantizer in hyperbolic space: keep a quantized radius, code the direction with sign bits."""
    dk = k.shape[-1]
    r = jnp.sqrt(jnp.sum(k * k, -1, keepdims=True) + 1e-9)
    if RBITS == 0:
        rq = jnp.ones_like(r) * 3.0                                   # radius discarded (fixed)
    else:
        lo, hi, n = math.log(0.1), math.log(20.0), 2 ** RBITS
        idx = jnp.clip(jnp.round((jnp.log(r) - lo) / (hi - lo) * (n - 1)), 0, n - 1)
        rq = jnp.exp(lo + idx / (n - 1) * (hi - lo))
    if DBITS <= 1:
        u = jnp.where(k >= 0, 1.0, -1.0)                                  # sign code: 1 bit per coordinate
    else:                                                                  # uniform scalar code per coordinate
        z = jnp.clip(k / r * math.sqrt(dk), -2.0, 2.0)
        levels = 2 ** DBITS
        u = -2.0 + (jnp.round((z + 2.0) / 4.0 * (levels - 1)) / (levels - 1)) * 4.0
    u = u / jnp.sqrt(jnp.sum(u * u, -1, keepdims=True) + 1e-9)
    kq = rq * u
    return k + jax.lax.stop_gradient(kq - k)


def codes(q, k, kind):
    if kind == "hypq":
        return q, radius_sign_quantize(k)
    if kind == "ham":
        return sign_ste(q), sign_ste(k)
    if kind == "e8":
        return e8_snap(q), e8_snap(k)
    return q, k


def score(p, q, k, kind):
    dk = q.shape[-1]
    kt = jnp.swapaxes(k, -1, -2)
    if kind == "dot":
        return q @ kt / math.sqrt(dk)
    b = jnp.exp(p["logb"])
    if kind == "cos":
        qn = q * jax.lax.rsqrt(jnp.sum(q * q, -1, keepdims=True) + 1e-6)
        kn = k * jax.lax.rsqrt(jnp.sum(k * k, -1, keepdims=True) + 1e-6)
        return b * (qn @ jnp.swapaxes(kn, -1, -2))
    if kind in ("hyp", "hypq"):
        q0 = jnp.sqrt(1.0 + jnp.sum(q * q, -1, keepdims=True))
        k0 = jnp.sqrt(1.0 + jnp.sum(k * k, -1, keepdims=True))
        z = q0 * jnp.swapaxes(k0, -1, -2) - q @ kt
        return -b * jnp.arccosh(jnp.maximum(z, 1.0 + 1e-5))
    if kind == "ham":
        return b * (q @ kt) / dk
    if kind == "e8":
        return b * (q @ kt) / (dk // 8)
    raise ValueError(kind)


def parts(p, toks, pad, Q, kind):
    prev = jnp.concatenate([jnp.full((toks.shape[0], 1), pad, toks.dtype), toks[:, :-1]], 1)
    x, s = p["E"][toks], p["E"][prev]
    q, k = codes(x[:, -Q:] @ p["Wq"], s @ p["Wk"], kind)
    return x, s, q, k


def forward(p, toks, kind, pad, K, V, Q, mask_extra=None, hard=False):
    x, s, q, k = parts(p, toks, pad, Q, kind)
    L = toks.shape[1]
    Evals = p["E"][K:K + V]
    if kind == "bind":
        mem = jnp.sum((s @ p["Wk"]) * (x @ p["Wv"]), 1, keepdims=True)      # one superposed state per sequence
        o = ((q) * mem) @ p["Wo"]                                             # unbind with the query
        return jnp.exp(p["logout"]) * (o @ Evals.T) / o.shape[-1]
    sc = score(p, q, k, kind)                                                # (B, Q, L)
    causal = (jnp.arange(L)[None, :] < (L - Q + jnp.arange(Q))[:, None])[None]
    m = causal if mask_extra is None else (causal & mask_extra)
    sc = jnp.where(m, sc, -1e30)
    w = jax.nn.one_hot(jnp.argmax(sc, -1), L) if hard else jax.nn.softmax(sc, -1)
    o = w @ x
    return jnp.exp(p["logout"]) * (o @ Evals.T) / x.shape[-1]


def loss_fn(p, toks, tgt, kind, pad, K, V, Q, ans=None, route_c=16, route_w=0.0):
    logits = forward(p, toks, kind, pad, K, V, Q)
    lz = jax.nn.logsumexp(logits, -1)
    lt = jnp.take_along_axis(logits, tgt[..., None], -1)[..., 0]
    loss = jnp.mean(lz - lt)
    if route_w > 0.0:
        _, _, q, k = parts(p, toks, pad, Q, kind)
        B, L, dk = k.shape
        D2 = L - Q
        nC = D2 // route_c
        kc = k[:, :nC * route_c].reshape(B, nC, route_c, dk)
        bund = bundle(kc, kind, ste=True)
        cs = jnp.exp(p["logr"]) * score(p, q, bund, kind) / jnp.exp(p.get("logb", 0.0))
        tc = ans // route_c
        lzc = jax.nn.logsumexp(cs, -1)
        ltc = jnp.take_along_axis(cs, tc[..., None], -1)[..., 0]
        loss = loss + route_w * jnp.mean(lzc - ltc)
    return loss


def accuracy(logits, tgt):
    return float(jnp.mean(jnp.argmax(logits, -1) == tgt))


def bundle(kc, kind, ste=False):
    """Advertisement of a chunk: majority vote (ham), Lorentzian centroid on the hyperboloid (hyp), mean otherwise.
    kc: (..., c, dk) member keys (spatial coordinates for hyp)."""
    if kind == "ham":
        sm = kc.sum(-2)
        sg = jnp.where(sm >= 0, 1.0, -1.0)
        return sm + jax.lax.stop_gradient(sg - sm) if ste else sg
    if kind == "hyp":
        x0 = jnp.sqrt(1.0 + jnp.sum(kc * kc, -1, keepdims=True))
        s0, ss = x0.sum(-2), kc.sum(-2)
        nrm = jnp.sqrt(jnp.maximum(s0 * s0 - jnp.sum(ss * ss, -1, keepdims=True), 1e-9))
        return ss / nrm                        # spatial part of the centroid; its time part is sqrt(1+|.|^2)
    return kc.mean(-2)


def route_mask(p, toks, kind, pad, Q, D, c, r, g=None, r1=None):
    """Chunks of the key/value region [0, 2D) advertise a bundle; each query keeps the top-r chunks."""
    _, _, q, k = parts(p, toks, pad, Q, kind)
    B, L, dk = k.shape
    nC = (2 * D) // c
    kc = k[:, :nC * c].reshape(B, nC, c, dk)
    bund = bundle(kc, kind)
    cs = score(p, q, bund, kind)                                                # (B, Q, nC)
    comps = nC
    if g is not None:
        nS = nC // g
        sb = kc.reshape(B, nS, g * c, dk)
        sbund = bundle(sb, kind)
        ss = score(p, q, sbund, kind)                                           # (B, Q, nS)
        allowS = jax.nn.one_hot(jnp.argsort(-ss, -1)[..., :r1], nS).sum(-2) > 0
        cs = jnp.where(jnp.repeat(allowS, g, axis=-1), cs, -1e30)
        comps = nS + r1 * g
    selC = jax.nn.one_hot(jnp.argsort(-cs, -1)[..., :r], nC).sum(-2) > 0        # (B, Q, nC)
    pos_chunk = jnp.minimum(jnp.arange(L) // c, nC - 1)
    posmask = jnp.take_along_axis(selC, jnp.broadcast_to(pos_chunk, (B, Q, L)), -1)
    posmask = posmask & (jnp.arange(L) < nC * c)[None, None, :]
    return posmask, comps + r * c


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kind", required=True)
    ap.add_argument("--dk", type=int, default=64)
    ap.add_argument("--d", type=int, default=128)
    ap.add_argument("--D", type=int, default=64)
    ap.add_argument("--Q", type=int, default=32)
    ap.add_argument("--K", type=int, default=512)
    ap.add_argument("--V", type=int, default=512)
    ap.add_argument("--B", type=int, default=64)
    ap.add_argument("--steps", type=int, default=2000)
    ap.add_argument("--lr", type=float, default=3e-3)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--task", default="mqar", choices=["mqar", "tree"])
    ap.add_argument("--tree_file", default=None)
    ap.add_argument("--rbits", type=int, default=0)
    ap.add_argument("--dbits", type=int, default=1)
    ap.add_argument("--route_w", type=float, default=0.0)
    ap.add_argument("--route_c", type=int, default=16)
    ap.add_argument("--out", required=True)
    ap.add_argument("--save_params", default=None)
    ap.add_argument("--tree_branch", type=int, default=4)
    ap.add_argument("--tree_depth", type=int, default=5)
    ap.add_argument("--eval_Ds", default=None, help="comma list of stored-pair counts for the built-in eval")
    ap.add_argument("--no_route_eval", action="store_true")
    ap.add_argument("--anc_bias", type=int, default=0)
    a = ap.parse_args()
    global TREE, RBITS, DBITS, ANC_BIAS
    RBITS, DBITS = a.rbits, a.dbits
    ANC_BIAS = a.anc_bias
    if a.tree_file:
        TREE = tree_from_parent(np.load(a.tree_file)["parent"])
    elif (a.tree_branch, a.tree_depth) != (4, 5):
        TREE = build_tree(a.tree_branch, a.tree_depth)
    if a.task == "tree":
        a.K = len(TREE[0])                    # node tokens come first; values follow
    vocab, pad = a.K + a.V + 1, a.K + a.V
    batch = (lambda r, B, D: make_tree_batch(r, B, D, a.Q, a.V)) if a.task == "tree" else \
            (lambda r, B, D: make_batch(r, B, D, a.Q, a.K, a.V))
    rng = np.random.default_rng(a.seed)
    p = init(jax.random.PRNGKey(a.seed), a.kind, vocab, a.d, a.dk)
    m = jax.tree_util.tree_map(jnp.zeros_like, p); v2 = jax.tree_util.tree_map(jnp.zeros_like, p)

    @jax.jit
    def step(p, m, v2, toks, tgt, ans, lr, t):
        l, gr = jax.value_and_grad(loss_fn)(p, toks, tgt, a.kind, pad, a.K, a.V, a.Q, ans, a.route_c, a.route_w)
        gn = jnp.sqrt(sum(jnp.sum(x * x) for x in jax.tree_util.tree_leaves(gr)))
        sc = jnp.minimum(1.0, 1.0 / (gn + 1e-6))
        gr = jax.tree_util.tree_map(lambda x: x * sc, gr)
        m = jax.tree_util.tree_map(lambda a_, g: 0.9 * a_ + 0.1 * g, m, gr)
        v2 = jax.tree_util.tree_map(lambda a_, g: 0.99 * a_ + 0.01 * g * g, v2, gr)
        p = jax.tree_util.tree_map(lambda p_, mm, vv: p_ - lr * (mm / (1 - 0.9 ** t)) / (jnp.sqrt(vv / (1 - 0.99 ** t)) + 1e-8), p, m, v2)
        return p, m, v2, l

    log = []; t0 = time.time()
    for it in range(1, a.steps + 1):
        toks, tgt, ans = batch(rng, a.B, a.D)
        lr = a.lr * min(1.0, it / 100) * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * it / a.steps)))
        p, m, v2, l = step(p, m, v2, jnp.asarray(toks), jnp.asarray(tgt), jnp.asarray(ans), lr, it)
        if it % 250 == 0 or it == a.steps:
            log.append({"step": it, "loss": float(l)}); print(a.kind, it, round(float(l), 4), flush=True)
    train_s = time.time() - t0

    fwd = jax.jit(forward, static_argnames=("kind", "pad", "K", "V", "Q", "hard"))
    ev = {}
    erng = np.random.default_rng(12345)
    eval_Ds = tuple(int(x) for x in a.eval_Ds.split(",")) if a.eval_Ds else ((64, 128, 256, 512) if a.task == "mqar" else (48, 96, 192))
    for D in eval_Ds:
        accs, haccs, nr_hit, nr_tot, base_hit = [], [], 0, 0, 0
        for _ in range(8):
            toks, tgt, ans = batch(erng, 32, D)
            logits = fwd(p, jnp.asarray(toks), kind=a.kind, pad=pad, K=a.K, V=a.V, Q=a.Q)
            accs.append(accuracy(logits, tgt))
            if a.task == "tree":                     # accuracy on queries whose answer is not the (always stored) root
                root_pos = 2 * np.argmax(np.asarray(toks)[:, 0:2 * D:2] == 0, axis=1) + 1
                nonroot = ans != root_pos[:, None]
                correct = np.asarray(jnp.argmax(logits, -1)) == tgt
                nr_hit += int(correct[nonroot].sum()); nr_tot += int(nonroot.sum()); base_hit += int((~nonroot).sum())
            toks = jnp.asarray(toks)
            if a.kind != "bind":
                haccs.append(accuracy(fwd(p, toks, kind=a.kind, pad=pad, K=a.K, V=a.V, Q=a.Q, hard=True), tgt))
        ev[f"acc_D{D}"] = float(np.mean(accs))
        if a.task == "tree":
            ev[f"acc_nonroot_D{D}"] = nr_hit / max(nr_tot, 1)
            ev[f"always_root_D{D}"] = base_hit / (base_hit + nr_tot)
        if haccs:
            ev[f"hard_acc_D{D}"] = float(np.mean(haccs))
    routing = []
    if a.task == "mqar" and a.kind in ("ham", "e8", "dot", "cos", "hyp") and a.dk >= 64 and not a.no_route_eval:
        D = 512
        rrng = np.random.default_rng(777)
        batches = [make_batch(rrng, 32, D, a.Q, a.K, a.V)[:2] for _ in range(4)]
        for cfg in [dict(c=16, r=1), dict(c=16, r=2), dict(c=16, r=4), dict(c=32, r=2), dict(c=8, r=4),
                    dict(c=16, r=2, g=4, r1=2), dict(c=16, r=4, g=8, r1=2)]:
            accs, haccs, comp = [], [], None
            for toks, tgt in batches:
                toks = jnp.asarray(toks)
                pm, comp = route_mask(p, toks, a.kind, pad, a.Q, D, **cfg)
                accs.append(accuracy(forward(p, toks, a.kind, pad, a.K, a.V, a.Q, mask_extra=pm), tgt))
                haccs.append(accuracy(forward(p, toks, a.kind, pad, a.K, a.V, a.Q, mask_extra=pm, hard=True), tgt))
            routing.append({**cfg, "comparisons_per_query": int(comp), "full_comparisons": 2 * D,
                            "acc": float(np.mean(accs)), "hard_acc": float(np.mean(haccs))})
            print("route", cfg, round(routing[-1]["acc"], 4), round(routing[-1]["hard_acc"], 4), comp, flush=True)
    res = {"tree_file": a.tree_file, "rbits": a.rbits, "dbits": a.dbits, "task": a.task, "route_w": a.route_w, "route_c": a.route_c, "kind": a.kind, "dk": a.dk, "d": a.d, "D_train": a.D, "Q": a.Q, "steps": a.steps, "seed": a.seed,
           "params": int(sum(x.size for x in jax.tree_util.tree_leaves(p))), "train_s": train_s,
           "beta": float(jnp.exp(p["logb"])) if "logb" in p else None, **ev, "routing": routing, "log": log}
    res["tree_branch"], res["tree_depth"], res["K"], res["V"] = a.tree_branch, a.tree_depth, a.K, a.V
    json.dump(res, open(a.out, "w"), indent=1)
    if a.save_params:
        np.savez(a.save_params, **{k_: np.asarray(v_) for k_, v_ in p.items()},
                 meta=json.dumps({"kind": a.kind, "dk": a.dk, "d": a.d, "task": a.task, "K": a.K, "V": a.V,
                                  "tree_file": a.tree_file, "tree_branch": a.tree_branch, "tree_depth": a.tree_depth,
                                  "rbits": a.rbits, "dbits": a.dbits, "route_w": a.route_w, "route_c": a.route_c,
                                  "D_train": a.D, "steps": a.steps, "seed": a.seed, "anc_bias": a.anc_bias}))
    print(json.dumps({k: v for k, v in res.items() if k != "log"}))


if __name__ == "__main__":
    main()
