import math
import time
from sympy import primerange, isprime

# Prime alignment nodes at smallest anchor (12)
PRIME_ALIGNMENT_NODES = [1, 5, 7, 11]

# Step 1: Generate Modular Anchors Dynamically from Primes Below sqrt(N)
def generate_anchors(limit):
    """Generate anchors for all primes below sqrt(N)."""
    primes = list(primerange(2, limit + 1))  # Generate all primes below sqrt(N)
    anchors = list(set(p * (p + 1) for p in primes if p * (p + 1) <= limit))
    return sorted(anchors, reverse=True)

# Step 2: Dynamic Modular Reduction
def modular_reduction(number):
    """Perform dynamic modular reduction with recalculating anchors."""
    reduction_path = []
    current_value = number
    
    while current_value > 12:
        sqrt_n = math.isqrt(current_value)
        anchors = generate_anchors(sqrt_n)
        
        if not anchors:
            break
        
        anchor = max(anchors)  # Take the largest valid anchor
        reduced_value = current_value % anchor
        reduction_path.append((anchor, reduced_value))
        current_value = reduced_value
        
        # Early Primality Check on Reduced Value
        if current_value < 12:
            break
        if current_value % 2 == 0 or current_value % 5 == 0:
            return reduction_path, False  # Clearly not prime
        
        if current_value < 1000 and not isprime(current_value):
            return reduction_path, False  # Check small primes explicitly
    
    return reduction_path, current_value in PRIME_ALIGNMENT_NODES

# Step 3: Main Function for User Input and Execution
def main():
    print("🔍 Modular Reduction Primality Testing Framework")
    number = int(input("Enter a number to evaluate for primality: "))
    
    start_time = time.time()
    
    # Perform reduction dynamically
    reduction_path, is_prime = modular_reduction(number)
    
    end_time = time.time()
    execution_time = end_time - start_time
    
    # Display results
    print("\n📝 Reduction Path:")
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

