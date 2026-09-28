"""E3b: RoPE-style kernels, far-field aliasing metric and zeta percentile vs random/stratified."""
import re, numpy as np
g = np.array([float(x) for x in re.findall(r"^\s+([0-9]+\.[0-9]+),", open("/home/user/uor-r4/crates/uor-r4-core/src/zeta_zeros.rs").read(), re.M)])
rng = np.random.default_rng(1)
D = np.arange(1, 256)
for N in (32, 128):
    wmin, wmax = 2*np.pi/2048, np.pi
    def affine(v): return wmin + (v - v.min())/(v.max()-v.min())*(wmax-wmin)
    def far(w):
        K = np.cos(np.outer(D, w)).mean(axis=1)
        return K[15:].max(), np.abs(K[15:]).mean()
    zeta = far(affine(g[:N]))
    rand = np.array([far(rng.uniform(wmin, wmax, N)) for _ in range(500)])
    edges = np.linspace(wmin, wmax, N+1)
    strat = np.array([far(rng.uniform(edges[:-1], edges[1:])) for _ in range(500)])
    geo = far(wmax*(wmin/wmax)**(np.arange(N)/(N-1)))
    print(f"N={N}: far-field (D>=16) max K / mean|K|")
    print(f"   zeta affine          {zeta[0]:.3f} / {zeta[1]:.3f}   percentile of zeta max-K among random: {np.mean(rand[:,0] <= zeta[0])*100:.0f}%, among stratified: {np.mean(strat[:,0] <= zeta[0])*100:.0f}%")
    print(f"   uniform random       {rand[:,0].mean():.3f} / {rand[:,1].mean():.3f}  (mean of 500)")
    print(f"   stratified jitter    {strat[:,0].mean():.3f} / {strat[:,1].mean():.3f}  (mean of 500)")
    print(f"   geometric (RoPE)     {geo[0]:.3f} / {geo[1]:.3f}")
