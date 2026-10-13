import math
from sympy import isprime

def modular_prime_anchor_chain_verbose(n):
    steps = []
    while n > 6:
        x = int(n**(1/3))  # Cube root rounded down to an integer
        # Ensure there's a valid prime in range
        primes_in_range = [i for i in range(2, x+1) if all(i % d != 0 for d in range(2, int(i**0.5)+1))]
        if primes_in_range:
            p = max(primes_in_range)  # Largest prime <= x
            nearby_primes = sorted(primes_in_range)[-3:]  # Take top 3 largest primes if available
            a = math.prod([p * (p + 1) for p in nearby_primes])  # Combined modulus
        else:
            p = 2  # Default to the smallest prime if none found
            a = p * (p + 1)
        reduced = n % a
        steps.append((n, x, p, a, reduced))
        n = reduced
    return n % 2, steps  # Final modulo 2 check to ensure binary result

def is_prime_modular_anchor(n):
    final_value, steps = modular_prime_anchor_chain_verbose(n)
    is_prime_result = final_value == 1  # 1 for prime, 0 for composite
    return is_prime_result, steps

def main():
    print("Welcome to the Modular Anchor Prime Checker!")
    try:
        number = int(input("Enter a number to check if it's prime: "))
        print(f"Evaluating primality for: {number}\n")
        
        # Perform Modular Anchor Reduction
        is_prime_result, steps = is_prime_modular_anchor(number)
        
        # Display step-by-step reduction
        print("Reduction Steps:")
        for idx, (n, x, p, a, reduced) in enumerate(steps):
            print(f"Step {idx+1}: n={n}, CubeRoot={x}, Prime={p}, Anchor={a}, Reduced={reduced}")
        
        # Display final result
        if is_prime_result:
            print(f"\n✅ The number {number} is likely PRIME based on the modular anchor method!")
        else:
            print(f"\n❌ The number {number} is COMPOSITE based on the modular anchor method.")
        
        # Cross-verify with sympy's primality test
        standard_check = isprime(number)
        print(f"(Verification with SymPy primality test: {'PRIME' if standard_check else 'COMPOSITE'})")
        
    except ValueError:
        print("Invalid input. Please enter a valid integer.")
    
if __name__ == '__main__':
    main()

