import math

MAX_LIMIT = 10_000_000
SMALL_PRIMES = [2,3,5,7,11,13]

# Allowed digit sum pairs as discovered assuming a true digital root:
ALLOWED_DIGIT_SUM_PAIRS = {(2,4), (5,8), (8,1)}

def digital_root(n):
    # Compute the iterative digit sum until one digit remains (the digital root)
    while n >= 10:
        s = 0
        while n > 0:
            s += n % 10
            n //= 10
        n = s
    return n

def passes_digit_sum_filter(n):
    # We previously removed multiples of 3 by checking mod9 conditions.
    # If we still want to exclude multiples of 3, we need to ensure the digital root is not 3,6,9.
    # Multiples of 3 have a digital root of 3,6, or 9.
    # So exclude digital roots in {3,6,9}.
    dr = digital_root(n)
    return dr not in [3,6,9]

def divisible_by_small_primes(n):
    for p in SMALL_PRIMES:
        if n > p and n % p == 0:
            return True
    return False

def sieve_primes_up_to(n):
    """Sieve of Eratosthenes to identify primes up to n."""
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
        # 6k±1 filter for n > 3
        if n > 3 and (n % 6 not in [1,5]):
            continue
        
        # Digital root filter (exclude multiples of 3 via their digital roots)
        if not passes_digit_sum_filter(n):
            continue
        
        # Small prime divisibility check
        if divisible_by_small_primes(n):
            continue
        
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

