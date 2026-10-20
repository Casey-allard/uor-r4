"""E3: what fixed zeta-zero phases do and do not offer (vs golden/linear/geometric/random)."""
import re, math, sys, time
import numpy as np
SRC = "/home/user/uor-r4/crates/uor-r4-core/src/zeta_zeros.rs"
g = np.array([float(x) for x in re.findall(r"^\s+([0-9]+\.[0-9]+),", open(SRC).read(), re.M)])
print("zeros parsed:", len(g), "first", g[0], "last", g[-1])
rng = np.random.default_rng(0)
PHI = (1 + 5 ** 0.5) / 2

# 1. verify table against mpmath
import mpmath
mpmath.mp.dps = 20
t0 = time.time()
for n in (1, 2, 100, 256, 512):
    z = float(mpmath.zetazero(n).imag)
    print(f"  zero #{n}: table {g[n-1]:.9f}  mpmath {z:.9f}  |diff| {abs(z-g[n-1]):.1e}")
print("  (mpmath check took %.1fs)" % (time.time() - t0))
# gaps sanity: table sorted, strictly increasing
print("strictly increasing:", bool(np.all(np.diff(g) > 0)))

# 2. Landau bias: (1/N) sum_n cos(gamma_n log x)  ~  -(T/2pi) Lambda(x) / (sqrt(x) N)
T = g[-1]; Nz = len(g)
def vonmangoldt(x):
    for p in range(2, x + 1):
        if x % p == 0:
            y = x
            while y % p == 0: y //= p
            return math.log(p) if y == 1 else 0.0
    return 0.0
print("\nLandau bias of zeta phases at log x (x integer):")
print("   x  Lambda(x)  mean_n cos(g_n ln x)   prediction -(T/2pi)Lambda/(sqrt(x) N)")
for x in range(2, 13):
    m = np.mean(np.cos(g * math.log(x)))
    pred = -(T / (2 * math.pi)) * vonmangoldt(x) / (math.sqrt(x) * Nz)
    print(f"  {x:2d}   {vonmangoldt(x):.3f}      {m:+.4f}              {pred:+.4f}")
m_rand = [abs(np.mean(np.cos(rng.uniform(0, 2*np.pi, Nz)))) for _ in range(2000)]
print("  random-phase reference |mean cos|: 95th percentile %.4f" % np.quantile(m_rand, 0.95))

# 3. star discrepancy of 1-D sequences mod 1
def star_disc(u):
    u = np.sort(np.mod(u, 1.0)); n = len(u); i = np.arange(1, n + 1)
    return max(np.max(i / n - u), np.max(u - (i - 1) / n))
print("\nStar discrepancy D*_N (N=512), lower = more uniform:")
for p in (2, 3, 5, 7, 101, 1009, 10007):
    print(f"  zeta phases gamma_n ln({p})/2pi : {star_disc(g * math.log(p) / (2 * math.pi)):.4f}")
print(f"  golden Weyl  n*phi            : {star_disc(np.arange(1, Nz + 1) * PHI):.4f}")
print(f"  Kronecker    n*sqrt2          : {star_disc(np.arange(1, Nz + 1) * 2 ** 0.5):.4f}")
rd = [star_disc(rng.uniform(size=Nz)) for _ in range(200)]
print(f"  uniform random (mean of 200)  : {np.mean(rd):.4f}  (5-95%: {np.quantile(rd,0.05):.4f}-{np.quantile(rd,0.95):.4f})")

# 4. spacing statistics (unfolded zeros)
def Nsmooth(t):
    return t / (2 * math.pi) * np.log(t / (2 * math.pi * math.e)) + 7 / 8
s = np.diff(Nsmooth(g)); s /= s.mean()
print("\nNearest-neighbour spacing of unfolded zeros (mean 1): min %.3f, frac<0.25: %.4f "
      "(Poisson 0.2212, GUE-Wigner approx 0.0161); variance %.3f (Poisson 1, GUE ~0.178)"
      % (s.min(), np.mean(s < 0.25), s.var()))
wg = np.sort(np.mod(np.arange(1, Nz + 1) * PHI, 1)); gw = np.diff(wg) * Nz
print("golden Weyl points: distinct gap values (three-gap theorem):", len(np.unique(np.round(gw, 6))),
      "min normalized gap %.3f" % gw.min())

# 5. RoPE-style positional kernels, N=32 frequencies in [w_min, w_max]
N = 32; wmin, wmax = 2 * np.pi / 2048, np.pi
def affine(v): return wmin + (v - v.min()) / (v.max() - v.min()) * (wmax - wmin)
sets = {
    'geometric (RoPE ladder)': wmax * (wmin / wmax) ** (np.arange(N) / (N - 1)),
    'linear ladder': np.linspace(wmin, wmax, N),
    'zeta, affine to range': affine(g[:N]),
    'zeta, proportional (g_k*pi/g_32)': g[:N] * np.pi / g[N - 1],
    'golden Weyl in range': wmin + (wmax - wmin) * np.mod(np.arange(1, N + 1) * PHI, 1),
}
D = np.arange(1, 256)
def stats(w):
    K = np.cos(np.outer(D, w)).mean(axis=1)
    return K.max(), np.abs(K[15:]).mean(), np.argmax(K) + 1
print("\nRoPE-style kernel K(D)=mean_k cos(D w_k), D=1..255 (N=32 freqs in [2pi/2048, pi]):")
print("  set                               max K (worst alias)  at D   mean|K| D>=16")
for name, w in sets.items():
    mx, ml, at = stats(w)
    print(f"  {name:34s} {mx:8.3f}            {at:4d}   {ml:.3f}")
for name, gen in [('uniform random in range', lambda: rng.uniform(wmin, wmax, N)),
                  ('log-uniform random', lambda: np.exp(rng.uniform(np.log(wmin), np.log(wmax), N)))]:
    r = np.array([stats(gen())[:2] for _ in range(200)])
    print(f"  {name:34s} {r[:,0].mean():8.3f} (mean of 200)       {r[:,1].mean():.3f}")

# 6. zeta phases as token-identity codes (the repo's use): v_p = e^{i gamma_j ln p}
def primes(n):
    lim = 50000; sieve = np.ones(lim, bool); sieve[:2] = False
    for i in range(2, int(lim ** 0.5) + 1):
        if sieve[i]: sieve[i*i::i] = False
    return np.nonzero(sieve)[0][:n]
P = primes(4096).astype(float); L = np.log(P)
def nn_sim(freqs, logs):
    # cosine similarity of complex phase codes = mean_j cos(freq_j (ln p - ln q))
    C = np.cos(np.outer(logs, freqs)); S = np.sin(np.outer(logs, freqs))
    M = (C @ C.T + S @ S.T) / len(freqs)
    np.fill_diagonal(M, -np.inf)
    return M.max(axis=1)
print("\nToken codes: nearest-neighbour cosine similarity among the first V primes (1.0 = collision)")
for V, chans in [(258, 8), (4096, 8), (4096, 512)]:
    Lv = L[:V]
    rows = [('zeta gamma_1..gamma_%d' % chans, g[:chans])]
    rows.append(('random freqs in same range', rng.uniform(g[0], g[chans - 1], chans)))
    for name, fr in rows:
        nn = nn_sim(fr, Lv)
        print(f"  V={V:4d} channels={chans:3d} {name:28s}: median NN sim {np.median(nn):.4f}; "
              f"frac tokens with NN sim>0.99: {np.mean(nn > 0.99):.3f}")
    # i.i.d. random phase code (a hash) with same channel count
    ph = rng.uniform(0, 2 * np.pi, (V, chans))
    C, S = np.cos(ph), np.sin(ph); M = (C @ C.T + S @ S.T) / chans; np.fill_diagonal(M, -np.inf)
    nn = M.max(axis=1)
    print(f"  V={V:4d} channels={chans:3d} {'iid random phases (hash)':28s}: median NN sim {np.median(nn):.4f}; "
          f"frac tokens with NN sim>0.99: {np.mean(nn > 0.99):.3f}")
