import math
import numpy as np
from bitarray import bitarray

def compute_initial_primes(M):
    """
    Computes all prime numbers up to M using the unified equation with revised sieve notation.
    This function is similar to compute_primes but tailored for smaller values of M.
    """
    if M < 2:
        return []

    primes = [2, 3] if M >= 3 else [2] if M >= 2 else []

    # Initialize is_prime array with False
    is_prime = bitarray(M + 1)
    is_prime.setall(False)

    # Generate candidate numbers N = 6k ± 1
    max_k = M // 6 + 2
    for k in range(1, max_k):
        n1 = 6 * k - 1
        n2 = 6 * k + 1
        if n1 <= M:
            is_prime[n1] = True
        if n2 <= M:
            is_prime[n2] = True

    max_P = int(math.isqrt(M)) + 1

    # Perform the sieve
    for P in range(5, max_P + 1):
        if not is_prime[P]:
            continue
        # Mark multiples of P as non-prime
        for multiple in range(P * P, M + 1, P):
            is_prime[multiple] = False

    # Extract primes from the is_prime array
    primes.extend([i for i in range(5, M + 1) if is_prime[i]])
    primes.sort()
    return primes

def compute_primes_segmented(M, segment_size):
    """
    Computes all prime numbers up to M using segmented sieving and the unified equation.
    """
    if M < 2:
        return []

    primes = [2, 3] if M >= 3 else [2] if M >= 2 else []

    max_P = int(math.isqrt(M)) + 1
    initial_primes = compute_initial_primes(max_P - 1)

    for low in range(5, M + 1, segment_size):
        high = min(low + segment_size - 1, M)
        segment_length = high - low + 1

        # Initialize bit array for the segment
        is_prime_segment = bitarray(segment_length)
        is_prime_segment.setall(True)

        for P in initial_primes:
            # Find the starting index in the segment
            start = max(P * P, ((low + P - 1) // P) * P)
            if start > high:
                continue
            indices = range(start - low, high - low + 1, P)
            for idx in indices:
                is_prime_segment[idx] = False

        # Collect primes in the segment
        segment_primes = [low + i for i in range(segment_length) if is_prime_segment[i] and (low + i) % 6 in [1, 5]]
        primes.extend(segment_primes)

    primes.sort()
    return primes

# Example usage:
if __name__ == "__main__":
    import time

    M = int(input("Enter the value of M: "))
    segment_size = input("Enter the segment size (Default: 800000000): ")
    if type(segment_size) != int:
       segment_size = int(800000000)
    start_time = time.time()
    primes = compute_primes_segmented(M, segment_size)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
    # Uncomment the following line to print all primes
    print(f"Last Prime Found: " + str(primes.pop()))
    print(str(math.ceil(math.log(M, 2))) + " bits required for Max " + str(M))
