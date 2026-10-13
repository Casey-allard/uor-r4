def optimized_sieve(limit):
    sieve = [True] * (limit + 1)
    sieve[0] = sieve[1] = False
    primes = []

    for p in range(2, int(limit**0.5) + 1):
        if sieve[p]:
            for i in range(p * p, limit + 1, p):
                sieve[i] = False

    for p in range(2, limit + 1):
        if sieve[p]:
            primes.append(p)

    return primes

# Generate and output all prime numbers using the optimized Sieve of Eratosthenes
limit = 20_000_000  # Adjust the limit as needed
primes = optimized_sieve(limit)

for prime in primes:
    print(prime)

