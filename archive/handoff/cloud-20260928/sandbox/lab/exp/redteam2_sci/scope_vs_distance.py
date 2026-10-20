"""redteam2_sci R2: is math2's "Lorentz wins on enclosing-scope copies" a hierarchy effect or a copy-distance effect?

Enclosing-scope copies are, by construction, older than same-scope copies (the enclosing scope's tokens precede the
current scope's opening). A read whose advantage grows with copy distance would show up as an "enclosing" win.
Uses math2's saved per-target log-probs (e3/*.logp.f32) and math2's own category rule (copied from e3_analyze.py).

Output: Lorentz-minus-Dot NLL per (category x distance bin), per seed, with window-bootstrap 95% CIs, and a per-seed
least-squares fit  diff = a_cat + b * log2(distance)  on copyable targets (does the enclosing term survive distance?).
"""
import json, sys
import numpy as np

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
sys.path.insert(0, f"{S}/lab/exp/math2")
E = f"{S}/lab/exp/math2/e3"
rng = np.random.default_rng(0)

toks = np.fromfile(f"{S}/c3/data/code/valid.u16", "<u2").astype(np.int64)
depth = np.load(f"{S}/c3/data/code/valid_depth.npy").astype(np.int64)


def scopes(depth):  # verbatim logic from math2 e3_analyze.scopes
    N = len(depth); scope_of = np.zeros(N, np.int64); start, end = [0], [N]; stack = [0]
    for t in range(N):
        if t > 0 and depth[t] > depth[t - 1]:
            sid = len(start); start.append(t); end.append(N); stack.append(sid)
        elif t > 0 and depth[t] < depth[t - 1] and len(stack) > 1:
            end[stack.pop()] = t
        scope_of[t] = stack[-1]
    return scope_of, np.array(start), np.array(end)


so, st, en = scopes(depth)
T, W = 128, 800
cat = np.zeros((W, T - 1), np.int8); dist = np.zeros((W, T - 1), np.int64)
for w in range(W):
    win = toks[w * T:(w + 1) * T]; last = {}
    for t in range(T - 1):
        if t >= 1:
            last[win[t - 1]] = t - 1
        tgt = win[t + 1]
        if tgt not in last:
            continue
        jj = last[tgt]; dist[w, t] = t - jj
        j, p = w * T + jj, w * T + t
        a, b = so[j], so[p]
        if a == b: cat[w, t] = 1
        elif st[a] <= st[b] and en[a] >= en[b]: cat[w, t] = 2
        elif st[a] >= st[b] and en[a] <= en[b]: cat[w, t] = 3
        else: cat[w, t] = 4
labels = {1: "same", 2: "enclosing", 3: "closed_child", 4: "other"}
bins = [(1, 4), (5, 16), (17, 48), (49, 127)]


def boot(per_w, B=2000):
    x = per_w[~np.isnan(per_w)]
    if len(x) < 5: return (float("nan"),) * 3
    bs = [x[rng.integers(0, len(x), len(x))].mean() for _ in range(B)]
    return float(x.mean()), float(np.percentile(bs, 2.5)), float(np.percentile(bs, 97.5))


out = {"counts": {}, "mean_distance": {}}
for c, lab in labels.items():
    m = cat == c
    out["counts"][lab] = {f"{lo}-{hi}": int((m & (dist >= lo) & (dist <= hi)).sum()) for lo, hi in bins}
    out["mean_distance"][lab] = float(dist[m].mean())
print("counts", json.dumps(out["counts"]))
print("mean copy distance by category", json.dumps({k: round(v, 1) for k, v in out["mean_distance"].items()}))

for seed in (2, 3, 4):
    ld = np.fromfile(f"{E}/code_dot_s{seed}.logp.f32", "<f4").astype(np.float64).reshape(W, T - 1)
    ll = np.fromfile(f"{E}/code_lorentzflat_s{seed}.logp.f32", "<f4").astype(np.float64).reshape(W, T - 1)
    diff = (-ll) - (-ld)
    res = {}
    for c, lab in labels.items():
        for lo, hi in bins:
            m = (cat == c) & (dist >= lo) & (dist <= hi)
            per_w = np.array([diff[w][m[w]].mean() if m[w].any() else np.nan for w in range(W)])
            res[f"{lab}_{lo}-{hi}"] = [round(v, 4) for v in boot(per_w)] + [int(m.sum())]
    # least squares on copyable targets: diff ~ category dummies (same = reference) + log2(distance)
    m = cat > 0
    y = diff[m]; ld2 = np.log2(dist[m].astype(np.float64)); cc = cat[m]
    X = np.stack([np.ones_like(y), (cc == 2).astype(float), (cc == 3).astype(float), (cc == 4).astype(float), ld2], 1)
    Xn = np.stack([np.ones_like(y), (cc == 2).astype(float), (cc == 3).astype(float), (cc == 4).astype(float)], 1)
    beta = np.linalg.lstsq(X, y, rcond=None)[0]; beta_nodist = np.linalg.lstsq(Xn, y, rcond=None)[0]
    # window-bootstrap CI for the enclosing coefficient with distance control
    wid = np.repeat(np.arange(W)[:, None], T - 1, 1)[m]
    bs = []
    for _ in range(300):
        pick = rng.integers(0, W, W)
        sel = np.concatenate([np.nonzero(wid == p)[0] for p in pick])
        bs.append(np.linalg.lstsq(X[sel], y[sel], rcond=None)[0])
    bs = np.array(bs)
    res["fit_with_distance"] = {"same_intercept": round(beta[0], 4), "enclosing": round(beta[1], 4),
                                "closed_child": round(beta[2], 4), "other": round(beta[3], 4),
                                "per_log2_distance": round(beta[4], 4),
                                "enclosing_CI": [round(float(np.percentile(bs[:, 1], 2.5)), 4), round(float(np.percentile(bs[:, 1], 97.5)), 4)],
                                "slope_CI": [round(float(np.percentile(bs[:, 4], 2.5)), 4), round(float(np.percentile(bs[:, 4], 97.5)), 4)]}
    res["fit_without_distance"] = {"same_intercept": round(beta_nodist[0], 4), "enclosing": round(beta_nodist[1], 4),
                                   "closed_child": round(beta_nodist[2], 4), "other": round(beta_nodist[3], 4)}
    out[f"seed{seed}"] = res
    print(f"seed {seed}", json.dumps(res))
json.dump(out, open(f"{S}/lab/exp/redteam2_sci/scope_vs_distance.json", "w"), indent=1)
