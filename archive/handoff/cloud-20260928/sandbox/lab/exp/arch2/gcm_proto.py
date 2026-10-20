"""GCM prototype (scratch, JAX, 1 CPU thread).

Purpose: (1) check that a parallel associative-scan form of the geometric
recurrence (diagonal lanes + quaternion transport lanes with per-lane radial
decay) equals the sequential recurrence; (2) measure training throughput of a
GCM-S-shaped stack (12 layers: 9 recurrent + 3 read, ReLU^2 MLPs, tied 4096
vocab) on one thread, for comparison with the D8 learner's sequential unroll.
Optional (3): a short training run on WikiText-2 u16 tokens.
"""
import os, sys, time, json, math
os.environ.setdefault("XLA_FLAGS", "--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1")
import numpy as np
import jax, jax.numpy as jnp

jax.config.update("jax_enable_x64", False)

# ---------------------------------------------------------------- recurrence
def qmat(q):
    """Left-multiplication matrix L(q) for unit quaternion q=(w,a,b,c): L(q) x = q (x) x."""
    w, a, b, c = q[..., 0], q[..., 1], q[..., 2], q[..., 3]
    row0 = jnp.stack([w, -a, -b, -c], -1)
    row1 = jnp.stack([a, w, -c, b], -1)
    row2 = jnp.stack([b, c, w, -a], -1)
    row3 = jnp.stack([c, -b, a, w], -1)
    return jnp.stack([row0, row1, row2, row3], -2)  # [...,4,4]

def georecur_parallel(f_diag, b_diag, r_lane, q_lane, b_lane):
    """h_t = f_t*h_{t-1} + b_t (diagonal); h_t = r_t L(q_t) h_{t-1} + b_t (lanes).
    Shapes: f_diag,b_diag [B,T,Dd]; r_lane [B,T,Q]; q_lane [B,T,Q,4] unit; b_lane [B,T,Q,4]."""
    def comb_d(e1, e2):
        a1, x1 = e1; a2, x2 = e2
        return a2 * a1, a2 * x1 + x2
    _, h_d = jax.lax.associative_scan(comb_d, (f_diag, b_diag), axis=1)
    A = r_lane[..., None, None] * qmat(q_lane)          # [B,T,Q,4,4]
    def comb_l(e1, e2):
        A1, x1 = e1; A2, x2 = e2
        return A2 @ A1, jnp.einsum('...ij,...j->...i', A2, x1) + x2
    _, h_l = jax.lax.associative_scan(comb_l, (A, b_lane), axis=1)
    return h_d, h_l

def georecur_sequential(f_diag, b_diag, r_lane, q_lane, b_lane):
    B, T, Dd = f_diag.shape; Q = r_lane.shape[2]
    def step(carry, inp):
        hd, hl = carry
        fd, bd, r, q, bl = inp
        hd = fd * hd + bd
        hl = r[..., None] * jnp.einsum('bqij,bqj->bqi', qmat(q), hl) + bl
        return (hd, hl), (hd, hl)
    xs = tuple(jnp.moveaxis(a, 1, 0) for a in (f_diag, b_diag, r_lane, q_lane, b_lane))
    _, (hd, hl) = jax.lax.scan(step, (jnp.zeros((B, Dd)), jnp.zeros((B, Q, 4))), xs)
    return jnp.moveaxis(hd, 0, 1), jnp.moveaxis(hl, 0, 1)

# ---------------------------------------------------------------- model
def rms(x, g):
    return x * jax.lax.rsqrt(jnp.mean(x * x, -1, keepdims=True) + 1e-5) * g

def init_params(key, V, d, L, read_layers, H, Hkv, r=64, conv=4, mlp=4):
    ks = iter(jax.random.split(key, 16 * L + 8))
    def w(i, o, s=None):
        s = s if s is not None else 1.0 / math.sqrt(i)
        return jax.random.normal(next(ks), (i, o), jnp.float32) * s
    nq = d // 16                     # transport lanes (1/4 of channels)
    dd = d - 4 * nq                  # diagonal channels
    P = {"emb": jax.random.normal(next(ks), (V, d)) * 0.02, "g_out": jnp.ones(d), "layers": []}
    # retention init: log-spaced time constants 2..256 (like D8's 4..64 lanes)
    tau_d = jnp.exp(jnp.linspace(math.log(2), math.log(256), dd))
    tau_l = jnp.exp(jnp.linspace(math.log(2), math.log(256), nq))
    for i in range(L):
        lp = {"g1": jnp.ones(d), "g2": jnp.ones(d),
              "up": w(d, mlp * d), "down": w(mlp * d, d, 1.0 / math.sqrt(mlp * d) / math.sqrt(2 * L))}
        if i in read_layers:
            lp.update(kind=None, wq=w(d, H * r), wk=w(d, Hkv * r), wv=w(d, Hkv * r),
                      wo=w(H * r, d, 1.0 / math.sqrt(H * r) / math.sqrt(2 * L)), wnull=w(d, H, 0.0))
        else:
            lp.update(conv=jax.random.normal(next(ks), (conv, d)) * 0.1 + jnp.array([0, 0, 0, 1.0])[:, None],
                      wc=w(d, d), wf=w(d, dd + nq), wq4=w(d, 4 * nq), wg=w(d, d),
                      bf=jnp.concatenate([jnp.log(tau_d - 1.0), jnp.log(tau_l - 1.0)]),
                      wo=w(d, d, 1.0 / math.sqrt(d) / math.sqrt(2 * L)))
        P["layers"].append(lp)
    return P

def recurrent_mix(lp, x, d):
    B, T, _ = x.shape; nq = d // 16; dd = d - 4 * nq
    xp = jnp.pad(x, ((0, 0), (lp["conv"].shape[0] - 1, 0), (0, 0)))
    xc = sum(lp["conv"][k] * xp[:, k:k + T] for k in range(lp["conv"].shape[0]))   # causal depthwise conv
    c = xc @ lp["wc"]
    f = jax.nn.sigmoid(xc @ lp["wf"] + lp["bf"])            # retention (1 - 1/tau at init)
    q = xc @ lp["wq4"] + jnp.tile(jnp.array([4.0, 0, 0, 0]), nq)   # identity-biased, free 4-D (not e0+0.1*raw)
    q = q.reshape(B, T, nq, 4); q = q / jnp.linalg.norm(q, axis=-1, keepdims=True)
    fd, fl = f[..., :dd], f[..., dd:]
    cd, cl = c[..., :dd], c[..., dd:].reshape(B, T, nq, 4)
    hd, hl = georecur_parallel(fd, (1 - fd) * cd, fl, q, (1 - fl)[..., None] * cl)
    h = jnp.concatenate([hd, hl.reshape(B, T, 4 * nq)], -1)
    return (h * jax.nn.silu(xc @ lp["wg"])) @ lp["wo"]

def read_mix(lp, x, H, Hkv, r=64):
    B, T, d = x.shape
    q = (x @ lp["wq"]).reshape(B, T, H, r)
    k = (x @ lp["wk"]).reshape(B, T, Hkv, r)
    v = (x @ lp["wv"]).reshape(B, T, Hkv, r)
    rep = H // Hkv
    k = jnp.repeat(k, rep, axis=2); v = jnp.repeat(v, rep, axis=2)
    s = jnp.einsum('bthr,bshr->bhts', q, k) / math.sqrt(r)
    mask = jnp.tril(jnp.ones((T, T), bool), -1)            # strictly earlier events (D8 write order)
    s = jnp.where(mask[None, None], s, -1e30)
    null = (x @ lp["wnull"]).transpose(0, 2, 1)[..., None]  # NoRead slot [B,H,T,1]
    a = jax.nn.softmax(jnp.concatenate([null, s], -1), -1)[..., 1:]
    o = jnp.einsum('bhts,bshr->bthr', a, v).reshape(B, T, H * r)
    return o @ lp["wo"]

def forward(P, ids, cfg):
    d, H, Hkv = cfg["d"], cfg["H"], cfg["Hkv"]
    x = P["emb"][ids]
    for i, lp in enumerate(P["layers"]):
        y = rms(x, lp["g1"])
        x = x + (read_mix(lp, y, H, Hkv) if i in cfg["read_layers"] else recurrent_mix(lp, y, d))
        y = rms(x, lp["g2"])
        x = x + (jnp.square(jax.nn.relu(y @ lp["up"]))) @ lp["down"]
    return rms(x, P["g_out"]) @ P["emb"].T

def loss_fn(P, ids, cfg):
    logits = forward(P, ids[:, :-1], cfg)
    lp = jax.nn.log_softmax(logits, -1)
    return -jnp.mean(jnp.take_along_axis(lp, ids[:, 1:, None], -1))

def count(P):
    return sum(int(np.prod(a.shape)) for a in jax.tree_util.tree_leaves(P) if hasattr(a, "shape"))

# ---------------------------------------------------------------- main
if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "check":
        key = jax.random.PRNGKey(0)
        B, T, Dd, Q = 3, 257, 40, 6
        k = jax.random.split(key, 5)
        f = jax.nn.sigmoid(jax.random.normal(k[0], (B, T, Dd)) + 2)
        bd = jax.random.normal(k[1], (B, T, Dd))
        r = jax.nn.sigmoid(jax.random.normal(k[2], (B, T, Q)) + 2)
        q = jax.random.normal(k[3], (B, T, Q, 4)); q = q / jnp.linalg.norm(q, axis=-1, keepdims=True)
        bl = jax.random.normal(k[4], (B, T, Q, 4))
        p = georecur_parallel(f, bd, r, q, bl); s = georecur_sequential(f, bd, r, q, bl)
        err = [float(jnp.max(jnp.abs(a - b)) / jnp.max(jnp.abs(b))) for a, b in zip(p, s)]
        print(json.dumps({"rel_err_diag": err[0], "rel_err_lane": err[1], "T": T}))
    elif mode == "speed":
        d, L, V = int(sys.argv[2]), int(sys.argv[3]), 4096
        B, T = int(sys.argv[4]), int(sys.argv[5])
        reads = [int(t) for t in sys.argv[6].split(",")] if len(sys.argv) > 6 else []
        cfg = {"d": d, "H": d // 64, "Hkv": max(1, d // 256), "read_layers": reads}
        P = init_params(jax.random.PRNGKey(1), V, d, L, reads, cfg["H"], cfg["Hkv"])
        n = count(P)
        ids = jax.random.randint(jax.random.PRNGKey(2), (B, T + 1), 0, V)
        g = jax.jit(jax.value_and_grad(lambda P, ids: loss_fn(P, ids, cfg)))
        t0 = time.time(); l, gr = g(P, ids); jax.block_until_ready(gr); tc = time.time() - t0
        times = []
        for _ in range(4):
            t0 = time.time(); l, gr = g(P, ids); jax.block_until_ready(gr); times.append(time.time() - t0)
        tt = float(np.median(times))
        print(json.dumps({"d": d, "L": L, "reads": reads, "params": n, "nonemb": n - V * d,
                          "B": B, "T": T, "compile_s": round(tc, 1), "step_s": round(tt, 3),
                          "tok_per_s": round(B * T / tt, 1),
                          "gflops_6N": round(6 * n * B * T / tt / 1e9, 2), "loss0": float(l)}))
