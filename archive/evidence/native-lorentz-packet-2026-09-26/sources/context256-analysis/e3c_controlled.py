"""ctx256: scope-category effects on Lorentz - Dot NLL, controlling for log2 copy distance (science-review method)."""
import json, sys, numpy as np
sys.path.insert(0, "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/lead_ctx256")
import e3_analyze_ctx256 as E

S = E.S
D = f"{S}/c3/data"
toks = np.fromfile(f"{D}/code/valid.u16", "<u2").astype(np.int64)
sc_info = E.scopes(np.load(f"{D}/code/valid_depth.npy").astype(np.int64))
rng = np.random.default_rng(2)
names = ["same_scope", "enclosing_scope", "closed_child", "other_scope"]
out = {}
for tag, dn, ln in [("s1", "dot_s1", "lorentzflat_s1"), ("s2", "dot_s2", "lorentzflat_s2")]:
    dot, lor = E.load(dn), E.load(ln)
    T, W = dot["P"]["context"], dot["P"]["windows"]
    diff = (-lor["logp"].reshape(W, T - 1)) - (-dot["logp"].reshape(W, T - 1))
    cat = E.categories(toks, T, W, sc_info)
    dist = np.zeros((W, T - 1))
    for w in range(W):
        win = toks[w * T:(w + 1) * T]; last = {}
        for t in range(T - 1):
            if t >= 1: last[win[t - 1]] = t - 1
            tgt = win[t + 1]
            dist[w, t] = (t - last[tgt]) if tgt in last else 0
    m = cat > 0
    y = diff[m]; ld = np.log2(dist[m]); c = cat[m]; wid = np.repeat(np.arange(W)[:, None], T - 1, 1)[m]
    X = np.column_stack([np.ones_like(ld), ld] + [(c == k).astype(float) for k in (2, 3, 4)])
    def fit(idx):
        beta, *_ = np.linalg.lstsq(X[idx], y[idx], rcond=None); return beta
    full = fit(np.arange(len(y)))
    per_w = [np.where(wid == w)[0] for w in range(W)]
    bs = np.array([fit(np.concatenate([per_w[p] for p in rng.integers(0, W, W)])) for _ in range(400)])
    lo, hi = np.percentile(bs, 2.5, axis=0), np.percentile(bs, 97.5, axis=0)
    labels = ["intercept_same_scope", "per_doubling", "enclosing_minus_same", "closed_child_minus_same", "other_minus_same"]
    res = {lab: dict(mean=round(float(full[i]), 4), lo=round(float(lo[i]), 4), hi=round(float(hi[i]), 4)) for i, lab in enumerate(labels)}
    out[tag] = res
    print(tag, json.dumps(res), flush=True)
json.dump(out, open(f"{S}/lab/exp/lead_ctx256/e3c_controlled.json", "w"), indent=1)
