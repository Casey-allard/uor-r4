"""redteam2_sci R3: exp2's content-cell admission on REAL learned (q, k) pairs instead of near-copy synthetic queries.

exp2 measured >=99.8% admission at 2.8% scored with queries that are near-copies of their keys (cos 0.995 / 0.999996).
This is exp2's own top-ranked falsifier (P1), not yet run: dump real query/key pairs from the D8 read and index them.
Data: math2's dumps of the saved cycle-3 D8 models (64 windows x 128 positions; read width 64), code validation split.
Store = all 8,192 keys of one model (pooled over windows: a MIPS/geometry test, not causal attention; the learned age
bias and NoRead slot are left out). Queries = 1,024 random positions (Q->K), control = 1,024 keys used as queries (K->K).
Target = exact argmax of the model's own score over the store; also the softmax mass (model's own scale) captured.
Index = k-means on keys (128 cells, ~64 keys/cell); cells ranked by the model's score against the centroid (Lorentz:
centroid lifted like a key) or by the L2 rule <q,c> - |c|^2/2 (exp2's best for dot), probed until >= f*N keys admitted.
"""
import json, math, sys
import numpy as np

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
E = f"{S}/lab/exp/math2/e3"
rng = np.random.default_rng(0)
FRACS = (0.01, 0.02, 0.05, 0.10, 0.25)


def load(name):
    P = json.load(open(f"{E}/{name}.params.json"))
    n = P["dump"] * P["context"]; r = P["read_width"]
    q = np.fromfile(f"{E}/{name}.query.f32", "<f4").reshape(n, r).astype(np.float64)
    k = np.fromfile(f"{E}/{name}.key.f32", "<f4").reshape(n, r).astype(np.float64)
    return P, q, k


def score_fn(P, r):
    if P["geometry"] == "Lorentz":
        beta, off = P["beta"], P["offset"]
        def f(Q, K):
            z = np.sqrt(1 + (Q * Q).sum(1))[:, None] * np.sqrt(1 + (K * K).sum(1))[None, :] - Q @ K.T
            return beta * (off - np.arccosh(np.maximum(z, 1.0 + 1e-12)))
    else:
        def f(Q, K):
            return Q @ K.T / math.sqrt(r)
    return f


def kmeans(X, C, iters=25):
    cent = X[rng.choice(len(X), C, replace=False)].copy()
    for _ in range(iters):
        d2 = (X * X).sum(1)[:, None] - 2 * X @ cent.T + (cent * cent).sum(1)[None, :]
        a = d2.argmin(1)
        for c in range(C):
            m = a == c
            if m.any(): cent[c] = X[m].mean(0)
    d2 = (X * X).sum(1)[:, None] - 2 * X @ cent.T + (cent * cent).sum(1)[None, :]
    return cent, d2.argmin(1)


def admission(Q, K, f, cent, assign, route):
    N = len(K)
    full = f(Q, K)                                   # (nq, N) model scores
    top1 = full.argmax(1)
    pm = np.exp(full - full.max(1, keepdims=True)); pm /= pm.sum(1, keepdims=True)
    csc = route(Q, cent)                             # (nq, C) cell ranking scores
    order = np.argsort(-csc, 1)
    sizes = np.bincount(assign, minlength=len(cent))
    res = {}
    for frac in FRACS:
        need = frac * N; rec = mass = scored = 0.0
        for i in range(len(Q)):
            cum = np.cumsum(sizes[order[i]])
            ncell = int(np.searchsorted(cum, need) + 1)
            cells = order[i][:ncell]
            adm = np.isin(assign, cells)
            rec += float(adm[top1[i]]); mass += float(pm[i][adm].sum()); scored += float(adm.sum())
        nq = len(Q)
        res[f"{frac:.2f}"] = {"top1_recall": round(rec / nq, 4), "mass_captured": round(mass / nq, 4),
                              "scored_frac": round(scored / nq / N, 4)}
    ent = float((-(pm * np.log(pm + 1e-300)).sum(1)).mean())
    return res, ent, float(pm.max(1).mean())


out = {}
for name in ("code_dot_s3", "code_lorentzflat_s3", "code_dot_s2", "code_lorentzflat_s2"):
    P, q, k = load(name)
    r = P["read_width"]; f = score_fn(P, r)
    cent, assign = kmeans(k, 128)
    qi = rng.choice(len(q), 1024, replace=False); ki = rng.choice(len(k), 1024, replace=False)
    Q, KQ = q[qi], k[ki]
    # geometry of the gap: cosine of each query to its top-1 key, vs a key to its nearest other key
    s = f(Q, k); t1 = s.argmax(1)
    cosq = np.einsum("ij,ij->i", Q, k[t1]) / (np.linalg.norm(Q, axis=1) * np.linalg.norm(k[t1], axis=1))
    kk = f(KQ, k); kk[np.arange(1024), ki] = -np.inf; t2 = kk.argmax(1)
    cosk = np.einsum("ij,ij->i", KQ, k[t2]) / (np.linalg.norm(KQ, axis=1) * np.linalg.norm(k[t2], axis=1))
    res = {"geometry": P["geometry"], "q_norm": round(float(np.linalg.norm(q, axis=1).mean()), 2),
           "k_norm": round(float(np.linalg.norm(k, axis=1).mean()), 2),
           "cos_query_to_top1_key": round(float(cosq.mean()), 3), "cos_key_to_top1_other_key": round(float(cosk.mean()), 3)}
    own = (lambda Q_, C_: f(Q_, C_))
    l2 = (lambda Q_, C_: Q_ @ C_.T - 0.5 * (C_ * C_).sum(1)[None, :])
    mips = (lambda Q_, C_: Q_ @ C_.T)
    for rname, route in (("own_score", own), ("L2_rule", l2), ("MIPS", mips)):
        a, ent, pmax = admission(Q, k, f, cent, assign, route)
        res[f"QtoK_{rname}"] = a
        if rname == "own_score":
            res["QtoK_softmax_entropy_nats"] = round(ent, 3); res["QtoK_softmax_maxmass"] = round(pmax, 3)
    # K->K control (near-copy regime, like exp2): exclude the probe itself from the target
    def f_excl(Q_, K_, _f=f, _ki=ki):
        s_ = _f(Q_, K_); s_[np.arange(len(Q_)), _ki] = -np.inf; return s_
    a, _, _ = admission(KQ, k, f_excl, cent, assign, own)
    res["KtoK_own_score"] = a
    out[name] = res
    print(name, json.dumps(res), flush=True)
json.dump(out, open(f"{S}/lab/exp/redteam2_sci/real_qk_admission.json", "w"), indent=1)
