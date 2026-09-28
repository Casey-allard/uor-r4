"""Controlled ablation of the owner's literal claim: 'preserve the radial direction'.

Same rotation, same shape code, same bit budget; only the radial treatment changes:
  proj   : Sabin-Gray MSE-optimal gain = Lloyd-quantized projection r<u,c>   (radius shrinks)
  radius : Lloyd-quantized block radius r, reconstruct r_hat * c            (radius preserved)
  ablate : no per-block radius (Lg=1): every block gets the same gain       (radius ablated)
and for TurboQuant-MSE:
  tq     : standard (norm stored, Lloyd reconstruction, norm shrinks by 1-D)
  tq-renorm: same bits, reconstruction rescaled to the stored norm (radius preserved)
usage: python3 radial_test.py <dataset> [out.json]
"""
import json
import math
import os
import sys
import numpy as np

from quantlib import Scalar, lloyd1d, rotation
from qcodecs import D, make_data, ip_argmax, LloydScalarQ, LogNormQ, metrics
from sweep import get_codes

HERE = os.path.dirname(os.path.abspath(__file__))


def run(ds, outpath):
    codes = get_codes()
    R = rotation("rht", D, np.random.default_rng(101))
    Xtr = make_data(ds, 8000, np.random.default_rng(1))
    Xdb = make_data(ds, 10000, np.random.default_rng(3))
    Q = make_data(ds, 100, np.random.default_rng(4))
    normq = LogNormQ().fit(np.linalg.norm(Xtr, axis=1))
    Ziso = np.random.default_rng(5).standard_normal((8000, D))

    def pre(X):
        n = np.linalg.norm(X, axis=1)
        return (X @ R.T) * (math.sqrt(D) / n)[:, None], n

    def post(Y, n):
        return (Y * (normq(n) / math.sqrt(D))[:, None]) @ R

    Yz, _ = pre(Ziso)
    Y, n = pre(Xdb)
    B = Y.reshape(-1, 4)
    Bz = Yz.reshape(-1, 4)
    rows = []
    for code in ("2T24", "2I120", "C600", "G2760"):
        C = codes[code]
        iz, pz = ip_argmax(Bz, C)
        rz = np.linalg.norm(Bz, axis=1)
        idx, p = ip_argmax(B, C)
        r = np.linalg.norm(B, axis=1)
        for Lg in (1, 2, 4, 8, 16):
            bits = (8 + math.ceil(16 * math.log2(len(C) * Lg) - 1e-9)) / D
            variants = {}
            gq = Scalar(lloyd1d(pz, Lg))
            variants["proj"] = gq(p)
            rq = Scalar(lloyd1d(rz, Lg))
            variants["radius"] = rq(r)
            for name, g in variants.items():
                if Lg == 1 and name == "radius":
                    continue
                Xh = post((g[:, None] * C[idx]).reshape(Y.shape), n)
                m = metrics(Xdb, Xh, Q)
                m.update(code=code, Lg=Lg, gain=("ablate" if Lg == 1 else name), bits_per_dim=bits)
                rows.append(m)
                print(f"{code:6s} Lg={Lg:2d} {m['gain']:7s} bits={bits:.3f} relmse={m['relmse']:.4f} "
                      f"slope={m['ip_slope']:.3f} ip_abs={m['ip_abs']:.3f} rec10={m['recall10']:.3f} "
                      f"cos={m['cos']:.4f}", flush=True)
    # TurboQuant-MSE vs norm-preserving (renormalized) reconstruction at identical bits
    Rq = rotation("qr", D, np.random.default_rng(101))
    Yq = (Xdb @ Rq.T) * (math.sqrt(D) / n)[:, None]
    Yzq = (Ziso @ Rq.T) * (math.sqrt(D) / np.linalg.norm(Ziso, axis=1))[:, None]
    for L in (2, 3, 4, 6, 8, 16):
        q = LloydScalarQ(L).fit(Yzq.reshape(-1, 1))
        Yh = q(Yq.reshape(-1, 1)).reshape(Yq.shape)
        bits = (8 + math.ceil(D * math.log2(L) - 1e-9)) / D
        for name in ("tq", "tq-renorm"):
            Yr = Yh if name == "tq" else Yh * (math.sqrt(D) / np.linalg.norm(Yh, axis=1))[:, None]
            Xh = (Yr * (normq(n) / math.sqrt(D))[:, None]) @ Rq
            m = metrics(Xdb, Xh, Q)
            m.update(code=f"TQ-L{L}", Lg=0, gain=name, bits_per_dim=bits)
            rows.append(m)
            print(f"TQ L={L:2d} {name:9s} bits={bits:.3f} relmse={m['relmse']:.4f} slope={m['ip_slope']:.3f} "
                  f"ip_abs={m['ip_abs']:.3f} rec10={m['recall10']:.3f} cos={m['cos']:.4f}", flush=True)
    json.dump(dict(dataset=ds, rows=rows), open(outpath, "w"), indent=1)


if __name__ == "__main__":
    ds = sys.argv[1]
    run(ds, sys.argv[2] if len(sys.argv) > 2 else os.path.join(HERE, f"radial_{ds}.json"))
