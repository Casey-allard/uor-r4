import math

MAX_LIMIT = 10_000_000
p = 1007  # Example prime p > 3
start = p*p
end = p*p + 12*p + 36
if end > MAX_LIMIT:
    raise ValueError("Increase MAX_LIMIT or choose a smaller p so that p^2+12p+36 <= MAX_LIMIT")

ALLOWED_DIGIT_SUM_PAIRS = {(2,4), (5,7), (8,1)}

def digital_root(n):
    # Compute the iterative digit sum (digital root)
    while n >= 10:
        s = 0
        while n > 0:
            s += n % 10
            n //= 10
        n = s
    return n

def is_6kpm1(n):
    # Primes > 3 are of form 6k ± 1
    if n > 3 and (n % 6 not in [1,5]):
        return False
    return True

def not_multiple_of_3_by_digit_sum(n):
    # Exclude multiples of 3: digital roots 3,6,9
    dr = digital_root(n)
    return dr not in [3,6,9]

def digit_sum_mod9(n):
    s = 0
    while n > 0:
        s += n % 10
        n //=10
    return s % 9

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

    print(f"Interval: [{start}, {end}] for p={p}")
    survivors = []
    count_survived = 0

    print("Filtering integers in the given interval...")
    for n in range(start, end+1):
        if not is_6kpm1(n):
            continue
        if not not_multiple_of_3_by_digit_sum(n):
            continue
        # Additional Filter from analysis:
        # Keep only n if n % 30 in {11,29}, as derived from the proof's examples.
        if (n % 30) not in {11,29}:
            continue
        survivors.append(n)
        count_survived += 1

    print("Checking primality of survivors...")
    count_prime = sum(1 for s in survivors if is_prime[s])

    print("Checking twin primes with allowed digit sum pairs...")
    survivor_set = set(survivors)
    twin_count = 0
    for x in survivors:
        if (x+2) in survivor_set and is_prime[x] and is_prime[x+2]:
            dr1 = digital_root(x)
            dr2 = digital_root(x+2)
            # Convert digit roots to mod 9 equivalents for allowed pairs:
            # Actually the proof states pairs (2,4), (5,7), (8,1) are from examples of reduced digit sums.
            # The reduced digit sum is a single digit (1-9), directly compare them:
            if (dr1, dr2) in ALLOWED_DIGIT_SUM_PAIRS:
                twin_count += 1

    print(f"p={p}, Interval: [{start}, {end}]")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs (with allowed digit sum pairs) in survivors: {twin_count}")
    print("Done.")

if __name__ == "__main__":
    main()

