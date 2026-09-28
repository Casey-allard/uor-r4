"""math2 E10: does a horocycle (hyperbolic) positional prior extrapolate in window length?

Derived: positions placed at unit spacing on a horocycle of H^2 have d(i,j) = 2 arcsinh(|i-j|/2); a Gibbs read
exp(-beta*d) is then ~ |i-j|^(-2 beta) at long lag (power law, the Lin-Tegmark hierarchical-MI form; the same family as
KERPLE-log) and ~ exp(-beta|i-j|) at short lag.
Test (copy-plus-bigram oracle of E2, code corpus): fit the recency kernel at W=128, then evaluate unchanged at W=512 and
W=2048 (length extrapolation), for
  free   : 7 free log2-distance bins (bins beyond the fitted range reuse the last fitted bin)
  expo   : -m * dist                         (ALiBi-like)
  horo   : -2 b * arcsinh(lam * dist / 2)    (horocycle distance; 2 parameters)
  plus the induction features and NoRead constant of E2 in every model (no scope features).
"""
import json, time, numpy as np
import jax, jax.numpy as jnp
from scipy.optimize import minimize

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
D = f"{S}/c3/data/code"
tr = np.fromfile(f"{D}/train.u16", dtype="<u2").astype(np.int64)
va = np.fromfile(f"{D}/valid.u16", dtype="<u2").astype(np.int64)
N, V = len(va), 4096
uni = np.bincount(tr, minlength=V).astype(np.float64) + 1.0; uni /= uni.sum()
big = np.zeros((V, V), np.float32); np.add.at(big, (tr[:-1], tr[1:]), 1.0); rowsum = big.sum(1).astype(np.float64)
alpha = 32.0
pbg = lambda prev, cur: (big[prev, cur].astype(np.float64) + alpha * uni[cur]) / (rowsum[prev] + alpha)
rng = np.random.default_rng(0)
Wmax, T = 2048, 12000
t_all = rng.choice(np.arange(Wmax + 2, N), size=T, replace=False)
fit_t, ev_t = t_all[: T // 2], t_all[T // 2:]


def build(t_idx):
    j = t_idx[:, None] - 1 - np.arange(Wmax)[None, :]
    dist = (t_idx[:, None] - j).astype(np.float32)
    ind = va[j - 1] == va[t_idx - 1][:, None]
    ind2 = ind & (va[j - 2] == va[t_idx - 2][:, None])
    corr = va[j] == va[t_idx][:, None]
    return dict(dist=dist, ind=ind, ind2=ind2, corr=corr, lpb=np.log(pbg(va[t_idx - 1], va[t_idx])))


FIT, EV = build(fit_t), build(ev_t)


def make(dat, W, kind):
    dist = jnp.asarray(dat["dist"][:, :W]); ind = jnp.asarray(dat["ind"][:, :W].astype(np.float32))
    ind2 = jnp.asarray(dat["ind2"][:, :W].astype(np.float32)); corr = jnp.asarray(dat["corr"][:, :W])
    lpb = jnp.asarray(dat["lpb"].astype(np.float32))
    bins = jnp.minimum(jnp.floor(jnp.log2(dist)).astype(jnp.int32), 6)

    def nll(th):
        w_ind, w_ind2, c0 = th[0], th[1], th[2]
        if kind == "free":
            rec = th[3:10][bins]
        elif kind == "expo":
            rec = -jnp.exp(th[3]) * dist
        else:
            rec = -2 * jnp.exp(th[3]) * jnp.arcsinh(jnp.exp(th[4]) * dist / 2)
        f = rec + w_ind * ind + w_ind2 * ind2
        lz = jnp.logaddexp(c0, jax.nn.logsumexp(f, 1))
        lnum = jnp.logaddexp(c0 + lpb, jax.nn.logsumexp(jnp.where(corr, f, -1e30), 1))
        return jnp.mean(lz - lnum)
    return jax.jit(jax.value_and_grad(nll)), jax.jit(nll)


npar = {"free": 10, "expo": 4, "horo": 5}
out = {}
for kind in ("free", "expo", "horo"):
    vg, _ = make(FIT, 128, kind)
    fun = lambda th: tuple(np.asarray(x, np.float64) for x in vg(jnp.asarray(th, jnp.float32)))
    th0 = np.zeros(npar[kind]); th0[0] = 5.0
    if kind == "expo": th0[3] = np.log(0.02)
    if kind == "horo": th0[3], th0[4] = np.log(0.5), np.log(0.2)
    r = minimize(fun, th0, jac=True, method="L-BFGS-B", options={"maxiter": 400})
    res = {"fit_nll_W128": float(r.fun), "theta": [round(float(x), 4) for x in r.x]}
    for W in (128, 512, 2048):
        _, ev = make(EV, W, kind)
        res[f"eval_W{W}"] = float(ev(jnp.asarray(r.x, jnp.float32)))
    out[kind] = res
    print(kind, json.dumps(res), flush=True)
json.dump(out, open(f"{S}/lab/exp/math2/e10_horocycle.json", "w"), indent=1)
