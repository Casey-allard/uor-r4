"""math2 E9: converting a trained dot-product read into a Lorentz (hyperbolic) read.

Teacher: the cycle-3 D8 Dot read (code, seed 3): exact queries/keys/NoRead logits rebuilt by the probe (8,192 queries).
Teacher read p = softmax([null, q.k/sqrt(r) + age]).
(1) Conversions without training (KL(teacher||student), mean over queries with >=16 candidates):
    naive   : lift q,k as they are; s = beta*(delta - arcosh(q0 k0 - q.k)) + age, beta/delta fitted (2 scalars)
    complete: 'norm completion' (hyperbolic analogue of Neyshabur-Srebro SIMPLE-ALSH): keys get an extra coordinate
              u_k = sqrt(C^2-1-|k|^2), queries a different one w_q = sqrt(Q^2-1-|q|^2), so q0=Q, k0=C for all rows and
              z = QC - q.k exactly; beta = QC/sqrt(r) (first-order match), delta fitted. Sweep QC.
(2) Precision: quantize log2(z) to m fractional bits (the table index) in the 'complete' conversion.
(3) Compression: learn P_q, P_k (64 -> k) so a k-dim student reproduces the teacher read; dot vs Lorentz, k = 4, 8, 16.
(4) RoPE: rotating the spatial block of lifted q, k by position-dependent rotations is a Lorentz isometry.
"""
import json, numpy as np, time
import jax, jax.numpy as jnp
from scipy.optimize import minimize

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
E = f"{S}/lab/exp/math2/e3"
name = "code_dot_s3"
P = json.load(open(f"{E}/{name}.params.json")); T, r = P["context"], P["read_width"]; n = P["dump"] * T
q = np.fromfile(f"{E}/{name}.query.f32", "<f4").reshape(n, r).astype(np.float64)
k = np.fromfile(f"{E}/{name}.key.f32", "<f4").reshape(n, r).astype(np.float64)
nl = np.fromfile(f"{E}/{name}.null.f32", "<f4").astype(np.float64)
age = np.array(P["age"])
out = {}

# padded batch of (query, candidates) with t >= 16
rows = [(w, t) for w in range(P["dump"]) for t in range(16, T)]
Qm = np.stack([q[w*T+t] for w, t in rows]); NL = np.array([nl[w*T+t] for w, t in rows])
Kc = np.zeros((len(rows), T - 1, r)); A = np.zeros((len(rows), T - 1)); M = np.zeros((len(rows), T - 1), bool)
for i, (w, t) in enumerate(rows):
    Kc[i, :t] = k[w*T:w*T+t]; A[i, :t] = age[t - 1 - np.arange(t)]; M[i, :t] = True
dot = np.einsum("nr,ncr->nc", Qm, Kc) / np.sqrt(r)


def read(scores):
    lg = np.concatenate([NL[:, None], np.where(M, scores + A, -np.inf)], 1)
    lg = lg - lg.max(1, keepdims=True); p = np.exp(lg); return p / p.sum(1, keepdims=True)


pt = read(dot)
ent = -np.sum(np.where(pt > 0, pt * np.log(pt + 1e-300), 0), 1)
spread = np.array([dot[i][M[i]].max() - np.median(dot[i][M[i]]) for i in range(len(rows))])
out["teacher"] = {"queries": len(rows), "mean_noread_mass": float(pt[:, 0].mean()), "mean_read_entropy_nats": float(ent.mean()),
                  "logit_max_minus_median_mean": float(spread.mean()), "logit_max_minus_median_p95": float(np.percentile(spread, 95)),
                  "query_norm_mean": float(np.linalg.norm(Qm, axis=1).mean()), "key_norm_mean": float(np.linalg.norm(k, axis=1).mean())}


def kl(ps):
    return float(np.mean(np.sum(np.where(pt > 0, pt * (np.log(pt + 1e-300) - np.log(ps + 1e-300)), 0), 1)))


# (1a) naive lift, fit beta, delta
q0 = np.sqrt(1 + (Qm**2).sum(1)); k0 = np.sqrt(1 + (Kc**2).sum(2))
dist_naive = np.arccosh(np.maximum(q0[:, None] * k0 - dot * np.sqrt(r), 1 + 1e-12))
f = lambda th: kl(read(np.exp(th[0]) * (th[1] - dist_naive)))
best = minimize(f, x0=[np.log(np.sinh(np.median(dist_naive)) / np.sqrt(r)), np.median(dist_naive)], method="Nelder-Mead",
                options={"maxiter": 300, "xatol": 1e-4, "fatol": 1e-7})
out["naive_lift_fitted"] = {"KL": float(best.fun), "beta": float(np.exp(best.x[0])), "delta": float(best.x[1])}
# first-order (cycle-3 Dot-matched) init without fitting, for reference
d0 = np.median(dist_naive)
out["naive_lift_dot_matched_init"] = {"KL": kl(read(np.sinh(d0) / np.sqrt(r) * (d0 - dist_naive))), "delta0": float(d0)}

# (1b) norm completion
qmax, kmax = np.linalg.norm(Qm, axis=1).max(), np.linalg.norm(k, axis=1).max()
comp = {}
for QC in (1e1, 1e2, 1e3, 1e4, 1e5):
    Qr = max(np.sqrt(1 + qmax**2), np.sqrt(QC)); C = QC / Qr
    if C < np.sqrt(1 + kmax**2):
        C = np.sqrt(1 + kmax**2); Qr = QC / C
    if Qr < np.sqrt(1 + qmax**2):
        comp[f"QC={QC:g}"] = "infeasible"; continue
    z = Qr * C - dot * np.sqrt(r)
    dist = np.arccosh(np.maximum(z, 1 + 1e-12)); beta = Qr * C / np.sqrt(r)
    g = lambda th: kl(read(beta * (th[0] - dist)))
    b2 = minimize(g, x0=[np.log(2 * Qr * C)], method="Nelder-Mead", options={"maxiter": 200, "xatol": 1e-6, "fatol": 1e-9})
    comp[f"QC={QC:g}"] = {"KL": float(b2.fun), "Q": float(Qr), "C": float(C), "radius_q": float(np.arccosh(Qr)), "radius_k": float(np.arccosh(C))}
    if QC == 1e3:
        # (2) precision of the table index: quantize log2(z) to m fractional bits
        prec = {}
        for mb in (2, 4, 6, 8, 10, 12):
            lz = np.log2(np.maximum(z, 1.0)); lzq = np.round(lz * 2**mb) / 2**mb
            dq = np.arccosh(np.maximum(2**lzq, 1 + 1e-12))
            prec[f"{mb}_fraction_bits"] = kl(read(beta * (b2.x[0] - dq)))
        prec["z_integer_bits_needed"] = float(np.ceil(np.log2(z.max())))
        out["precision_log2z_table_QC1e3"] = prec
out["norm_completion"] = comp

# (3) compression: k-dim students (dot vs Lorentz), fitted to the teacher read (train half / eval half of windows)
half = np.array([w < P["dump"] // 2 for w, t in rows])
jQ, jK, jA, jM, jNL, jP = map(jnp.asarray, (Qm, Kc, A, M, NL, pt))


def student(params, kind, idx):
    qs = jQ[idx] @ params["Pq"]; ks = jnp.einsum("ncr,rk->nck", jK[idx], params["Pk"])
    if kind == "dot":
        s = jnp.exp(params["lb"]) * jnp.einsum("nk,nck->nc", qs, ks)
    else:
        z = jnp.sqrt(1 + jnp.sum(qs**2, -1))[:, None] * jnp.sqrt(1 + jnp.sum(ks**2, -1)) - jnp.einsum("nk,nck->nc", qs, ks)
        s = jnp.exp(params["lb"]) * (params["delta"] - jnp.arccosh(jnp.maximum(z, 1 + 1e-6)))
    lg = jnp.concatenate([jNL[idx][:, None], jnp.where(jM[idx], s + jA[idx], -1e30)], 1)
    return jax.nn.log_softmax(lg, 1)


def loss(params, kind, idx):
    lp = student(params, kind, idx); pt_ = jP[idx]
    return jnp.mean(jnp.sum(jnp.where(pt_ > 0, pt_ * (jnp.log(pt_ + 1e-30) - lp), 0.0), 1))


tr_idx, ev_idx = jnp.asarray(np.where(half)[0]), jnp.asarray(np.where(~half)[0])
comp_res = {}
for kd in (4, 8, 16):
    for kind in ("dot", "lorentz"):
        key = jax.random.PRNGKey(kd)
        # init from the teacher's top principal directions of q and k (fair start for both kinds)
        Uq = np.linalg.svd(Qm[half], full_matrices=False)[2][:kd].T; Uk = np.linalg.svd(k, full_matrices=False)[2][:kd].T
        params = {"Pq": jnp.asarray(Uq), "Pk": jnp.asarray(Uk), "lb": jnp.array(np.log(1 / np.sqrt(r))),
                  "delta": jnp.array(3.0)}
        if kind == "lorentz":
            params["lb"] = jnp.array(0.0)
        opt_m = jax.tree_util.tree_map(jnp.zeros_like, params); opt_v = jax.tree_util.tree_map(jnp.zeros_like, params)
        vg = jax.jit(jax.value_and_grad(lambda p_, i_: loss(p_, kind, i_)))
        rngn = np.random.default_rng(0); t0 = time.time()
        for it in range(1, 1501):
            bi = tr_idx[rngn.integers(0, len(tr_idx), 512)]
            l, g = vg(params, bi)
            lr = 1e-2 * (0.1 + 0.9 * 0.5 * (1 + np.cos(np.pi * it / 1500)))
            opt_m = jax.tree_util.tree_map(lambda a_, b_: 0.9 * a_ + 0.1 * b_, opt_m, g)
            opt_v = jax.tree_util.tree_map(lambda a_, b_: 0.999 * a_ + 0.001 * b_ * b_, opt_v, g)
            params = jax.tree_util.tree_map(lambda p_, m_, v_: p_ - lr * (m_ / (1 - 0.9**it)) / (jnp.sqrt(v_ / (1 - 0.999**it)) + 1e-8), params, opt_m, opt_v)
        ev = float(jax.jit(lambda p_: loss(p_, kind, ev_idx))(params))
        comp_res[f"k{kd}_{kind}"] = {"eval_KL": ev, "train_s": round(time.time() - t0, 1)}
        print("compress", kd, kind, round(ev, 5), flush=True)
out["compression_KL_eval"] = comp_res

# (4) RoPE isometry check on lifted vectors (spatial block rotated, time coordinate untouched)
def rope(x, pos, base=10000.0):
    d = x.shape[-1]; fr = base ** (-np.arange(0, d, 2) / d); ang = pos * fr
    c, s = np.cos(ang), np.sin(ang); x1, x2 = x[..., 0::2], x[..., 1::2]
    y = np.empty_like(x); y[..., 0::2] = x1 * c - x2 * s; y[..., 1::2] = x1 * s + x2 * c; return y
qq, kk = Qm[:200], k[:200]
zq = lambda a, b: np.sqrt(1 + (a**2).sum(-1)) * np.sqrt(1 + (b**2).sum(-1)) - (a * b).sum(-1)
m_, n_ = 37, 91
err = np.abs(zq(rope(qq, m_), rope(kk, n_)) - zq(qq, rope(kk, n_ - m_))).max()
out["rope_relative_z_max_abs_err"] = float(err)
print(json.dumps(out, indent=1))
json.dump(out, open(f"{S}/lab/exp/math2/e9_convert.json", "w"), indent=1)
