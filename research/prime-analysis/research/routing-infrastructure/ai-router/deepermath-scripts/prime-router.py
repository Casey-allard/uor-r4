import math
import time
import numpy as np
import mpmath as mp
from sympy import primerange

# ============================================================
# Configuration
# ============================================================
X_MIN = 1e4
X_MAX = 1e6
NUM_WINDOWS = 16          # reduced a bit for runtime
RHO = 4.0                 # H(x) = RHO * sqrt(x)
N_SAMPLES = 257
SUBWINDOWS = 6
M_LIST = [72, 89, 96, 120]

mp.mp.dps = 50

# ============================================================
# Basic metrics
# ============================================================
def sigma_q_from_weights(p: np.ndarray) -> float:
    p = np.asarray(p, dtype=float)
    n = len(p)
    if n <= 1:
        return 1.0
    return 1.0 - np.sum((p - 1.0 / n) ** 2) / (1.0 - 1.0 / n)

def sigma_kl_from_weights(p: np.ndarray) -> float:
    p = np.asarray(p, dtype=float)
    n = len(p)
    if n <= 1:
        return 1.0
    eps = 1e-300
    p = np.clip(p, eps, None)
    p = p / p.sum()
    return 1.0 - np.sum(p * np.log(n * p)) / np.log(n)

def state_metrics_from_weights(p: np.ndarray):
    p = np.asarray(p, dtype=float)
    p = np.clip(p, 0.0, None)
    s = p.sum()
    if s <= 0:
        raise ValueError("State weights sum to zero.")
    p = p / s
    sigma_q = sigma_q_from_weights(p)
    sigma_kl = sigma_kl_from_weights(p)
    one_minus = max(1e-300, 1.0 - sigma_q)
    Lambda = -math.log(one_minus)
    kappa = float(np.max(p))
    return {
        "sigma_q": float(sigma_q),
        "sigma_kl": float(sigma_kl),
        "Lambda": float(Lambda),
        "kappa": float(kappa),
    }

def summarize_metrics(metric_list):
    keys = ["sigma_q", "sigma_kl", "Lambda", "kappa"]
    out = {}
    for k in keys:
        vals = np.array([m[k] for m in metric_list], dtype=float)
        out[k + "_mean"] = float(np.mean(vals))
        out[k + "_std"] = float(np.std(vals))
        out[k + "_min"] = float(np.min(vals))
        out[k + "_max"] = float(np.max(vals))
    out["Lambda_minus_pi_mean"] = out["Lambda_mean"] - math.pi
    return out

def centered_l2_normalize(y: np.ndarray) -> np.ndarray:
    y = np.asarray(y, dtype=float)
    y = y - np.mean(y)
    nrm = np.linalg.norm(y)
    if nrm <= 0:
        return y
    return y / nrm

# ============================================================
# Build psi(x) using von Mangoldt weights
# ============================================================
def build_psi_table(x_max: float, rho: float) -> np.ndarray:
    max_needed = int(math.floor(x_max + rho * math.sqrt(x_max) + 10))
    vm = np.zeros(max_needed + 1, dtype=float)

    for p in primerange(2, max_needed + 1):
        lp = math.log(p)
        pk = p
        while pk <= max_needed:
            vm[pk] = lp
            if pk > max_needed // p:
                break
            pk *= p

    psi = np.cumsum(vm)
    return psi

# ============================================================
# Window signal E(x) = psi(x) - x
# ============================================================
def make_windows(psi: np.ndarray, x_grid: np.ndarray, rho: float, n_samples: int):
    windows = []
    for x in x_grid:
        H = rho * math.sqrt(x)
        t = np.linspace(-H, H, n_samples)
        xx = x + t
        idx = np.floor(xx).astype(int)
        y = psi[idx] - xx
        y = centered_l2_normalize(y)
        windows.append((x, xx, y))
    return windows

# ============================================================
# True-zero basis
# ============================================================
def first_zeta_zero_ordinates(M: int):
    return np.array([float(mp.im(mp.zetazero(k))) for k in range(1, M + 1)], dtype=float)

def design_matrix_true_zero(xx: np.ndarray, gammas: np.ndarray) -> np.ndarray:
    # Columns are channels exp(i * gamma * log(x))
    return np.exp(1j * np.outer(np.log(xx), gammas))

# ============================================================
# Extraction methods
# ============================================================
def raw_amplitude_state(Phi: np.ndarray, y: np.ndarray) -> np.ndarray:
    a = Phi.conj().T @ y
    amps = np.abs(a)
    return amps

def qr_orthonormal_state(Phi: np.ndarray, y: np.ndarray) -> np.ndarray:
    Q, _ = np.linalg.qr(Phi, mode="reduced")
    a = Q.conj().T @ y
    amps = np.abs(a)
    return amps

def covariance_eigenvalue_state(Phi: np.ndarray, y: np.ndarray, n_subwindows: int = 6) -> np.ndarray:
    """
    Better fix:
    - orthogonalize once on the full window
    - reuse that fixed orthonormal channel family inside each subwindow
    - form covariance of subwindow coefficients
    - use covariance eigenvalues as state
    """
    N, M = Phi.shape
    seg_len = N // n_subwindows
    if seg_len < 8:
        raise ValueError("Subwindows too short. Reduce SUBWINDOWS or increase N_SAMPLES.")

    # Orthogonalize once on the full window
    Q, _ = np.linalg.qr(Phi, mode="reduced")   # shape: (N, r)
    r = Q.shape[1]

    coeffs = []
    for s in range(n_subwindows):
        start = s * seg_len
        end = (s + 1) * seg_len if s < n_subwindows - 1 else N

        ys = centered_l2_normalize(y[start:end])
        Qs = Q[start:end, :]   # fixed column count r

        a_s = Qs.conj().T @ ys
        coeffs.append(a_s)

    A = np.vstack(coeffs)   # shape: (S, r)

    # Channel covariance
    C = (A.conj().T @ A) / A.shape[0]

    # Hermitian eigenvalues
    evals = np.linalg.eigvalsh(C)
    evals = np.real(evals)
    evals = np.clip(evals, 0.0, None)

    # Sort descending
    evals = np.sort(evals)[::-1]
    return evals

# ============================================================
# Main experiment
# ============================================================
def run_experiment():
    t0 = time.time()

    print("Building psi table...")
    psi = build_psi_table(X_MAX, RHO)

    x_grid = np.exp(np.linspace(math.log(X_MIN), math.log(X_MAX), NUM_WINDOWS))
    print(f"Preparing {NUM_WINDOWS} windows...")
    windows = make_windows(psi, x_grid, RHO, N_SAMPLES)

    results = {}

    for M in M_LIST:
        print(f"\n=== Running M={M} ===")
        gammas = first_zeta_zero_ordinates(M)

        raw_metrics = []
        qr_metrics = []
        cov_metrics = []

        for j, (x, xx, y) in enumerate(windows, start=1):
            Phi = design_matrix_true_zero(xx, gammas)

            raw_state = raw_amplitude_state(Phi, y)
            raw_metrics.append(state_metrics_from_weights(raw_state))

            qr_state = qr_orthonormal_state(Phi, y)
            qr_metrics.append(state_metrics_from_weights(qr_state))

            cov_state = covariance_eigenvalue_state(Phi, y, SUBWINDOWS)
            cov_metrics.append(state_metrics_from_weights(cov_state))

            print(f"  window {j:02d}/{len(windows)} done", end="\r")

        results[M] = {
            "raw": summarize_metrics(raw_metrics),
            "qr": summarize_metrics(qr_metrics),
            "cov_eig": summarize_metrics(cov_metrics),
        }

        print(f"  window {len(windows):02d}/{len(windows)} done")

    elapsed = time.time() - t0
    print(f"\nFinished in {elapsed:.1f} seconds.\n")

    for M in M_LIST:
        print(f"===== M = {M} =====")
        for name in ["raw", "qr", "cov_eig"]:
            r = results[M][name]
            print(f"[{name}]")
            print(f"  sigma_Q mean      = {r['sigma_q_mean']:.10f}")
            print(f"  sigma_KL mean     = {r['sigma_kl_mean']:.10f}")
            print(f"  Lambda mean       = {r['Lambda_mean']:.10f}")
            print(f"  kappa mean        = {r['kappa_mean']:.10f}")
            print(f"  Lambda - pi mean  = {r['Lambda_minus_pi_mean']:.10f}")
            print(f"  sigma_Q range     = [{r['sigma_q_min']:.10f}, {r['sigma_q_max']:.10f}]")
            print(f"  Lambda range      = [{r['Lambda_min']:.10f}, {r['Lambda_max']:.10f}]")
        print()

    return results

if __name__ == "__main__":
    run_experiment()
