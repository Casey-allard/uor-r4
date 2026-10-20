"""ctx256: Lorentz - Dot NLL by copy distance (bootstrap over windows), slope per doubling, and read geometry."""
import json, sys, numpy as np
sys.path.insert(0, "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/lead_ctx256")
import e3_analyze_ctx256 as E

S = E.S
toks = np.fromfile(f"{S}/c3/data/code/valid.u16", "<u2").astype(np.int64)
rng = np.random.default_rng(1)
bins = [(1, 4), (5, 16), (17, 64), (65, 255)]
out = {}
for tag, dn, ln in [("s1", "dot_s1", "lorentzflat_s1"), ("s2", "dot_s2", "lorentzflat_s2")]:
    dot, lor = E.load(dn), E.load(ln)
    T, W = dot["P"]["context"], dot["P"]["windows"]
    diff = (-lor["logp"].reshape(W, T - 1)) - (-dot["logp"].reshape(W, T - 1))
    dist = np.zeros((W, T - 1), np.int64)
    for w in range(W):
        win = toks[w * T:(w + 1) * T]; last = {}
        for t in range(T - 1):
            if t >= 1: last[win[t - 1]] = t - 1
            tgt = win[t + 1]
            dist[w, t] = (t - last[tgt]) if tgt in last else 0
    res = {}
    for lo, hi in bins:
        m = (dist >= lo) & (dist <= hi)
        per_w = np.array([diff[w][m[w]].mean() if m[w].any() else np.nan for w in range(W)])
        mean, a, b = E.boot(per_w)
        res[f"{lo}-{hi}"] = dict(frac=float(m.mean()), mean=round(mean, 4), lo=round(a, 4), hi=round(b, 4))
    # slope per doubling on copyable tokens, bootstrap over windows
    m = dist > 0
    x = np.log2(dist[m]); y = diff[m]; wid = np.repeat(np.arange(W)[:, None], T - 1, 1)[m]
    def slope(idx):
        xx, yy = x[idx], y[idx]; xx = xx - xx.mean(); return float((xx * (yy - yy.mean())).sum() / (xx * xx).sum())
    per_window_index = [np.where(wid == w)[0] for w in range(W)]
    bs = []
    for _ in range(500):
        pick = rng.integers(0, W, W)
        idx = np.concatenate([per_window_index[p] for p in pick]); bs.append(slope(idx))
    s0 = slope(np.arange(len(x)))
    res["slope_per_doubling"] = dict(mean=round(s0, 4), lo=round(float(np.percentile(bs, 2.5)), 4), hi=round(float(np.percentile(bs, 97.5)), 4))
    res["geometry_lorentz"] = {k: round(v, 4) for k, v in E.read_geometry(lor).items()}
    res["geometry_dot"] = {k: round(v, 4) for k, v in E.read_geometry(dot).items()}
    out[tag] = res
    print(tag, json.dumps(res), flush=True)
json.dump(out, open(f"{S}/lab/exp/lead_ctx256/e3b_distance_geometry.json", "w"), indent=1)
