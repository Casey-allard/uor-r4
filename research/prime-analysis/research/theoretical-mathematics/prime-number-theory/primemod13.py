import math
from sympy import isprime

def get_max_prime_below(x):
    """Find the largest prime number less than or equal to x."""
    if x < 2:
        return 2
    for i in range(x, 1, -1):
        if i == 2 or all(i % d != 0 for d in range(2, int(math.sqrt(i)) + 1)):
            print(f"[DEBUG] get_max_prime_below: Found prime {i} for x={x}")
            return i
    print(f"[ERROR] get_max_prime_below: No valid prime found for x={x}, defaulting to 2")
    return 2

def modular_prime_anchor_chain_verbose(n):
    steps = []
    previous_n = n
    iteration = 0
    while n > 6:
        iteration += 1
        x = int(n**(1/3))  # Cube root rounded down to an integer
        p = get_max_prime_below(x)  # Largest prime <= x
        if p == 2 and x > 2:
            print(f"[ERROR] Failed to find a valid prime below cube root {x}. Fallback to 2.")
        
        # Alternate anchor calculation for stability
        if iteration % 2 == 0:
            a = p * (p + 1)
        else:
            a = p * (p - 1)
        
        reduced = n % a
        print(f"[DEBUG - Iteration {iteration}] n={n}, Previous_n={previous_n}, CubeRoot={x}, Prime={p}, Anchor={a}, Reduced={reduced}")
        steps.append((n, x, p, a, reduced))
        
        if reduced == previous_n:
            print(f"[ERROR] Reduction stalled at Iteration {iteration}: {n} did not change. Breaking loop.")
            break
        previous_n = n
        n = reduced
    print(f"[FINAL DEBUG] Final reduced value: {n}")
    return n, steps  # Return final reduced value

def validate_final_residue(n):
    """Ensure the final residue aligns with valid prime indicators."""
    if n in [1, 5]:
        print(f"[VALIDATION] Final residue {n} aligns with prime indicators (1, 5)")
        return True
    print(f"[VALIDATION ERROR] Final residue {n} does NOT align with prime indicators (1, 5)")
    return False

def is_prime_modular_anchor(n):
    final_value, steps = modular_prime_anchor_chain_verbose(n)
    is_prime_result = validate_final_residue(final_value)
    print(f"[DEBUG] Final Value: {final_value}, Prime Check: {is_prime_result}")
    return is_prime_result, steps

def interactive_test_case():
    try:
        number = int(input("Enter a number to check if it's prime: "))
        print(f"\n[INTERACTIVE TEST CASE] Testing number: {number}")
        is_prime_result, steps = is_prime_modular_anchor(number)
        print("Reduction Steps:")
        for idx, (n, x, p, a, reduced) in enumerate(steps):
            print(f"Step {idx+1}: n={n}, CubeRoot={x}, Prime={p}, Anchor={a}, Reduced={reduced}")
        print(f"Final Prime Check: {'PRIME' if is_prime_result else 'COMPOSITE'}")
        print(f"Verification with SymPy primality test: {'PRIME' if isprime(number) else 'COMPOSITE'}")
        print(f"[SUMMARY] Final Value: {steps[-1][-1] if steps else n}, Steps Taken: {len(steps)}")
    except ValueError:
        print("Invalid input. Please enter a valid integer.")

def main():
    print("[USER INTERACTIVE MODE]")
    interactive_test_case()
    
if __name__ == '__main__':
    main()

