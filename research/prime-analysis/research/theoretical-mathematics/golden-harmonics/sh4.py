import math

def sieve_eratosthenes(limit):
    if limit < 2:
        return []

    sqrt_limit = int(math.sqrt(limit))
    sieve_size = (limit + 1) // 2
    sieve = [True] * sieve_size

    for i in range(1, (sqrt_limit + 1) // 2):
        if sieve[i]:
            current_prime = 2 * i + 1
            start_index = (current_prime * current_prime) // 2
            sieve[start_index::current_prime] = [False] * ((sieve_size - start_index - 1) // current_prime + 1)

    primes = [2] + [2 * i + 1 for i, is_prime in enumerate(sieve) if is_prime and i > 0]

    return primes

# Find and print all prime numbers up to a specified limit
limit = 100_000_000  # Adjust the limit as needed
primes = sieve_eratosthenes(limit)

for prime in primes:
    print(prime)

