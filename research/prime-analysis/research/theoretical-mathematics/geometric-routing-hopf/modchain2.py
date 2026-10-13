import math
import time

# Optimized Prime Selection
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


# Modular Reduction with Correct Anchor Handling
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

