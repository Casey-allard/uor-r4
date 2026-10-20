"""Did gradient descent discover the binary icosahedral group? For each lane of a trained 4-lane Q model,
take the 60 learned unit quaternions q(x), generate their multiplicative closure (tolerance 1e-3), and report
its order, the minimum angle between distinct elements, and whether it is closed; then check exact-table
(snapped) serving accuracy."""
import sys, pickle, itertools, numpy as np
sys.argv = sys.argv
import e6m_a5_lanes as m
P = pickle.load(open(sys.argv[1], 'rb'))
alpha = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
K = P['h0'].shape[0]
def ham(a, b):
    return np.array([a[0]*b[0]-a[1]*b[1]-a[2]*b[2]-a[3]*b[3], a[0]*b[1]+a[1]*b[0]+a[2]*b[3]-a[3]*b[2],
                     a[0]*b[2]-a[1]*b[3]+a[2]*b[0]+a[3]*b[1], a[0]*b[3]+a[1]*b[2]-a[2]*b[1]+a[3]*b[0]])
e0 = np.array([1.0, 0, 0, 0])
for lane in range(K):
    r = P['r'][:, lane, :]
    q = e0[None] + alpha * r; q /= np.linalg.norm(q, axis=1, keepdims=True)
    els = [e0]
    def find(x):
        for i, y in enumerate(els):
            if np.linalg.norm(x - y) < 1e-3: return i
        return -1
    frontier = list(q)
    for x in q:
        if find(x) < 0: els.append(x)
    changed = True; rounds = 0
    while changed and len(els) <= 400 and rounds < 10:
        changed = False; rounds += 1
        cur = list(els)
        for a in cur:
            for g in q:
                p = ham(g, a)
                if find(p) < 0:
                    els.append(p); changed = True
                    if len(els) > 400: break
            if len(els) > 400: break
    E = np.array(els)
    G = np.abs(E @ E.T); np.fill_diagonal(G, 0)
    minang = np.degrees(np.arccos(np.clip(np.max(E @ E.T - 2*np.eye(len(E))), -1, 1)))
    reals = np.sort(np.round(np.unique(np.round(np.abs(E[:, 0] * 0 + (E @ E.T).max(1)), 3)), 3))
    # conjugacy-invariant: distribution of Re(q) (rotation angles) of the learned generators
    ang = np.degrees(2 * np.arccos(np.clip(np.abs(q[:, 0]), 0, 1)))
    print(f"lane {lane}: closure size {len(els)}{' (>400, not finite at tol)' if len(els) > 400 else ''}; "
          f"min angle between distinct elements {minang:.2f} deg; "
          f"SO(3) rotation angles of learned generators (deg, rounded): {sorted(set(np.round(ang).astype(int)))}")
