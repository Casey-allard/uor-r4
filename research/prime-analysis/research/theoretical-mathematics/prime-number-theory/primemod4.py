import math
import time
from sympy import primerange

# Prime alignment nodes at smallest anchor (12)
PRIME_ALIGNMENT_NODES = [1, 5, 7, 11]

# Step 1: Generate Modular Anchors from Primes Below sqrt(N)
def generate_anchors(limit):
    """Dynamically generate modular anchors from prime matrices below sqrt(N)."""
    primes = list(primerange(2, limit + 1))  # Generate all primes up to sqrt(N)
    anchors = list(set(p * (p + 1) for p in primes if p * (p + 1) <= limit))
    return sorted(anchors, reverse=True)

# Step 2: Modular Reduction
def modular_reduction(number, anchors):
    """Perform modular reduction through the selected anchors in descending order."""
    reduction_path = []
    current_value = number
    
    for anchor in anchors:
        reduced_value = current_value % anchor
        reduction_path.append((anchor, reduced_value))
        current_value = reduced_value  # Pass reduced value to next step
    
    return reduction_path

# Step 3: Alignment Verification
def verify_alignment(reduction_path):
    """Verify if the final reduction aligns with prime nodes at anchor 12."""
    for anchor, value in reduction_path:
        if anchor == 12:
            return value % 12 in PRIME_ALIGNMENT_NODES
    return False

# Step 4: Main Function for User Input and Execution
def main():
    print("🔍 Modular Reduction Primality Testing Framework")
    number = int(input("Enter a number to evaluate for primality: "))
    
    start_time = time.time()
    
    # Calculate square root and anchors dynamically
    sqrt_n = math.isqrt(number)
    anchors = generate_anchors(sqrt_n)
    anchor_chain_length = len(anchors)
    
    print(f"✅ Anchor chain length: {anchor_chain_length}")
    print(f"🔗 Anchors used (descending order): {anchors}")
    
    # Perform reduction
    reduction_path = modular_reduction(number, anchors)
    is_prime = verify_alignment(reduction_path)
    
    end_time = time.time()
    execution_time = end_time - start_time
    
    # Display results
    print("\n📝 Reduction Path (in descending order):")
    for anchor, value in reduction_path:
        print(f"  Mod {anchor}: {value}")
    
    if is_prime:
        print("\n🎯 Result: The number is PRIME!")
    else:
        print("\n❌ Result: The number is NOT PRIME.")
    
    print(f"⏱️ Execution Time: {execution_time:.6f} seconds")

# Entry point
if __name__ == "__main__":
    main()

