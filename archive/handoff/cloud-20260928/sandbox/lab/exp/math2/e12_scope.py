"""math2 E1/E2: how much scope hierarchy does next-token retrieval on code actually need, by window length?

E1 (statistics): for each window length W, the scope relation of every in-window candidate position j to the target t:
    same scope / enclosing (ancestor) scope / closed child scope (descendant) / other (sibling branch, e.g. an
    earlier function), plus depth spread within windows.
E2 (oracle value): a fitted copy-plus-bigram predictor (the shape of the D8 read: a NoRead slot with the bigram
    background competing with a softmax over in-window positions that copies x_j), with position features
      recency (log2 distance bins), induction match (x_{j-1}==x_{t-1}), 2nd-order match,
    with and without ground-truth scope-relation features (and scope x induction interactions).
    The NLL gain from adding scope features at window W measures the value of hierarchy-aware retrieval in this data
    (an oracle: the scope relation is given exactly, as a perfectly hierarchical key geometry could supply it).
Data: $S/c3/data/code (this repo's Rust, 4096 BPE); depth from valid_depth.npy (cycle-3 lexer).
"""
import sys, time, json
import numpy as np
import jax, jax.numpy as jnp
from scipy.optimize import minimize

jax.config.update("jax_enable_x64", False)
S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
D = f"{S}/c3/data/code"
tr = np.fromfile(f"{D}/train.u16", dtype="<u2").astype(np.int64)
va = np.fromfile(f"{D}/valid.u16", dtype="<u2").astype(np.int64)
depth = np.load(f"{D}/valid_depth.npy").astype(np.int64)
N, V = len(va), 4096
out = {}

# ---------- scopes from the depth sequence (depth = unmatched '{' before the token) ----------
scope_of = np.zeros(N, np.int64)
start, end, sdepth, parent = [0], [N], [0], [-1]
stack = [0]
for t in range(N):
    if t > 0 and depth[t] > depth[t - 1]:          # token t-1 opened a scope; it starts at t
        sid = len(start); start.append(t); end.append(N); sdepth.append(depth[t]); parent.append(stack[-1]); stack.append(sid)
    elif t > 0 and depth[t] < depth[t - 1]:        # token t-1 closed the innermost scope
        if len(stack) > 1:
            end[stack.pop()] = t
    scope_of[t] = stack[-1]
start, end = np.array(start), np.array(end)
print("scopes", len(start), "max depth", depth.max(), flush=True)


def relation(t_idx, W):
    """[len(t_idx), W] scope relation of candidate j=t-1-i (i=0..W-1) to target t: 0 same, 1 anc, 2 desc, 3 other."""
    j = t_idx[:, None] - 1 - np.arange(W)[None, :]
    st, sj = scope_of[t_idx][:, None], scope_of[j]
    same = sj == st
    anc = (~same) & (start[sj] <= start[st]) & (end[sj] >= end[st])
    desc = (~same) & (start[sj] >= start[st]) & (end[sj] <= end[st])
    rel = np.full(j.shape, 3, np.int8)
    rel[desc] = 2; rel[anc] = 1; rel[same] = 0
    return rel, j


# ---------- E1: statistics ----------
rng = np.random.default_rng(0)
stats = {}
for W in (128, 256, 512, 2048, 8192):
    t_idx = rng.integers(W + 1, N, size=4000)
    rel, j = relation(t_idx, W)
    frac = [float((rel == c).mean()) for c in range(4)]
    dwin = depth[j]
    stats[W] = {"frac_same": frac[0], "frac_anc": frac[1], "frac_desc": frac[2], "frac_other": frac[3],
                "mean_depth_std_in_window": float(dwin.std(1).mean()),
                "mean_distinct_scopes": float(np.mean([len(np.unique(scope_of[jj])) for jj in j[:500]]))}
    print("E1", W, json.dumps({k: round(v, 3) for k, v in stats[W].items()}), flush=True)
stats["global_depth_std"] = float(depth.std())
out["E1"] = stats

# ---------- bigram background from the training split ----------
alpha = None
uni = np.bincount(tr, minlength=V).astype(np.float64) + 1.0; uni /= uni.sum()
big = np.zeros((V, V), np.float32)
np.add.at(big, (tr[:-1], tr[1:]), 1.0)
rowsum = big.sum(1).astype(np.float64)


def p_bg(prev, cur, a):
    return (big[prev, cur].astype(np.float64) + a * uni[cur]) / (rowsum[prev] + a)


tt = rng.integers(2050, N, size=20000)
best = min((float(-np.log(p_bg(va[tt - 1], va[tt], a)).mean()), a) for a in (0.5, 1, 2, 4, 8, 16, 32))
alpha = best[1]
print("bigram NLL", best, flush=True)

# ---------- E2: oracle copy model ----------
Wmax = 2048
T = 12000
t_all = rng.choice(np.arange(Wmax + 2, N), size=T, replace=False)
fit_t, ev_t = t_all[: T // 2], t_all[T // 2:]


def build(t_idx):
    rel, j = relation(t_idx, Wmax)
    dist = t_idx[:, None] - j
    rbin = np.floor(np.log2(dist)).astype(np.int8)                     # 0..10 for dist<=2048
    ind = va[j - 1] == va[t_idx - 1][:, None]
    ind2 = ind & (va[j - 2] == va[t_idx - 2][:, None])
    corr = va[j] == va[t_idx][:, None]
    pb = p_bg(va[t_idx - 1], va[t_idx], alpha)
    return dict(rel=rel, rbin=rbin, ind=ind, ind2=ind2, corr=corr, pb=pb)


t0 = time.time()
FIT, EV = build(fit_t), build(ev_t)
print("built", time.time() - t0, flush=True)
NB = 11


def make_loss(dat, W, use_scope):
    rel = jnp.asarray(dat["rel"][:, :W].astype(np.int32)); rb = jnp.asarray(dat["rbin"][:, :W].astype(np.int32))
    ind = jnp.asarray(dat["ind"][:, :W].astype(np.float32)); ind2 = jnp.asarray(dat["ind2"][:, :W].astype(np.float32))
    corr = jnp.asarray(dat["corr"][:, :W]); lpb = jnp.asarray(np.log(dat["pb"]).astype(np.float32))

    def nll(theta):
        w_rec = theta[:NB]; w_ind, w_ind2, c0 = theta[NB], theta[NB + 1], theta[NB + 2]
        f = w_rec[rb] + w_ind * ind + w_ind2 * ind2
        if use_scope:
            ws = jnp.concatenate([jnp.zeros(1), theta[NB + 3:NB + 6]])       # same scope = reference 0
            wsi = jnp.concatenate([jnp.zeros(1), theta[NB + 6:NB + 9]])      # scope x induction interaction
            f = f + ws[rel] + wsi[rel] * ind
        lz = jnp.logaddexp(c0, jax.nn.logsumexp(f, 1))
        lnum = jnp.logaddexp(c0 + lpb, jax.nn.logsumexp(jnp.where(corr, f, -1e30), 1))
        return jnp.mean(lz - lnum)
    return jax.jit(jax.value_and_grad(nll)), jax.jit(nll)


res = {}
for W in (128, 512, 2048):
    for use_scope in (False, True):
        npar = NB + 3 + (6 if use_scope else 0)
        vg, _ = make_loss(FIT, W, use_scope)
        _, ev = make_loss(EV, W, use_scope)
        fun = lambda th: tuple(np.asarray(x, np.float64) for x in vg(jnp.asarray(th, jnp.float32)))
        th0 = np.zeros(npar); th0[NB] = 2.0
        r = minimize(fun, th0, jac=True, method="L-BFGS-B", options={"maxiter": 300})
        e = float(ev(jnp.asarray(r.x, jnp.float32)))
        key = f"W{W}_{'scope' if use_scope else 'noscope'}"
        res[key] = {"fit_nll": float(r.fun), "eval_nll": e, "theta": [round(float(x), 3) for x in r.x]}
        print("E2", key, round(float(r.fun), 4), round(e, 4), "iters", r.nit, round(time.time() - t0, 1), flush=True)
res["bigram_eval_nll"] = float(-np.log(EV["pb"]).mean())
res["copyable_frac"] = {W: float(EV["corr"][:, :W].any(1).mean()) for W in (128, 512, 2048)}
out["E2"] = res
out["alpha"] = alpha
json.dump(out, open(f"{S}/lab/exp/math2/e12_scope.json", "w"), indent=1)
print(json.dumps({k: (v["eval_nll"] if isinstance(v, dict) and "eval_nll" in v else v) for k, v in res.items()}))
