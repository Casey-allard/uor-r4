"""math2 E8: does prime/CRT/Galois addressing beat ordinary hashing for an exact n-gram memory?

Keys: every distinct token trigram of the code training split (4096-token BPE), packed as x = t1*2^32 + t2*2^16 + t3.
Tables of m = 2^20 buckets. Hash families:
  naive      x mod 2^20                       (structured: ignores t1 and most of t2)
  prime      ((a*x + b) mod p) mod m, p = 2^61-1   (Carter-Wegman prime-field hash; needs a multiplier)
  mulshift   (a*x mod 2^64) >> 44              (Dietzfelbinger multiply-shift; needs a multiplier)
  tabulation T1[b0]^...^T6[b5] over the 6 bytes of x (Patrascu-Thorup simple tabulation; table reads + XOR only)
  crt_pair   (x mod p1, x mod p2) with p1*p2 ~ m (joint address, primes 1021, 1031)
Metric: colliding pairs vs the uniform-hashing expectation n(n-1)/(2m); max bucket load.
Also: sequential positions t=0..W-1 into coprime ring buffers -> collision-free by CRT for W <= p1*p2 (checked).
"""
import numpy as np, json
S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
tr = np.fromfile(f"{S}/c3/data/code/train.u16", "<u2").astype(np.uint64)
x = np.unique((tr[:-2] << np.uint64(32)) | (tr[1:-1] << np.uint64(16)) | tr[2:])
n, m = len(x), 1 << 20
rng = np.random.default_rng(1)
out = {"distinct_trigrams": int(n), "buckets": m, "expected_pairs_uniform": n * (n - 1) / (2 * m)}


def stats(h, M=m):
    c = np.bincount(h.astype(np.int64), minlength=M)
    return {"colliding_pairs": int((c * (c - 1) // 2).sum()), "max_load": int(c.max()), "empty_frac": float((c == 0).mean())}


out["naive_mod_2^20"] = stats(x & np.uint64(m - 1))
# prime-field hash with p = 2^61 - 1 (exact 128-bit arithmetic via Python ints on a sample-free vectorized path)
p = (1 << 61) - 1
a, b = int(rng.integers(1, p)), int(rng.integers(0, p))
hp = np.array([((a * int(v) + b) % p) % m for v in x.tolist()], dtype=np.uint64)
out["prime_2^61-1"] = stats(hp)
am = np.uint64(int(rng.integers(1, 1 << 63)) | 1)
with np.errstate(over="ignore"):
    out["multiply_shift"] = stats((x * am) >> np.uint64(44))
T = rng.integers(0, np.iinfo(np.uint64).max, size=(6, 256), dtype=np.uint64, endpoint=True)
h = np.zeros(n, np.uint64)
for i in range(6):
    h ^= T[i][((x >> np.uint64(8 * i)) & np.uint64(255)).astype(np.int64)]
out["simple_tabulation"] = stats(h >> np.uint64(44))
p1, p2 = 1021, 1031
out["crt_pair_1021x1031"] = stats((x % np.uint64(p1)) * np.uint64(p2) + (x % np.uint64(p2)), M=p1 * p2)
out["crt_pair_1021x1031"]["expected_pairs_uniform"] = n * (n - 1) / (2 * p1 * p2)
# sequential positions into coprime rings: joint residue is unique for W <= p1*p2
W = p1 * p2
t = np.arange(W, dtype=np.int64)
joint = (t % p1) * p2 + (t % p2)
out["crt_positions_unique_for_W=p1p2"] = bool(len(np.unique(joint)) == W)
hpos = ((t.astype(np.uint64) * am) >> np.uint64(44)) % np.uint64(W) if False else rng.integers(0, W, size=W)
out["random_positions_colliding_pairs_same_W"] = stats(hpos, M=W)["colliding_pairs"]
print(json.dumps(out, indent=1))
json.dump(out, open(f"{S}/lab/exp/math2/e8_hashing.json", "w"), indent=1)
