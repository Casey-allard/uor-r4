import math
import time

# --- Pre-Validation Stage ---
def pre_validation_optimized(n):
    """Check pre-validation rules for primality."""
    # Divisibility check for small primes
    for p in [2, 3, 5, 7, 11]:
        if n % p == 0:
            print(f"❌ Failed pre-validation: Divisible by {p}")
            return False  # Eliminate if divisible by any of these primes

    # Modulo-6 alignment check
    if n % 6 not in [1, 5]:
        print("❌ Failed pre-validation: Invalid modulo-6 residue.")
        return False  # Eliminate if mod 6 is not 1 or 5

    print("✅ Passed pre-validation checks.")
    return True


# --- Optimized Prime Selection ---
def largest_prime_below_sqrt(n):
    """Find the largest prime below or equal to sqrt(n) efficiently."""
    root_n = int(math.sqrt(n))
    if root_n < 2:
        return None

    # Start from root_n and move downwards by 2 (skip evens)
    for i in range(root_n if root_n % 2 != 0 else root_n - 1, 1, -2):
        if all(i % j != 0 for j in range(3, int(math.sqrt(i)) + 1, 2)):
            return i
    return 2  # If nothing else, return 2 as the fallback prime


# --- Modular Reduction Stage ---
def matrix_reduction_optimized(n):
    """Perform modular reduction with anchor chaining."""
    trace = []
    anchor_chain = []
    start_time = time.time()

    previous_n = None  # Track the previous value to prevent infinite loops

    while n >= 6:
        if time.time() - start_time > 10:  # Timeout safeguard
            print("⏳ Timeout: Reduction taking too long. Exiting.")
            break

        # Detect if n is stuck in a loop
        if n == previous_n:
            print("❌ Stuck in Reduction Loop. Exiting.")
            break

        # Update previous value
        previous_n = n

        # Step 1: Find the largest prime below or equal to sqrt(n)
        p = largest_prime_below_sqrt(n)
        if p is None:
            trace.append({'n': n, 'p': None, 'a': None, 'n_mod_a': None})
            break

        # Step 2: Calculate anchor
        a = p * (p + 1)
        anchor_chain.append({'prime': p, 'anchor': a})

        # Step 3: Modular reduction
        if a > n:
            print(f"⚠️ Anchor ({a}) larger than n ({n}). Proceeding to smaller primes.")
            n_mod_a = n  # Skip reduction, just use the current n
        else:
            n_mod_a = n % a

        # Log the step
        print(f"🔗 Step: n={n}, sqrt(n)={math.sqrt(n):.2f}, prime={p}, anchor={a}, n mod a={n_mod_a}")
        trace.append({
            'n': n,
            'sqrt(n)': math.sqrt(n),
            'p (largest prime <= sqrt(n))': p,
            'a (anchor = p * (p+1))': a,
            'n mod a': n_mod_a
        })

        # Update n for next iteration
        n = n_mod_a

    # Final residue
    trace.append({'Final residue': n, 'Is Prime': n in [1, 2, 3, 5]})
    return trace, anchor_chain


# --- Final Validation Stage ---
def primality_test(n):
    """Evaluate the primality of any number using modular reduction."""
    print(f"\n🛠️ Testing Primality for n = {n}")

    # Step 1: Pre-Validation
    if not pre_validation_optimized(n):
        print("❌ Number failed pre-validation. Not Prime.")
        return {'n': n, 'Valid for Matrix Reduction': False, 'Prime by Reduction': False}

    print("✅ Passed pre-validation. Proceeding to modular reduction...")

    # Step 2: Modular Reduction
    reduction_trace, anchor_chain = matrix_reduction_optimized(n)
    final_residue = reduction_trace[-1]['Final residue']
    is_prime = reduction_trace[-1]['Is Prime']

    # Step 3: Display Results
    print("\n🔗 Anchor Chain:")
    for i, anchor in enumerate(anchor_chain):
        print(f"  Step {i+1}: Prime = {anchor['prime']}, Anchor = {anchor['anchor']}")

    print("\n🌀 Reduction Trace:")
    for step in reduction_trace:
        print(step)

    print(f"\n🏁 Final Residue: {final_residue}")
    print(f"✅ Prime by Reduction: {is_prime}")

    return {
        'n': n,
        'Valid for Matrix Reduction': True,
        'Matrix Reduction Steps': reduction_trace,
        'Anchor Chain': anchor_chain,
        'Final Residue': final_residue,
        'Prime by Reduction': is_prime
    }


# --- Main Program Interface ---
if __name__ == "__main__":
    try:
        user_input = int(input("🔢 Enter a number to test for primality: "))
        result = primality_test(user_input)
    except ValueError:
        print("❌ Invalid input. Please enter an integer.")

