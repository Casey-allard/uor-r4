"""math2 E5: 'geodesic triangulation' as exact admission: horospherical (Busemann) landmark lower bounds.

In the Lorentz model an ideal landmark xi = (1, u), |u| = 1, gives the Busemann coordinate B_u(x) = log(x0 - <x_s, u>);
|B_u(q) - B_u(k)| <= d(q, k) (1-Lipschitz), so LB(q,k) = max_i |B_i(q) - B_i(k)| never excludes the true nearest key.
Derived: B_u(y) - B_u(x) = d(x,y) - 2 (y|xi)_x, an ADDITIVE error set by the angle at x, versus the Euclidean projection
bound <y - x, u> = |y - x| cos(phi), a MULTIPLICATIVE error.
Test: same tree (4-ary, depth 6), keys = random 30% of nodes, queries = random leaves (not stored); exact 1-NN in each
geometry's own metric; admitted fraction = share of keys with LB <= d(q, NN) (they must be scored exactly).
  H: cone embedding in H^8 (radius tau*depth, child directions spread ~exp(-tau*depth)); landmarks: m ideal points,
     random or placed at the directions of m random keys (data-adaptive).
  E: path-sum embedding in R^64 (x_v = sum of Gaussian edge vectors on the root path; |x_u-x_v|^2 ~ tree distance);
     landmarks: m unit directions, random or toward m random keys.
"""
import numpy as np, json
rng = np.random.default_rng(0)
b, L, n_h, n_e, tau = 4, 6, 8, 64, 1.5
parent, depth = [-1], [0]
frontier = [0]
for l in range(1, L + 1):
    new = []
    for p in frontier:
        for _ in range(b):
            parent.append(p); depth.append(l); new.append(len(parent) - 1)
    frontier = new
parent, depth = np.array(parent), np.array(depth); N = len(parent)

# hyperbolic cone embedding
U = np.zeros((N, n_h)); U[0] = rng.normal(size=n_h); U[0] /= np.linalg.norm(U[0])
theta0 = 1.2
for v in range(1, N):
    up = U[parent[v]]; g = rng.normal(size=n_h); g -= (g @ up) * up; g /= np.linalg.norm(g)
    ang = theta0 * np.exp(-tau * (depth[v] - 1))
    U[v] = np.cos(ang) * up + np.sin(ang) * g
rad = tau * depth
XH = np.concatenate([np.cosh(rad)[:, None], np.sinh(rad)[:, None] * U], 1)   # hyperboloid points (x0, x_s)
# Euclidean path-sum embedding
G = rng.normal(size=(N, n_e)) / np.sqrt(n_e)
XE = np.zeros((N, n_e))
for v in range(1, N):
    XE[v] = XE[parent[v]] + G[v]


def dH(a, B):  # a: (n+1,), B: (m, n+1)
    return np.arccosh(np.maximum(a[0] * B[:, 0] - B[:, 1:] @ a[1:], 1.0))


def busemann(X, dirs):  # X (m, n+1), dirs (L, n) unit
    return np.log(X[:, [0]] - X[:, 1:] @ dirs.T)


leaves = np.where(depth == L)[0]
res = {}
for trial_keys in range(1):
    stored = rng.choice(np.arange(N), size=int(0.3 * N), replace=False)
    stored = np.setdiff1d(stored, leaves[:0])
    queries = rng.choice(np.setdiff1d(leaves, stored), size=300, replace=False)
    for m in (8, 32, 128, 512):
        for mode in ("random", "adaptive"):
            if mode == "random":
                dh = rng.normal(size=(m, n_h)); dh /= np.linalg.norm(dh, axis=1, keepdims=True)
                de = rng.normal(size=(m, n_e)); de /= np.linalg.norm(de, axis=1, keepdims=True)
            else:
                pick = rng.choice(stored, size=m, replace=False)
                dh = XH[pick, 1:] / np.linalg.norm(XH[pick, 1:], axis=1, keepdims=True).clip(1e-12)
                de = XE[pick] / np.linalg.norm(XE[pick], axis=1, keepdims=True).clip(1e-12)
            BK = busemann(XH[stored], dh); PK = XE[stored] @ de.T
            adm_h, adm_e, ok_h, ok_e = [], [], 0, 0
            for qv in queries:
                d_true = dH(XH[qv], XH[stored]); nn = d_true.min()
                lb = np.abs(busemann(XH[[qv]], dh) - BK).max(1)
                adm_h.append(float((lb <= nn + 1e-9).mean())); ok_h += int(lb[d_true.argmin()] <= nn + 1e-9)
                de_true = np.linalg.norm(XE[stored] - XE[qv], axis=1); nne = de_true.min()
                lbe = np.abs(XE[qv] @ de.T - PK).max(1)
                adm_e.append(float((lbe <= nne + 1e-9).mean())); ok_e += int(lbe[de_true.argmin()] <= nne + 1e-9)
            res[f"m{m}_{mode}"] = {"H8_admitted_frac": float(np.mean(adm_h)), "R64_admitted_frac": float(np.mean(adm_e)),
                                  "H8_nn_always_admitted": ok_h == len(queries), "R64_nn_always_admitted": ok_e == len(queries)}
            print(m, mode, json.dumps(res[f"m{m}_{mode}"]), flush=True)
# tree-likeness sanity: NN in each embedding equals the tree-metric NN how often?
def tree_dist(u, v):
    du, dv = u, v; dist = 0
    while du != dv:
        if depth[du] >= depth[dv]: du = parent[du]
        else: dv = parent[dv]
        dist += 1
    return dist
agree_h = agree_e = 0
for qv in queries[:100]:
    td = np.array([tree_dist(qv, s) for s in stored])
    agree_h += int(td[dH(XH[qv], XH[stored]).argmin()] == td.min())
    agree_e += int(td[np.linalg.norm(XE[stored] - XE[qv], axis=1).argmin()] == td.min())
res["nn_matches_tree_nn_H8"] = agree_h / 100; res["nn_matches_tree_nn_R64"] = agree_e / 100
print(json.dumps({k: v for k, v in res.items() if k.startswith("nn_")}))
json.dump(res, open("/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/math2/e5_busemann.json", "w"), indent=1)
