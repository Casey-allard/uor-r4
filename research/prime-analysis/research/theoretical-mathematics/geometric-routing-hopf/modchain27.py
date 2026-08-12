import math

# ✅ Pre-validation Optimized
def pre_validation_optimized(n):
    """
    Pre-validates if n can be prime.
    - Checks divisibility by small primes.
    - Checks modulo-6 alignment.
    """
    if n <= 1:
        return False

    small_primes = [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89
    ]
    for p in small_primes:
        if n % p == 0 and n != p:
            return False
    if n % 6 not in [1, 5]:
        return False
    return True

# ✅ Prime Check Utility
def is_prime(num):
    """
    Check if a number is prime using trial division.
    """
    if num <= 1:
        return False
    for i in range(2, int(math.sqrt(num)) + 1):
        if num % i == 0:
            return False
    return True

# ✅ Recursive Modular Reduction
def recursive_reduction_with_state_reset(n):
    """
    Recursively reduces n using modular anchors derived from primes below sqrt(n+1).
    Explicitly validates small numbers (<100) directly before reduction.
    """
    # Early Prime Check for Small Numbers
    if n <= 100:
        return [], is_prime(n)
    
    trace = []
    previous_n = None
    prime_exponents = [9, 25, 49]

    while n >= 12:
        if n == previous_n:
            break
        previous_n = n

        # Increment
        n = n + 1

        # Square root and prime selection
        root_n = int(math.sqrt(n))
        p = next((i for i in range(root_n if root_n % 2 != 0 else root_n - 1, 1, -2) if is_prime(i)), 2)

        # Anchor calculation
        a = p * (p + 1)
        n_mod_a = n % a

        # Log the step
        trace.append({'n': n, 'p': p, 'a': a, 'n_mod_a': n_mod_a})
        n = n_mod_a

    # Explicit validation for small residues
    if n < 12:
        return trace, is_prime(n)

    # Final Validation for larger numbers
    if n in prime_exponents:
        return trace, False
    final_residue = n % 6
    return trace, final_residue in [1, 5]

# ✅ User Interaction
def main():
    print("🛠️ Modular Prime Evaluation Program 🛠️")
    print("Enter a number to check if it's prime:")
    
    try:
        n = int(input("🔢 Number: "))
        if n <= 0:
            print("❌ Please enter a positive integer.")
            return
        
        print("\n🔍 Evaluating...\n")
        
        if not pre_validation_optimized(n):
            print(f"❌ {n} failed pre-validation and is likely NOT prime.")
            return
        
        trace, is_prime_result = recursive_reduction_with_state_reset(n)
        
        if is_prime_result:
            print(f"✅ {n} is PRIME!")
        else:
            print(f"❌ {n} is NOT prime.")
        
        print("\n🔗 Trace Steps:")
        if trace:
            for step in trace:
                print(f"n: {step['n']}, p: {step['p']}, a: {step['a']}, n_mod_a: {step['n_mod_a']}")
        else:
            print("ℹ️ Direct validation was applied, no recursive reduction trace.")
    
    except ValueError:
        print("❌ Invalid input. Please enter an integer.")

if __name__ == "__main__":
    main()

