"""Quantization stress-test library (scratch analysis only; not project code).

All quantizers are simulated in float64 numpy: encode -> decode -> reconstruction.
Bit accounting is exact fixed-rate mixed-radix: bits/vector = sum(log2 |alphabet|)
over every stored symbol + side bits (norms, scales, radii), rounded up per vector.
"""
import itertools
import math
import numpy as np

PHI = (1 + 5 ** 0.5) / 2

# --------------------------------------------------------------------------
# quaternion helpers and H4 / 2I codebooks
# --------------------------------------------------------------------------

def qmul(a, b):
    a0, a1, a2, a3 = np.moveaxis(a, -1, 0)
    b0, b1, b2, b3 = np.moveaxis(b, -1, 0)
    return np.stack([
        a0 * b0 - a1 * b1 - a2 * b2 - a3 * b3,
        a0 * b1 + a1 * b0 + a2 * b3 - a3 * b2,
        a0 * b2 - a1 * b3 + a2 * b0 + a3 * b1,
        a0 * b3 + a1 * b2 - a2 * b1 + a3 * b0,
    ], axis=-1)


def _perm_parity(p):
    p = list(p)
    parity = 0
    for i in range(len(p)):
        for j in range(i + 1, len(p)):
            if p[i] > p[j]:
                parity ^= 1
    return parity


def dedupe(points, decimals=9):
    keys = np.round(points, decimals)
    _, idx = np.unique(keys, axis=0, return_index=True)
    return points[np.sort(idx)]


def hurwitz24():
    """Binary tetrahedral group 2T = 24 unit Hurwitz quaternions (a 24-cell)."""
    pts = []
    for i in range(4):
        for s in (1.0, -1.0):
            v = np.zeros(4)
            v[i] = s
            pts.append(v)
    for signs in itertools.product((1.0, -1.0), repeat=4):
        pts.append(np.array(signs) / 2)
    return np.array(pts)


def d4_roots24():
    """D4 root system normalized: (+-1,+-1,0,0)/sqrt2 and permutations (a rotated 24-cell)."""
    pts = []
    for i, j in itertools.combinations(range(4), 2):
        for si in (1.0, -1.0):
            for sj in (1.0, -1.0):
                v = np.zeros(4)
                v[i] = si
                v[j] = sj
                pts.append(v / math.sqrt(2))
    return np.array(pts)


def icosians120(parity=0):
    """Binary icosahedral group 2I = 120 unit icosians = vertices of the 600-cell."""
    base = list(hurwitz24())
    vals = np.array([0.0, 1.0, 1.0 / PHI, PHI]) / 2
    for perm in itertools.permutations(range(4)):
        if _perm_parity(perm) != parity:
            continue
        for signs in itertools.product((1.0, -1.0), repeat=3):
            v = np.zeros(4)
            s = (1.0,) + signs
            for k in range(4):
                v[perm[k]] = vals[k] * s[k]
            base.append(v)
    return dedupe(np.array(base))


def is_group(points, tol=1e-9):
    prod = qmul(points[:, None, :], points[None, :, :]).reshape(-1, 4)
    keys = set(map(tuple, np.round(points, 7)))
    return all(tuple(r) in keys for r in np.round(prod, 7))


def two_I():
    for parity in (0, 1):
        pts = icosians120(parity)
        if len(pts) == 120 and is_group(pts):
            return pts
    raise RuntimeError("failed to build 2I")


def cells_600(V):
    """Edges, triangles and tetrahedral cells of the 600-cell with vertex set V (120 x 4)."""
    G = V @ V.T
    adj = np.abs(G - PHI / 2) < 1e-9
    n = len(V)
    edges = [(i, j) for i in range(n) for j in range(i + 1, n) if adj[i, j]]
    tris = []
    for i, j in edges:
        common = np.nonzero(adj[i] & adj[j])[0]
        for k in common:
            if k > j:
                tris.append((i, j, k))
    cells = []
    for i, j, k in tris:
        common = np.nonzero(adj[i] & adj[j] & adj[k])[0]
        for l in common:
            if l > k:
                cells.append((i, j, k, l))
    return np.array(edges), np.array(tris), np.array(cells)


def normalize_rows(P):
    return P / np.linalg.norm(P, axis=1, keepdims=True)


def geodesic_600(V, cells, f):
    """Normalized barycentric grid of frequency f on every tetrahedral cell of the 600-cell."""
    pts = []
    combos = [c for c in itertools.product(range(f + 1), repeat=4) if sum(c) == f]
    W = np.array(combos, dtype=float) / f  # (m,4)
    for cell in cells:
        P = W @ V[cell]
        pts.append(P)
    P = normalize_rows(np.concatenate(pts))
    return dedupe(P, 8)


def h4_codes():
    """Dictionary name -> unit codebook on S^3 (all derived from 2I / 600-cell geometry)."""
    V = two_I()
    edges, tris, cells = cells_600(V)
    assert len(edges) == 720 and len(tris) == 1200 and len(cells) == 600, (len(edges), len(tris), len(cells))
    codes = {
        "2T24": hurwitz24(),
        "D4r24": d4_roots24(),
        "2I120": V,
        "C600": dedupe(normalize_rows(V[cells].mean(axis=1))),  # 120-cell vertices
        "R720": dedupe(normalize_rows(V[edges].mean(axis=1))),   # rectified 600-cell
        "F1200": dedupe(normalize_rows(V[tris].mean(axis=1))),   # face centroids
    }
    for f in (2, 3, 4, 5):
        g = geodesic_600(V, cells, f)
        codes[f"G{len(g)}"] = g
    return codes


def random_code(n, rng):
    return normalize_rows(rng.standard_normal((n, 4)))


def spherical_kmeans(samples, n, rng, iters=40, init=None):
    C = normalize_rows(samples[rng.choice(len(samples), n, replace=False)]) if init is None else init.copy()
    for _ in range(iters):
        idx = nearest_by_ip(samples, C)
        S = np.zeros_like(C)
        np.add.at(S, idx, samples)
        cnt = np.bincount(idx, minlength=n)
        empty = cnt == 0
        S[empty] = samples[rng.choice(len(samples), empty.sum(), replace=False)]
        C = normalize_rows(S)
    return C


def nearest_by_ip(Y, C, chunk=4096):
    """argmax_j <y, c_j> (for unit codebooks: nearest point)."""
    out = np.empty(len(Y), dtype=np.int64)
    Ct = C.T.astype(np.float64)
    for s in range(0, len(Y), chunk):
        out[s:s + chunk] = np.argmax(Y[s:s + chunk] @ Ct, axis=1)
    return out


def nearest_by_l2(Y, C, chunk=4096):
    out = np.empty(len(Y), dtype=np.int64)
    Ct = C.T
    c2 = (C * C).sum(1)
    for s in range(0, len(Y), chunk):
        out[s:s + chunk] = np.argmin(c2[None, :] - 2 * (Y[s:s + chunk] @ Ct), axis=1)
    return out


# --------------------------------------------------------------------------
# 1-D Lloyd-Max
# --------------------------------------------------------------------------

def lloyd1d(samples, L, iters=300, symmetric=False):
    s = np.sort(np.asarray(samples, dtype=np.float64))
    if L == 1:
        return np.array([s.mean()])
    c = np.quantile(s, (np.arange(L) + 0.5) / L)
    cs = np.concatenate([[0.0], np.cumsum(s)])
    for _ in range(iters):
        t = (c[:-1] + c[1:]) / 2
        idx = np.searchsorted(s, t)
        b = np.concatenate([[0], idx, [len(s)]])
        cnt = np.diff(b)
        sums = cs[b[1:]] - cs[b[:-1]]
        new = np.where(cnt > 0, sums / np.maximum(cnt, 1), c)
        if symmetric:
            new = (new - new[::-1]) / 2
        if np.max(np.abs(new - c)) < 1e-12:
            c = new
            break
        c = np.sort(new)
    return c


class Scalar:
    def __init__(self, levels):
        self.c = np.asarray(levels, dtype=np.float64)
        self.t = (self.c[:-1] + self.c[1:]) / 2

    def index(self, x):
        return np.searchsorted(self.t, x)

    def __call__(self, x):
        return self.c[self.index(x)]

    @property
    def L(self):
        return len(self.c)


# --------------------------------------------------------------------------
# rotations
# --------------------------------------------------------------------------

def hadamard(d):
    H = np.array([[1.0]])
    while H.shape[0] < d:
        H = np.block([[H, H], [H, -H]])
    assert H.shape[0] == d
    return H


def rotation(kind, d, rng):
    if kind == "none":
        return None
    if kind == "qr":
        A = rng.standard_normal((d, d))
        Q, R = np.linalg.qr(A)
        Q = Q * np.sign(np.diag(R))[None, :]
        return Q
    if kind == "rht":
        s = rng.choice([-1.0, 1.0], size=d)
        return hadamard(d) * s[None, :] / math.sqrt(d)  # H diag(s)/sqrt(d): orthogonal
    raise ValueError(kind)


# --------------------------------------------------------------------------
# lattices
# --------------------------------------------------------------------------

def decode_Dn(z):
    f = np.round(z)
    odd = (np.sum(f, axis=1) % 2) != 0
    if np.any(odd):
        zo, fo = z[odd], f[odd]
        err = zo - fo
        k = np.argmax(np.abs(err), axis=1)
        r = np.arange(len(zo))
        step = np.where(err[r, k] >= 0, 1.0, -1.0)
        step[err[r, k] == 0] = 1.0
        fo[r, k] += step
        f[odd] = fo
    return f


def decode_E8(z):
    a = decode_Dn(z)
    b = decode_Dn(z - 0.5) + 0.5
    da = ((z - a) ** 2).sum(1)
    db = ((z - b) ** 2).sum(1)
    return np.where((da <= db)[:, None], a, b)


def sigma3(m):
    return sum(d ** 3 for d in range(1, m + 1) if m % d == 0)


def e8_ball_count(M):
    """# E8 points with squared norm <= 2M (theta series 1 + 240 sum sigma3)."""
    return 1 + 240 * sum(sigma3(m) for m in range(1, M + 1))


def d4_ball_count(R2):
    r = int(math.isqrt(int(R2))) + 1
    g = np.array(list(itertools.product(range(-r, r + 1), repeat=4)), dtype=float)
    g = g[(g.sum(1) % 2) == 0]
    return int(((g * g).sum(1) <= R2 + 1e-9).sum())


class LatticeBall:
    """Nearest lattice point inside a ball of squared radius R2 (lattice units), scale s."""

    def __init__(self, kind, R2):
        self.kind = kind
        self.R2 = R2
        self.k = 8 if kind == "E8" else 4
        if kind == "E8":
            assert R2 % 2 == 0
            self.N = e8_ball_count(R2 // 2)
        else:
            self.N = d4_ball_count(R2)
        self.s = 1.0

    def _decode(self, z):
        return decode_E8(z) if self.kind == "E8" else decode_Dn(z)

    def points(self):
        """Enumerate the ball codebook (only used when it is small)."""
        if getattr(self, "_pts", None) is not None:
            return self._pts
        r = int(math.isqrt(int(self.R2))) + 1

        def ball(vals):
            part = np.zeros((1, 0))
            nrm = np.zeros(1)
            for _ in range(self.k):
                ext = np.repeat(part, len(vals), axis=0)
                col = np.tile(vals, len(part))
                n2 = np.repeat(nrm, len(vals)) + col * col
                keep = n2 <= self.R2 + 1e-9
                part = np.concatenate([ext[keep], col[keep, None]], axis=1)
                nrm = n2[keep]
            return part[(part.sum(1) % 2) == 0]

        pts = [ball(np.arange(-r, r + 1, dtype=np.float64))]
        if self.kind == "E8":
            pts.append(ball(np.arange(-r - 0.5, r + 1.0, 1.0)))
        P = np.concatenate(pts)
        assert len(P) == self.N, (len(P), self.N)
        self._pts = P
        return P

    def quant(self, Y, s=None, exact=True):
        """Nearest lattice point; points decoding outside the ball are re-decoded from a
        grid of radially shrunk inputs and the in-ball candidate closest to the input kept
        (exact=True and a small ball: brute force over the enumerated ball instead)."""
        s = self.s if s is None else s
        z = Y / s
        lam = self._decode(z)
        out = np.nonzero((lam * lam).sum(1) > self.R2 + 1e-9)[0]
        if len(out) and exact and self.N <= 60000:
            P = self.points()
            c2 = (P * P).sum(1)
            for st in range(0, len(out), 2048):
                ii = out[st:st + 2048]
                lam[ii] = P[np.argmin(c2[None, :] - 2 * (z[ii] @ P.T), axis=1)]
            return lam * s
        if len(out):
            zz = z[out]
            nrm = np.linalg.norm(zz, axis=1, keepdims=True)
            best = np.zeros_like(zz)
            bestd = ((zz - best) ** 2).sum(1)  # origin is always in the ball
            rad = math.sqrt(self.R2)
            for f in np.linspace(1.15, 0.3, 18):
                cand = self._decode(zz * np.minimum(1.0, f * rad / np.maximum(nrm, 1e-12)))
                inside = (cand * cand).sum(1) <= self.R2 + 1e-9
                dist = ((zz - cand) ** 2).sum(1)
                better = inside & (dist < bestd)
                best[better] = cand[better]
                bestd[better] = dist[better]
            lam[out] = best
        return lam * s

    def fit(self, Ytrain):
        if len(Ytrain) > 16000:
            Ytrain = Ytrain[np.random.default_rng(0).choice(len(Ytrain), 16000, replace=False)]
        best = None
        base = math.sqrt(np.mean((Ytrain ** 2).sum(1))) / math.sqrt(self.R2 + 1e-9)
        for f in np.geomspace(0.35, 2.8, 13):
            s = base * f
            err = np.mean(((Ytrain - self.quant(Ytrain, s, exact=False)) ** 2).sum(1))
            if best is None or err < best[0]:
                best = (err, s)
        # refine
        s0 = best[1]
        for f in np.geomspace(0.85, 1.15, 9):
            s = s0 * f
            err = np.mean(((Ytrain - self.quant(Ytrain, s, exact=False)) ** 2).sum(1))
            if err < best[0]:
                best = (err, s)
        self.s = best[1]
        return self

    @property
    def bits(self):
        return math.log2(self.N)

    def __call__(self, Y):
        return self.quant(Y, exact=True)
