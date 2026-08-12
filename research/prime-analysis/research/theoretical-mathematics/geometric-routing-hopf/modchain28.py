import math

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


# ✅ Modular Anchor Chain Reduction with 210 Stability Threshold
def modular_anchor_chain_with_210_threshold(n):
    """
    Implements anchor chain reduction with 210 as the stability threshold.
    - Stops propagation below anchor 210.
    - Performs explicit primality validation for residues below this threshold.
    """
    trace = []
    threshold = 12  # Stabilization threshold
    stability_threshold = 210  # Critical anchor boundary

    propagation_chain = []  # To track propagation history

    while n >= threshold:
        # Calculate root and find the largest prime below sqrt(n+1)
        x = int(math.sqrt(n + 1))
        p = next((i for i in range(x if x % 2 != 0 else x - 1, 1, -2) if is_prime(i)), 2)
        
        # Calculate the anchor
        a = p * (p + 1)
        
        # Check if anchor is below the stability threshold (210)
        if a < stability_threshold:
            final_residue = n % a
            is_residue_prime = is_prime(final_residue)
            propagation_chain.append(final_residue)
            trace.append({
                'n': n,
                'x': x,
                'p': p,
                'a': a,
                'final_residue': final_residue,
                'explicit_validation': is_residue_prime,
                'propagation_chain': propagation_chain,
                'stopped_at_threshold': True
            })
            return trace, is_residue_prime
        
        n_mod_a = n % a

        # Log the reduction step
        propagation_chain.append(n_mod_a)
        trace.append({
            'n': n,
            'x': x,
            'p': p,
            'a': a,
            'n_mod_a': n_mod_a,
            'propagation_chain': propagation_chain.copy()
        })
        
        # Update n for the next iteration
        n = n_mod_a

    # Explicit final validation and backtrace verification
    final_validation = is_prime(n)
    propagation_chain.append(n)
    trace.append({
        'n': n,
        'explicit_final_validation': final_validation,
        'propagation_chain': propagation_chain
    })
    return trace, final_validation


# ✅ User Interaction and Program Execution
def main():
    print("🛠️ Modular Anchor Chain Validation Program 🛠️")
    print("Enter a number to check if it's prime using modular anchor reduction with a stability threshold of 210.")
    
    try:
        n = int(input("🔢 Enter a number: "))
        if n <= 0:
            print("❌ Please enter a positive integer.")
            return
        
        print("\n🔍 Evaluating...\n")
        
        trace, is_prime_result = modular_anchor_chain_with_210_threshold(n)
        
        if is_prime_result:
            print(f"✅ {n} is PRIME!")
        else:
            print(f"❌ {n} is NOT prime.")
        
        print("\n🔗 Detailed Trace Steps:")
        for step in trace:
            print(step)
    
    except ValueError:
        print("❌ Invalid input. Please enter an integer.")


# ✅ Run the Program
if __name__ == "__main__":
    main()

