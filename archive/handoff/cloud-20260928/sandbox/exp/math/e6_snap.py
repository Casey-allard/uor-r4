"""Snap learned quaternion lanes to their generated finite group and serve by exact table lookup."""
import pickle, numpy as np, e6m_a5_lanes as m
P = pickle.load(open('params_Q_1.0_all60_0_K4.pkl', 'rb'))
e0 = np.array([1.0, 0, 0, 0]); K = P['h0'].shape[0]
def ham(a, b):
    return np.array([a[0]*b[0]-a[1]*b[1]-a[2]*b[2]-a[3]*b[3], a[0]*b[1]+a[1]*b[0]+a[2]*b[3]-a[3]*b[2],
                     a[0]*b[2]-a[1]*b[3]+a[2]*b[0]+a[3]*b[1], a[0]*b[3]+a[1]*b[2]-a[2]*b[1]+a[3]*b[0]])
Q = e0[None, None] + P['r']; Q = Q / np.linalg.norm(Q, axis=2, keepdims=True)   # (60, K, 4)
for lane in range(K):
    q = Q[:, lane]
    ang = np.degrees(2 * np.arccos(np.clip(np.abs(q[:, 0]), 0, 1)))
    hist = {a: int(np.sum(np.abs(ang - a) < 1.5)) for a in (0, 72, 120, 144, 180)}
    print(f"lane {lane}: learned generators by SO(3) angle (+-1.5 deg): {hist} (A5 class sizes: 0:1, 72:12, 120:20, 144:12, 180:15)")
# closure with tolerance 0.05 for the icosahedral lanes
for lane in (2, 3):
    q = Q[:, lane]; els = [e0]
    def find(x, tol=0.05):
        d = np.linalg.norm(np.array(els) - x, axis=1); i = int(np.argmin(d))
        return i if d[i] < tol else -1
    for _ in range(6):
        grew = False
        for a in list(els):
            for g in q:
                p = ham(g, a)
                if find(p) < 0: els.append(p); grew = True
        if not grew or len(els) > 300: break
    E = np.array(els)
    G = E @ E.T; np.fill_diagonal(G, -2)
    print(f"lane {lane}: closure size at tol 0.05 = {len(E)}; min angle between distinct elements "
          f"{np.degrees(np.arccos(G.max())):.2f} deg (2I: 36.00)")
    if len(E) == 120:
        # exact table serving: token -> element index; state index composed by nearest-element table
        tok = np.array([find(g) for g in q])
        T = np.array([[find(ham(E[i], E[j])) for j in range(120)] for i in range(120)])
        print(f"   token->element map complete: {bool((tok >= 0).all())}; product table closed: {bool((T >= 0).all())}")
        E_snap = E  # representatives
        # evaluate table-served lane (with the other lanes left continuous) at long lengths
        for L in (256, 1024):
            rs = np.random.default_rng(7); toks, lab = m.make_batch(rs, 1000, L, list(range(60)))
            A = np.array(m.trans_mats(P, 'Q', 1.0, 60)).reshape(60, K, 4, 4)
            h0 = P['h0'] / np.linalg.norm(P['h0'], axis=1, keepdims=True)
            h = np.tile(h0[None], (1000, 1, 1)); s = np.full(1000, find(e0))
            for t in range(L):
                h = np.einsum('bkij,bkj->bki', A[toks[:, t]], h)
                s = T[tok[toks[:, t]], s]                       # exact index composition (left action)
                hs = h.copy()
                hs[:, lane] = np.stack([ham(E_snap[i], h0[lane]) for i in s])  # replace lane by exact table state
            pred = np.argmax(np.array(m.features(hs)) @ P['W'].T + P['b'], axis=1)
            predc = np.argmax(np.array(m.features(h)) @ P['W'].T + P['b'], axis=1)
            print(f"   L={L}: accuracy with lane {lane} table-served: {np.mean(pred == lab[:, -1]):.3f}; continuous: {np.mean(predc == lab[:, -1]):.3f}")
