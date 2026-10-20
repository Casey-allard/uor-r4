"""Red-team numerical checks (scratch, not a project artifact).
A. associative scan of (r*q, b) vs sequential; frame decomposition h_t = U_t g_t; float32 drift at long T.
B. integer-quaternion codebook q ~ (2^k,a,b,c): size, finest angle (quaternion vs SO(3) convention),
   how many codewords have a rational normaliser, and the error of folding 1/|p| into a shift-add radius.
C. finite-subgroup claim: binary dihedral groups are non-abelian and arbitrarily fine.
"""
import itertools, math
import numpy as np
rng = np.random.default_rng(0)

def ham(a, b):
    aw, ax, ay, az = a[..., 0], a[..., 1], a[..., 2], a[..., 3]
    bw, bx, by, bz = b[..., 0], b[..., 1], b[..., 2], b[..., 3]
    return np.stack([aw*bw-ax*bx-ay*by-az*bz, aw*bx+ax*bw+ay*bz-az*by,
                     aw*by-ax*bz+ay*bw+az*bx, aw*bz+ax*by-ay*bx+az*bw], -1)
def conj(q):
    return q * np.array([1, -1, -1, -1], dtype=q.dtype)

print("=== A. scan / frame decomposition ===")
for dtype in (np.float64, np.float32):
    for T in (256, 4096, 16384):
        raw = rng.normal(scale=0.3, size=(T, 3))
        q = np.concatenate([np.ones((T, 1)), raw], 1); q /= np.linalg.norm(q, axis=1, keepdims=True)
        r = 1 - 2.0 ** -rng.integers(1, 11, size=T)          # dyadic decays, timescales 2..1024
        b = rng.normal(size=(T, 4))
        q, r, b = q.astype(dtype), r.astype(dtype), b.astype(dtype)
        # sequential reference (in float64 always)
        h = np.zeros(4); H = np.empty((T, 4))
        for t in range(T):
            h = r[t].astype(np.float64) * ham(q[t].astype(np.float64), h) + b[t].astype(np.float64); H[t] = h
        # sequential in dtype
        hd = np.zeros(4, dtype=dtype); Hd = np.empty((T, 4), dtype=dtype)
        for t in range(T):
            hd = r[t] * ham(q[t], hd) + b[t]; Hd[t] = hd
        # frame decomposition in dtype: U_t = q_t U_{t-1}; g_t = r_t g_{t-1} + conj(U_t) b_t; h_t = U_t g_t
        U = np.array([1, 0, 0, 0], dtype=dtype); g = np.zeros(4, dtype=dtype); Hf = np.empty((T, 4), dtype=dtype)
        unorm = np.empty(T)
        for t in range(T):
            U = ham(q[t], U); unorm[t] = np.linalg.norm(U.astype(np.float64))
            g = r[t] * g + ham(conj(U), b[t]); Hf[t] = ham(U, g)
        # Blelloch-style associative scan (log-depth tree) in dtype over pairs (A=r*q, b)
        A = (r[:, None] * q).astype(dtype); Bv = b.copy()
        n = T; step = 1
        As, Bs = A.copy(), Bv.copy()
        while step < n:  # Hillis-Steele inclusive scan: (A2,b2)o(A1,b1)=(A2A1, A2 b1 + b2)
            A_prev, B_prev = As.copy(), Bs.copy()
            As[step:] = ham(A_prev[step:], A_prev[:-step])
            Bs[step:] = ham(A_prev[step:], B_prev[:-step]) + B_prev[step:]
            step *= 2
        scale = np.abs(H).max()
        e_seq = np.abs(Hd - H).max() / scale; e_frame = np.abs(Hf - H).max() / scale; e_scan = np.abs(Bs - H).max() / scale
        print(f"{np.dtype(dtype).name:8s} T={T:6d}  rel.err seq={e_seq:.2e} frame={e_frame:.2e} scan={e_scan:.2e}  max| |U_t|-1 |={np.abs(unorm-1).max():.2e}")

print("\n=== B. integer-quaternion codebook ===")
rows = []
for k in range(5):
    for a, bb, c in itertools.product((-1, 0, 1), repeat=3):
        rows.append((2 ** k, a, bb, c))
for a, bb, c in itertools.product((-1, 0, 1), repeat=3):
    if (a, bb, c) != (0, 0, 0):
        rows.append((0, a, bb, c))
P = np.array(rows, float)
N2 = (P ** 2).sum(1)
Un = P / np.sqrt(N2)[:, None]
uniq, idx = np.unique(np.round(Un, 12), axis=0, return_index=True)
P = P[idx]; N2 = N2[idx]; Un = uniq
print("codebook size:", len(P))
theta = np.degrees(np.arccos(np.clip(np.abs(Un[:, 0]), 0, 1)))   # quaternion half-angle
nonid = theta > 1e-9
print(f"finest non-identity: quaternion angle {theta[nonid].min():.3f} deg, SO(3)/SU(2)_L-on-S3 rotation angle {2*theta[nonid].min():.3f} deg")
print("2I finest for comparison: quaternion angle 36 deg (SO(3) 72 deg)")
sq = np.array([math.isqrt(int(n)) ** 2 == int(n) for n in N2])
print(f"codewords with rational 1/|p| (|p|^2 a perfect square): {sq.sum()} of {len(P)}")
# CSD / greedy signed-power-of-two approximation of s = r/|p|
def spt(x, terms, emax=24):
    approx, res, used = 0.0, x, []
    for _ in range(terms):
        if res == 0: break
        e = round(-math.log2(abs(res)))
        e = min(max(e, 0), emax)
        d = math.copysign(2.0 ** -e, res); approx += d; res = x - approx; used.append(d)
    return approx
for r in (1.0, 1 - 2 ** -4, 1 - 2 ** -8):
    for terms in (1, 2, 3, 4, 6):
        errs = np.array([abs(spt(r / math.sqrt(n), terms) * math.sqrt(n) / r - 1) for n in N2 if n > 0])
        print(f"r={r:.6f} terms={terms}: effective-radius rel.err median {np.median(errs):.1e}  max {errs.max():.1e}")
# error growth: fixed-point lane, rotation by integer p (exact), scale by SPT approximation with rounding
def run(r, terms, T, F=16, seed=1, inputs=True):
    g = np.random.default_rng(seed)
    ids = g.integers(0, len(P), size=T)
    b = g.normal(scale=0.5, size=(T, 4)) if inputs else np.zeros((T, 4))
    h_ref = np.array([1.0, 0, 0, 0]); h_int = np.array([1 << F, 0, 0, 0], dtype=object)
    coef = {}
    worst = 0.0
    for t in range(T):
        p = P[ids[t]]; n = N2[ids[t]]
        h_ref = r * ham(p / math.sqrt(n), h_ref) + b[t]
        pi = [int(x) for x in p]
        y = [pi[0]*h_int[0]-pi[1]*h_int[1]-pi[2]*h_int[2]-pi[3]*h_int[3],
             pi[0]*h_int[1]+pi[1]*h_int[0]+pi[2]*h_int[3]-pi[3]*h_int[2],
             pi[0]*h_int[2]-pi[1]*h_int[3]+pi[2]*h_int[0]+pi[3]*h_int[1],
             pi[0]*h_int[3]+pi[1]*h_int[2]-pi[2]*h_int[1]+pi[3]*h_int[0]]   # small-int multipliers = shifts/adds
        key = (ids[t], r, terms)
        if key not in coef:
            s = r / math.sqrt(n); ds = []; approx = 0.0; res = s
            for _ in range(terms):
                if res == 0: break
                e = min(max(round(-math.log2(abs(res))), 0), 30); d = 1 if res > 0 else -1
                ds.append((d, e)); approx += d * 2.0 ** -e; res = s - approx
            coef[key] = ds
        out = []
        for v in y:
            acc = 0
            for d, e in coef[key]:
                acc += d * ((v + (1 << (e - 1))) >> e if e > 0 else v)   # rounded shift
            out.append(acc)
        bi = [int(round(x * (1 << F))) for x in b[t]]
        h_int = np.array([out[i] + bi[i] for i in range(4)], dtype=object)
        hr = np.array([float(x) / (1 << F) for x in h_int])
        worst = max(worst, np.linalg.norm(hr - h_ref) / max(np.linalg.norm(h_ref), 1e-9))
    return worst, np.linalg.norm(hr) / max(np.linalg.norm(h_ref), 1e-12)
print("\nfixed-point lane (F=16) vs float64 reference; worst relative state error / final norm ratio")
for inputs in (False, True):
    for r in (1.0, 1 - 2 ** -4, 1 - 2 ** -8):
        for terms in (2, 3, 4, 6):
            for T in (256, 4096):
                w, nr = run(r, terms, T, inputs=inputs)
                print(f"inputs={inputs!s:5s} r={r:.6f} terms={terms} T={T:5d}: worst rel.err {w:.2e}  final norm ratio {nr:.3f}")

print("\n=== C. finite subgroups of SU(2) ===")
for n in (5, 12, 60):
    # binary dihedral Dic_n: <a = exp(i*pi/n), x = j>; order 4n, non-abelian, smallest angle pi/n
    a = np.array([math.cos(math.pi / n), math.sin(math.pi / n), 0, 0]); j = np.array([0, 0, 1.0, 0])
    print(f"Dic_{n}: order {4*n}, non-abelian (a*j != j*a: {not np.allclose(ham(a,j), ham(j,a))}), smallest quaternion angle {180/n:.2f} deg")
