"""E9: steps until quantized quaternion transport (project grids: unit 2^-14, state 2^-11, no renormalization)
misidentifies the exact 2I product (decoded up to sign, i.e. as an A5 element)."""
import itertools, numpy as np
PHI = (1 + 5 ** 0.5) / 2
pts = set()
for k in range(4):
    for s in (1, -1):
        v = [0.0] * 4; v[k] = s; pts.add(tuple(v))
for s in itertools.product((0.5, -0.5), repeat=4): pts.add(s)
base = [0.0, 0.5, PHI / 2, (PHI - 1) / 2]
ev = [p for p in itertools.permutations(range(4)) if sum(1 for i in range(4) for j in range(i + 1, 4) if p[i] > p[j]) % 2 == 0]
for p in ev:
    for s1, s2, s3 in itertools.product((1, -1), repeat=3):
        sg = [base[0], s1 * base[1], s2 * base[2], s3 * base[3]]
        pts.add(tuple(round(sg[p[i]], 12) for i in range(4)))
G = np.array(sorted(pts)); assert len(G) == 120
def ham(q, x):
    w, a, b, c = q.T; x0, x1, x2, x3 = x.T
    return np.stack([w*x0-a*x1-b*x2-c*x3, w*x1+a*x0+b*x3-c*x2, w*x2-a*x3+b*x0+c*x1, w*x3+a*x2-b*x1+c*x0], 1)
rng = np.random.default_rng(0)
B, T = 512, 200_000
checks = np.unique(np.round(np.logspace(1, np.log10(T), 60)).astype(int))
for scale in (1.0, 4.0):
    for mode in ('unit only', 'unit+state'):
        exact = np.tile(np.array([1.0, 0, 0, 0]), (B, 1)) * scale
        approx = exact.copy(); first_fail = np.full(B, np.inf)
        qg = np.clip(np.round(G * 2**14), -32767, 32767) / 2**14
        for t in range(1, T + 1):
            k = rng.integers(0, 120, B)
            exact = ham(G[k], exact)
            approx = ham(qg[k], approx)
            if mode == 'unit+state':
                approx = np.clip(np.round(approx * 2**11), -32767, 32767) / 2**11
            if t in checks:
                de = np.abs(exact @ G.T).argmax(1)            # decode up to sign (A5 class)
                da = np.abs(approx @ G.T).argmax(1)
                bad = (de != da) & np.isinf(first_fail)
                first_fail[bad] = t
                if exact.shape and t % 20000 == 0: exact = G[np.abs(exact @ G.T).argmax(1)] * scale * np.sign((exact * G[np.abs(exact @ G.T).argmax(1)]).sum(1, keepdims=True))
        ff = first_fail[np.isfinite(first_fail)]
        print(f"lane norm {scale}, {mode:10s}: failed within {T} steps: {len(ff)}/{B}; "
              f"first failure step min {ff.min() if len(ff) else None}, median {np.median(ff) if len(ff) else None}; "
              f"final norm ratio mean {np.linalg.norm(approx,axis=1).mean()/scale:.4f}")
