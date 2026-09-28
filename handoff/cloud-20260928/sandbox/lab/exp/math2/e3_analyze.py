"""math2 E3: where does the cycle-3 Lorentz gain come from, and how much curvature does the trained read use?

Inputs: probe dumps in e3/ (exact queries/keys/NoRead logits rebuilt from saved cycle-3 JointModels; verified against the
models' own read masses). Paired models share seeds, initial arrays, training windows and evaluation windows.

(1) Per-token NLL difference (Lorentz - Dot) by target category, bootstrap over windows:
    A not copyable | B..E copyable, most recent earlier occurrence in same / enclosing / closed-child / other scope.
(2) Read geometry of the trained Lorentz read: key/query hyperbolic radii, and how much the read distribution changes
    when every key's radius in the window is replaced by the window mean (direction kept) = the purely spherical part.
    Same test for Dot (key norm replaced by window mean).
"""
import json, sys, numpy as np

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
E = f"{S}/lab/exp/math2/e3"
rng = np.random.default_rng(0)


def load(name):
    P = json.load(open(f"{E}/{name}.params.json"))
    T, r, n = P["context"], P["read_width"], P["dump"] * P["context"]
    d = dict(P=P, logp=np.fromfile(f"{E}/{name}.logp.f32", "<f4").astype(np.float64),
             q=np.fromfile(f"{E}/{name}.query.f32", "<f4").reshape(n, r).astype(np.float64),
             k=np.fromfile(f"{E}/{name}.key.f32", "<f4").reshape(n, r).astype(np.float64),
             nl=np.fromfile(f"{E}/{name}.null.f32", "<f4").astype(np.float64))
    return d


def scopes(depth):
    N = len(depth); scope_of = np.zeros(N, np.int64); start, end = [0], [N]; stack = [0]
    for t in range(N):
        if t > 0 and depth[t] > depth[t - 1]:
            sid = len(start); start.append(t); end.append(N); stack.append(sid)
        elif t > 0 and depth[t] < depth[t - 1] and len(stack) > 1:
            end[stack.pop()] = t
        scope_of[t] = stack[-1]
    return scope_of, np.array(start), np.array(end)


def categories(toks, T, W, scope_info=None):
    """category per (window, position t=0..T-2) for target toks[t+1]; candidates are positions 0..t-1."""
    cat = np.zeros((W, T - 1), np.int8)
    for w in range(W):
        win = toks[w * T:(w + 1) * T]
        last = {}
        for t in range(T - 1):
            if t >= 1:
                last[win[t - 1]] = t - 1           # candidate t-1 becomes available at position t
            tgt = win[t + 1]
            if tgt not in last:
                cat[w, t] = 0; continue
            if scope_info is None:
                cat[w, t] = 1; continue
            so, st, en = scope_info
            j, p = w * T + last[tgt], w * T + t
            a, b = so[j], so[p]
            if a == b: cat[w, t] = 1
            elif st[a] <= st[b] and en[a] >= en[b]: cat[w, t] = 2
            elif st[a] >= st[b] and en[a] <= en[b]: cat[w, t] = 3
            else: cat[w, t] = 4
    return cat


def boot(diff_w, B=2000):
    """diff_w: per-window mean differences (nan where empty); returns mean and 95% CI over windows."""
    x = diff_w[~np.isnan(diff_w)]
    if len(x) == 0: return (np.nan, np.nan, np.nan)
    bs = [x[rng.integers(0, len(x), len(x))].mean() for _ in range(B)]
    return float(x.mean()), float(np.percentile(bs, 2.5)), float(np.percentile(bs, 97.5))


def paired(dot, lor, toks, scope_info, labels):
    T = dot["P"]["context"]; W = dot["P"]["windows"]
    ld = dot["logp"].reshape(W, T - 1); ll = lor["logp"].reshape(W, T - 1)
    diff = (-ll) - (-ld)                                          # Lorentz NLL - Dot NLL (negative = Lorentz better)
    cat = categories(toks, T, W, scope_info)
    out = {"all": dict(zip(("mean", "lo", "hi"), boot(diff.mean(1)))),
           "nll_dot": float(-ld.mean()), "nll_lorentz": float(-ll.mean())}
    for c, lab in enumerate(labels):
        m = cat == c
        frac = float(m.mean())
        per_w = np.array([diff[w][m[w]].mean() if m[w].any() else np.nan for w in range(W)])
        mean, lo, hi = boot(per_w)
        out[lab] = dict(frac=frac, mean=mean, lo=lo, hi=hi, contrib=frac * float(diff[m].mean()) if m.any() else 0.0,
                        nll_dot=float(-ld[m].mean()) if m.any() else np.nan)
    return out


def read_geometry(mod):
    P = mod["P"]; T, r = P["context"], P["read_width"]; age = np.array(P["age"])
    q, k, nl = mod["q"], mod["k"], mod["nl"]
    lor = P["geometry"] == "Lorentz"
    kr = np.linalg.norm(k, axis=1); qr = np.linalg.norm(q, axis=1)
    res = {"key_norm_mean": float(kr.mean()), "query_norm_mean": float(qr.mean())}
    if lor:
        rk, rq = np.arcsinh(kr), np.arcsinh(qr)
        res.update(key_radius_mean=float(rk.mean()), key_radius_std=float(rk.std()),
                   query_radius_mean=float(rq.mean()), query_radius_std=float(rq.std()),
                   key_radius_within_window_std=float(np.mean([rk[w*T:(w+1)*T].std() for w in range(P["dump"])])))
    else:
        res.update(key_norm_std=float(kr.std()),
                   key_norm_within_window_cv=float(np.mean([kr[w*T:(w+1)*T].std() / kr[w*T:(w+1)*T].mean() for w in range(P["dump"])])))
    kls, tvs, top1, r2 = [], [], [], []
    for w in range(P["dump"]):
        Q, K = q[w*T:(w+1)*T], k[w*T:(w+1)*T]
        for t in range(16, T):
            kk = K[:t]; qq = Q[t]
            nk = np.linalg.norm(kk, axis=1); u = kk / nk[:, None]
            if lor:
                def sc(knorm):
                    z = np.sqrt(1 + qq @ qq) * np.sqrt(1 + knorm**2) - knorm * (u @ qq)
                    return P["beta"] * (P["offset"] - np.arccosh(np.maximum(z, 1 + 1e-6)))
            else:
                def sc(knorm):
                    return knorm * (u @ qq) / np.sqrt(r)
            s_full = sc(nk); s_sph = sc(np.full_like(nk, nk.mean()))
            a = age[t - 1 - np.arange(t)]
            def dist(s):
                lg = np.concatenate([[nl[w*T+t]], s + a]); p = np.exp(lg - lg.max()); return p / p.sum()
            p, p2 = dist(s_full), dist(s_sph)
            kls.append(float(np.sum(p * (np.log(p + 1e-300) - np.log(p2 + 1e-300)))))
            tvs.append(0.5 * float(np.abs(p - p2).sum()))
            top1.append(float(np.argmax(p[1:]) == np.argmax(p2[1:])))
            cos = u @ qq / np.linalg.norm(qq)
            cc = np.corrcoef(s_full, cos)[0, 1]; r2.append(cc * cc)
    res.update(KL_full_vs_spherical=float(np.mean(kls)), TV_full_vs_spherical=float(np.mean(tvs)),
               top1_agree=float(np.mean(top1)), R2_score_vs_cos=float(np.mean(r2)))
    return res


if __name__ == "__main__":
    D = f"{S}/c3/data"
    code_toks = np.fromfile(f"{D}/code/valid.u16", "<u2").astype(np.int64)
    wiki_toks = np.fromfile(f"{D}/wiki/valid.u16", "<u2").astype(np.int64)
    sc_info = scopes(np.load(f"{D}/code/valid_depth.npy").astype(np.int64))
    labels_code = ["not_copyable", "copy_same_scope", "copy_enclosing_scope", "copy_closed_child", "copy_other_scope"]
    out = {}
    pairs = [("code_s2", "code_dot_s2", "code_lorentzflat_s2", "code"), ("code_s3", "code_dot_s3", "code_lorentzflat_s3", "code"),
             ("code_s4", "code_dot_s4", "code_lorentzflat_s4", "code"), ("wiki_s1", "wiki_dot_s1", "wiki_lorentzflat_s1", "wiki")]
    cache = {}
    for tag, dn, ln, corpus in pairs:
        try:
            dot, lor = load(dn), load(ln)
        except FileNotFoundError:
            print("missing", tag); continue
        cache[dn], cache[ln] = dot, lor
        if corpus == "code":
            out[tag] = paired(dot, lor, code_toks, sc_info, labels_code)
        else:
            out[tag] = paired(dot, lor, wiki_toks, None, ["not_copyable", "copyable"])
        print(tag, json.dumps({k: (round(v, 4) if isinstance(v, float) else {kk: round(vv, 4) for kk, vv in v.items()}) for k, v in out[tag].items()}), flush=True)
    geo = {}
    for name in ["code_lorentzflat_s3", "code_dot_s3", "code_lorentzflat_s2", "code_dot_s2", "code_lorentz_s3", "wiki_lorentzflat_s1", "wiki_dot_s1"]:
        try:
            mod = cache.get(name) or load(name)
        except FileNotFoundError:
            continue
        geo[name] = read_geometry(mod)
        print("geometry", name, json.dumps({k: round(v, 4) for k, v in geo[name].items()}), flush=True)
    out["geometry"] = geo
    json.dump(out, open(f"{S}/lab/exp/math2/e3_analysis.json", "w"), indent=1)
