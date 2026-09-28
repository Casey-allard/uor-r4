"""Covering test for icosahedral golden-gate rotation codebooks (lead scratch; not a project artifact).

Codebooks on SO(3) (unit quaternions modulo sign):
  2I          : the 120 binary-icosahedral units (60 rotations)
  GG1         : words c0 * T * c1, c0,c1 in 2I, T = (2+phi) i + j + k normalised (norm^2 = 7 + 5 phi, a prime of Z[phi])
  GG2         : words c0 T c1 T c2 (sampled subset if large)
  random-N    : N Haar-random rotations (same size as the golden-gate level)
Metric: rotation angle (degrees) from a Haar-random test rotation to the nearest codeword; mean, 99th percentile, max.
Also checks exactness: every GG1 codeword scaled by sqrt(7+5phi) has coordinates in (1/2)Z[phi] (integer pairs).
"""
import itertools, math, sys
import numpy as np

PHI = (1 + 5 ** 0.5) / 2
rng = np.random.default_rng(0)


def ham(a, b):
    aw, ax, ay, az = a[..., 0], a[..., 1], a[..., 2], a[..., 3]
    bw, bx, by, bz = b[..., 0], b[..., 1], b[..., 2], b[..., 3]
    return np.stack([aw * bw - ax * bx - ay * by - az * bz, aw * bx + ax * bw + ay * bz - az * by,
                     aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw], -1)


def two_i():
    pts = []
    for i in range(4):
        for s in (1, -1):
            v = [0.0] * 4; v[i] = s; pts.append(v)
    pts += [list(s) for s in itertools.product((0.5, -0.5), repeat=4)]
    base = [0.0, 0.5, 1 / (2 * PHI), PHI / 2]
    for perm in itertools.permutations(range(4)):
        if sum(perm[i] > perm[j] for i in range(4) for j in range(i + 1, 4)) % 2:
            continue
        for sg in itertools.product((1, -1), repeat=4):
            v = [0.0] * 4
            for k in range(4):
                v[perm[k]] = base[k] * sg[k]
            pts.append(v)
    c = np.unique(np.round(np.array(pts), 12), axis=0)
    assert c.shape == (120, 4)
    return c


def canon(q):
    # identify q ~ -q (SO(3)); choose first nonzero coordinate positive, then dedupe
    q = q.copy()
    s = np.sign(q[np.arange(len(q)), np.argmax(np.abs(q) > 1e-9, axis=1)])
    q *= s[:, None]
    return np.unique(np.round(q, 9), axis=0)


def haar(n):
    x = rng.normal(size=(n, 4))
    return x / np.linalg.norm(x, axis=1, keepdims=True)


def nearest_angle_deg(test, code, chunk=2000):
    out = np.empty(len(test))
    for i in range(0, len(test), chunk):
        d = np.abs(test[i:i + chunk] @ code.T).max(axis=1)
        out[i:i + chunk] = np.degrees(2 * np.arccos(np.clip(d, -1, 1)))
    return out


def stats(name, code, test):
    a = nearest_angle_deg(test, code)
    print(f"{name:28s} N_rot={len(code):8d} bits={math.log2(len(code)):6.2f}  mean={a.mean():7.3f}  p99={np.percentile(a, 99):7.3f}  max={a.max():7.3f} deg")
    return a


def main():
    c = two_i()
    T = np.array([0.0, 2 + PHI, 1.0, 1.0]); nT = math.sqrt(7 + 5 * PHI)
    assert abs(np.dot(T, T) - (7 + 5 * PHI)) < 1e-12
    That = T / nT
    g1 = ham(ham(c[:, None, None, :], That[None, None, None, :]), c[None, :, None, :]).reshape(-1, 4)
    g1 = canon(g1)
    # exactness: sqrt(7+5phi) * codeword in (1/2) Z[phi]: each coordinate 2x = a + b phi with integers a,b
    raw = g1 * nT * 2
    ok = True
    for x in raw.ravel():
        b = round((x - round(x - 0 * PHI)) / PHI) if False else None
        # solve x = a + b*phi with integers via nearest lattice point search on small b
        found = False
        for bb in range(-12, 13):
            a = x - bb * PHI
            if abs(a - round(a)) < 1e-7:
                found = True; break
        ok &= found
    print("GG1 exact (1/2)Z[phi] coordinates after scaling by sqrt(7+5phi):", ok)
    test = haar(40000)
    rot2i = canon(c)
    stats("2I (600-cell rotations)", rot2i, test)
    a1 = stats("golden-gate level 1 (c T c)", g1, test)
    stats("random, same N as level 1", canon(haar(len(g1))), test)
    # level 2 (sampled): c0 T c1 T c2
    idx = rng.integers(0, 120, size=(60000, 3))
    w = ham(ham(ham(ham(c[idx[:, 0]], That[None]), c[idx[:, 1]]), That[None]), c[idx[:, 2]])
    g2 = canon(w)
    stats("golden-gate level 2 (sample)", g2, test)
    stats("random, same N as level-2 sample", canon(haar(len(g2))), test)
    print("theory: 60 * 60 = 3600 rotations at T-count 1 (Parzanchevski-Sarnak N(1)=|C|^2); level-1 found:", len(g1))


if __name__ == "__main__":
    main()
