import math

# Configuration
MAX_LIMIT = 100000  # Increase as needed after initial tests

# Small primes to exclude
SMALL_PRIMES = [2,3,5,7,11,13]

def reduced_digit_sum_mod9(n):
    # Compute the reduced digit sum until a single digit and return mod 9
    # Actually, for mod 9, the reduced digit sum mod 9 is just n mod 9
    # But to strictly follow the "reduced digit sum" idea:
    # We'll do a proper digit sum repeatedly until one digit remains.
    # However, for checking divisible by 3 conditions, n mod 9 == digit sum mod 9 is enough.
    s = 0
    while n > 0:
        s += n % 10
        n //=10
    # If s >= 10, reduce again
    # But for mod 9 checks, this is equivalent to s mod 9 directly.
    return s % 9

def passes_digit_sum_filter(n):
    # We keep numbers whose reduced digit sum mod 9 is NOT divisible by 3
    # That means digit_sum_mod_9(n) != 0, 3, 6
    # Actually, since mod 9, multiples of 3 are 0, 3, or 6 mod 9.
    r = reduced_digit_sum_mod9(n)
    return r not in [0,3,6]

def divisible_by_small_primes(n):
    # Check divisibility by {2,3,5,7,11,13}
    # Since we are already going to skip even and multiples of 3 due to mod conditions,
    # let's just be explicit here:
    for p in SMALL_PRIMES:
        if n > p and n % p == 0:
            return True
    return False

# A basic primality test for demonstration:
# For up to 10^7 or 10^8, a sieve would be faster, but let's start simple.
def is_prime(n):
    if n < 2:
        return False
    for p in SMALL_PRIMES:
        if p < n and n % p == 0:
            return n == p
    # trial division up to sqrt(n) for simplicity
    limit = int(math.isqrt(n))
    for i in range(17, limit+1, 2): # skip even numbers
        if n % i == 0:
            return False
    return True

def main():
    survivors = []
    # We know primes > 3 are in form 6k ± 1, but let's not rely solely on that initially.
    # We'll filter everything straightforwardly first.

    # To possibly optimize:
    # - Skip even numbers right away
    # - Skip multiples of 3 (digit sum filter will handle it, but let's rely on the filter itself)
    
    count_checked = 0
    count_survived = 0
    count_prime = 0
    
    for n in range(2, MAX_LIMIT+1):
        # Check digit sum filter
        if not passes_digit_sum_filter(n):
            continue
        # Check small prime divisibility
        if divisible_by_small_primes(n):
            continue
        
        # If we reach here, n is in F
        survivors.append(n)
        count_survived += 1
        
    # Now check how many of these are prime
    for s in survivors:
        if is_prime(s):
            count_prime += 1
    
    # Check for twin primes
    survivor_set = set(survivors)
    twin_count = 0
    for p in survivors:
        if (p+2) in survivor_set:
            # (p, p+2) is a twin prime pair
            if p < p+2:
                twin_count += 1
    
    # Print out basic results
    print(f"Max Limit: {MAX_LIMIT}")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs in survivors: {twin_count}")
    print("Done.")

if __name__ == "__main__":
    main()

