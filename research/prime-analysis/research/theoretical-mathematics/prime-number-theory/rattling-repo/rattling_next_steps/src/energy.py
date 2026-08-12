import numpy as np


def compute_energy_series(N: int, M: int, alpha: float = 2.0, start: int = 2, dtype=np.float64) -> np.ndarray:
    """Compute E(n; M) for n = 1..N (inclusive) as defined in the paper.

    E(n;M) = sum_{m=2..M} [ alpha*log(m+1) if m|n else log((n mod m)+1) ]

    Notes:
    - This is O(N*M). Use moderate N/M or optimize further.
    - Returns array of length N+1 with E[0]=0 unused.
    """
    if N < 2:
        out = np.zeros(N + 1, dtype=dtype)
        return out

    n = np.arange(0, N + 1, dtype=np.int64)
    E = np.zeros(N + 1, dtype=dtype)

    # Precompute logs once (avoid log(0))
    log_m1 = np.zeros(M + 2, dtype=dtype)
    log_m1[1:] = np.log(np.arange(1, M + 2, dtype=dtype))  # log(k) indexed by k
    # log(m+1) for m
    log_m_plus_1 = log_m1[3: M + 2]  # m=2 -> log(3) ... m=M -> log(M+1)

    # Iterate modulus cap
    for m in range(2, M + 1):
        rem = n % m
        term = np.log(rem.astype(dtype) + 1.0)
        div_mask = (rem == 0)
        term[div_mask] = alpha * log_m1[m + 1]  # log(m+1)
        E += term

    # E(0) and E(1) are not meaningful in this scheme; keep for indexing
    return E


def compute_rii(E: np.ndarray) -> np.ndarray:
    """Resonance Interference Index: discrete curvature of E.

    RII(n) = E(n-1) - 2E(n) + E(n+1)

    Returns array same length as E with undefined edges set to 0.
    """
    R = np.zeros_like(E)
    if len(E) < 3:
        return R
    R[1:-1] = E[:-2] - 2.0 * E[1:-1] + E[2:]
    return R


def local_extrema_indices(x: np.ndarray, kind: str = "min") -> np.ndarray:
    """Return indices of strict local minima or maxima."""
    if len(x) < 3:
        return np.array([], dtype=np.int64)
    if kind == "min":
        mask = (x[1:-1] < x[:-2]) & (x[1:-1] < x[2:])
    elif kind == "max":
        mask = (x[1:-1] > x[:-2]) & (x[1:-1] > x[2:])
    else:
        raise ValueError("kind must be 'min' or 'max'")
    return np.where(mask)[0] + 1
