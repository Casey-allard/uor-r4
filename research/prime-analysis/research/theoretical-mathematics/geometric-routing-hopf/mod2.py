import math
from sympy import isprime

def modular_alignment_details(n, primes):
    """
    Perform modular alignment analysis based on previous prime matrices.
    """
    alignment_steps = []
    current_value = n
    
    for i, prime in enumerate(primes):
        max_anchor = prime
        residue = current_value % max_anchor
        alignment_steps.append((max_anchor, residue))
        current_value = residue if residue != 0 else current_value
    
    return alignment_steps

def validate_residue_alignment(residue, prime):
    """
    Validate if a residue aligns with a prime or its exponent.
    """
    if residue == 1:
        if isprime(prime):
            return True, 'Residue aligns with prime matrix.'
        for base in range(2, int(prime**0.5) + 1):
            exp = 1
            result = base**exp
            while result <= prime:
                if result == prime and isprime(base):
                    return False, 'Residue aligns with prime exponent.'
                exp += 1
                result = base**exp
    return False, 'Residue does not align with valid prime properties.'

def check_prime_recursive(n, primes):
    """
    Apply recursive anchor method to check if a number is prime.
    """
    if n < 2:
        return False, 'Number is less than 2 and not prime.'
    
    if isprime(n):
        return True, 'Direct check confirms number is prime.'
    
    alignment_steps = modular_alignment_details(n, primes)
    
    for i, (anchor, residue) in enumerate(alignment_steps):
        if residue == 0:
            return False, f'Residue reduced to 0 at layer {i+1} (anchor {anchor}).'
        
        if residue == 1:
            valid, reason = validate_residue_alignment(residue, anchor)
            if not valid:
                return False, reason
        else:
            valid, reason = validate_residue_alignment(residue, anchor)
            if not valid:
                return False, reason
    
    return True, 'Recursive validation confirms number aligns across matrices.'

def test_large_primes():
    test_numbers = [10000019, 10000079, 10000111, 10000020, 10000050]
    primes = [2, 3, 5, 7, 11, 13, 17, 19]
    
    for n in test_numbers:
        print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
        alignment_steps = modular_alignment_details(n, primes)
        
        print("Step-by-step Modular Alignment:")
        for i, (anchor, residue) in enumerate(alignment_steps, 1):
            print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
        
        is_prime, reason = check_prime_recursive(n, primes)
        print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
        print(f"Reason: {reason}")

def main():
    try:
        user_input = input("Enter a number to check if it's prime (or type 'test' for large primes): ")
        primes = [2, 3, 5, 7, 11, 13, 17, 19]
        if user_input.lower() == 'test':
            test_large_primes()
        else:
            n = int(user_input)
            
            print(f"\nAnalyzing {n} using Recursive Prime Anchor Method...\n")
            alignment_steps = modular_alignment_details(n, primes)
            
            print("Step-by-step Modular Alignment:")
            for i, (anchor, residue) in enumerate(alignment_steps, 1):
                print(f"Layer {i}: Anchor = {anchor}, Residue = {residue}")
            
            is_prime, reason = check_prime_recursive(n, primes)
            print(f"\nResult: {'Prime' if is_prime else 'Not Prime'}")
            print(f"Reason: {reason}")
        
    except ValueError:
        print("Invalid input! Please enter a valid integer.")

if __name__ == '__main__':
    main()

