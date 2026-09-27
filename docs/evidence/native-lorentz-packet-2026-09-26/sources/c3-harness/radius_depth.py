"""Scratch analysis: does the written key's radius track code scope depth?

Inputs: <prefix>.key_norm.f32 from the `keys` tool (consecutive windows from token 0), valid_depth.npy,
valid.u16. For Lorentz keys the hyperbolic radius is arcsinh(|k|); for Dot keys we report |k| (and arcsinh).
Controls for token identity: subtract each token id's mean radius, then correlate the residual with depth.
"""
import sys, numpy as np
from scipy.stats import spearmanr

data_dir, *prefixes = sys.argv[1:]
depth_all = np.load(f"{data_dir}/valid_depth.npy")
toks_all = np.fromfile(f"{data_dir}/valid.u16", dtype="<u2")
for prefix in prefixes:
    norm = np.fromfile(f"{prefix}.key_norm.f32", dtype="<f4").astype(np.float64)
    n = len(norm)
    depth, toks = depth_all[:n], toks_all[:n]
    radius = np.arcsinh(norm)
    rho, _ = spearmanr(radius, depth)
    # token-identity control: residual radius after removing each token id's mean
    sums = np.bincount(toks, weights=radius, minlength=4096)
    counts = np.bincount(toks, minlength=4096)
    means = sums / np.maximum(counts, 1)
    resid = radius - means[toks]
    keep = counts[toks] >= 5
    rho_resid, _ = spearmanr(resid[keep], depth[keep])
    # within-token correlation for the most frequent tokens
    top = np.argsort(-counts)[:50]
    within = []
    for t in top:
        m = toks == t
        if m.sum() >= 50 and depth[m].std() > 0:
            within.append(spearmanr(radius[m], depth[m])[0])
    by_depth = [f"{d}:{radius[depth == d].mean():.3f}" for d in range(7) if (depth == d).sum() > 100]
    print(f"{prefix.split('/')[-1]:22s} n={n} |k| mean {norm.mean():.2f}  rho(radius,depth)={rho:+.3f}  "
          f"rho(resid,depth)={rho_resid:+.3f}  median within-token rho (top-50 tokens)={np.median(within):+.3f}")
    print("    mean radius by depth:", " ".join(by_depth))
