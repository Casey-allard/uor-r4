import math
import numpy as np
from multiprocessing import Pool, cpu_count

def sieve_segment(args):
    """
    Function to sieve a segment based on the unified equation.
    """
    P, M = args
    start = P * P
    if start > M:
        return None
    n_max = (M - start) // P + 1
    if n_max <= 0:
        return None
    n_values = np.arange(n_max)
    S_P_indices = start + n_values * P
    return S_P_indices

def compute_primes(M):
    """
    Computes all prime numbers up to M using the unified equation with revised sieve notation.
    """
    if M < 2:
        return []

    # Manually include 2 and 3, as they are primes but not of the form 6k ± 1
    primes = [2, 3] if M >= 3 else [2] if M >= 2 else []

    # Initialize is_prime array with False
    is_prime = np.zeros(M + 1, dtype=bool)

    # Generate candidate numbers N = 6k ± 1
    max_k = M // 6 + 2
    N_values = []
    for k in range(1, max_k):
        n1 = 6 * k - 1
        n2 = 6 * k + 1
        if n1 <= M:
            is_prime[n1] = True
            N_values.append(n1)
        if n2 <= M:
            is_prime[n2] = True
            N_values.append(n2)

    N_values = np.array(N_values, dtype=int)
    max_P = int(math.isqrt(M)) + 1

    # Perform the sieve using vectorized operations
    for P in N_values:
        if P > max_P:
            break
        if not is_prime[P]:
            continue
        # Compute S_P = P^2 + n * P where P^2 + n * P ≤ M
        start = P * P
        if start > M:
            continue
        n_max = (M - start) // P + 1
        n_values = np.arange(n_max)
        S_P_indices = start + n_values * P
        is_prime[S_P_indices] = False

    # Extract primes from the is_prime array
    prime_indices = np.flatnonzero(is_prime)
    primes.extend(prime_indices.tolist())

    primes.sort()
    return primes

def compute_primes_parallel(M):
    """
    Computes all prime numbers up to M using the unified equation with revised sieve notation,
    utilizing parallel processing for improved performance.
    """
    if M < 2:
        return []

    # Manually include 2 and 3
    primes = [2, 3] if M >= 3 else [2] if M >= 2 else []

    # Generate candidate numbers N = 6k ± 1
    max_k = M // 6 + 2
    N_values_set = set()
    for k in range(1, max_k):
        n1 = 6 * k - 1
        n2 = 6 * k + 1
        if n1 <= M:
            N_values_set.add(n1)
        if n2 <= M:
            N_values_set.add(n2)

    N_values = np.array(sorted(N_values_set), dtype=int)
    max_P = int(math.isqrt(M)) + 1

    # Initialize the is_prime array
    is_prime = np.zeros(M + 1, dtype=bool)
    is_prime[N_values] = True

    # Prepare P values to sieve (only those up to sqrt(M) and marked as prime)
    sieve_Ps = [P for P in N_values if P <= max_P and is_prime[P]]

    # Prepare arguments for the sieve_segment function
    args_list = [(P, M) for P in sieve_Ps]

    # Use multiprocessing Pool to parallelize the sieve
    num_processes = min(cpu_count(), len(sieve_Ps))
    with Pool(processes=num_processes) as pool:
        results = pool.map(sieve_segment, args_list)

    # Combine results
    for indices in results:
        if indices is not None:
            is_prime[indices] = False

    # Extract primes from the is_prime array
    prime_indices = np.flatnonzero(is_prime)
    primes.extend(prime_indices.tolist())

    primes.sort()
    return primes

# Example usage:
if __name__ == "__main__":
    import time

    M = int(input("Enter the value of M: "))
    use_parallel = input("Use parallel processing? (y/n): ").strip().lower() == 'y'

    start_time = time.time()
    if use_parallel:
        primes = compute_primes_parallel(M)
    else:
        primes = compute_primes(M)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
    # Uncomment the following line to print all primes
    # print(primes)
