"""redteam2_sci R4: first-order curvature expansion of the flat-limit conversion scores (Derived; numeric check).
Lift x -> (sqrt(1/k + |x|^2), x); d_k = arcosh(k z)/sqrt(k), z = x0 y0 - <x,y>.
Claim: d_k^2 = |x-y|^2 - k[(|x|^2-|y|^2)^2/4 + |x-y|^4/12] + O(k^2), hence
  key-norm score  s = (|k|^2 - d^2)/2       = q.k - |q|^2/2 + k[(|q|^2-|k|^2)^2/8 + |q-k|^4/24] + O(k^2)
  intrinsic score s = (d(o,k)^2 - d^2)/2    = same - k|k|^4/6 + O(k^2)
i.e. near the flat limit, 'curvature' is one extra scalar multiplying a fixed quartic norm feature."""
import numpy as np
rng = np.random.default_rng(1)
def d2(x, y, k):
    x0 = np.sqrt(1/k + x @ x); y0 = np.sqrt(1/k + y @ y)
    u = 0.5 * k * (np.sum((x - y)**2) - ((x @ x - y @ y) / (x0 + y0))**2)   # stable k z - 1
    return (np.arccosh(1 + u) / np.sqrt(k))**2
rows = []
for kap in (1e-6, 1e-5, 1e-4, 1e-3):
    errs0, errs1 = [], []
    for _ in range(200):
        q = rng.normal(0, 1.5, 64); kk = rng.normal(0, 1.5, 64)
        dd = d2(q, kk, kap)
        flat = np.sum((q - kk)**2)
        first = flat - kap * ((q @ q - kk @ kk)**2 / 4 + np.sum((q - kk)**2)**2 / 12)
        errs0.append(abs(dd - flat)); errs1.append(abs(dd - first))
    rows.append((kap, np.mean(errs0), np.mean(errs1)))
    print(f"kappa={kap:.0e}  |d^2 - flat| = {np.mean(errs0):.3e}   |d^2 - first-order| = {np.mean(errs1):.3e}")
# intrinsic radius check: d(o,k)^2 = |k|^2 - k|k|^4/3 + O(k^2)
for kap in (1e-5, 1e-4):
    kk = rng.normal(0, 1.5, 64); r2 = (np.arcsinh(np.sqrt(kap) * np.linalg.norm(kk)) / np.sqrt(kap))**2
    print(f"kappa={kap:.0e}  d(o,k)^2={r2:.6f}  |k|^2 - k|k|^4/3 = {kk@kk - kap*(kk@kk)**2/3:.6f}  |k|^2={kk@kk:.6f}")
