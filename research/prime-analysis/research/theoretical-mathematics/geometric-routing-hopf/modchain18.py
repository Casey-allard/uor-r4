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


# --- Check if a number is prime ---
def is_prime(num):
    """Check if a number is prime."""
    if num <= 1:
        return False
    for i in range(2, int(math.sqrt(num)) + 1):
        if num % i == 0:
            return False
    return True


# --- Optimized Prime Selection ---
def largest_prime_below_sqrt_minus_1(n):
    """Find the largest prime below or equal to sqrt(n)-1 efficiently."""
    x = int(math.sqrt(n)) - 1
    if x <= 2:
        return 2  # Default to prime 2 for very small numbers

    # Start from x and move downwards by 2 (skip evens)
    for i in range(x if x % 2 != 0 else x - 1, 1, -2):
        if is_prime(i):
            return i
    return 2  # Fallback to 2 if nothing else is valid


# --- Modular Reduction Stage ---
def matrix_reduction_optimized(n):
    """Perform modular reduction with anchor chaining and square root exponent validation."""
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

        # Step 1: Calculate square root and check prime exponents
        root_n = math.sqrt(n)
        if root_n.is_integer():
            root_n_int = int(root_n)
            if is_prime(root_n_int):
                print(f"❌ Square root {root_n_int} is a prime exponent. Marked as Composite.")
                trace.append({'n': n, 'sqrt(n)': root_n, 'Prime Exponent Check': 'Composite'})
                return trace, anchor_chain, False

        # Step 2: Calculate sqrt(n)-1 and find largest prime
        p = largest_prime_below_sqrt_minus_1(n)
        if p is None:
            trace.append({'n': n, 'p': None, 'a': None, 'n_mod_a': None})
            break

        # Step 3: Calculate anchor
        a = p * (p + 1)
        anchor_chain.append({'prime': p, 'anchor': a})

        # Step 4: Modular reduction
        n_mod_a = n % a

        # Log the step
        print(f"🔗 Step: n={n}, sqrt(n)-1={int(math.sqrt(n)) - 1}, prime={p}, anchor={a}, n mod a={n_mod_a}")
        trace.append({
            'n': n,
            'sqrt(n)-1': int(math.sqrt(n)) - 1,
            'p (largest prime <= sqrt(n)-1)': p,
            'a (anchor = p * (p+1))': a,
            'n mod a': n_mod_a
        })

        # Update n for next iteration
        n = n_mod_a

    # Final Validation in the 2x3 Matrix
    final_mod_2 = n % 2
    print(f"🏁 Final Residue: {n}, Final Modulo-2 Check: {final_mod_2}")
    trace.append({
        'Final residue': n,
        'Modulo-2 Check': final_mod_2,
        'Is Prime': n == 1 or final_mod_2 == 1
    })

    return trace, anchor_chain, n == 1 or final_mod_2 == 1


# --- Main Program Interface ---
if __name__ == "__main__":
    try:
        user_input = int(input("🔢 Enter a number to test for primality: "))
        if pre_validation_optimized(user_input):
            trace, anchor_chain, is_prime_result = matrix_reduction_optimized(user_input)
            print("\n🔗 Reduction Trace:")
            for step in trace:
                print(step)
            print(f"✅ Prime by Reduction: {is_prime_result}")
        else:
            print("❌ Number is not prime after pre-validation.")
    except ValueError:
        print("❌ Invalid input. Please enter an integer.")

