"""E8: is transformerless/cd_space.rs Octonion::mul a genuine octonion product? (norm-multiplicative, alternative,
non-associative). Replicates the Rust formula exactly."""
import numpy as np
F = [(1,2,4),(2,3,5),(3,4,6),(4,5,7),(5,6,1),(6,7,2),(7,1,3)]
def mul(a, b):
    o = np.zeros(8); o[0] = a[0]*b[0]
    for i in range(1, 8):
        o[0] -= a[i]*b[i]; o[i] += a[0]*b[i] + a[i]*b[0]
    for i, j, k in F:
        o[k] += a[i]*b[j] - a[j]*b[i]
        o[i] += a[j]*b[k] - a[k]*b[j]
        o[j] += a[k]*b[i] - a[i]*b[k]
    return o
rng = np.random.default_rng(0)
worst_norm = worst_alt = 0; assoc = []
for _ in range(2000):
    a, b, c = rng.normal(size=(3, 8))
    worst_norm = max(worst_norm, abs(np.linalg.norm(mul(a, b)) - np.linalg.norm(a)*np.linalg.norm(b)))
    worst_alt = max(worst_alt, np.abs(mul(mul(a, a), b) - mul(a, mul(a, b))).max(), np.abs(mul(mul(b, a), a) - mul(b, mul(a, a))).max())
    assoc.append(np.abs(mul(mul(a, b), c) - mul(a, mul(b, c))).max())
print(f"max | |ab| - |a||b| | = {worst_norm:.2e}; max alternativity defect = {worst_alt:.2e}; "
      f"median associator = {np.median(assoc):.2f} (nonzero => non-associative, as octonions must be)")
