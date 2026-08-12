import math

def compute_primes(M):
    # Manually include 2 and 3, as they are prime but not of the form 6k ± 1
    primes = [2, 3]

    # Generate candidate numbers N = 6k ± 1
    candidates = set()
    max_k = M // 6 + 1
    for k in range(1, max_k):
        n1 = 6 * k - 1
        n2 = 6 * k + 1
        if n1 <= M:
            candidates.add(n1)
        if n2 <= M:
            candidates.add(n2)

    # Perform the sieve
    max_P = int(math.sqrt(M)) + 1
    for P in sorted(candidates):
        if P > max_P:
            break
        # Skip P if it has been removed
        if P not in candidates:
            continue
        # Remove multiples of P using the set S_P
        n = 1
        while True:
            S_P = P * P + n * P
            if S_P > M:
                break
            candidates.discard(S_P)
            n += 1

    # Combine the remaining candidates with the initial primes
    primes.extend(sorted(candidates))
    primes = [p for p in primes if p <= M]

    return primes

# Example usage:
if __name__ == "__main__":
    M = int(input("Enter the value of M: "))
    primes = compute_primes(M)
    print(f"Primes up to {M}:")
    print(primes)
    print(str(math.ceil(math.log(M, 2))) + " bits required for Max " + str(M))
