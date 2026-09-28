"""Rate-distortion check of the owner's original idea:
"like TurboQuant/PolarQuant but keep the radius and use 4-D blocks instead of 2-D".

Source: i.i.d. N(0,1) coordinates (the distribution TurboQuant/PolarQuant/QuIP# induce
by random rotation). Metric: MSE per dimension on a held-out test set.
Every quantizer uses centroid (MSE-optimal) decoding for its own encoder partition,
fitted on a training set and evaluated on an independent test set.
"""
import itertools
import numpy as np

rng = np.random.default_rng(12345)
N_TRAIN = 400_000
N_TEST = 400_000
PHI = (1 + 5 ** 0.5) / 2


def lloyd_1d(samples, levels, iters=200):
    qs = np.quantile(samples, (np.arange(levels) + 0.5) / levels)
    for _ in range(iters):
        edges = (qs[1:] + qs[:-1]) / 2
        idx = np.searchsorted(edges, samples)
        new = np.array([samples[idx == k].mean() if np.any(idx == k) else qs[k] for k in range(levels)])
        if np.allclose(new, qs, atol=1e-9):
            break
        qs = new
    return qs


def scalar_lloyd_mse(bits):
    tr = rng.standard_normal(N_TRAIN)
    te = rng.standard_normal(N_TEST)
    qs = lloyd_1d(tr, 2 ** bits)
    edges = (qs[1:] + qs[:-1]) / 2
    rec = qs[np.searchsorted(edges, te)]
    return float(np.mean((te - rec) ** 2))


def centroid_codec_mse(train_x, train_cell, test_x, test_cell, n_cells):
    """Optimal decoder for a fixed encoder partition: per-cell mean."""
    d = train_x.shape[1]
    sums = np.zeros((n_cells, d))
    np.add.at(sums, train_cell, train_x)
    counts = np.bincount(train_cell, minlength=n_cells).astype(float)
    cents = sums / np.maximum(counts, 1)[:, None]
    rec = cents[test_cell]
    return float(np.mean(np.sum((test_x - rec) ** 2, axis=1)) / d)


def polar2d_mse(total_bits_per_pair):
    """2-D polar product quantizer: b_r radius bits (Lloyd on radius) x b_t uniform angle bits."""
    best = None
    tr = rng.standard_normal((N_TRAIN, 2))
    te = rng.standard_normal((N_TEST, 2))
    for b_t in range(0, total_bits_per_pair + 1):
        b_r = total_bits_per_pair - b_t
        def cells(x, rq):
            r = np.linalg.norm(x, axis=1)
            th = np.mod(np.arctan2(x[:, 1], x[:, 0]), 2 * np.pi)
            ti = np.minimum((th / (2 * np.pi) * 2 ** b_t).astype(int), 2 ** b_t - 1)
            edges = (rq[1:] + rq[:-1]) / 2
            ri = np.searchsorted(edges, r)
            return ri * (2 ** b_t) + ti
        rq = lloyd_1d(np.linalg.norm(tr, axis=1), 2 ** b_r)
        mse = centroid_codec_mse(tr, cells(tr, rq), te, cells(te, rq), 2 ** total_bits_per_pair)
        if best is None or mse < best[0]:
            best = (mse, b_r, b_t)
    return best


def cell24():
    pts = [np.array(v, float) for v in itertools.product([0], [0], [0], [1])]
    pts = []
    for i in range(4):
        for s in (1, -1):
            v = np.zeros(4); v[i] = s; pts.append(v)
    for signs in itertools.product((0.5, -0.5), repeat=4):
        pts.append(np.array(signs))
    return np.array(pts)


def even_perms4():
    out = []
    for p in itertools.permutations(range(4)):
        inv = sum(1 for i in range(4) for j in range(i + 1, 4) if p[i] > p[j])
        if inv % 2 == 0:
            out.append(p)
    return out


def cell600():
    pts = list(cell24())
    base = [0.0, 0.5, PHI / 2, 1 / (2 * PHI)]
    for p in even_perms4():
        for s in itertools.product((1, -1), repeat=3):
            v = np.array([base[p[0]], base[p[1]], base[p[2]], base[p[3]]])
            # apply signs to the three nonzero entries (base[0] = 0)
            nz = [k for k in range(4) if p[k] != 0]
            for k, sk in zip(nz, s):
                v[k] *= sk
            pts.append(v)
    pts = np.unique(np.round(np.array(pts), 12), axis=0)
    return pts


def gain_shape4_mse(shape_cb, n_radius, tr, te):
    def cells(x, rq):
        r = np.linalg.norm(x, axis=1)
        u = x / r[:, None]
        si = argmax_dot(u, shape_cb)
        edges = (rq[1:] + rq[:-1]) / 2
        ri = np.searchsorted(edges, r)
        return ri * len(shape_cb) + si
    rq = lloyd_1d(np.linalg.norm(tr, axis=1), n_radius) if n_radius > 1 else np.array([np.linalg.norm(tr, axis=1).mean()])
    n_cells = n_radius * len(shape_cb)
    return centroid_codec_mse(tr, cells(tr, rq), te, cells(te, rq), n_cells)


def kmeans(x, k, iters=60, seed=0):
    r = np.random.default_rng(seed)
    c = x[r.choice(len(x), k, replace=False)].copy()
    for _ in range(iters):
        idx = assign(x, c)
        sums = np.zeros_like(c); np.add.at(sums, idx, x)
        cnt = np.bincount(idx, minlength=k)
        empty = cnt == 0
        c[~empty] = sums[~empty] / cnt[~empty, None]
        if empty.any():
            c[empty] = x[r.choice(len(x), empty.sum(), replace=False)]
    return c


def assign(x, c):
    chunk = max(1000, int(1e7 // len(c)))
    out = np.empty(len(x), dtype=np.int64)
    cc = np.sum(c * c, axis=1)
    for s in range(0, len(x), chunk):
        xb = x[s:s + chunk]
        out[s:s + chunk] = np.argmin(cc[None, :] - 2 * xb @ c.T, axis=1)
    return out


def argmax_dot(u, c):
    chunk = max(1000, int(1e7 // len(c)))
    out = np.empty(len(u), dtype=np.int64)
    for s in range(0, len(u), chunk):
        out[s:s + chunk] = np.argmax(u[s:s + chunk] @ c.T, axis=1)
    return out


def max_dot(u, c):
    chunk = max(1000, int(1e7 // len(c)))
    out = np.empty(len(u))
    for s in range(0, len(u), chunk):
        out[s:s + chunk] = np.max(u[s:s + chunk] @ c.T, axis=1)
    return out


def vq_mse(dim, bits_per_dim, iters=60):
    k = 2 ** int(round(bits_per_dim * dim))
    tr = rng.standard_normal((N_TRAIN // 2, dim))
    te = rng.standard_normal((N_TEST // 2, dim))
    c = kmeans(tr, k, iters=iters)
    rec = c[assign(te, c)]
    return float(np.mean(np.sum((te - rec) ** 2, axis=1)) / dim)


def spherical_kmeans(u, k, iters=60, seed=1):
    r = np.random.default_rng(seed)
    c = u[r.choice(len(u), k, replace=False)].copy()
    for _ in range(iters):
        idx = argmax_dot(u, c)
        sums = np.zeros_like(c); np.add.at(sums, idx, u)
        nrm = np.linalg.norm(sums, axis=1)
        ok = nrm > 0
        c[ok] = sums[ok] / nrm[ok, None]
    return c


if __name__ == "__main__":
    c24, c600 = cell24(), cell600()
    assert c24.shape == (24, 4) and c600.shape == (120, 4), (c24.shape, c600.shape)
    assert np.allclose(np.linalg.norm(c600, axis=1), 1)
    g = c600 @ c600.T
    np.fill_diagonal(g, -2)
    print(f"600-cell: {len(c600)} pts, max cos between distinct = {g.max():.6f} (expect phi/2={PHI/2:.6f}, 36 deg)")

    tr4 = rng.standard_normal((N_TRAIN, 4))
    te4 = rng.standard_normal((N_TEST, 4))
    u_tr = tr4 / np.linalg.norm(tr4, axis=1, keepdims=True)

    for R in (2.0, 3.0):
        print(f"\n=== rate R = {R} bits/dim  (Shannon bound D = 2^-2R = {2 ** (-2 * R):.5f}) ===")
        print(f"scalar Lloyd-Max ({int(R)} bits/coord): {scalar_lloyd_mse(int(R)):.5f}")
        m, br, bt = polar2d_mse(int(2 * R))
        print(f"2-D polar product quantizer (best split radius {br} b + angle {bt} b per pair): {m:.5f}")
        budget = int(2 ** (4 * R))
        results = []
        for name, cb in (("24-cell", c24), ("600-cell", c600)):
            n_r = budget // len(cb)
            results.append((gain_shape4_mse(cb, n_r, tr4, te4), f"4-D gain-shape {name} x {n_r} radii ({len(cb)*n_r} codewords <= {budget})"))
        for n_shape in ((128, 2), (256, 1), (64, 4), (32, 8)) if R == 2.0 else ((1024, 4), (512, 8), (256, 16)):
            k, n_r = n_shape
            cb = spherical_kmeans(u_tr, k)
            results.append((gain_shape4_mse(cb, n_r, tr4, te4), f"4-D gain-shape learned spherical code {k} x {n_r} radii"))
        for mse, label in sorted(results):
            print(f"{label}: {mse:.5f}")
        if R == 2.0:
            print(f"unconstrained 2-D VQ (k-means, 16 codewords): {vq_mse(2, R):.5f}")
            print(f"unconstrained 4-D VQ (k-means, 256 codewords): {vq_mse(4, R):.5f}")
        else:
            print(f"unconstrained 2-D VQ (k-means, 64 codewords): {vq_mse(2, R):.5f}")
            print(f"unconstrained 4-D VQ (k-means, 4096 codewords, 25 iters): {vq_mse(4, R, iters=25):.5f}")

    # exact-radius (\"preserve the radius\") variant: angular-only error with a float16 radius per block
    for name, cb in (("24-cell", c24), ("600-cell", c600)):
        cosmax = max_dot(te4 / np.linalg.norm(te4, axis=1, keepdims=True), cb)
        mse = np.mean(np.sum(te4 ** 2, axis=1) * 2 * (1 - cosmax)) / 4
        bits = np.log2(len(cb)) + 16
        print(f"exact fp16 radius + {name} direction: MSE/dim {mse:.5f} at {bits/4:.3f} bits/dim "
              f"(direction-only cost {np.log2(len(cb))/4:.3f} bits/dim)")
