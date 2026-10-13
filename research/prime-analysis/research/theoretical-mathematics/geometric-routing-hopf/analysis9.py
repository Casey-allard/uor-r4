import math

MAX_LIMIT = 10000000
SMALL_PRIMES = [2,3,5,7,11,13]

def reduced_digit_sum_mod9(n):
    s = 0
    while n > 0:
        s += n % 10
        n //=10
    return s % 9

def passes_digit_sum_filter(n):
    r = reduced_digit_sum_mod9(n)
    return r not in [0,3,6]  # exclude multiples of 3

def divisible_by_small_primes(n):
    for p in SMALL_PRIMES:
        if n > p and n % p == 0:
            return True
    return False

def is_prime(n):
    if n < 2:
        return False
    # We already removed multiples of SMALL_PRIMES from the set,
    # but we still need to confirm primality for those that remain.
    # Just do a standard trial division up to sqrt(n).
    limit = int(math.isqrt(n))
    # Check divisibility by odd numbers from 17 upwards:
    # Actually, to be completely safe, check all numbers from 2 upwards,
    # skipping multiples of known small primes might speed it up but let's keep it simple.
    for i in range(2, limit+1):
        if n % i == 0:
            return i == n
    return True

def main():
    survivors = []
    count_survived = 0
    count_prime = 0
    
    for n in range(2, MAX_LIMIT+1):
        if not passes_digit_sum_filter(n):
            continue
        if divisible_by_small_primes(n):
            continue
        # If we reach here, n is in F
        survivors.append(n)
        count_survived += 1
        
    # Check primality
    count_prime = sum(1 for s in survivors if is_prime(s))
    
    # Check twin primes
    survivor_set = set(survivors)
    twin_count = 0
    for p in survivors:
        if (p+2) in survivor_set:
            twin_count += 1

    print(f"Max Limit: {MAX_LIMIT}")
    print(f"Survivors: {count_survived}")
    print(f"Primes in survivors: {count_prime}")
    print(f"Twin prime pairs in survivors: {twin_count}")
    print("Done.")

if __name__ == "__main__":
    main()

