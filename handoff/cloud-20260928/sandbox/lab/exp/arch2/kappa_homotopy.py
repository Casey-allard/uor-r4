"""Curvature-homotopy conversion of a dot-product attention head into a Lorentz head.

Hyperboloid of curvature -kappa: x -> (x0, x), x0 = sqrt(1/kappa + |x|^2).
Distance d_k(x,y) = arcosh(kappa*z)/sqrt(kappa), z = x0*y0 - <x,y>.
Stable form: kappa*z - 1 = (kappa/2) * (|x-y|^2 - ((|x|^2-|y|^2)/(x0+y0))^2).
Score s_k(q,k) = -(1/(2 sqrt r)) d_k(q,k)^2 + |k|^2/(2 sqrt r).
Claim (Derived): as kappa -> 0, d_k -> |q-k|, so s_k -> <q,k>/sqrt r - |q|^2/(2 sqrt r),
whose softmax over keys equals the teacher's dot-product softmax exactly.
RoPE acts on the spatial part by rotations, which preserve |x|, x0 and <.,.>, hence d_k.
"""
import numpy as np, json, sys

rng = np.random.default_rng(0)
r = 64

def lift_d2(q, K, kappa, dtype=np.float64):
    q = q.astype(dtype); K = K.astype(dtype)
    nq = q @ q; nk = np.einsum('ij,ij->i', K, K)
    diff = K - q; dd = np.einsum('ij,ij->i', diff, diff)
    if kappa == 0.0:
        return dd                                            # flat limit: |q-k|^2
    q0 = np.sqrt(1.0 / kappa + nq); k0 = np.sqrt(1.0 / kappa + nk)
    u = 0.5 * kappa * (dd - ((nq - nk) / (q0 + k0)) ** 2)    # kappa*z - 1 >= 0
    u = np.maximum(u, 0)
    d = np.arccosh(1.0 + u) / np.sqrt(kappa)
    return d * d

def scores_kappa(q, K, kappa, dtype=np.float64):
    nk = np.einsum('ij,ij->i', K.astype(dtype), K.astype(dtype))
    return (-0.5 * lift_d2(q, K, kappa, dtype) + 0.5 * nk) / np.sqrt(r)

def softmax(s):
    s = s - s.max(); e = np.exp(s); return e / e.sum()

def kl(p, q):
    m = p > 0
    return float(np.sum(p[m] * (np.log(p[m]) - np.log(np.maximum(q[m], 1e-300)))))

def heads(T=512):
    """Synthetic trained-like heads (no real teacher available: huggingface.co is blocked)."""
    out = {}
    # diffuse: moderate norms, broad attention
    q = rng.normal(0, 1.0, r); K = rng.normal(0, 1.0, (T, r)); out["diffuse"] = (q, K)
    # sharp: a few keys aligned with a large-norm query
    q = rng.normal(0, 2.0, r); K = rng.normal(0, 1.0, (T, r))
    for j in rng.choice(T, 3, replace=False): K[j] += 0.6 * q
    out["sharp"] = (q, K)
    # sink: a first key with very large norm along a shared direction
    q = rng.normal(0, 1.5, r); K = rng.normal(0, 1.0, (T, r)); K[0] = 5.0 * q / np.linalg.norm(q) * 12
    out["sink"] = (q, K)
    # rope-like positional head: keys = rotations of a template
    return out

def rope(x, pos, base=10000.0):
    half = x.shape[-1] // 2
    freqs = base ** (-np.arange(half) / half)
    ang = pos * freqs
    c, s = np.cos(ang), np.sin(ang)
    x1, x2 = x[..., :half], x[..., half:]
    return np.concatenate([x1 * c - x2 * s, x1 * s + x2 * c], -1)

def quant(x, bits):
    qmax = 2 ** (bits - 1) - 1
    scale = np.max(np.abs(x)) / qmax
    return np.round(x / scale).clip(-qmax, qmax) * scale

res = {"kl_vs_teacher": {}, "cycle3_kappa1_best_beta_kl": {}, "quant": {}, "rope_invariance_maxerr": None,
       "float32_kl_small_kappa": {}}
for name, (q, K) in heads().items():
    teach = softmax(K @ q / np.sqrt(r))
    ent = float(-np.sum(teach * np.log(teach + 1e-300)))
    res["kl_vs_teacher"][name] = {"teacher_entropy_nats": round(ent, 3), "top1_mass": round(float(teach.max()), 3)}
    for kap in [0.0, 1e-6, 1e-4, 1e-3, 1e-2, 1e-1, 1.0]:
        res["kl_vs_teacher"][name][str(kap)] = kl(teach, softmax(scores_kappa(q, K, kap)))
        if kap in (1e-6, 1e-4, 1e-2):
            res["float32_kl_small_kappa"].setdefault(name, {})[str(kap)] = kl(
                teach, softmax(scores_kappa(q, K, kap, np.float32).astype(np.float64)))
    # cycle-3 style Lorentz score at kappa=1: beta*(delta - arcosh z); delta cancels in softmax; best beta by grid
    q0 = np.sqrt(1 + q @ q); k0 = np.sqrt(1 + np.einsum('ij,ij->i', K, K))
    dist = np.arccosh(np.maximum(q0 * k0 - K @ q, 1.0))
    best = min((kl(teach, softmax(-b * dist)), b) for b in np.geomspace(1e-2, 1e3, 400))
    res["cycle3_kappa1_best_beta_kl"][name] = {"kl": best[0], "beta": round(float(best[1]), 3)}
    # multiplier-free serving: 4-bit / 8-bit q and k (per-tensor symmetric), dot-equivalent and kappa=1e-2
    for bits in (4, 8):
        qq, KK = quant(q, bits), quant(K, bits)
        res["quant"].setdefault(name, {})[f"int{bits}_dot"] = kl(teach, softmax(KK @ qq / np.sqrt(r)))
        res["quant"][name][f"int{bits}_kappa1e-2"] = kl(softmax(scores_kappa(q, K, 1e-2)),
                                                       softmax(scores_kappa(qq, KK, 1e-2)))

# RoPE: d_kappa(R_m q, R_n k) == d_kappa(q, R_{n-m} k)
q = rng.normal(0, 1, r); k = rng.normal(0, 1, (1, r)); errs = []
for m, n in [(3, 17), (100, 250), (0, 999), (512, 40)]:
    a = lift_d2(rope(q, m), rope(k, n), 0.05); b = lift_d2(q, rope(k, n - m), 0.05)
    errs.append(float(abs(a[0] - b[0]) / b[0]))
res["rope_invariance_maxerr"] = max(errs)
print(json.dumps(res, indent=1))
