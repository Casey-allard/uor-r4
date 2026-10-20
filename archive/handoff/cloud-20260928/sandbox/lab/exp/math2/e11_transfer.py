"""math2 E11: can a dot-product head converted to a Lorentz head exceed its teacher on a real hierarchy?
And is the hyperbolic advantage on the code-scope tree curvature, or just a learned per-key potential?

Task: cycle-2 code-scope tree (this repository's Rust scopes), deepest-stored-ancestor retrieval, retrieval head of
cycle 1 (q = Wq E[leaf], k_j = Wk E[tok_{j-1}], value = E[tok_j]), dk = 64, non-root accuracy at 48/96/192 stored.
Arms (seed 0, 1500 steps each, same batches):
  A  dot from scratch                                   (cycle-2 result: dot fails at any width)
  B  dot + learned per-key potential b_j = w_b . E[tok_{j-1}]   (from scratch)
  D  convert A exactly to a norm-completed Lorentz head (q0 = Q, k0_j = C e^{rho_j}, z = q0 k0 - q.k,
     score = beta(delta - arcosh z), beta = QC/sqrt(dk), delta = ln 2QC, rho = w_rho . E[tok_{j-1}] init 0),
     then fine-tune with free radii rho_j and learnable Q, C, beta, delta
  E  continue A as dot (control for D)
  F  continue A as dot + per-key potential (control for D: potential without curvature)
Derived: arcosh(QC e^rho - x) ~ ln(2QC) + rho - x/(QC e^rho): in the near-dot regime the radius IS a per-key potential
(plus a per-key temperature e^-rho), so F is the right control for D.
"""
import importlib.util, json, math, sys, time
import numpy as np
import jax, jax.numpy as jnp

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
spec = importlib.util.spec_from_file_location("geo", f"{S}/exp/lead/geoattn.py"); geo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(geo)
geo.TREE = geo.tree_from_parent(np.load(f"{S}/exp/lead/data/code_tree.npz")["parent"])
K = len(geo.TREE[0]); V = 512; Q = 32; DK = 64; D_MODEL = 128; pad = K + V; vocab = K + V + 1
STEPS = int(sys.argv[1]) if len(sys.argv) > 1 else 1500


def parts(p, toks):
    prev = jnp.concatenate([jnp.full((toks.shape[0], 1), pad, toks.dtype), toks[:, :-1]], 1)
    x, s = p["E"][toks], p["E"][prev]
    return x, s, x[:, -Q:] @ p["Wq"], s @ p["Wk"]


def scores(p, s, q, k, kind):
    dot = q @ jnp.swapaxes(k, -1, -2)                                     # (B,Q,L)
    if kind == "dot":
        return dot / math.sqrt(DK)
    if kind == "dotb":
        return dot / math.sqrt(DK) + (s @ p["wb"])[:, None, :]
    # norm-completed Lorentz with per-key radial offsets
    Qt = jnp.exp(p["logQ"]); C = jnp.exp(p["logC"])
    qn2 = jnp.sum(q * q, -1, keepdims=True); kn2 = jnp.sum(k * k, -1)
    q0 = jnp.maximum(Qt, jnp.sqrt(1 + qn2))                               # completion keeps q0 = Q when valid
    k0 = jnp.maximum(C * jnp.exp(s @ p["wr"]), jnp.sqrt(1 + kn2))[:, None, :]
    z = jnp.maximum(q0 * k0 - dot, 1 + 1e-6)
    return jnp.exp(p["lb"]) * (p["delta"] - jnp.arccosh(z))


def forward(p, toks, kind):
    x, s, q, k = parts(p, toks)
    L = toks.shape[1]
    sc = scores(p, s, q, k, kind)
    causal = (jnp.arange(L)[None, :] < (L - Q + jnp.arange(Q))[:, None])[None]
    w = jax.nn.softmax(jnp.where(causal, sc, -1e30), -1)
    Evals = p["E"][K:K + V]
    return jnp.exp(p["logout"]) * ((w @ x) @ Evals.T) / x.shape[-1]


def loss_fn(p, toks, tgt, kind):
    lg = forward(p, toks, kind)
    return jnp.mean(jax.nn.logsumexp(lg, -1) - jnp.take_along_axis(lg, tgt[..., None], -1)[..., 0])


def train(p, kind, steps, seed):
    m = jax.tree_util.tree_map(jnp.zeros_like, p); v2 = jax.tree_util.tree_map(jnp.zeros_like, p)

    @jax.jit
    def step(p, m, v2, toks, tgt, lr, t):
        l, gr = jax.value_and_grad(loss_fn)(p, toks, tgt, kind)
        gn = jnp.sqrt(sum(jnp.sum(x * x) for x in jax.tree_util.tree_leaves(gr)))
        gr = jax.tree_util.tree_map(lambda x: x * jnp.minimum(1.0, 1.0 / (gn + 1e-6)), gr)
        m = jax.tree_util.tree_map(lambda a_, g: 0.9 * a_ + 0.1 * g, m, gr)
        v2 = jax.tree_util.tree_map(lambda a_, g: 0.99 * a_ + 0.01 * g * g, v2, gr)
        p = jax.tree_util.tree_map(lambda p_, mm, vv: p_ - lr * (mm / (1 - 0.9 ** t)) / (jnp.sqrt(vv / (1 - 0.99 ** t)) + 1e-8), p, m, v2)
        return p, m, v2, l
    rng = np.random.default_rng(seed)
    for it in range(1, steps + 1):
        toks, tgt, _ = geo.make_tree_batch(rng, 64, 48, Q, V)
        lr = 3e-3 * min(1.0, it / 100) * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * it / steps)))
        p, m, v2, l = step(p, m, v2, jnp.asarray(toks), jnp.asarray(tgt), lr, it)
    return p


def evaluate(p, kind):
    fwd = jax.jit(lambda p_, t_: forward(p_, t_, kind))
    erng = np.random.default_rng(12345); ev = {}
    for Dn in (48, 96, 192):
        hit = tot = 0; allhit = alltot = 0
        for _ in range(8):
            toks, tgt, ans = geo.make_tree_batch(erng, 32, Dn, Q, V)
            pred = np.asarray(jnp.argmax(fwd(p, jnp.asarray(toks)), -1)); ok = pred == tgt
            root_pos = 2 * np.argmax(toks[:, 0:2 * Dn:2] == 0, axis=1) + 1
            nr = ans != root_pos[:, None]
            hit += int(ok[nr].sum()); tot += int(nr.sum()); allhit += int(ok.sum()); alltot += ok.size
        ev[f"D{Dn}"] = {"nonroot_acc": hit / max(tot, 1), "acc": allhit / alltot}
    return ev


def base_params(seed):
    ks = iter(jax.random.split(jax.random.PRNGKey(seed), 8))
    lin = lambda i, o: jax.random.normal(next(ks), (i, o)) / math.sqrt(i)
    return {"E": jax.random.normal(next(ks), (vocab, D_MODEL)) / math.sqrt(D_MODEL), "Wq": lin(D_MODEL, DK),
            "Wk": lin(D_MODEL, DK), "logout": jnp.array(math.log(10.0 * D_MODEL))}


out = {}; t0 = time.time()
pA = train(base_params(0), "dot", STEPS, 0); out["A_dot_scratch"] = evaluate(pA, "dot"); print("A", out["A_dot_scratch"], round(time.time() - t0), flush=True)
pB = dict(base_params(0), wb=jnp.zeros(D_MODEL)); pB = train(pB, "dotb", STEPS, 0)
out["B_dot_potential_scratch"] = evaluate(pB, "dotb"); print("B", out["B_dot_potential_scratch"], round(time.time() - t0), flush=True)
# exact conversion of A
QC = 1e4; Qt = C = math.sqrt(QC)
pD = dict(pA, logQ=jnp.array(math.log(Qt)), logC=jnp.array(math.log(C)), wr=jnp.zeros(D_MODEL),
          lb=jnp.array(math.log(QC / math.sqrt(DK))), delta=jnp.array(math.log(2 * QC)))
out["D0_converted_before_finetune"] = evaluate(pD, "lor"); print("D0", out["D0_converted_before_finetune"], flush=True)
pD = train(pD, "lor", STEPS, 1); out["D_converted_lorentz_finetuned"] = evaluate(pD, "lor")
out["D_final_scalars"] = {k: float(pD[k]) for k in ("logQ", "logC", "lb", "delta")}
rho = np.asarray(pD["E"][:K] @ pD["wr"]); lvl = geo.TREE[1]
out["D_rho_vs_depth_spearman"] = float(np.corrcoef(np.argsort(np.argsort(rho)), np.argsort(np.argsort(lvl)))[0, 1])
print("D", out["D_converted_lorentz_finetuned"], out["D_final_scalars"], out["D_rho_vs_depth_spearman"], round(time.time() - t0), flush=True)
pE = train(dict(pA), "dot", STEPS, 1); out["E_dot_continued"] = evaluate(pE, "dot"); print("E", out["E_dot_continued"], flush=True)
pF = train(dict(pA, wb=jnp.zeros(D_MODEL)), "dotb", STEPS, 1); out["F_dot_potential_continued"] = evaluate(pF, "dotb")
print("F", out["F_dot_potential_continued"], round(time.time() - t0), flush=True)
json.dump(out, open(f"{S}/lab/exp/math2/e11_transfer.json", "w"), indent=1)
