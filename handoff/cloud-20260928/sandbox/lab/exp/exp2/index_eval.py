"""exp2: does a geometry-organized memory index admit the right key while scoring 1-3% of 4K-16K candidates?
(Scratch research code, numpy only, one thread.  NOT a project artifact.)

Loads encoders trained by geoattn_exp2.py (E, Wq, Wk), builds stores of N stored items, and compares admission
indices.  Every index ranks "cells" for a query by comparing the query with each cell's advertisement, admits whole
cells in rank order until at least f*N items are admitted, and then scores the admitted items exactly.

Index types
  chunk  : storage-order chunks of c consecutive items; advertisement = bundle (mean / majority / Lorentz centroid)
  kmeans : geometry-organized clusters (Euclidean k-means for dot, spherical for cos, Lorentzian k-means for hyp,
           k-majority in Hamming space for ham); advertisement = cluster centroid in the same geometry
  lsh    : data-independent sign buckets (SimHash hyperplanes; bit sampling for ham); advertisement = the bucket's
           sign pattern, ranked by the soft multi-probe score sum_i s_i <r_i, q>
  radir  : hyperbolic only: a "core" of the smallest-radius keys is always admitted, the rest are clustered by
           DIRECTION (spherical k-means on unit spatial parts) and routed by cosine
  hamscan: flat prefilter, every key compared by an m-bit sign code (XOR+popcount), top f*N admitted
  oracle : top f*N by the exact score itself (upper bound for any admission rule at that budget)

Metrics per (index, budget f): admission recall = P(target admitted); final top-1 = P(argmax of the exact score over
the admitted set is the target); mean admitted fraction; comparisons per query = advertisements compared + items scored.
"""
import argparse, json, math, time
import numpy as np


# ----------------------------------------------------------------------------------------------------------- model
def load(path):
    z = np.load(path)
    meta = json.loads(str(z["meta"]))
    P = {k: np.asarray(z[k], np.float64) for k in z.files if k != "meta"}
    return P, meta


def code(x, kind):
    if kind == "ham":
        return np.where(x >= 0, 1.0, -1.0)
    return x


def exact_scores(q, k, kind):
    """Ranking-equivalent exact score (larger = better) of queries (n, dk) against keys (N, dk)."""
    if kind in ("dot", "ham"):
        return q @ k.T
    if kind == "cos":
        qn = q / np.linalg.norm(q, axis=1, keepdims=True)
        kn = k / np.linalg.norm(k, axis=1, keepdims=True)
        return qn @ kn.T
    if kind == "hyp":        # -arcosh(z) is monotone decreasing in z = q0 k0 - <q,k>
        q0 = np.sqrt(1.0 + (q * q).sum(1, keepdims=True))
        k0 = np.sqrt(1.0 + (k * k).sum(1, keepdims=True))
        return -(q0 * k0.T - q @ k.T)
    raise ValueError(kind)


# ----------------------------------------------------------------------------------------------------------- bundles
def bundle_cells(K, labels, n_cells, kind):
    """Advertisement per cell from its members (same rule for storage chunks and for k-means centroids)."""
    dk = K.shape[1]
    cnt = np.bincount(labels, minlength=n_cells).astype(np.float64)
    if kind == "hyp":
        k0 = np.sqrt(1.0 + (K * K).sum(1))
        S = np.zeros((n_cells, dk)); np.add.at(S, labels, K)
        S0 = np.bincount(labels, weights=k0, minlength=n_cells)
        nrm = np.sqrt(np.maximum(S0 * S0 - (S * S).sum(1), 1e-12))
        return S / nrm[:, None]
    if kind == "cos":
        Kn = K / np.linalg.norm(K, axis=1, keepdims=True)
        S = np.zeros((n_cells, dk)); np.add.at(S, labels, Kn)
        return S / np.maximum(np.linalg.norm(S, axis=1, keepdims=True), 1e-12)
    S = np.zeros((n_cells, dk)); np.add.at(S, labels, K)
    if kind == "ham":
        return np.where(S >= 0, 1.0, -1.0)
    return S / np.maximum(cnt, 1.0)[:, None]


def kmeans(K, n_cells, kind, rng, iters=12):
    """Lloyd iterations in the key geometry.  Returns labels (N,)."""
    N = len(K)
    C = K[rng.choice(N, size=n_cells, replace=False)].copy()
    if kind == "cos":
        C = C / np.linalg.norm(C, axis=1, keepdims=True)
    labels = None
    for it in range(iters):
        if kind == "dot":            # Euclidean assignment
            d = (C * C).sum(1)[None, :] - 2.0 * K @ C.T
            new = np.argmin(d, 1)
        else:                        # cos / ham / hyp: nearest = largest exact score to the centroid
            new = np.argmax(exact_scores(K, C, kind), 1)
        if labels is not None and np.array_equal(new, labels):
            break
        labels = new
        C = bundle_cells(K, labels, n_cells, kind)
        empty = np.bincount(labels, minlength=n_cells) == 0
        if empty.any():
            C[empty] = K[rng.choice(N, size=int(empty.sum()), replace=False)]
    return labels


def dir_kmeans(K, n_cells, rng, iters=12):
    U = K / np.maximum(np.linalg.norm(K, axis=1, keepdims=True), 1e-12)
    return kmeans(U, n_cells, "cos", rng, iters)


def cell_radius(K, labels, cent, kind):
    """Covering radius of each cell around its advertisement, in the geometry's own distance."""
    nC = len(cent)
    if kind == "hyp":
        k0 = np.sqrt(1.0 + (K * K).sum(1)); c0 = np.sqrt(1.0 + (cent * cent).sum(1))
        z = k0 * c0[labels] - (K * cent[labels]).sum(1)
        d = np.arccosh(np.maximum(z, 1.0))
    elif kind == "ham":
        d = 0.5 * (K.shape[1] - (K * cent[labels]).sum(1))             # Hamming distance
    elif kind == "cos":
        Kn = K / np.linalg.norm(K, axis=1, keepdims=True)
        d = np.arccos(np.clip((Kn * cent[labels]).sum(1), -1, 1))
    else:
        d = np.linalg.norm(K - cent[labels], axis=1)
    r = np.zeros(nC); np.maximum.at(r, labels, d)
    return r


def ball_scores(q, cent, rad, kind, rho):
    """Optimistic bound on the best member score of each cell: centroid score relaxed by rho * covering radius."""
    if kind == "hyp":
        q0 = np.sqrt(1.0 + (q * q).sum(1, keepdims=True)); c0 = np.sqrt(1.0 + (cent * cent).sum(1))
        d = np.arccosh(np.maximum(q0 * c0[None, :] - q @ cent.T, 1.0))
        return -(d - rho * rad[None, :])                            # smaller lower-bound distance first
    if kind == "ham":
        return q @ cent.T + 2.0 * rho * rad[None, :]
    if kind == "cos":
        qn = q / np.linalg.norm(q, axis=1, keepdims=True)
        return -(np.arccos(np.clip(qn @ cent.T, -1, 1)) - rho * rad[None, :])
    return q @ cent.T + rho * np.linalg.norm(q, axis=1, keepdims=True) * rad[None, :]


def code_scores(q, cent, kind, Rm):
    """Route with sign codes (XOR+popcount angle estimate) plus exact radii: multiplier-free up to scalar tables."""
    m = Rm.shape[1]
    qc = np.where(q @ Rm >= 0, 1.0, -1.0); cc = np.where(cent @ Rm >= 0, 1.0, -1.0)
    cosang = np.cos(np.pi * 0.5 * (m - qc @ cc.T) / m)              # Hamming/m estimates angle/pi
    qn = np.linalg.norm(q, axis=1, keepdims=True); cn = np.linalg.norm(cent, axis=1)[None, :]
    if kind == "hyp":
        return -(np.sqrt(1 + qn ** 2) * np.sqrt(1 + cn ** 2) - qn * cn * cosang)
    if kind == "cos":
        return cosang
    return qn * cn * cosang - 0.5 * cn ** 2                         # dot: Euclidean (l2) routing form


# ----------------------------------------------------------------------------------------------------------- admission
BUDGETS = (0.01, 0.02, 0.03, 0.05, 0.10)


def admit_eval(cell_scores, labels, sizes, S_exact, tgt, N, n_adverts, budgets=BUDGETS, fixed_extra=None):
    """cell_scores (nq, nC); labels (N,) cell of each item; sizes (nC,); S_exact (nq, N); tgt (nq,).
    fixed_extra: optional boolean (N,) of items always admitted (not counted as cells)."""
    nq, nC = cell_scores.shape
    order = np.argsort(-cell_scores, axis=1, kind="stable")
    rank = np.empty_like(order); rank[np.arange(nq)[:, None], order] = np.arange(nC)[None, :]
    csum = np.cumsum(sizes[order], axis=1)                          # (nq, nC)
    item_rank = rank[:, labels]                                     # (nq, N) rank of each item's cell
    if fixed_extra is not None:
        item_rank = np.where(fixed_extra[None, :], -1, item_rank)
        n_fixed = int(fixed_extra.sum())
    else:
        n_fixed = 0
    st = S_exact[np.arange(nq), tgt]
    beat = S_exact > st[:, None]                                    # items that outrank the target exactly
    beat_min_rank = np.where(beat, item_rank, nC + 1).min(1)
    t_rank = item_rank[np.arange(nq), tgt]
    out = []
    for f in budgets:
        need = max(f * N - n_fixed, 0)
        # number of cells admitted: smallest prefix with cumulative size >= need (at least 1 cell if need > 0)
        ncell = (csum < need).sum(1) + (1 if need > 0 else 0)
        ncell = np.minimum(ncell, nC)
        adm_items = np.where(ncell > 0, csum[np.arange(nq), np.maximum(ncell - 1, 0)], 0) + n_fixed
        admitted = t_rank < ncell
        final = admitted & (beat_min_rank >= ncell)
        out.append({"f": f, "recall": float(admitted.mean()), "final": float(final.mean()),
                    "frac": float(adm_items.mean() / N), "comps": float(np.mean(n_adverts) + adm_items.mean())})
    return out


def cascade_eval(cell_scores, labels, sizes, S_exact, tgt, N, n_adverts, qc, kc, f1, rng, budgets=BUDGETS):
    """Stage 1: admit whole cells until >= f1*N items.  Stage 2: among those, keep the top f*N by sign-code
    agreement (XOR+popcount).  Stage 3: exact score only those.  Comparisons: adverts + stage-1 items (cheap codes)
    + f*N exact."""
    nq, nC = cell_scores.shape
    order = np.argsort(-cell_scores, axis=1, kind="stable")
    rank = np.empty_like(order); rank[np.arange(nq)[:, None], order] = np.arange(nC)[None, :]
    csum = np.cumsum(sizes[order], axis=1)
    ncell = np.minimum((csum < f1 * N).sum(1) + 1, nC)
    item_rank = rank[:, labels]
    adm1 = item_rank < ncell[:, None]                                # (nq, N) stage-1 admitted
    n1 = adm1.sum(1)
    A = qc @ kc.T + rng.random((nq, N)) * 1e-3
    A = np.where(adm1, A, -1e9)
    at = A[np.arange(nq), tgt]
    r = (A > at[:, None]).sum(1)
    st = S_exact[np.arange(nq), tgt]
    beat = S_exact > st[:, None]
    out = []
    for f in budgets:
        m = max(int(round(f * N)), 1)
        kth = -np.partition(-A, m - 1, axis=1)[:, m - 1]
        adm = (A >= kth[:, None]) & adm1
        admitted = adm1[np.arange(nq), tgt] & (r < m)
        final = admitted & ~(beat & adm).any(1)
        out.append({"f": f, "recall": float(admitted.mean()), "final": float(final.mean()),
                    "frac": float(np.minimum(n1, m).mean() / N),
                    "comps": float(np.mean(n_adverts) + np.minimum(n1, m).mean()),
                    "cheap_comps": float(n1.mean())})
    return out


def oracle_eval(S_exact, tgt, N, budgets=BUDGETS):
    nq = len(tgt)
    st = S_exact[np.arange(nq), tgt]
    r = (S_exact > st[:, None]).sum(1)                             # 0 = target is the exact argmax
    return [{"f": f, "recall": float((r < max(int(round(f * N)), 1)).mean()), "final": float((r == 0).mean()),
             "frac": f, "comps": float(f * N)} for f in budgets]


def hamscan_eval(qc, kc, S_exact, tgt, N, rng, budgets=BUDGETS):
    """Flat prefilter: every key compared by its sign code; admit the top f*N by agreement (random tie-break)."""
    nq = len(tgt)
    A = qc @ kc.T + rng.random((nq, N)) * 1e-3                     # agreements (+ tiny random tie-break)
    at = A[np.arange(nq), tgt]
    r = (A > at[:, None]).sum(1)                                    # target's rank under the code
    st = S_exact[np.arange(nq), tgt]
    beat = S_exact > st[:, None]
    out = []
    for f in budgets:
        m = max(int(round(f * N)), 1)
        kth = -np.partition(-A, m - 1, axis=1)[:, m - 1]           # m-th largest agreement
        adm = A >= kth[:, None]
        admitted = r < m
        final = admitted & ~(beat & adm).any(1)
        out.append({"f": f, "recall": float(admitted.mean()), "final": float(final.mean()), "frac": m / N,
                    "comps": float(m), "cheap_comps": N})
    return out


# ----------------------------------------------------------------------------------------------------------- tree helpers
def build_tree(branch, depth):
    parent, frontier = [-1], [0]
    for _ in range(depth):
        new = []
        for p_ in frontier:
            for _ in range(branch):
                parent.append(p_); new.append(len(parent) - 1)
        frontier = new
    return np.array(parent)


def tree_info(parent):
    parent = np.asarray(parent); N = len(parent)
    level = np.zeros(N, int)
    for n in range(1, N):
        level[n] = level[parent[n]] + 1
    anc = np.full((N, level.max() + 1), -1)
    for n in range(N):
        m = n
        while m >= 0:
            anc[n, level[m]] = m; m = parent[m]
    has_child = np.zeros(N, bool); has_child[parent[parent >= 0]] = True
    children = [[] for _ in range(N)]
    for n in range(1, N):
        children[parent[n]].append(n)
    pre = np.zeros(N, int); stack = [0]; i = 0
    while stack:                                                   # pre-order (DFS) index = order in source text
        n = stack.pop(); pre[n] = i; i += 1
        stack.extend(reversed(children[n]))
    return dict(parent=parent, level=level, anc=anc, internal=np.where(has_child)[0],
                leaves=np.where(~has_child)[0], pre=pre)


# ----------------------------------------------------------------------------------------------------------- stores
def mqar_store(rng, P, meta, N, nq, noise):
    K_, kind = meta["K"], meta["kind"]
    items = rng.choice(K_, size=N, replace=False)                  # storage order = insertion order (random)
    qi = rng.choice(N, size=nq, replace=False)
    keys = code(P["E"][items] @ P["Wk"], kind)
    qraw = P["E"][items[qi]] @ P["Wq"]
    qs = {}
    for s in noise:
        g = rng.standard_normal(qraw.shape)
        qn = qraw + s * np.linalg.norm(qraw, axis=1, keepdims=True) / math.sqrt(qraw.shape[1]) * g
        qs[s] = code(qn, kind)
    extra = {"value_keys": code(P["E"][meta["K"]:meta["K"] + meta["V"]] @ P["Wk"], kind)}
    return keys, qs, qi, extra


def tree_store(rng, P, meta, T, n_int, n_leaf, nq, order):
    kind = meta["kind"]
    internal, leaves = T["internal"], T["leaves"]
    stored = np.concatenate([[0], rng.choice(internal[internal != 0], size=n_int - 1, replace=False)])
    if n_leaf > 0:
        stored = np.concatenate([stored, rng.choice(leaves, size=n_leaf, replace=False)])
    if order == "dfs":
        stored = stored[np.argsort(T["pre"][stored])]
    else:
        rng.shuffle(stored)
    is_st = np.zeros(len(T["parent"]), bool); is_st[stored] = True
    pos = np.full(len(T["parent"]), -1); pos[stored] = np.arange(len(stored))
    free = leaves[~is_st[leaves]]
    ql = rng.choice(free, size=min(nq, len(free)), replace=False)
    A = T["anc"][ql]                                                # (nq, depth+1)
    ok = (A >= 0) & is_st[np.maximum(A, 0)]
    deepest = A.shape[1] - 1 - np.argmax(ok[:, ::-1], axis=1)
    tnode = A[np.arange(len(ql)), deepest]
    tgt = pos[tnode]
    keys = code(P["E"][stored] @ P["Wk"], kind)
    q = code(P["E"][ql] @ P["Wq"], kind)
    nonroot = tnode != 0
    return keys, {0.0: q}, tgt, {"nonroot": nonroot, "tdepth": T["level"][tnode], "stored": stored}


def global_codebook(P, meta, T, nC, rng):
    """Centroids learned once from every key the memory could ever hold (not from the current store)."""
    kind = meta["kind"]
    if meta["task"] == "mqar":
        allk = code(P["E"][:meta["K"]] @ P["Wk"], kind)
    else:
        allk = code(P["E"][T["internal"]] @ P["Wk"], kind)
    nC = min(nC, len(allk) // 2)
    lab = kmeans(allk, nC, kind, rng)
    return bundle_cells(allk, lab, nC, kind)


def assign(keys, cent, kind):
    if kind == "dot":
        return np.argmin((cent * cent).sum(1)[None, :] - 2.0 * keys @ cent.T, 1)
    return np.argmax(exact_scores(keys, cent, kind), 1)


# ----------------------------------------------------------------------------------------------------------- main
def route_scores(q, cent, kind, rule, dk):
    """Query-vs-advertisement score under a routing rule.
    exact : the model's own score applied to the advertisement (MIPS for dot; cosine; Lorentz; Hamming)
    l2    : dot only, the Euclidean rule <q,c> - |c|^2/2 (the advertisement also publishes its squared norm)
    codeM : M-bit sign codes (XOR+popcount angle estimate) plus exact norms; for dot in the l2 form"""
    if rule == "exact":
        return exact_scores(q, cent, kind)
    if rule == "l2":
        return q @ cent.T - 0.5 * (cent * cent).sum(1)[None, :]
    m = int(rule[4:])
    Rm = np.random.default_rng(7).standard_normal((dk, m))
    return code_scores(q, cent, kind, Rm)


def rules_for(kind, name, cfg):
    rs = ["exact"]
    if kind == "dot":
        rs.append("l2")
    if kind != "ham" and name in cfg["code_on"]:
        rs += [f"code{m}" for m in cfg["code_bits"]]
    return rs


def run_indices(keys, qd, tgt, kind, N, rng, cfg, extra_mask=None):
    """Evaluate all index types for one store. qd: {noise: queries}. Returns list of result rows."""
    rows = []
    t0 = time.time()
    dk = keys.shape[1]
    # --- build the indices once per store (they depend on keys only)
    built = []
    for c in cfg["chunk"]:
        if c >= N:
            continue
        nC = N // c
        lab = np.minimum(np.arange(N) // c, nC - 1)
        built.append(("chunk", f"c={c}", lab, nC, bundle_cells(keys, lab, nC, kind), None))
    for c in cfg["kmeans"]:
        nC = max(N // c, 2)
        lab = kmeans(keys, nC, kind, rng)
        cent = bundle_cells(keys, lab, nC, kind)
        built.append(("kmeans", f"size~{c}", lab, nC, cent, None))
        if cfg["ball"]:
            rad = cell_radius(keys, lab, cent, kind)
            for rho in cfg["ball"]:
                built.append(("kmeans+ball", f"size~{c},rho={rho}", lab, nC, cent, ("ball", rad, rho)))
    for c in cfg["kmeans2"]:
        nF = max(N // c, 4)
        labF = kmeans(keys, nF, kind, rng)
        centF = bundle_cells(keys, labF, nF, kind)
        nCo = max(int(round(math.sqrt(nF))), 2)
        parent = kmeans(centF, nCo, kind, rng)                      # coarse clusters of fine advertisements
        labCo = parent[labF]
        centCo = bundle_cells(keys, labCo, nCo, kind)
        built.append(("kmeans2", f"size~{c},coarse={nCo}", labF, nF, centF, ("two", parent, centCo)))
    for cname, cent in cfg.get("codebooks", []):
        lab = assign(keys, cent, kind)
        built.append(("codebook", cname, lab, len(cent), cent, None))
    for c in cfg["lsh"]:
        b = max(int(round(math.log2(N / c))), 1)
        if kind == "ham":
            R = np.eye(dk)[:, rng.choice(dk, size=b, replace=False)]
        else:
            R = rng.standard_normal((dk, b))
        bits = (keys @ R >= 0).astype(np.int64)
        ids = bits @ (1 << np.arange(b))
        uniq, lab = np.unique(ids, return_inverse=True)
        pat = ((uniq[:, None] >> np.arange(b)[None, :]) & 1) * 2.0 - 1.0     # (nC, b) sign pattern
        built.append(("lsh", f"bits={b}", lab, len(uniq), (R, pat), None))
    if kind in ("hyp", "dot") and cfg.get("core_kmeans"):
        rad = np.linalg.norm(keys, axis=1)
        for core_frac in cfg.get("radir_core", ()):
            if core_frac <= 0:
                continue
            n_core = int(round(core_frac * N))
            core = np.zeros(N, bool); core[np.argsort(rad)[:n_core]] = True
            rest = np.where(~core)[0]
            for c in cfg["core_kmeans"]:
                nC = max(len(rest) // c, 2)
                lab_rest = kmeans(keys[rest], nC, kind, rng)
                lab = np.zeros(N, int); lab[rest] = lab_rest
                cent = bundle_cells(keys[rest], lab_rest, nC, kind)
                sizes_r = np.bincount(lab_rest, minlength=nC)
                built.append(("kmeans+core", f"core={core_frac:.3f},size~{c}", lab, nC, cent, (core, sizes_r)))
    if kind == "hyp":
        rad = np.linalg.norm(keys, axis=1)
        for core_frac in cfg.get("radir_core", ()):
            for c in cfg["radir"]:
                n_core = int(round(core_frac * N))
                core = np.zeros(N, bool)
                if n_core > 0:
                    core[np.argsort(rad)[:n_core]] = True
                rest = np.where(~core)[0]
                nC = max(len(rest) // c, 2)
                lab_rest = dir_kmeans(keys[rest], nC, rng)
                lab = np.zeros(N, int); lab[rest] = lab_rest
                U = keys / np.maximum(np.linalg.norm(keys, axis=1, keepdims=True), 1e-12)
                cent = bundle_cells(U[rest], lab_rest, nC, "cos")
                sizes = np.bincount(lab_rest, minlength=nC)
                built.append(("radir", f"core={core_frac:.3f},size~{c}", lab, nC, cent, (core, sizes)))
    t_build = time.time() - t0
    for noise, q in qd.items():
        S = exact_scores(q, keys, kind)
        if kind == "ham":                                           # integer scores: break ties at random
            S = S + rng.random(S.shape) * 1e-3
        base = {"noise": noise}
        orc = oracle_eval(S, tgt, N)
        rows.append({**base, "index": "oracle", "cfg": "exact top-fN", "res": orc})
        for (name, cfgs, lab, nC, adv, aux) in built:
            sizes = np.bincount(lab, minlength=nC)
            if name == "lsh":
                R, pat = adv
                res = admit_eval((q @ R) @ pat.T, lab, sizes, S, tgt, N, nC)
                rows.append({**base, "index": name, "cfg": cfgs, "n_cells": int(nC), "res": res})
            elif name == "radir":
                core, sizes_r = aux
                qu = q / np.maximum(np.linalg.norm(q, axis=1, keepdims=True), 1e-12)
                res = admit_eval(qu @ adv.T, lab, sizes_r, S, tgt, N, nC, fixed_extra=core)
                rows.append({**base, "index": name, "cfg": cfgs, "n_cells": int(nC), "res": res})
            elif name == "kmeans+core":
                core, sizes_r = aux
                for rule in (["exact", "l2"] if kind == "dot" else ["exact"]):
                    res = admit_eval(route_scores(q, adv, kind, rule, dk), lab, sizes_r, S, tgt, N, nC, fixed_extra=core)
                    rows.append({**base, "index": f"{name}[{rule}]", "cfg": cfgs, "n_cells": int(nC), "res": res})
            elif name == "kmeans+ball":
                _, rad, rho = aux
                res = admit_eval(ball_scores(q, adv, rad, kind, rho), lab, sizes, S, tgt, N, nC)
                rows.append({**base, "index": name, "cfg": cfgs, "n_cells": int(nC), "res": res})
            elif name == "kmeans2":
                _, parent, centCo = aux
                for rule in rules_for(kind, name, cfg):
                    sco, scf = route_scores(q, centCo, kind, rule, dk), route_scores(q, adv, kind, rule, dk)
                    order_co = np.argsort(-sco, axis=1)
                    for p in cfg["probe"]:
                        if p >= len(centCo):
                            continue
                        allow = np.zeros_like(sco, bool); allow[np.arange(len(q))[:, None], order_co[:, :p]] = True
                        fa = allow[:, parent]                           # (nq, nF) fine cells inside probed coarse
                        cs = np.where(fa, scf, -1e30)
                        n_adv = len(centCo) + fa.sum(1)
                        res = admit_eval(cs, lab, sizes, S, tgt, N, n_adv)
                        rows.append({**base, "index": f"{name}[{rule}]", "cfg": f"{cfgs},probe={p}",
                                     "n_cells": int(nC), "res": res})
            else:                                                   # chunk, kmeans, codebook
                for rule in rules_for(kind, name, cfg):
                    res = admit_eval(route_scores(q, adv, kind, rule, dk), lab, sizes, S, tgt, N, nC)
                    rows.append({**base, "index": f"{name}[{rule}]", "cfg": cfgs, "n_cells": int(nC), "res": res})
        for (name, cfgs, lab, nC, adv, aux) in built:
            if name != "kmeans" or not cfg.get("cascade"):
                continue
            rule = "l2" if kind == "dot" else "exact"
            cs = route_scores(q, adv, kind, rule, dk)
            sizes = np.bincount(lab, minlength=nC)
            for m in cfg["hamscan"]:
                if kind == "ham" and m == dk:
                    qc, kc = q, keys
                else:
                    Rm = np.random.default_rng(99).standard_normal((dk, m))
                    qc, kc = np.where(q @ Rm >= 0, 1.0, -1.0), np.where(keys @ Rm >= 0, 1.0, -1.0)
                for f1 in cfg["cascade"]:
                    res = cascade_eval(cs, lab, sizes, S, tgt, N, nC, qc, kc, f1, rng)
                    rows.append({**base, "index": f"cascade[{rule}]", "cfg": f"{cfgs},stage1={f1},m={m}",
                                 "n_cells": int(nC), "res": res})
        for m in cfg["hamscan"]:
            if kind == "ham" and m == dk:
                qc, kc = q, keys
            else:
                Rm = np.random.default_rng(99).standard_normal((dk, m))
                qc, kc = np.where(q @ Rm >= 0, 1.0, -1.0), np.where(keys @ Rm >= 0, 1.0, -1.0)
            res = hamscan_eval(qc, kc, S, tgt, N, rng)
            rows.append({**base, "index": "hamscan", "cfg": f"m={m}", "res": res})
    return rows, t_build


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--params", required=True)
    ap.add_argument("--Ns", default="1024,4096,16384")
    ap.add_argument("--stores", default="4,2,2", help="stores per N")
    ap.add_argument("--nq", type=int, default=256)
    ap.add_argument("--noise", default="0")
    ap.add_argument("--order", default="random", help="tree only: random or dfs storage order")
    ap.add_argument("--leaf_frac", type=float, default=0.0, help="tree only: fraction of N that are leaf distractors")
    ap.add_argument("--chunk", default="16,64")
    ap.add_argument("--kmeans", default="16,64,256")
    ap.add_argument("--lsh", default="16,64")
    ap.add_argument("--radir", default="16,64")
    ap.add_argument("--radir_core", default="0,0.005")
    ap.add_argument("--core_kmeans", default="", help="cell sizes for core + k-means (tree runs)")
    ap.add_argument("--cascade", default="", help="stage-1 fractions for cells -> codes -> exact (e.g. 0.1,0.2)")
    ap.add_argument("--hamscan", default="64,256")
    ap.add_argument("--ball", default="0.5")
    ap.add_argument("--kmeans2", default="16")
    ap.add_argument("--codebook", default="16,64")
    ap.add_argument("--probe", default="4,8,16")
    ap.add_argument("--code_bits", default="64,256")
    ap.add_argument("--code_on", default="kmeans,kmeans2,codebook")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    P, meta = load(a.params)
    kind = meta["kind"]
    ints = lambda s: tuple(int(x) for x in s.split(",") if x != "")
    flts = lambda s: tuple(float(x) for x in s.split(",") if x != "")
    cfg = {"chunk": ints(a.chunk), "kmeans": ints(a.kmeans), "lsh": ints(a.lsh), "radir": ints(a.radir),
           "radir_core": flts(a.radir_core), "core_kmeans": ints(a.core_kmeans), "cascade": flts(a.cascade), "hamscan": ints(a.hamscan), "ball": flts(a.ball),
           "kmeans2": ints(a.kmeans2), "probe": ints(a.probe), "code_bits": ints(a.code_bits),
           "code_on": tuple(a.code_on.split(","))}
    rng = np.random.default_rng(1000 + a.seed)
    T = None
    if meta["task"] == "tree":
        parent = np.load(meta["tree_file"].replace("../../../", "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/"))["parent"] \
            if meta["tree_file"] else build_tree(meta["tree_branch"], meta["tree_depth"])
        T = tree_info(parent)
    out = {"params": a.params, "meta": meta, "args": vars(a), "per_N": []}
    for N, ns in zip(ints(a.Ns), ints(a.stores)):
        allrows, diag = [], {}
        tcb = time.time()
        cfg["codebooks"] = [(f"size~{c}", global_codebook(P, meta, T, max(N // c, 2), rng)) for c in ints(a.codebook)]
        tcb = time.time() - tcb
        t0 = time.time(); tb = 0.0
        for s in range(ns):
            if meta["task"] == "mqar":
                keys, qd, tgt, extra = mqar_store(rng, P, meta, N, a.nq, flts(a.noise))
                q0 = qd[flts(a.noise)[0]]
                # diagnostics: exact full scan, including the value-keyed positions of the MQAR layout
                S = exact_scores(q0, keys, kind)
                if kind == "ham":
                    S = S + rng.random(S.shape) * 1e-3
                Sv = exact_scores(q0, extra["value_keys"], kind)
                st = S[np.arange(len(tgt)), tgt]
                diag.setdefault("full_acc", []).append(float((S.max(1) <= st).mean()))
                diag.setdefault("value_pos_beats_target", []).append(float((Sv.max(1) > st).mean()))
                kt = keys[tgt]
                cosqk = (q0 * kt).sum(1) / (np.linalg.norm(q0, axis=1) * np.linalg.norm(kt, axis=1))
                diag.setdefault("cos_q_ktarget", []).append(float(cosqk.mean()))
                for s_ in flts(a.noise)[1:]:
                    Sn = exact_scores(qd[s_], keys, kind)
                    if kind == "ham":
                        Sn = Sn + rng.random(Sn.shape) * 1e-3
                    diag.setdefault(f"full_acc_noise{s_}", []).append(
                        float((Sn.max(1) <= Sn[np.arange(len(tgt)), tgt]).mean()))
            else:
                n_leaf = int(round(a.leaf_frac * N))
                keys, qd, tgt, extra = tree_store(rng, P, meta, T, N - n_leaf, n_leaf, a.nq, a.order)
                S = exact_scores(qd[0.0], keys, kind)
                st = S[np.arange(len(tgt)), tgt]
                full = S.max(1) <= st
                nr = extra["nonroot"]
                diag.setdefault("full_acc", []).append(float(full.mean()))
                diag.setdefault("full_acc_nonroot", []).append(float(full[nr].mean()) if nr.any() else None)
                diag.setdefault("nonroot_frac", []).append(float(nr.mean()))
                diag.setdefault("target_depth_mean", []).append(float(extra["tdepth"].mean()))
                if kind == "hyp":
                    rad = np.linalg.norm(keys, axis=1)
                    lvl = T["level"][extra["stored"]]
                    diag.setdefault("radius_depth_corr", []).append(float(np.corrcoef(rad, lvl)[0, 1]))
            rows, tbs = run_indices(keys, qd, tgt, kind, N, rng, cfg)
            tb += tbs
            for r in rows:
                r["store"] = s
            allrows.extend(rows)
        # average over stores
        agg = {}
        for r in allrows:
            key = (r["noise"], r["index"], r["cfg"])
            agg.setdefault(key, []).append(r)
        summary = []
        for (noise, idx, cf), rs in agg.items():
            res = []
            for j in range(len(rs[0]["res"])):
                res.append({k: float(np.mean([x["res"][j][k] for x in rs])) for k in rs[0]["res"][j]})
            summary.append({"noise": noise, "index": idx, "cfg": cf,
                            "n_cells": rs[0].get("n_cells"), "res": res})
        d = {k: (float(np.mean([x for x in v if x is not None])) if any(x is not None for x in v) else None)
             for k, v in diag.items()}
        out["per_N"].append({"N": N, "stores": ns, "nq": a.nq, "diag": d, "summary": summary,
                             "build_s": tb, "codebook_s": tcb, "wall_s": time.time() - t0})
        print(f"N={N} diag={json.dumps(d)} build={tb:.1f}s wall={time.time() - t0:.1f}s", flush=True)
        for r in summary:
            if r["noise"] != flts(a.noise)[0]:
                continue
            print(f"  {r['index']:8s} {r['cfg']:24s} " + " ".join(
                f"f{x['f']:.2f}:R{x['recall']:.3f}/A{x['final']:.3f}/fr{x['frac']:.3f}/c{x['comps']:.0f}" for x in r["res"]), flush=True)
        json.dump(out, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
