"""E4: quantization geometry facts.
(a) normalized second moment G of Z^n, A3* (BCC), D4, E8 by Monte Carlo (Conway-Sloane decoders);
(b) the 600-cell (2I) as a 120-point quantizer of uniform directions on S^3 vs a Lloyd-optimized
    120-point code, a Hopf-coordinate product grid with ~120 cells, the 24-cell, and random codes.
"""
import itertools, numpy as np
rng = np.random.default_rng(0)
M = 400_000
def G_of(decode, n, vol):
    x = rng.uniform(0, 8, size=(M, n))
    e = x - decode(x)
    return (e ** 2).sum(1).mean() / n / vol ** (2 / n)
def dec_Z(x): return np.round(x)
def dec_D(x):
    f = np.round(x); s = f.sum(1) % 2 != 0
    err = x - f; k = np.argmax(np.abs(err), axis=1)
    rows = np.nonzero(s)[0]
    f[rows, k[rows]] += np.sign(err[rows, k[rows]] + 1e-300)
    return f
def dec_E8(x):
    a = dec_D(x); b = dec_D(x - 0.5) + 0.5
    da = ((x - a) ** 2).sum(1); db = ((x - b) ** 2).sum(1)
    return np.where((da <= db)[:, None], a, b)
def dec_BCC(x):
    a = np.round(x); b = np.round(x - 0.5) + 0.5
    da = ((x - a) ** 2).sum(1); db = ((x - b) ** 2).sum(1)
    return np.where((da <= db)[:, None], a, b)
res = {'Z^n (cubic)': G_of(dec_Z, 4, 1.0), 'A3* (BCC, 3-D)': G_of(dec_BCC, 3, 0.5),
       'D4 (24-cell lattice)': G_of(dec_D, 4, 2.0), 'E8': G_of(dec_E8, 8, 1.0)}
print("(a) normalized second moment G (lower is better; 1/12=0.08333, sphere bound limit 1/(2 pi e)=0.05855)")
for k, v in res.items():
    print(f"   {k:22s} G = {v:.5f}   gain over cubic = {10*np.log10((1/12)/v):.3f} dB "
          f"= {0.5*np.log2((1/12)/v):.3f} bit/dim")

# (b) spherical codes on S^3
PHI = (1 + 5 ** 0.5) / 2
def cell600():
    pts = set()
    for k in range(4):
        for s in (1, -1):
            v = [0.0] * 4; v[k] = s; pts.add(tuple(v))
    for s in itertools.product((0.5, -0.5), repeat=4): pts.add(s)
    base = [0.0, 0.5, PHI / 2, (PHI - 1) / 2]
    ev = [p for p in itertools.permutations(range(4))
          if sum(1 for i in range(4) for j in range(i + 1, 4) if p[i] > p[j]) % 2 == 0]
    for p in ev:
        for s1, s2, s3 in itertools.product((1, -1), repeat=3):
            sg = [base[0], s1 * base[1], s2 * base[2], s3 * base[3]]
            pts.add(tuple(round(sg[p[i]], 12) for i in range(4)))
    return np.array(sorted(pts))
C600 = cell600(); assert len(C600) == 120
C24 = C600[np.abs(C600).max(1) > 0.99].tolist() + [c for c in C600.tolist() if all(abs(abs(v) - 0.5) < 1e-9 for v in c)]
C24 = np.array(C24); assert len(C24) == 24
X = rng.normal(size=(300_000, 4)); X /= np.linalg.norm(X, axis=1, keepdims=True)
def mse(C, X=X):
    return (2 - 2 * (X @ C.T).max(1)).mean()   # E|x - c|^2 for unit x, unit c
def lloyd(K, iters=60, seed=0):
    r = np.random.default_rng(seed)
    C = r.normal(size=(K, 4)); C /= np.linalg.norm(C, axis=1, keepdims=True)
    Xs = X[:150_000]
    for _ in range(iters):
        a = np.argmax(Xs @ C.T, axis=1)
        for k in range(K):
            m = Xs[a == k]
            if len(m): C[k] = m.mean(0)
        C /= np.linalg.norm(C, axis=1, keepdims=True)
    return C
best = min((lloyd(120, seed=s) for s in range(4)), key=mse)
# Hopf-coordinate product grid: z1 = cos(chi) e^{i t1}, z2 = sin(chi) e^{i t2}; chi bins equal-mass in sin^2 chi
def hopf_grid_codebook(nchi, per):
    """nchi equal-mass chi bins; in each, n1 x n2 phase bins with n1:n2 ~ cos:sin, n1*n2 ~ per."""
    chi = np.arcsin(np.sqrt(np.clip(X[:, 2] ** 2 + X[:, 3] ** 2, 0, 1)))
    t1 = np.arctan2(X[:, 1], X[:, 0]); t2 = np.arctan2(X[:, 3], X[:, 2])
    q = np.sin(chi) ** 2
    cb = np.minimum((q * nchi).astype(int), nchi - 1)
    codes = []; label = np.zeros(len(X), int); total = 0
    for b in range(nchi):
        cc = np.sqrt((b + 0.5) / nchi)  # sin(chi_center)
        ratio = np.sqrt(1 - cc ** 2) / max(cc, 1e-9)
        n1 = max(1, int(round(np.sqrt(per * ratio)))); n2 = max(1, int(round(per / n1)))
        sel = cb == b
        l1 = np.minimum(((t1[sel] + np.pi) / (2 * np.pi) * n1).astype(int), n1 - 1)
        l2 = np.minimum(((t2[sel] + np.pi) / (2 * np.pi) * n2).astype(int), n2 - 1)
        label[sel] = total + l1 * n2 + l2
        total += n1 * n2
    C = np.zeros((total, 4))
    for k in range(total):
        m = X[label == k]
        if len(m): C[k] = m.mean(0)
    C /= np.maximum(np.linalg.norm(C, axis=1, keepdims=True), 1e-12)
    # MSE of the grid itself (cell assignment by coordinates, reconstruction = cell centroid)
    grid_mse = (2 - 2 * (X * C[label]).sum(1)).mean()
    return total, grid_mse
print("\n(b) direction quantization of uniform points on S^3, E|x - c|^2 (unit vectors)")
print(f"   600-cell / 2I (120 pts, {np.log2(120):.2f} bits): {mse(C600):.5f}")
print(f"   Lloyd-optimized 120-pt code (best of 4 restarts): {mse(best):.5f}")
for nchi, per in [(3, 40), (4, 30), (2, 60)]:
    tot, gm = hopf_grid_codebook(nchi, per)
    print(f"   Hopf (chi,t1,t2) product grid, {tot} cells ({np.log2(tot):.2f} bits): {gm:.5f}")
rnd = np.mean([mse((lambda C: C / np.linalg.norm(C, axis=1, keepdims=True))(rng.normal(size=(120, 4)))) for _ in range(10)])
print(f"   random 120-pt code (mean of 10): {rnd:.5f}")
print(f"   24-cell / 2T (24 pts, {np.log2(24):.2f} bits): {mse(C24):.5f}")
print("   min angle 600-cell: %.2f deg; Lloyd code: %.2f deg" % (
    np.degrees(np.arccos(np.sort((C600 @ C600.T).ravel())[-121])),
    np.degrees(np.arccos(np.sort((best @ best.T).ravel())[-121]))))
