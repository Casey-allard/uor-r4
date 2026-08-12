import math

MAX_LIMIT = 10_000_000

SMALL_PRIMES = [2,3,5,7,11,13]
# Primes > 5 are always in one of these residues mod 30:
PRIME_RESIDUES_MOD_30 = {1,7,11,13,17,19,23,29}

ALLOWED_DIGIT_SUM_PAIRS = {(2,4), (5,8), (8,1)}

def digital_root(n):
    # Compute the iterative digit sum (digital root)
    while n >= 10:
        s = 0
        while n > 0:
            s += n % 10
            n //= 10
        n = s
    return n

def passes_digit_sum_filter(n):
    # Exclude multiples of 3: digital roots in {3,6,9}
    dr = digital_root(n)
    return dr not in [3,6,9]

def divisible_by_small_primes(n):
    for p in SMALL_PRIMES:
        if n > p and n % p == 0:
            return True
    return False

def sieve_primes_up_to(n):
    sieve = [True]*(n+1)
    sieve[0] = False
    sieve[1] = False
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

    survivors = []
    count_survived = 0
    
    print("Filtering integers...")
    for n in range(2, MAX_LIMIT+1):
        # Primes > 3 must be 6k±1, and primes > 5 must also be in PRIME_RESIDUES_MOD_30 if > 5.
        # First, handle the mod 30 residue filter for n > 5
        if n > 5 and (n % 30 not in PRIME_RESIDUES_MOD_30):
            continue
        
        # Digit sum filter (excludes multiples of 3)
        if not passes_digit_sum_filter(n):
            continue
        
        # Small prime divisibility filter
        if divisible_by_small_primes(n):
            continue
        
        # If we reach here, n is a survivor
        survivors.append(n)
        count_survived += 1
    
    print("Checking primality of survivors...")
    count_prime = sum(1 for s in survivors if is_prime[s])
    
    print("Checking twin primes with allowed digit sum pairs...")
    survivor_set = set(survivors)
    twin_count = 0
    for p in survivors:
        if (p+2) in survivor_set and is_prime[p] and is_prime[p+2]:
            r1 = digital_root(p)
            r2 = digital_root(p+2)
            if (r1, r2) in ALLOWED_DIGIT_SUM_PAIRS:
                twin_count += 1

    print(f"Max Limit: {MAX_LIMIT}")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs (with allowed digit sum pairs) in survivors: {twin_count}")
    print("Done.")

if __name__ == "__main__":
    main()

