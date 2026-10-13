import math

MAX_LIMIT = 10_000_000
p = 1007  # example prime p>3
start = p * p
end = p * p + 12 * p + 36
if end > MAX_LIMIT:
    raise ValueError("Increase MAX_LIMIT or choose a smaller p so that (p+6)^2 <= MAX_LIMIT")

ALLOWED_DIGIT_SUM_PAIRS = {(2, 4), (5, 7), (8, 1)}


def digital_root(n):
    while n >= 10:
        s = 0
        while n > 0:
            s += n % 10
            n //= 10
        n = s
    return n


def is_6kpm1(n):
    return n > 3 and n % 6 in {1, 5}


def sieve_primes_up_to(n):
    sieve = [True] * (n + 1)
    sieve[0] = sieve[1] = False
    limit = int(math.isqrt(n))
    for i in range(2, limit + 1):
        if sieve[i]:
            for j in range(i * i, n + 1, i):
                sieve[j] = False
    return sieve


def main():
    print(f"Building sieve up to {MAX_LIMIT}...")
    is_prime = sieve_primes_up_to(MAX_LIMIT)
    print("Sieve built.")

    print(f"Interval: [{start}, {end}] for p={p}")

    survivors = []
    count_survived = 0

    print("Filtering integers in the given interval...")
    for n in range(start, end + 1):
        if not is_6kpm1(n):
            continue
        survivors.append(n)
        count_survived += 1

    print("Checking primality of survivors...")
    count_prime = sum(1 for s in survivors if is_prime[s])

    print("Checking twin primes with allowed digit sum pairs...")
    survivor_set = set(survivors)
    twin_count = 0
    for x in survivors:
        if (x + 2) in survivor_set and is_prime[x] and is_prime[x + 2]:
            dr1 = digital_root(x)
            dr2 = digital_root(x + 2)
            if (dr1, dr2) in ALLOWED_DIGIT_SUM_PAIRS:
                twin_count += 1

    print(f"p={p}, Interval: [{start}, {end}]")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs (with allowed digit sum pairs) in survivors: {twin_count}")
    print("Done.")


if __name__ == "__main__":
    main()

