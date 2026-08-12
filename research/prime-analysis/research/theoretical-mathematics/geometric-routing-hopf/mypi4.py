import sympy
import matplotlib.pyplot as plt
from mpmath import mp
from collections import Counter

# Function to compute digit sum mod 9
def digit_sum_mod9(n):
    return sum(map(int, str(n))) % 9

# Function to calculate digit sums of primes mod 9
def primes_digit_sums_mod9(prime_limit):
    primes = list(sympy.primerange(2, prime_limit))
    return [digit_sum_mod9(p) for p in primes]

# Function to find positions of 3 in π mod 9
def distribution_of_3s_in_pi_mod9(pi_digits):
    mod9_positions = [d % 9 for d in pi_digits]
    return [i for i, residue in enumerate(mod9_positions) if residue == 3]

# Function to reduce positions to digit sums mod 9
def reduce_positions_digit_sum_mod9(positions):
    return [digit_sum_mod9(pos) for pos in positions]

# Set precision and extract π digits
mp.dps = 1000  # Number of digits of π to use
pi_digits = [int(d) for d in str(mp.pi)[2:]]  # Remove "3." at the start

# Analysis
prime_limit = 10000  # Analyze primes up to 10,000
prime_digit_sums = primes_digit_sums_mod9(prime_limit)  # Prime digit sums mod 9
positions_of_3 = distribution_of_3s_in_pi_mod9(pi_digits)  # Positions of 3 in π mod 9
reduced_positions_of_3 = reduce_positions_digit_sum_mod9(positions_of_3)  # Reduced positions

# Count frequencies of residues
prime_counter = Counter(prime_digit_sums)
pi_counter = Counter(reduced_positions_of_3)

# Print frequency analysis
print("Frequency of Prime Residues Mod 9:", dict(prime_counter))
print("Frequency of π Residues Mod 9:", dict(pi_counter))

# Visualize alignment
plt.figure(figsize=(10, 6))

# Plot prime residues mod 9
plt.scatter(range(len(prime_digit_sums)), prime_digit_sums, color='blue', s=5, label='Prime Digit Sums Mod 9')

# Plot reduced positions of 3 in π mod 9
plt.scatter(positions_of_3, reduced_positions_of_3, color='red', s=5, label='Reduced Positions of 3 in Pi Mod 9')

# Configure plot
plt.title('Prime Digit Sums vs Reduced Positions of 3 in Pi Mod 9')
plt.xlabel('Index')
plt.ylabel('Residue Mod 9')
plt.legend()
plt.grid()
plt.show()

# Circular Mapping of Residues
fig, ax = plt.subplots(subplot_kw={'projection': 'polar'}, figsize=(8, 8))

# Map prime residues
prime_angles = [2 * 3.14159 * r / 9 for r in prime_digit_sums]
ax.scatter(prime_angles, [1] * len(prime_digit_sums), color='blue', label='Prime Residues Mod 9')

# Map π residues
pi_angles = [2 * 3.14159 * r / 9 for r in reduced_positions_of_3]
ax.scatter(pi_angles, [1.2] * len(reduced_positions_of_3), color='red', label='π Residues Mod 9')

# Configure polar plot
ax.set_title("Circular Mapping: Prime Residues vs π Residues Mod 9")
ax.legend()
plt.show()
