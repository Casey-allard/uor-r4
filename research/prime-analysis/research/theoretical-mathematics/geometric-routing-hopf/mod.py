import math
from sympy import isprime

def modular_alignment_details(n, layers=4):
    """
    Perform modular alignment analysis and display step-by-step details.
    """
    alignment_steps = []
    current_value = n
    
    for i in range(1, layers + 1):
        max_anchor = i * 4  # Simulating anchor nodes at quarter-turn increments
        residue = current_value % max_anchor
        alignment_steps.append((max_anchor, residue))
        current_value = residue if residue != 0 else current_value
    
    return alignment_steps

def check_prime_recursive(n):
    """
    Apply recursive anchor method to check if a number is prime.
    """
    if n < 2:
        return False, 'Number is less than 2 and not prime.'
    
    if isprime(n):
        return True, 'Direct check confirms number is prime.'
    
    alignment_steps = modular_alignment_details(n)
    reset_nodes = [0, 90, 180, 270]
    tangential_nodes = []
    
    for step in alignment_steps:
        anchor, residue = step
        if anchor % 90 == 0 and residue == 1:
            return True, f'Number aligns with reset node at anchor {anchor}.'
        if residue in reset_nodes:
            tangential_nodes.append(anchor)
    
    if tangential_nodes:
        return True, f'Number shows tangential alignment at nodes: {tangential_nodes}'
    
    return False, 'Number does not align with reset or tangential nodes.'

def test_large_primes():
    test_numbers = [10000019, 10000079, 10000111, 10000020, 10000050]
    
    for n in test_numbers:
        print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
        alignment_steps = modular_alignment_details(n)
        
        print("Step-by-step Modular Alignment:")
        for i, (anchor, residue) in enumerate(alignment_steps, 1):
            print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
        
        is_prime, reason = check_prime_recursive(n)
        print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
        print(f"Reason: {reason}")

def main():
    try:
        user_input = input("Enter a number to check if it's prime (or type 'test' for large primes): ")
        if user_input.lower() == 'test':
            test_large_primes()
        else:
            n = int(user_input)
            
            print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
            alignment_steps = modular_alignment_details(n)
            
            print("Step-by-step Modular Alignment:")
            for i, (anchor, residue) in enumerate(alignment_steps, 1):
                print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
            
            is_prime, reason = check_prime_recursive(n)
            print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
            print(f"Reason: {reason}")
        
    except ValueError:
        print("Invalid input! Please enter a valid integer.")

if __name__ == '__main__':
    main()

