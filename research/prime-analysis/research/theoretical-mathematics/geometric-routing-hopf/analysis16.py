import math

# Configuration
MAX_LIMIT = 10_000_000
p = 1007  # Example prime p > 3, adjust as needed
# Interval [p^2, p^2 + 12p + 36] = [p^2, (p+6)^2]
start = p*p
end = (p+6)*(p+6)
if end > MAX_LIMIT:
    raise ValueError("Increase MAX_LIMIT or choose a smaller p so that (p+6)^2 <= MAX_LIMIT")

SMALL_PRIMES = [5,7,11,13]  # from original proof beyond 2 and 3 which we exclude by digit sum & 6k±1

ALLOWED_MOD9_PAIRS = {(2,4), (5,8), (8,1)}

def digit_sum_mod9(n):
    s = 0
    while n > 0:
        s += n % 10
        n //=10
    return s % 9

def passes_digit_sum_filter_mod9(n):
    # Remove multiples of 3: digit_sum mod9 in {0,3,6}
    r = digit_sum_mod9(n)
    return r not in [0,3,6]

def is_6kpm1(n):
    # For n > 3, primes are 6k ± 1.
    # Keep n if n <= 3 or n % 6 in {1,5}.
    if n > 3 and (n % 6 not in [1,5]):
        return False
    return True

def divisible_by_small_primes(n):
    # We assume multiples of 2 and 3 are removed by digit sum and 6k±1 conditions
    # Check only 5,7,11,13 now
    for q in SMALL_PRIMES:
        if n > q and n % q == 0:
            return True
    return False

def sieve_primes_up_to(n):
    sieve = [True]*(n+1)
    sieve[0] = sieve[1] = False
    limit = int(math.isqrt(n))
    for i in range(2, limit+1):
        if sieve[i]:
            for j in range(i*i, n+1, i):
                sieve[j] = False
    return sieve

def main():
    print(f"Building sieve up to {MAX_LIMIT}...")
    is_prime = sieve_primes_up_to(MAX_LIMIT)
    print("Sieve built.")

    # Construct the interval [p^2, (p+6)^2]
    print(f"Interval: [{start}, {end}] for p={p}")

    survivors = []
    count_survived = 0

    # Filtering according to original proof style
    print("Filtering integers in the given interval...")
    for n in range(start, end+1):
        # 6k±1 filter
        if not is_6kpm1(n):
            continue
        # digit sum mod9 filter to remove multiples of 3
        if not passes_digit_sum_filter_mod9(n):
            continue
        # remove multiples of 5,7,11,13
        if divisible_by_small_primes(n):
            continue
        # If reached here, n is a survivor
        survivors.append(n)
        count_survived += 1

    print("Checking primality of survivors...")
    count_prime = sum(1 for s in survivors if is_prime[s])

    print("Checking twin primes with allowed digit sum pairs...")
    survivor_set = set(survivors)
    twin_count = 0
    for x in survivors:
        if (x+2) in survivor_set and is_prime[x] and is_prime[x+2]:
            r1 = digit_sum_mod9(x)
            r2 = digit_sum_mod9(x+2)
            if (r1,r2) in ALLOWED_MOD9_PAIRS:
                twin_count += 1

    print(f"p={p}, Interval: [{start}, {end}]")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs (with allowed digit sum pairs) in survivors: {twin_count}")
    print("Done.")

if __name__ == "__main__":
    main()

