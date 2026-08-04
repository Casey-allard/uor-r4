#!/usr/bin/env python3
"""#395 E8/icosian codebook spike, measurement 1: neighborhood preservation.

Question: at an exactly matched bit budget (288 bits/vector), does E8 lattice
quantization of the real store content preserve neighborhood structure better
than the shipped 288-sign-bit orthant code?

Data: 50,000 centered context-bundle vectors (dim 288) dumped from the
checked-in fixture corpus by e8_dump_bundles.rs — the exact f32 objects the
shipped path sign-codes into 36 bytes.

Codes compared, both 288 bits/vector:
  SIGN   — shipped: sign bit per dim, Hamming ranking.
  E8-256 — 36 blocks x 8 dims; per block, decode to the nearest E8 lattice
           point (Conway-Sloane: best of D8 and D8+1/2) at a per-block scale
           chosen from a small grid on train data; codebook capped to the 256
           most frequent train lattice points (8 bits/block); ranking by L2
           between dequantized vectors.

Metric: recall@10 vs float-space ground truth (cosine and L2), mean over 500
held-out queries against a 20,000-vector pool; plus relative reconstruction
MSE. Train/eval split disjoint.
"""
import numpy as np

rng = np.random.default_rng(395)
D, B = 288, 8
NBLK = D // B
POOL, NQ, K = 20_000, 500, 10

X = np.fromfile("/tmp/e8_spike/bundles.f32", dtype="<f4").reshape(-1, D).astype(np.float64)
rng.shuffle(X)
train, pool, queries = X[:25_000], X[25_000 : 25_000 + POOL], X[45_000 : 45_000 + NQ]
print(f"vectors: train {len(train)}, pool {len(pool)}, queries {len(queries)}")


def d8_round(y):
    """Nearest D8 point (integer coords, even sum) per row of (..., 8)."""
    f = np.rint(y)
    s = f.sum(axis=-1)
    odd = (np.mod(s, 2) != 0)
    if odd.any():
        err = np.abs(y - f)
        idx = np.argmax(err, axis=-1)
        rows = np.nonzero(odd)[0]
        for r in rows:  # flip the worst coordinate the other way
            j = idx[r]
            f[r, j] += 1.0 if y[r, j] > f[r, j] else -1.0
    return f


def e8_decode(y):
    """Nearest E8 point: best of D8(y) and D8(y - 1/2) + 1/2."""
    a = d8_round(y)
    b = d8_round(y - 0.5) + 0.5
    da = ((y - a) ** 2).sum(axis=-1)
    db = ((y - b) ** 2).sum(axis=-1)
    return np.where((da <= db)[:, None], a, b)


def blocks(M):
    return M.reshape(len(M), NBLK, B)


# --- per-block scale: grid over multiples of block RMS, pick min train MSE ---
tb = blocks(train)
rms = np.sqrt((tb**2).mean(axis=(0, 2))) + 1e-9  # (NBLK,)
scales = np.empty(NBLK)
for blk in range(NBLK):
    best = (np.inf, None)
    for mult in (0.25, 0.5, 1.0, 2.0, 4.0):
        s = rms[blk] * mult
        y = tb[:, blk, :] / s
        q = e8_decode(y) * s
        mse = ((tb[:, blk, :] - q) ** 2).mean()
        if mse < best[0]:
            best = (mse, s)
    scales[blk] = best[1]

# --- capped codebook: top-256 train lattice points per block ---
codebooks = []
oov_blocks = 0
for blk in range(NBLK):
    y = tb[:, blk, :] / scales[blk]
    pts = e8_decode(y)
    key = np.round(pts * 2).astype(np.int64)  # half-integer safe
    uniq, counts = np.unique(key, axis=0, return_counts=True)
    order = np.argsort(-counts)
    cb = uniq[order[:256]].astype(np.float64) / 2.0
    if len(cb) < 256:
        pad = np.zeros((256 - len(cb), B))
        cb = np.vstack([cb, pad])
    codebooks.append(cb)
    oov_blocks += max(0, len(uniq) - 256)
print(f"codebooks built; mean distinct train lattice points/block: "
      f"{np.mean([len(np.unique(np.round(e8_decode(blocks(train)[:, b, :] / scales[b]) * 2).astype(np.int64), axis=0)) for b in range(0, NBLK, 12)]):.0f} (sampled)")


def encode_e8(M):
    """Quantize to capped codebook; return dequantized vectors."""
    Mb = blocks(M)
    out = np.empty_like(Mb)
    for blk in range(NBLK):
        y = Mb[:, blk, :] / scales[blk]
        cb = codebooks[blk]
        d2 = ((y[:, None, :] - cb[None, :, :]) ** 2).sum(axis=-1)
        out[:, blk, :] = cb[np.argmin(d2, axis=1)] * scales[blk]
    return out.reshape(len(M), D)


def recall_at_k(rank_pred, rank_true, k=K):
    return np.mean([len(set(p[:k]) & set(t[:k])) / k for p, t in zip(rank_pred, rank_true)])


def top_idx(dist, k=K):
    return np.argpartition(dist, k, axis=1)[:, : k + 1][
        np.arange(len(dist))[:, None],
        np.argsort(np.take_along_axis(dist, np.argpartition(dist, k, axis=1)[:, : k + 1], axis=1), axis=1),
    ][:, :k]


# --- ground truths ---
qn = queries / (np.linalg.norm(queries, axis=1, keepdims=True) + 1e-12)
pn = pool / (np.linalg.norm(pool, axis=1, keepdims=True) + 1e-12)
gt_cos = top_idx(1.0 - qn @ pn.T)
d_l2 = ((queries**2).sum(1)[:, None] - 2 * queries @ pool.T + (pool**2).sum(1)[None, :])
gt_l2 = top_idx(d_l2)

# --- SIGN code ranking (Hamming == mismatched sign count) ---
qs = (queries >= 0).astype(np.int8)
ps = (pool >= 0).astype(np.int8)
ham = (qs[:, None, :] != ps[None, :, :]).sum(axis=2)
r_sign = top_idx(ham.astype(np.float64))

# --- E8-256 ranking (L2 between dequantized) ---
pq = encode_e8(pool)
qq = encode_e8(queries)
d_e8 = ((qq**2).sum(1)[:, None] - 2 * qq @ pq.T + (pq**2).sum(1)[None, :])
r_e8 = top_idx(d_e8)

# --- reconstruction MSE (relative) ---
pool_mse_e8 = ((pool - pq) ** 2).mean() / (pool**2).mean()
sign_recon = np.sign(pool) * np.sqrt((pool**2).mean(axis=0, keepdims=True))
pool_mse_sign = ((pool - sign_recon) ** 2).mean() / (pool**2).mean()

print("\n=== #395 spike, measurement 1 (matched 288-bit budget) ===")
print(f"recall@{K} vs cosine GT : SIGN {recall_at_k(r_sign, gt_cos):.3f} | E8-256 {recall_at_k(r_e8, gt_cos):.3f}")
print(f"recall@{K} vs L2 GT     : SIGN {recall_at_k(r_sign, gt_l2):.3f} | E8-256 {recall_at_k(r_e8, gt_l2):.3f}")
print(f"relative recon MSE      : SIGN(+rms) {pool_mse_sign:.4f} | E8-256 {pool_mse_e8:.4f}")
