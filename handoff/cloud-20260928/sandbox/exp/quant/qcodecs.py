"""Block quantizers, vector codecs, datasets, stage-1 sweep and stage-2 metrics."""
import math
import numpy as np
from quantlib import (Scalar, lloyd1d, rotation, LatticeBall, normalize_rows, h4_codes,
                      random_code, spherical_kmeans, nearest_by_l2)

D = 64
NORM_BITS = 8

# --------------------------------------------------------------------------
# datasets
# --------------------------------------------------------------------------
OUT_CH = np.array([3, 17, 40, 58])


def make_data(name, n, rng, d=D):
    if name == "gauss":
        return rng.standard_normal((n, d))
    if name == "t3":  # iid heavy-tailed coordinates
        return rng.standard_t(3, size=(n, d))
    if name == "mvt3":  # isotropic direction, heavy-tailed radius (multivariate t, nu=3)
        g = rng.standard_normal((n, d))
        w = np.sqrt(3.0 / rng.chisquare(3, size=(n, 1)))
        return g * w
    if name == "outlier":  # 4 fixed outlier channels: large mean offset + large std (KV-key-like)
        x = rng.standard_normal((n, d))
        sign = np.array([1.0, -1.0, 1.0, -1.0])
        x[:, OUT_CH] = 6.0 * sign[None, :] + 3.0 * rng.standard_normal((n, len(OUT_CH)))
        return x
    if name == "aniso":  # embedding-like cone: common mean + power-law spectrum, random basis
        r2 = np.random.default_rng(12345)
        Q, _ = np.linalg.qr(r2.standard_normal((d, d)))
        lam = 1.0 / np.arange(1, d + 1)
        lam = lam / lam.sum() * d
        mu = Q[:, -1] * math.sqrt(0.5 * d)  # mean along a low-variance direction
        z = rng.standard_normal((n, d)) * np.sqrt(lam)[None, :]
        return z @ Q.T + mu[None, :]
    raise ValueError(name)


DATASETS = ["gauss", "t3", "mvt3", "outlier", "aniso"]

# --------------------------------------------------------------------------
# norm side-information quantizer
# --------------------------------------------------------------------------


class LogNormQ:
    def __init__(self, bits=NORM_BITS):
        self.bits = bits

    def fit(self, n):
        ln = np.log(np.maximum(n, 1e-30))
        self.lo = ln.min() - 0.5
        self.hi = ln.max() + 0.5
        return self

    def __call__(self, n):
        L = 2 ** self.bits
        ln = np.clip(np.log(np.maximum(n, 1e-30)), self.lo, self.hi)
        step = (self.hi - self.lo) / L
        idx = np.clip(np.floor((ln - self.lo) / step), 0, L - 1)
        return np.exp(self.lo + (idx + 0.5) * step)


# --------------------------------------------------------------------------
# block quantizers (operate on (n, k) arrays in the preprocessed domain)
# --------------------------------------------------------------------------


class LloydScalarQ:
    k = 1

    def __init__(self, L, symmetric=True):
        self.L = L
        self.sym = symmetric

    def fit(self, B):
        self.q = Scalar(lloyd1d(B.ravel(), self.L, symmetric=self.sym))
        return self

    @property
    def bits(self):
        return math.log2(self.L)

    def __call__(self, B):
        return self.q(B)


def ip_argmax(Y, C, chunk=4096):
    idx = np.empty(len(Y), dtype=np.int64)
    val = np.empty(len(Y))
    Ct = C.T.astype(np.float32)
    Yf = Y.astype(np.float32)
    for s in range(0, len(Y), chunk):
        G = Yf[s:s + chunk] @ Ct
        j = np.argmax(G, axis=1)
        idx[s:s + chunk] = j
    # exact projection in float64
    val[:] = np.einsum("ij,ij->i", Y, C[idx])
    return idx, val


class GainShapeQ:
    """Sabin-Gray gain-shape VQ: shape = argmax <y, c>, gain = Lloyd-quantized projection."""

    def __init__(self, code, Lg, name=""):
        self.C = code
        self.k = code.shape[1]
        self.Lg = Lg
        self.name = name

    def fit(self, B, proj=None):
        if proj is None:
            _, proj = ip_argmax(B, self.C)
        self.g = Scalar(lloyd1d(proj, self.Lg))
        return self

    @property
    def bits(self):
        return math.log2(len(self.C) * self.Lg)

    def encode(self, B):
        idx, p = ip_argmax(B, self.C)
        return idx, self.g.index(p)

    def __call__(self, B, cached=None):
        idx, p = ip_argmax(B, self.C) if cached is None else cached
        return self.g(p)[:, None] * self.C[idx]


class Polar2Q:
    k = 2

    def __init__(self, Lt, Lg):
        self.Lt, self.Lg = Lt, Lg

    def _shape(self, B):
        th = np.arctan2(B[:, 1], B[:, 0])
        step = 2 * math.pi / self.Lt
        j = np.round(th / step) % self.Lt
        thq = j * step
        p = B[:, 0] * np.cos(thq) + B[:, 1] * np.sin(thq)
        return thq, p

    def fit(self, B):
        _, p = self._shape(B)
        self.g = Scalar(lloyd1d(p, self.Lg))
        return self

    @property
    def bits(self):
        return math.log2(self.Lt * self.Lg)

    def __call__(self, B):
        thq, p = self._shape(B)
        g = self.g(p)
        return np.stack([g * np.cos(thq), g * np.sin(thq)], axis=1)


class LatticeQ:
    def __init__(self, kind, R2):
        self.lat = LatticeBall(kind, R2)
        self.k = self.lat.k
        self.kind = kind
        self.R2 = R2

    def fit(self, B):
        self.lat.fit(B)
        return self

    @property
    def bits(self):
        return self.lat.bits

    def __call__(self, B):
        return self.lat(B)


class VQQ:
    k = 4

    def __init__(self, codebook):
        self.C = codebook

    def fit(self, B):
        return self

    @property
    def bits(self):
        return math.log2(len(self.C))

    def __call__(self, B):
        return self.C[nearest_by_l2(B, self.C)]


def kmeans(X, n, rng, iters=25):
    C = X[rng.choice(len(X), n, replace=False)].copy()
    for _ in range(iters):
        idx = nearest_by_l2(X, C)
        S = np.zeros_like(C)
        np.add.at(S, idx, X)
        cnt = np.bincount(idx, minlength=n)
        empty = cnt == 0
        C[~empty] = S[~empty] / cnt[~empty, None]
        if empty.any():
            C[empty] = X[rng.choice(len(X), empty.sum(), replace=False)]
    return C


def _polar_levels(B):
    """Recursive polar transform of 16-dim blocks (PolarQuant, L=4 levels)."""
    n = len(B)
    r = B
    angles = []
    for lvl in range(4):
        a, b = r[:, 0::2], r[:, 1::2]
        psi = np.arctan2(b, a)
        if lvl == 0:
            psi = np.mod(psi, 2 * math.pi)
        angles.append(psi)
        r = np.sqrt(a * a + b * b)
    return angles, r[:, 0]


def _polar_inverse(angles, radius):
    r = radius[:, None]
    for lvl in (3, 2, 1, 0):
        psi = angles[lvl]
        a = r * np.cos(psi)
        b = r * np.sin(psi)
        out = np.empty((len(r), 2 * r.shape[1]))
        out[:, 0::2] = a
        out[:, 1::2] = b
        r = out
    return r


class PolarQuantQ:
    """PolarQuant (Han et al. 2025) practical variant: 16-dim blocks, 4 recursion levels,
    level-1 angle uniform on [0,2pi), levels 2-4 Lloyd on [0,pi/2], block radius stored
    with br bits (Lloyd in log domain; the paper stores it in fp16)."""
    k = 16

    def __init__(self, L1, L2, L3, L4, br):
        self.Ls = (L1, L2, L3, L4)
        self.br = br

    def fit(self, B, design=None):
        angles, rad = _polar_levels(B if design is None else design)
        self.aq = [None]
        for lvl in (1, 2, 3):
            self.aq.append(Scalar(lloyd1d(angles[lvl].ravel(), self.Ls[lvl])))
        _, radB = _polar_levels(B)
        self.rq = Scalar(lloyd1d(np.log(np.maximum(radB, 1e-30)), 2 ** self.br))
        return self

    @property
    def bits(self):
        L1, L2, L3, L4 = self.Ls
        return 8 * math.log2(L1) + 4 * math.log2(L2) + 2 * math.log2(L3) + math.log2(L4) + self.br

    def __call__(self, B):
        angles, rad = _polar_levels(B)
        step = 2 * math.pi / self.Ls[0]
        qa = [np.floor(angles[0] / step) * step + step / 2]
        for lvl in (1, 2, 3):
            qa.append(self.aq[lvl](angles[lvl]))
        qr = np.exp(self.rq(np.log(np.maximum(rad, 1e-30))))
        return _polar_inverse(qa, qr)


# --------------------------------------------------------------------------
# vector codec: rotation + optional vector-norm side info + per-position blocks
# --------------------------------------------------------------------------


class Pre:
    def __init__(self, rot, norm_mode, seed, d=D, center=False):
        self.rotkind = rot
        self.R = rotation(rot, d, np.random.default_rng(seed))
        self.norm_mode = norm_mode
        self.d = d
        self.center = center
        self.mu = np.zeros(d)

    def fit_center(self, Xtr):
        if self.center:
            self.mu = Xtr.mean(0)

    def fwd(self, X):
        X = X - self.mu
        Y = X if self.R is None else X @ self.R.T
        if self.norm_mode == "vec":
            n = np.linalg.norm(X, axis=1)
            return Y * (math.sqrt(self.d) / n)[:, None], n
        return Y, np.linalg.norm(X, axis=1)

    def inv(self, Yhat, nhat):
        Z = Yhat if self.norm_mode != "vec" else Yhat * (nhat / math.sqrt(self.d))[:, None]
        return (Z if self.R is None else Z @ self.R) + self.mu

    @property
    def side_bits(self):
        return NORM_BITS if self.norm_mode == "vec" else 0


def blocks(Y, k):
    n, d = Y.shape
    return Y.reshape(n, d // k, k)


# --------------------------------------------------------------------------
# metrics
# --------------------------------------------------------------------------


def metrics(X, Xh, Q, topk=10):
    err = ((X - Xh) ** 2).sum(1)
    nx = (X * X).sum(1)
    rel = float(np.mean(err / nx))
    nmse = float(err.sum() / nx.sum())
    cos = float(np.mean((X * Xh).sum(1) / np.sqrt(nx * np.maximum((Xh * Xh).sum(1), 1e-300))))
    S = Q @ X.T
    Sh = Q @ Xh.T
    e = Sh - S
    scale = np.mean(np.abs(S))
    ip_abs = float(np.mean(np.abs(e)) / scale)
    ip_bias = float(np.mean(e) / scale)
    slope = float((Sh * S).sum() / (S * S).sum())
    true_top = np.argsort(-S, axis=1)[:, :topk]
    appr_top = np.argsort(-Sh, axis=1)[:, :topk]
    rec = np.mean([len(set(a) & set(b)) / topk for a, b in zip(true_top, appr_top)])
    r1 = np.mean([t[0] in set(a) for t, a in zip(true_top, appr_top)])
    return dict(relmse=rel, nmse=nmse, cos=cos, ip_abs=ip_abs, ip_bias=ip_bias,
                ip_slope=slope, recall10=float(rec), r1at10=float(r1))
