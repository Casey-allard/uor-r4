"""Scratch: integer Lorentz read, naive vs stable z-1, on Q8 codes (as the integer runtime stores q, k)."""
import math, numpy as np
rng = np.random.default_rng(0)
def isqrt(n): return math.isqrt(n)
def naive(qc, kc, guard):
    # q0 = sqrt(1+|q|^2) with `guard` fractional bits; codes are Q8 (value = code/256)
    q2 = int(np.dot(qc, qc)); k2 = int(np.dot(kc, kc)); qk = int(np.dot(qc, kc))    # Q16 exact
    one = 1 << 16
    q0 = isqrt((one + q2) << (2 * guard - 16))          # Q(guard)
    k0 = isqrt((one + k2) << (2 * guard - 16))
    z = q0 * k0 - (qk << (2 * guard - 16))                # Q(2*guard)
    return max(z / 2.0 ** (2 * guard), 1.0)
def stable(qc, kc, guard):
    q2 = int(np.dot(qc, qc)); k2 = int(np.dot(kc, kc)); qk = int(np.dot(qc, kc))
    A = q2 + k2 - 2 * qk                                   # |q-k|^2, Q16, exact
    B = q2 - k2                                            # Q16, exact
    one = 1 << 16
    C = isqrt((one + q2) << (2 * guard - 16)) + isqrt((one + k2) << (2 * guard - 16))   # q0+k0, Q(guard)
    # B^2/C^2 in Q16: (B^2 << 2*guard) / C^2 ; B^2 is Q32, C^2 is Q(2 guard)
    frac = (B * B << (2 * guard - 16)) // (C * C)
    u = max(A - frac, 0) / 2.0 / 2.0 ** 16                 # z - 1
    return 1.0 + u
errs = {k: [] for k in ("naive16", "naive24", "stable16")}
for trial in range(4000):
    scale = rng.choice([1.0, 3.0, 10.0])
    q = rng.normal(0, scale / 8, 64)
    k = q + rng.normal(0, rng.choice([1e-3, 1e-2, 0.1, 1.0]) * scale / 8, 64) if trial % 2 else rng.normal(0, scale / 8, 64)
    qc = np.round(q * 256).astype(np.int64); kc = np.round(k * 256).astype(np.int64)
    qv, kv = qc / 256.0, kc / 256.0
    zt = math.sqrt(1 + qv @ qv) * math.sqrt(1 + kv @ kv) - qv @ kv                 # float64 on the same codes
    ut = 0.5 * (np.sum((qv - kv) ** 2) - ((qv @ qv - kv @ kv) / (math.sqrt(1 + qv @ qv) + math.sqrt(1 + kv @ kv))) ** 2)
    dt = math.acosh(1 + ut)
    errs["naive16"].append(abs(math.acosh(naive(qc, kc, 16)) - dt))
    errs["naive24"].append(abs(math.acosh(naive(qc, kc, 24)) - dt))
    errs["stable16"].append(abs(math.acosh(stable(qc, kc, 16)) - dt))
for k, v in errs.items():
    v = np.array(v); print(f"{k:9s} max |d err| {v.max():.2e}  p99 {np.quantile(v, .99):.2e}  median {np.median(v):.2e}")
