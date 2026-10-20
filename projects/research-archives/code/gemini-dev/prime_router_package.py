import os
# Set single-thread environment variables BEFORE importing numpy to avoid deadlocks on macOS
os.environ["VECLIB_MAXIMUM_THREADS"] = "1"
os.environ["OMP_NUM_THREADS"] = "1"
os.environ["MKL_NUM_THREADS"] = "1"
os.environ["OPENBLAS_NUM_THREADS"] = "1"

import math
import time
import urllib.request
import numpy as np
from sympy import primerange

# ============================================================
# Package Configuration & File Paths
# ============================================================
CACHE_DIR = "./zeta_data"
ZEROS_FILE = os.path.join(CACHE_DIR, "zeta_zeros_100k.txt")
# Public academic URL containing the first 100,000 imaginary parts of zeta zeros
ZEROS_URL = "https://www.dtc.umn.edu/~odlyzko/zeta_tables/zeros1"

X_MIN = 1e4
X_MAX = 1e6
NUM_WINDOWS = 16          
RHO = 4.0                 # H(x) = RHO * sqrt(x)
N_SAMPLES = 257
SUBWINDOWS = 6
M_LIST = [120, 256, 512]  # Upgraded milestones to hit the stability plateau
SPARSE_RADIUS = 0.3       

# ============================================================
# Automatic Dependency and Data Fetching
# ============================================================
def ensure_zeta_data(max_m: int = 512):
    """Ensures the local environment has the true pre-calculated zeta zeroes."""
    if not os.path.exists(CACHE_DIR):
        os.makedirs(CACHE_DIR)
        
    enough_lines = False
    if os.path.exists(ZEROS_FILE):
        try:
            with open(ZEROS_FILE, 'r') as f:
                lines_count = sum(1 for line in f if line.strip())
            if lines_count >= max_m:
                enough_lines = True
        except Exception:
            pass
            
    if not enough_lines:
        print(f"[*] Local zeta data not found or insufficient (need {max_m} zeroes).")
        print(f"[*] Fetching from Odlyzko repository...")
        print(f"    Source: {ZEROS_URL}")
        download_success = False
        try:
            # Set a 5-second timeout for quick fallback
            req = urllib.request.Request(
                ZEROS_URL, 
                headers={'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'}
            )
            with urllib.request.urlopen(req, timeout=5) as response:
                content = response.read().decode('utf-8')
            with open(ZEROS_FILE, 'w') as f:
                f.write(content)
            print(f"[+] Download complete. Saved to {ZEROS_FILE}")
            
            # Recheck line count
            with open(ZEROS_FILE, 'r') as f:
                lines_count = sum(1 for line in f if line.strip())
            if lines_count >= max_m:
                download_success = True
            else:
                print(f"[!] Downloaded file only contains {lines_count} zeroes, need {max_m}. Falling back to mpmath generator...")
        except Exception as e:
            print(f"[-] Error downloading zeroes: {e}")
            
        if not download_success:
            print(f"[*] Generating the first {max_m} zeroes using mpmath...")
            try:
                import mpmath
                mpmath.mp.dps = 50
                gammas = []
                t_start = time.time()
                for k in range(1, max_m + 1):
                    val = float(mpmath.im(mpmath.zetazero(k)))
                    gammas.append(val)
                    if k % 100 == 0 or k == max_m:
                        print(f"    Calculated {k}/{max_m} zeroes...")
                print(f"[+] Calculation complete in {time.time() - t_start:.2f} seconds.")
                
                with open(ZEROS_FILE, 'w') as f:
                    for idx, val in enumerate(gammas, 1):
                        f.write(f"{idx} {val:.10f}\n")
                print(f"[+] Saved generated zeroes to local cache: {ZEROS_FILE}")
            except Exception as mp_err:
                print(f"[-] Failed to calculate zeroes: {mp_err}")
                raise mp_err

def load_true_zeros(M: int) -> np.ndarray:
    """Reads M lines from the cached text file directly into an optimized NumPy array."""
    ensure_zeta_data(M)
    gammas = []
    with open(ZEROS_FILE, 'r') as f:
        for line in f:
            stripped = line.strip()
            if stripped:
                try:
                    gammas.append(float(stripped.split()[-1]))
                except ValueError:
                    continue
            if len(gammas) >= M:
                break
    return np.array(gammas, dtype=float)

# ============================================================
# Core Analytical Metrics (Geometric Upgrades)
# ============================================================
def sigma_q_from_weights(p: np.ndarray) -> float:
    n = len(p)
    if n <= 1: return 1.0
    return float(1.0 - np.sum((p - 1.0 / n) ** 2) / (1.0 - 1.0 / n))

def sigma_kl_from_weights(p: np.ndarray) -> float:
    n = len(p)
    if n <= 1: return 1.0
    eps = 1e-300
    p = np.clip(p, eps, None)
    p = p / p.sum()
    return float(1.0 - np.sum(p * np.log(n * p)) / np.log(n))

def state_metrics_from_weights(p: np.ndarray):
    p = np.clip(p, 0.0, None)
    s = p.sum()
    if s <= 0: raise ValueError("State weights sum to zero.")
    p = p / s
    sigma_q = sigma_q_from_weights(p)
    sigma_kl = sigma_kl_from_weights(p)
    one_minus = max(1e-300, 1.0 - sigma_q)
    Lambda = -math.log(one_minus)
    kappa = float(np.max(p))
    
    # R4 Coordinate Translation: Hyperbolic deficit angle mapping
    deficit_angle = math.pi - Lambda
    
    return {
        "sigma_q": sigma_q,
        "sigma_kl": sigma_kl,
        "Lambda": Lambda,
        "kappa": kappa,
        "deficit_angle": deficit_angle
    }

def summarize_metrics(metric_list):
    keys = ["sigma_q", "sigma_kl", "Lambda", "kappa", "deficit_angle"]
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
    y = y - np.mean(y)
    nrm = np.linalg.norm(y)
    if nrm <= 0: return y
    return y / nrm

# ============================================================
# Signal Generation Pipeline
# ============================================================
def build_psi_table(x_max: float, rho: float) -> np.ndarray:
    max_needed = int(math.floor(x_max + rho * math.sqrt(x_max) + 10))
    vm = np.zeros(max_needed + 1, dtype=float)

    for p in primerange(2, max_needed + 1):
        lp = math.log(p)
        pk = p
        while pk <= max_needed:
            vm[pk] = lp
            if pk > max_needed // p: break
            pk *= p

    psi = np.cumsum(vm)
    return psi

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
# Sublinear R4 Routing & Matrix Projections
# ============================================================
def design_matrix_sparse_r4(xx: np.ndarray, gammas: np.ndarray, x_center: float, x_max: float) -> tuple:
    M = len(gammas)
    angular_phase = math.log(x_center) / math.log(x_max)
    center_idx = int(angular_phase * M)
    
    window_radius = max(4, int(M * SPARSE_RADIUS // 2))
    start_idx = max(0, center_idx - window_radius)
    end_idx = min(M, center_idx + window_radius)
    
    active_gammas = gammas[start_idx:end_idx]
    Phi_sparse = np.exp(1j * np.outer(np.log(xx), active_gammas))
    return Phi_sparse, start_idx, end_idx

def raw_amplitude_state(Phi: np.ndarray, y: np.ndarray, total_channels: int, start: int, end: int) -> np.ndarray:
    a_sparse = Phi.conj().T @ y
    full_state = np.zeros(total_channels)
    full_state[start:end] = np.abs(a_sparse)
    return full_state

def qr_orthonormal_state(Phi: np.ndarray, y: np.ndarray, total_channels: int, start: int, end: int) -> np.ndarray:
    Q, _ = np.linalg.qr(Phi, mode="reduced")
    a_sparse = Q.conj().T @ y
    full_state = np.zeros(total_channels)
    full_state[start:end] = np.abs(a_sparse)
    return full_state

def covariance_eigenvalue_state(Phi: np.ndarray, y: np.ndarray, total_channels: int, n_subwindows: int) -> np.ndarray:
    N, _ = Phi.shape
    seg_len = N // n_subwindows
    Q, _ = np.linalg.qr(Phi, mode="reduced")   
    
    coeffs = []
    for s in range(n_subwindows):
        start_t = s * seg_len
        end_t = (s + 1) * seg_len if s < n_subwindows - 1 else N
        ys = centered_l2_normalize(y[start_t:end_t])
        Qs = Q[start_t:end_t, :]   
        coeffs.append(Qs.conj().T @ ys)

    A = np.vstack(coeffs)   
    C = (A.conj().T @ A) / A.shape[0]

    evals = np.clip(np.real(np.linalg.eigvalsh(C)), 0.0, None)
    evals = np.sort(evals)[::-1]
    
    full_evals = np.zeros(total_channels)
    full_evals[:len(evals)] = evals
    return full_evals

# ============================================================
# Main Execution Runner
# ============================================================
def run_sandbox():
    print("=" * 60)
    print(" RUNNING R4 SUBLINEAR PRIME ROUTER TOY PACKAGE")
    print("=" * 60)
    
    t_start = time.time()
    
    print("[*] Pre-computing Von Mangoldt tables...")
    psi = build_psi_table(X_MAX, RHO)

    x_grid = np.exp(np.linspace(math.log(X_MIN), math.log(X_MAX), NUM_WINDOWS))
    print(f"[*] Constructing structural signal across {NUM_WINDOWS} windows...")
    windows = make_windows(psi, x_grid, RHO, N_SAMPLES)

    results = {}

    for M in M_LIST:
        print(f"\n[!] Accelerating Matrix to M={M} via Local Array Cache...")
        t_m0 = time.time()
        
        gammas = load_true_zeros(M)
        
        raw_metrics, qr_metrics, cov_metrics = [], [], []

        for j, (x, xx, y) in enumerate(windows, start=1):
            Phi, s_idx, e_idx = design_matrix_sparse_r4(xx, gammas, x, X_MAX)

            raw_metrics.append(state_metrics_from_weights(raw_amplitude_state(Phi, y, M, s_idx, e_idx)))
            qr_metrics.append(state_metrics_from_weights(qr_orthonormal_state(Phi, y, M, s_idx, e_idx)))
            cov_metrics.append(state_metrics_from_weights(covariance_eigenvalue_state(Phi, y, M, SUBWINDOWS)))

        results[M] = {
            "raw": summarize_metrics(raw_metrics),
            "qr": summarize_metrics(qr_metrics),
            "cov_eig": summarize_metrics(cov_metrics),
        }
        print(f"    M={M} Block processing resolved in {time.time() - t_m0:.2f} seconds.")

    print("\n" + "=" * 60)
    print(" EXECUTION PROFILE COMPLETED SUCCESSFULLY")
    print("=" * 60)
    print(f"Total time elapsed: {time.time() - t_start:.2f} seconds.\n")

    for M in M_LIST:
        print(f"##### TARGET MILESTONE RESOLUTION: M = {M} #####")
        for name in ["raw", "qr", "cov_eig"]:
            r = results[M][name]
            print(f" [{name.upper()}]")
            print(f"   sigma_Q Mean       = {r['sigma_q_mean']:.8f}")
            print(f"   sigma_KL Mean      = {r['sigma_kl_mean']:.8f}")
            print(f"   Lambda Mean        = {r['Lambda_mean']:.8f}")
            print(f"   Hyperbolic Deficit = {r['deficit_angle_mean']:.8f}")
            print(f"   Lambda - pi        = {r['Lambda_minus_pi_mean']:.8f}")
            print(f"   sigma_Q Dispersion = [{r['sigma_q_min']:.5f} -> {r['sigma_q_max']:.5f}]")
        print()

if __name__ == "__main__":
    run_sandbox()
