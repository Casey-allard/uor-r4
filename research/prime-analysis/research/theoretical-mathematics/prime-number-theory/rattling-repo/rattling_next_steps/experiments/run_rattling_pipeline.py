"""Reproducible 'next steps' pipeline for Modular Prime Rattling.

Runs:
- Compute E(n;M) and RII up to N.
- Evaluate correlation of E local minima (and/or RII features) with primality.
- Run null controls (permutation).
- Perform basic spectral tests:
  * FFT of E and RII
  * Peak-spacing stats vs first K zeta zeros

Outputs artifacts into ../output.

Usage:
  python experiments/run_rattling_pipeline.py --N 50000 --M 200 --alpha 2.0 --zeta_k 500
"""

import argparse
import json
from pathlib import Path
import sys

import numpy as np

# Allow running as a script without installing a package
ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from src.energy import compute_energy_series, compute_rii, local_extrema_indices
from src.primes import sieve_is_prime


def nearest_neighbor_spacings(xs: np.ndarray) -> np.ndarray:
    xs = np.sort(xs)
    return np.diff(xs)


def unfold_spacings(spacings: np.ndarray) -> np.ndarray:
    """Simple unfolding: normalize to unit mean spacing."""
    if len(spacings) == 0:
        return spacings
    m = float(np.mean(spacings))
    if m == 0:
        return spacings
    return spacings / m


def basic_stats(spacings: np.ndarray) -> dict:
    if len(spacings) == 0:
        return {"count": 0}
    return {
        "count": int(len(spacings)),
        "mean": float(np.mean(spacings)),
        "std": float(np.std(spacings)),
        "p10": float(np.percentile(spacings, 10)),
        "p50": float(np.percentile(spacings, 50)),
        "p90": float(np.percentile(spacings, 90)),
    }


def ks_statistic(a: np.ndarray, b: np.ndarray) -> float:
    """Two-sample KS statistic (no p-value; avoids scipy dependency)."""
    a = np.sort(a)
    b = np.sort(b)
    if len(a) == 0 or len(b) == 0:
        return float("nan")
    vals = np.sort(np.unique(np.concatenate([a, b])))
    cdf_a = np.searchsorted(a, vals, side="right") / len(a)
    cdf_b = np.searchsorted(b, vals, side="right") / len(b)
    return float(np.max(np.abs(cdf_a - cdf_b)))


def get_zeta_zero_imag_parts(k: int) -> np.ndarray:
    """Return imag parts of first k nontrivial zeta zeros using mpmath."""
    import mpmath as mp

    mp.mp.dps = 50
    gammas = []
    for i in range(1, k + 1):
        z = mp.zetazero(i)  # 0.5 + i*gamma
        gammas.append(float(mp.im(z)))
    return np.array(gammas, dtype=np.float64)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--N", type=int, default=50000)
    ap.add_argument("--M", type=int, default=200)
    ap.add_argument("--alpha", type=float, default=2.0)
    ap.add_argument("--zeta_k", type=int, default=300)
    ap.add_argument("--seed", type=int, default=7)
    args = ap.parse_args()

    outdir = ROOT / "output"
    outdir.mkdir(parents=True, exist_ok=True)

    N, M, alpha = args.N, args.M, args.alpha

    # Compute signals
    E = compute_energy_series(N=N, M=M, alpha=alpha)
    RII = compute_rii(E)

    is_prime = sieve_is_prime(N)
    baseline_prime_rate = float(is_prime[2:].mean())

    # Feature: local minima of E
    mins = local_extrema_indices(E, kind="min")
    mins = mins[(mins >= 2) & (mins <= N)]
    precision_mins = float(is_prime[mins].mean()) if len(mins) else 0.0

    # Feature: local maxima of RII (curvature peaks)
    rmax = local_extrema_indices(RII, kind="max")
    rmax = rmax[(rmax >= 2) & (rmax <= N)]
    precision_rmax = float(is_prime[rmax].mean()) if len(rmax) else 0.0

    # Null: permute indices of minima (same count)
    rng = np.random.default_rng(args.seed)
    perm_idx = (
        rng.choice(np.arange(2, N + 1), size=len(mins), replace=False)
        if len(mins)
        else np.array([], dtype=int)
    )
    precision_perm = float(is_prime[perm_idx].mean()) if len(perm_idx) else 0.0

    # Spectral: FFT of E and RII
    def fft_power(x: np.ndarray) -> np.ndarray:
        y = x[2:].astype(np.float64)
        y = y - y.mean()
        F = np.fft.rfft(y)
        P = (F * np.conj(F)).real
        return P

    P_E = fft_power(E)
    P_R = fft_power(RII)

    def top_freqs(P: np.ndarray, k: int = 10):
        idx = np.argsort(P[1:])[-k:][::-1] + 1
        return [{"bin": int(i), "power": float(P[i])} for i in idx]

    topE = top_freqs(P_E)
    topR = top_freqs(P_R)

    # Peak spacings (RII maxima positions)
    spac_rmax = np.diff(np.sort(rmax))
    spac_rmax_u = unfold_spacings(spac_rmax)

    # Zeta spacings
    zetas = get_zeta_zero_imag_parts(args.zeta_k)
    spac_z = unfold_spacings(np.diff(np.sort(zetas)))

    ks = ks_statistic(spac_rmax_u, spac_z)

    summary = {
        "params": {"N": N, "M": M, "alpha": alpha, "zeta_k": args.zeta_k},
        "counts": {"E_local_minima": int(len(mins)), "RII_local_maxima": int(len(rmax))},
        "prime_rates": {
            "baseline": baseline_prime_rate,
            "E_minima_precision": precision_mins,
            "RII_maxima_precision": precision_rmax,
            "perm_minima_precision": precision_perm,
        },
        "lift": {
            "E_minima": (precision_mins / baseline_prime_rate) if baseline_prime_rate else None,
            "RII_maxima": (precision_rmax / baseline_prime_rate) if baseline_prime_rate else None,
        },
        "spectral": {
            "top_freq_bins_E": topE,
            "top_freq_bins_RII": topR,
            "rmax_spacing_stats_unfolded": basic_stats(spac_rmax_u),
            "zeta_spacing_stats_unfolded": basic_stats(spac_z),
            "ks_stat_unfolded_rmax_vs_zeta": ks,
        },
    }

    (outdir / "summary.json").write_text(json.dumps(summary, indent=2))

    arr = np.column_stack(
        [
            np.arange(0, N + 1),
            E,
            RII,
            is_prime.astype(int),
            np.isin(np.arange(0, N + 1), mins).astype(int),
            np.isin(np.arange(0, N + 1), rmax).astype(int),
        ]
    )
    np.savetxt(
        outdir / "signals.csv",
        arr,
        delimiter=",",
        header="n,E,RII,is_prime,is_E_min,is_RII_max",
        comments="",
    )

    print(json.dumps(summary, indent=2))
    print(f"\nWrote: {outdir/'summary.json'}")
    print(f"Wrote: {outdir/'signals.csv'}")


if __name__ == "__main__":
    main()
