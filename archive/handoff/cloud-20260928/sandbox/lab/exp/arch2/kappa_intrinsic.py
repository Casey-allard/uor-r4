"""Variant of kappa_homotopy.py: intrinsic key-radius bias d_k(o,k)^2/2 = (asinh(sqrt(k)|k|)/sqrt(k))^2/2
instead of the Euclidean |k|^2/2. Same synthetic heads (same seed) as kappa_homotopy.py."""
import numpy as np, json
import kappa_homotopy as kh   # re-runs kh's own report on import (seeded); we only reuse its functions
rng = np.random.default_rng(0)
kh.rng = rng
r = kh.r
def scores_intrinsic(q, K, kappa):
    nk = np.sqrt(np.einsum('ij,ij->i', K, K))
    rad2 = nk ** 2 if kappa == 0 else (np.arcsinh(np.sqrt(kappa) * nk) / np.sqrt(kappa)) ** 2
    return (-0.5 * kh.lift_d2(q, K, kappa) + 0.5 * rad2) / np.sqrt(r)
out = {}
for name, (q, K) in kh.heads().items():
    teach = kh.softmax(K @ q / np.sqrt(r))
    out[name] = {str(k): kh.kl(teach, kh.softmax(scores_intrinsic(q, K, k))) for k in [0.0, 1e-4, 1e-2, 1e-1, 1.0]}
print("INTRINSIC", json.dumps(out))
