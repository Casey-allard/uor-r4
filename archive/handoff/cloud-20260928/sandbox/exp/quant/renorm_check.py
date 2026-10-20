"""Higher-power check of norm-preserving reconstruction for TurboQuant-MSE (1000 queries, 3 seeds)."""
import math
import sys
import numpy as np
from quantlib import rotation
from qcodecs import D, make_data, LloydScalarQ, LogNormQ


def recall(S, Sh, k=10):
    t = np.argsort(-S, axis=1)[:, :k]
    a = np.argsort(-Sh, axis=1)[:, :k]
    return np.array([len(set(x) & set(y)) / k for x, y in zip(t, a)])


ds = sys.argv[1] if len(sys.argv) > 1 else "gauss"
Ziso = np.random.default_rng(5).standard_normal((8000, D))
for seed in (11, 12, 13):
    Rq = rotation("qr", D, np.random.default_rng(100 + seed))
    Xtr = make_data(ds, 8000, np.random.default_rng(seed))
    Xdb = make_data(ds, 10000, np.random.default_rng(seed + 100))
    Q = make_data(ds, 1000, np.random.default_rng(seed + 200))
    normq = LogNormQ().fit(np.linalg.norm(Xtr, axis=1))
    S = Q @ Xdb.T
    n = np.linalg.norm(Xdb, axis=1)
    Y = (Xdb @ Rq.T) * (math.sqrt(D) / n)[:, None]
    Yz = (Ziso @ Rq.T) * (math.sqrt(D) / np.linalg.norm(Ziso, axis=1))[:, None]
    for L in (3, 4, 6, 8, 16):
        q = LloydScalarQ(L).fit(Yz.reshape(-1, 1))
        Yh = q(Y.reshape(-1, 1)).reshape(Y.shape)
        res = {}
        for name in ("tq", "renorm"):
            Yr = Yh if name == "tq" else Yh * (math.sqrt(D) / np.linalg.norm(Yh, axis=1))[:, None]
            Xh = (Yr * (normq(n) / math.sqrt(D))[:, None]) @ Rq
            res[name] = recall(S, Q @ Xh.T)
        d = res["renorm"] - res["tq"]
        print(f"{ds} seed={seed} L={L:2d} recall tq={res['tq'].mean():.4f} renorm={res['renorm'].mean():.4f} "
              f"diff={d.mean():+.4f} +/- {d.std(ddof=1) / math.sqrt(len(d)):.4f} (paired SE, 1000 queries)", flush=True)
