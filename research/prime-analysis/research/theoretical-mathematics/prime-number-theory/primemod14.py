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
        
        # Handle small final residues explicitly
        if n <= 27:
            print(f"[DEBUG - Iteration {iteration}] Small residue handling triggered for n={n}")
            valid_residues_6 = {1, 5}
            valid_residues_27 = {1, 5, 7, 11, 13, 17, 19, 23}
            valid_residues_30 = {1, 7, 11, 13, 17, 19, 23, 29}
            
            if n <= 6 and n in valid_residues_6:
                print(f"[VALIDATION] Final small residue {n} aligns with mod 6 valid residues {valid_residues_6}")
                return n, steps
            elif n <= 27 and n in valid_residues_27:
                print(f"[VALIDATION] Final small residue {n} aligns with mod 27 valid residues {valid_residues_27}")
                return n, steps
            elif n <= 30 and n in valid_residues_30:
                print(f"[VALIDATION] Final small residue {n} aligns with mod 30 valid residues {valid_residues_30}")
                return n, steps
            
            print(f"[VALIDATION ERROR] Final small residue {n} does NOT align with valid residues for mod 6, 27, or 30")
            return n, steps
        
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
    valid_residues_6 = {1, 5}
    valid_residues_27 = {1, 5, 7, 11, 13, 17, 19, 23}
    valid_residues_30 = {1, 7, 11, 13, 17, 19, 23, 29}
    
    if n <= 6 and n in valid_residues_6:
        print(f"[VALIDATION] Final residue {n} aligns with mod 6 valid residues {valid_residues_6}")
        return True
    if n <= 27 and n in valid_residues_27:
        print(f"[VALIDATION] Final residue {n} aligns with mod 27 valid residues {valid_residues_27}")
        return True
    if n <= 30 and n in valid_residues_30:
        print(f"[VALIDATION] Final residue {n} aligns with mod 30 valid residues {valid_residues_30}")
        return True
    print(f"[VALIDATION ERROR] Final residue {n} does NOT align with valid residues for mod 6, 27, or 30")
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

