"""Lead's wave-2 scratch experiment (NOT a project artifact; the project policy keeps model code in Rust).

Question: inside an otherwise identical small byte-level LM, how do time-mixing mechanisms compare on real text?
  diag    : input-dependent real diagonal decay (Mamba/HGRN/minGRU-like, commutative)
  complex : input-dependent 2-D rotation + decay per channel pair (LRU-like, commutative)
  quat    : input-dependent unit-quaternion rotation (left Hamilton product) + radial decay per 4-D lane (non-commutative)
  quat2i  : quat with the rotation snapped to the nearest of the 120 binary-icosahedral (600-cell) unit quaternions
            in the forward pass (straight-through gradient) -> exact finite-group transport at serving
  attn    : single-head causal softmax attention with RoPE (a transformer-style reference, not a serving candidate)
All linear recurrences are trained with an associative (parallel) scan.  Parameter budgets are matched: every
time-mix uses four d x d maps (value, gate, transition, output).
"""
import argparse, json, math, os, sys, time
import numpy as np
import jax, jax.numpy as jnp

PHI = (1 + 5 ** 0.5) / 2


def two_i_codebook():
    pts = []
    for i in range(4):
        for s in (1.0, -1.0):
            v = [0.0] * 4; v[i] = s; pts.append(v)
    for sx in (0.5, -0.5):
        for sy in (0.5, -0.5):
            for sz in (0.5, -0.5):
                for sw in (0.5, -0.5):
                    pts.append([sx, sy, sz, sw])
    import itertools
    base = [0.0, 0.5, 1 / (2 * PHI), PHI / 2]  # standard icosian reference order (w, x, y, z)
    for perm in itertools.permutations(range(4)):
        if sum(perm[i] > perm[j] for i in range(4) for j in range(i + 1, 4)) % 2:
            continue  # even permutations only
        for sg in itertools.product((1, -1), repeat=4):
            v = [0.0] * 4
            for k in range(4):
                v[perm[k]] = base[k] * sg[k]
            pts.append(v)
    c = np.unique(np.round(np.array(pts), 12), axis=0)
    assert c.shape == (120, 4), c.shape
    assert np.allclose(np.linalg.norm(c, axis=1), 1.0)
    # closure under the Hamilton product (group check)
    aw, ax, ay, az = [c[:, None, i] for i in range(4)]
    bw, bx, by, bz = [c[None, :, i] for i in range(4)]
    prod = np.stack([aw * bw - ax * bx - ay * by - az * bz, aw * bx + ax * bw + ay * bz - az * by,
                     aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw], -1).reshape(-1, 4)
    assert np.max(np.min(np.linalg.norm(prod[:, None, :] - c[None, :, :], axis=2), axis=1)) < 1e-9
    return jnp.asarray(c, dtype=jnp.float32)


C2I = two_i_codebook()


def hamilton(a, b):
    aw, ax, ay, az = a[..., 0], a[..., 1], a[..., 2], a[..., 3]
    bw, bx, by, bz = b[..., 0], b[..., 1], b[..., 2], b[..., 3]
    return jnp.stack([aw * bw - ax * bx - ay * by - az * bz,
                      aw * bx + ax * bw + ay * bz - az * by,
                      aw * by - ax * bz + ay * bw + az * bx,
                      aw * bz + ax * by - ay * bx + az * bw], -1)


def int_codebook():
    rows = []
    for k in range(5):  # m = 1, 2, 4, 8, 16 -> fine rotations near identity
        for a in (-1, 0, 1):
            for b in (-1, 0, 1):
                for c in (-1, 0, 1):
                    rows.append([2.0 ** k, a, b, c])
    for a in (-1, 0, 1):  # half-turns (m = 0)
        for b in (-1, 0, 1):
            for c in (-1, 0, 1):
                if (a, b, c) != (0, 0, 0):
                    rows.append([0.0, a, b, c])
    r = np.array(rows, dtype=np.float64)
    r = r / np.linalg.norm(r, axis=1, keepdims=True)
    return jnp.asarray(np.unique(np.round(r, 12), axis=0), dtype=jnp.float32)


CINT = int_codebook()


def snap_int(q):
    return CINT[jnp.argmax(q @ CINT.T, axis=-1)]


def snap_2i(q):
    idx = jnp.argmax(q @ C2I.T, axis=-1)
    return C2I[idx]


def rmsnorm(x, w):
    return x * jax.lax.rsqrt(jnp.mean(x * x, -1, keepdims=True) + 1e-6) * w


def timescale_bias(n, lo=2.0, hi=512.0):
    tau = np.exp(np.linspace(math.log(lo), math.log(hi), n))
    a = 1.0 - 1.0 / tau
    return jnp.asarray(np.log(a / (1 - a)), dtype=jnp.float32)


def init_params(key, kind, d, layers, vocab=256, mlp_mult=2):
    ks = iter(jax.random.split(key, 16 * layers + 8))
    def lin(i, o, s=None):
        s = s if s is not None else 1.0 / math.sqrt(i)
        return jax.random.normal(next(ks), (i, o), jnp.float32) * s
    p = {"emb": jax.random.normal(next(ks), (vocab, d)) * 0.5, "blocks": []}
    for _ in range(layers):
        b = {"n1": jnp.ones(d), "n2": jnp.ones(d), "Wv": lin(d, d), "Wg": lin(d, d), "Wo": lin(d, d, 0.5 / math.sqrt(d)),
             "W1": lin(d, mlp_mult * d), "W2": lin(d, mlp_mult * d), "W3": lin(mlp_mult * d, d, 0.5 / math.sqrt(mlp_mult * d)),
             "hn": jnp.ones(d)}
        if kind == "attn":
            b["Wq"] = lin(d, d); b["Wk"] = lin(d, d)
        elif kind == "gru":
            b["Wa"] = lin(d, d); b["Wr"] = lin(d, d)
            b["Uz"] = lin(d, d, 0.5 / math.sqrt(d)); b["Ur"] = lin(d, d, 0.5 / math.sqrt(d)); b["Un"] = lin(d, d, 0.5 / math.sqrt(d))
            b["ba"] = timescale_bias(d)
        else:
            b["Wa"] = lin(d, d, 0.1 / math.sqrt(d))
            if kind == "diag":
                b["ba"] = timescale_bias(d)
            elif kind == "complex":
                b["ba"] = jnp.concatenate([jnp.zeros(d // 2), timescale_bias(d // 2)])
            else:  # quat lanes: 3 rotation coords + 1 radius per 4-D lane
                lanes = d // 4
                b["ba"] = jnp.concatenate([jnp.zeros(3 * lanes), timescale_bias(lanes)])
        p["blocks"].append(b)
    p["nf"] = jnp.ones(d)
    p["out"] = lin(d, vocab, 1.0 / math.sqrt(d))
    return p


def rope(x):
    B, T, d = x.shape
    half = d // 2
    freqs = 1.0 / (10000 ** (jnp.arange(half) / half))
    ang = jnp.arange(T)[:, None] * freqs[None, :]
    c, s = jnp.cos(ang), jnp.sin(ang)
    x1, x2 = x[..., :half], x[..., half:]
    return jnp.concatenate([x1 * c - x2 * s, x1 * s + x2 * c], -1)


def time_mix(b, x, kind, snap):
    B, T, d = x.shape
    v = x @ b["Wv"]
    g = jax.nn.silu(x @ b["Wg"])
    if kind == "gru":
        xz = x @ b["Wa"] + b["ba"]; xr = x @ b["Wr"]; xn = v
        def cell(hprev, inp):
            zt, rt, nt = inp
            z = jax.nn.sigmoid(zt + hprev @ b["Uz"]); r = jax.nn.sigmoid(rt + hprev @ b["Ur"])
            n = jnp.tanh(nt + r * (hprev @ b["Un"]))
            hnew = z * hprev + (1 - z) * n
            return hnew, hnew
        _, hs = jax.lax.scan(cell, jnp.zeros((B, d)), (jnp.swapaxes(xz, 0, 1), jnp.swapaxes(xr, 0, 1), jnp.swapaxes(xn, 0, 1)))
        h = rmsnorm(jnp.swapaxes(hs, 0, 1), b["hn"])
        return (h * g) @ b["Wo"]
    if kind == "attn":
        q, k = rope(x @ b["Wq"]), rope(x @ b["Wk"])
        att = (q @ jnp.swapaxes(k, 1, 2)) / math.sqrt(d)
        mask = jnp.tril(jnp.ones((T, T), bool))
        att = jnp.where(mask, att, -1e30)
        h = jax.nn.softmax(att, -1) @ v
    else:
        z = x @ b["Wa"] + b["ba"]
        if kind == "diag":
            a = jax.nn.sigmoid(z)
            comb = lambda e1, e2: (e2[0] * e1[0], e2[0] * e1[1] + e2[1])
            _, h = jax.lax.associative_scan(comb, (a, (1 - a) * v), axis=1)
        elif kind == "complex":
            theta, r = z[..., : d // 2], jax.nn.sigmoid(z[..., d // 2:])
            A = r * jnp.exp(1j * theta)
            vc = v[..., : d // 2] + 1j * v[..., d // 2:]
            comb = lambda e1, e2: (e2[0] * e1[0], e2[0] * e1[1] + e2[1])
            _, hc = jax.lax.associative_scan(comb, (A, (1 - r) * vc), axis=1)
            h = jnp.concatenate([hc.real, hc.imag], -1)
        else:
            lanes = d // 4
            raw = z[..., : 3 * lanes].reshape(B, T, lanes, 3)
            r = jax.nn.sigmoid(z[..., 3 * lanes:])[..., None]
            qv = jnp.concatenate([jnp.ones((B, T, lanes, 1)), raw], -1)
            q = qv / jnp.linalg.norm(qv, axis=-1, keepdims=True)
            if kind == "quat2i" or snap is True or snap == "2i":
                q = q + jax.lax.stop_gradient(snap_2i(q) - q)
            elif kind == "quatint" or snap == "int":
                q = q + jax.lax.stop_gradient(snap_int(q) - q)
            comb = lambda e1, e2: (hamilton(e2[0], e1[0]), hamilton(e2[0], e1[1]) + e2[1])
            _, hq = jax.lax.associative_scan(comb, (r * q, (1 - r) * v.reshape(B, T, lanes, 4)), axis=1)
            h = hq.reshape(B, T, d)
        h = rmsnorm(h, b["hn"])
    return (h * g) @ b["Wo"]


def forward(p, tokens, kind, snap=False):
    x = p["emb"][tokens]
    for b in p["blocks"]:
        x = x + time_mix(b, rmsnorm(x, b["n1"]), kind, snap)
        y = rmsnorm(x, b["n2"])
        x = x + (jax.nn.silu(y @ b["W1"]) * (y @ b["W2"])) @ b["W3"]
    return rmsnorm(x, p["nf"]) @ p["out"]


def loss_fn(p, xb, yb, kind, snap=False):
    logits = forward(p, xb, kind, snap)
    logz = jax.nn.logsumexp(logits, -1)
    tgt = jnp.take_along_axis(logits, yb[..., None], -1)[..., 0]
    return jnp.mean(logz - tgt)


def count(p):
    return int(sum(x.size for x in jax.tree_util.tree_leaves(p)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--kind", required=True)
    ap.add_argument("--d", type=int, default=128)
    ap.add_argument("--layers", type=int, default=3)
    ap.add_argument("--T", type=int, default=128)
    ap.add_argument("--B", type=int, default=32)
    ap.add_argument("--steps", type=int, default=1200)
    ap.add_argument("--lr", type=float, default=3e-3)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--eval_T", type=int, default=512)
    ap.add_argument("--eval_bytes", type=int, default=262144)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    root = "/home/user/uor-r4/research/ai-research/ai-router/router-research/data/lm_proxy/raw/wikitext2/"
    train = np.frombuffer(open(root + "train.txt", "rb").read(), dtype=np.uint8)
    valid = np.frombuffer(open(root + "valid.txt", "rb").read(), dtype=np.uint8)
    rng = np.random.default_rng(a.seed)
    p = init_params(jax.random.PRNGKey(a.seed), a.kind, a.d, a.layers)
    nparam = count(p)
    m = jax.tree_util.tree_map(jnp.zeros_like, p); v = jax.tree_util.tree_map(jnp.zeros_like, p)
    b1, b2, eps, wd = 0.9, 0.95, 1e-8, 0.01

    @jax.jit
    def step(p, m, v, xb, yb, lr, t):
        l, gr = jax.value_and_grad(loss_fn)(p, xb, yb, a.kind)
        gn = jnp.sqrt(sum(jnp.sum(g * g) for g in jax.tree_util.tree_leaves(gr)))
        scale = jnp.minimum(1.0, 1.0 / (gn + 1e-6))
        gr = jax.tree_util.tree_map(lambda g: g * scale, gr)
        m = jax.tree_util.tree_map(lambda m_, g: b1 * m_ + (1 - b1) * g, m, gr)
        v = jax.tree_util.tree_map(lambda v_, g: b2 * v_ + (1 - b2) * g * g, v, gr)
        mh = jax.tree_util.tree_map(lambda m_: m_ / (1 - b1 ** t), m)
        vh = jax.tree_util.tree_map(lambda v_: v_ / (1 - b2 ** t), v)
        p = jax.tree_util.tree_map(lambda p_, mm, vv: p_ - lr * (mm / (jnp.sqrt(vv) + eps) + wd * p_ * (p_.ndim > 1)), p, mh, vh)
        return p, m, v, l, gn

    @jax.jit
    def ev_train(p, xb, yb):
        return ev_impl(p, xb, yb, False)

    @jax.jit
    def ev_snap(p, xb, yb):
        return ev_impl(p, xb, yb, True)

    @jax.jit
    def ev_snap_int(p, xb, yb):
        return ev_impl(p, xb, yb, "int")

    def ev(p, xb, yb, snap):
        if snap == "int":
            return ev_snap_int(p, xb, yb)
        return ev_snap(p, xb, yb) if snap else ev_train(p, xb, yb)

    def ev_impl(p, xb, yb, snap):
        logits = forward(p, xb, a.kind, snap)
        logz = jax.nn.logsumexp(logits, -1)
        tgt = jnp.take_along_axis(logits, yb[..., None], -1)[..., 0]
        return jnp.sum(logz - tgt)

    def evaluate(p, snap=False):
        n = (min(a.eval_bytes, len(valid) - 1) // a.eval_T) * a.eval_T
        xs = valid[:n].reshape(-1, a.eval_T); ys = valid[1:n + 1].reshape(-1, a.eval_T)
        tot = 0.0
        for i in range(0, xs.shape[0], 16):
            tot += float(ev(p, jnp.asarray(xs[i:i + 16], jnp.int32), jnp.asarray(ys[i:i + 16], jnp.int32), snap))
        return tot / n / math.log(2)

    log = []
    t0 = None
    for it in range(1, a.steps + 1):
        idx = rng.integers(0, len(train) - a.T - 1, size=a.B)
        xb = np.stack([train[i:i + a.T] for i in idx]); yb = np.stack([train[i + 1:i + a.T + 1] for i in idx])
        warm = min(1.0, it / 100)
        lr = a.lr * warm * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * it / a.steps)))
        p, m, v, l, gn = step(p, m, v, jnp.asarray(xb, jnp.int32), jnp.asarray(yb, jnp.int32), lr, it)
        if it == 5:
            jax.block_until_ready(l); t0 = time.time()
        if it % 100 == 0 or it == a.steps:
            log.append({"step": it, "train_bpb": float(l) / math.log(2), "gnorm": float(gn)})
            print(a.kind, it, round(float(l) / math.log(2), 4), flush=True)
    jax.block_until_ready(p)
    elapsed = time.time() - t0
    res = {"kind": a.kind, "params": nparam, "d": a.d, "layers": a.layers, "T": a.T, "B": a.B, "steps": a.steps,
           "seed": a.seed, "train_tokens": a.steps * a.B * a.T, "tokens_per_s": (a.steps - 5) * a.B * a.T / elapsed,
           "valid_bpb": evaluate(p), "log": log}
    if a.kind == "quat":
        res["valid_bpb_posthoc_2i_snap"] = evaluate(p, snap=True)
        res["valid_bpb_posthoc_int_snap"] = evaluate(p, snap="int")
    json.dump(res, open(a.out, "w"), indent=1)
    print(json.dumps({k: v for k, v in res.items() if k != "log"}))


if __name__ == "__main__":
    main()
