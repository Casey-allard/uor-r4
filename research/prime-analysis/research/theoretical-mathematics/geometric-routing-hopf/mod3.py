import math
from sympy import isprime

def modular_alignment_details(n, matrix_sizes):
    """
    Perform modular alignment analysis based on previous prime matrices.
    """
    alignment_steps = []
    current_value = n
    
    for i, matrix_size in enumerate(matrix_sizes):
        max_anchor = matrix_size * (matrix_size + 1)  # Calculate anchor for each prime
        residue = current_value % max_anchor
        alignment_steps.append((max_anchor, residue))
        current_value = residue if residue != 0 else current_value
    
    return alignment_steps

def validate_residue_alignment(residue, matrix_size):
    """
    Validate if a residue aligns with a prime, reset node, or exponent.
    """
    reset_nodes = [0, 90, 180, 270]
    final_positions = [1, 5]
    
    if residue in final_positions:
        return True, 'Residue aligns with final valid position (1,5).'
    
    if residue == 1:
        if isprime(matrix_size):
            return True, 'Residue aligns with prime matrix size.'
        for base in range(2, int(matrix_size**0.5) + 1):
            exp = 1
            result = base**exp
            while result <= matrix_size:
                if result == matrix_size and isprime(base):
                    return False, 'Residue aligns with a prime exponent.'
                exp += 1
                result = base**exp
    
    if residue in reset_nodes:
        return True, 'Residue aligns with a reset node.'
    
    return False, 'Residue does not align with valid properties.'

def check_prime_recursive(n, matrix_sizes):
    """
    Apply recursive anchor method to check if a number is prime.
    """
    if n < 2:
        return False, 'Number is less than 2 and not prime.'
    
    alignment_steps = modular_alignment_details(n, matrix_sizes)
    prime_anchor_detected = False
    
    for i, (anchor, residue) in enumerate(alignment_steps):
        if residue == 0:
            return False, f'Residue reduced to 0 at layer {i+1} (anchor {anchor}).'
        
        if residue in [1, 5]:
            return True, f'Residue aligns with final valid position (1,5) at layer {i+1}.'
        
        if residue == 1:
            if not prime_anchor_detected:
                prime_anchor_detected = True
                continue
            valid, reason = validate_residue_alignment(residue, anchor)
            if not valid:
                return False, reason
        elif residue in [0, 90, 180, 270]:
            return True, f'Residue aligns with reset node at layer {i+1} (anchor {anchor}).'
        else:
            valid, reason = validate_residue_alignment(residue, anchor)
            if not valid:
                return False, reason
            # Explicit failure checkpoint
            if residue not in reset_nodes and not isprime(residue):
                return False, f'Residue {residue} failed alignment at layer {i+1}.'
    
    return True, 'Recursive validation confirms number aligns across matrices.'

def test_large_primes():
    test_numbers = [10000019, 10000079, 10000111, 10000020, 10000050]
    matrix_sizes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    
    for n in test_numbers:
        print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
        alignment_steps = modular_alignment_details(n, matrix_sizes)
        
        print("Step-by-step Modular Alignment:")
        for i, (anchor, residue) in enumerate(alignment_steps, 1):
            print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
        
        is_prime, reason = check_prime_recursive(n, matrix_sizes)
        print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
        print(f"Reason: {reason}")

def main():
    try:
        user_input = input("Enter a number to check if it's prime (or type 'test' for large primes): ")
        matrix_sizes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
        if user_input.lower() == 'test':
            test_large_primes()
        else:
            n = int(user_input)
            
            print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
            alignment_steps = modular_alignment_details(n, matrix_sizes)
            
            print("Step-by-step Modular Alignment:")
            for i, (anchor, residue) in enumerate(alignment_steps, 1):
                print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
            
            is_prime, reason = check_prime_recursive(n, matrix_sizes)
            print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
            print(f"Reason: {reason}")
        
    except ValueError:
        print("Invalid input! Please enter a valid integer.")

if __name__ == '__main__':
    main()

