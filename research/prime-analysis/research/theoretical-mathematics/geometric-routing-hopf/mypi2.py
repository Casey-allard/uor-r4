import sympy

def digit_sum_mod9(n):
    return sum(map(int, str(n))) % 9

def primes_digit_sums_mod9(prime_limit):
    primes = list(sympy.primerange(2, prime_limit))
    return [digit_sum_mod9(p) for p in primes]

def distribution_of_3s_in_pi_mod9(pi_digits):
    mod9_positions = [d % 9 for d in pi_digits]
    return [i for i, residue in enumerate(mod9_positions) if residue == 3]

# First 100 digits of Pi
pi_digits = [int(d) for d in "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"]

# Analyze primes' digit sums mod 9
prime_digit_sums_mod9 = primes_digit_sums_mod9(1000)

# Analyze positions of '3' in Pi mod 9
positions_of_3 = distribution_of_3s_in_pi_mod9(pi_digits)

# Results
print("Prime Digit Sums Mod 9:", prime_digit_sums_mod9[:20])  # First 20 results
print("Positions of 3 in Pi Mod 9:", positions_of_3[:20])     # First 20 positions
