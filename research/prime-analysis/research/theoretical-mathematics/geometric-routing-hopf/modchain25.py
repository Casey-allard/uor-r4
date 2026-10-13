import math
import time

# --- Pre-Validation Stage ---
def pre_validation_optimized(n):
    """Check pre-validation rules for primality."""
    # Divisibility check for small primes
    small_primes = [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89
    ]
    for p in small_primes:
        if n % p == 0 and n != p:
            print(f"❌ Failed pre-validation: Divisible by {p}")
            return False
    if n % 6 not in [1, 5]:
        print("❌ Failed pre-validation: Invalid modulo-6 residue.")
        return False
    print("✅ Passed pre-validation checks.")
    return True


# --- Check if a number is prime ---
def is_prime(num):
    """Check if a number is prime."""
    if num <= 1:
        return False
    for i in range(2, int(math.sqrt(num)) + 1):
        if num % i == 0:
            return False
    return True


# --- Prime and Anchor Selection using sqrt(n+1) ---
def largest_prime_below_adjusted_root_plus_1(n):
    """Find the largest prime below or equal to sqrt(n+1) efficiently."""
    adjusted_n = n + 1
    if adjusted_n <= 0:
        return 2
    x = int(math.sqrt(adjusted_n))
    if x <= 2:
        return 2
    for i in range(x if x % 2 != 0 else x - 1, 1, -2):
        if is_prime(i):
            return i
    return 2


# --- Recursive Modular Reduction with Explicit Type Handling ---
def recursive_reduction(n):
    """Recursive modular reduction with residue validation."""
    trace = []
    anchor_chain = []
    start_time = time.time()
    previous_n = None

    prime_exponents = [9, 25, 49]

    while n >= 12:
        if time.time() - start_time > 10:
            print("⏳ Timeout: Reduction taking too long. Exiting.")
            break

        if n == previous_n:
            print("❌ Stuck in Reduction Loop. Exiting.")
            break

        previous_n = n

        try:
            root_n = int(math.sqrt(n + 1))
            p = largest_prime_below_adjusted_root_plus_1(n)
            if not isinstance(p, int):
                raise ValueError("Prime selection failed to return an integer.")

            a = p * (p + 1)
            if not isinstance(a, int):
                raise ValueError("Anchor calculation failed to return an integer.")

            n_mod_a = n % a
            if not isinstance(n_mod_a, int):
                raise ValueError("Modulo reduction did not return an integer.")

            if n_mod_a in prime_exponents:
                print(f"❌ Prime exponent residue detected: {n_mod_a}. Marking as Not Prime.")
                trace.append({'n': n, 'Residue': n_mod_a, 'Validation': 'Prime Exponent Detected'})
                return trace, anchor_chain, False

            print(f"🔗 Step: n={n}, sqrt(n+1)={root_n}, prime={p}, anchor={a}, n mod a={n_mod_a}")
            trace.append({
                'n': n,
                'sqrt(n+1)': root_n,
                'p (largest prime <= sqrt(n+1))': p,
                'a (anchor = p * (p+1))': a,
                'n mod a': n_mod_a
            })

            n = n_mod_a

        except Exception as e:
            print(f"❌ Exception occurred: {e}")
            trace.append({'error': str(e), 'n': n})
            return trace, anchor_chain, False

    # Final Validation for n < 12
    final_anchor = 6
    final_residue = n % final_anchor

    trace.append({
        'Final residue': final_residue,
        'Validation': 'Final 2x3 Matrix Validation'
    })

    if final_residue in [1, 5]:
        print(f"🏁 Final Residue: {final_residue} → ✅ Prime by Final Validation")
        return trace, anchor_chain, True

    print(f"🏁 Final Residue: {final_residue} → ❌ Not Prime by Final Validation")
    return trace, anchor_chain, False


# --- Main Program Interface ---
if __name__ == "__main__":
    try:
        user_input = int(input("🔢 Enter a number to test for primality: "))
        if pre_validation_optimized(user_input):
            trace, anchor_chain, is_prime_result = recursive_reduction(user_input)
            print("\n🔗 Reduction Trace:")
            for step in trace:
                print(step)
            print(f"✅ Prime by Reduction: {is_prime_result}")
        else:
            print("❌ Number is not prime after pre-validation.")
    except ValueError:
        print("❌ Invalid input. Please enter an integer.")

