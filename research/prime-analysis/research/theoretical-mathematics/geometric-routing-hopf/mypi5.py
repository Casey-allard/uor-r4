import sympy
import matplotlib.pyplot as plt
import numpy as np
from mpmath import mp
from collections import Counter

# Function to compute digit sum mod 9
def digit_sum_mod9(n):
    return sum(map(int, str(n))) % 9

# Function to calculate prime digit sums mod 9
def primes_digit_sums_mod9(prime_limit):
    primes = list(sympy.primerange(2, prime_limit))
    return [digit_sum_mod9(p) for p in primes]

# Function to find reduced positions of a specific digit in π mod 9
def reduced_positions_digit_mod9(pi_digits, target_digit):
    return [sum(map(int, str(i))) % 9 for i, d in enumerate(pi_digits) if d == target_digit]

# Function to map residues to angles on a circle
def residues_to_angles(residues, modulus):
    return [2 * np.pi * r / modulus for r in residues]

# Main program
def main():
    # Set precision and extract π digits
    mp.dps = 10000  # Set precision to 10,000 digits of π
    pi_digits = [int(d) for d in str(mp.pi)[2:]]  # Extract π digits (remove "3.")

    # Parameters
    prime_limit = 100000  # Limit for prime numbers
    target_digit = 3      # Target digit in π to analyze

    # Step 1: Calculate Prime Digit Sums Mod 9
    print("Calculating prime digit sums mod 9...")
    prime_digit_sums = primes_digit_sums_mod9(prime_limit)
    prime_counter = Counter(prime_digit_sums)

    # Step 2: Calculate Reduced Positions of π Digit Mod 9
    print("Calculating reduced positions of target digit in π mod 9...")
    pi_positions_digit3 = reduced_positions_digit_mod9(pi_digits, target_digit)
    pi_counter = Counter(pi_positions_digit3)

    # Step 3: Display Frequencies
    print("Frequency of Prime Residues Mod 9:", dict(prime_counter))
    print("Frequency of π Residues Mod 9:", dict(pi_counter))

    # Step 4: Histogram Comparison
    plt.figure(figsize=(10, 6))
    plt.hist(prime_digit_sums, bins=np.arange(10)-0.5, color='blue', alpha=0.7, label='Prime Residues Mod 9')
    plt.hist(pi_positions_digit3, bins=np.arange(10)-0.5, color='red', alpha=0.5, label='π Residues Mod 9 (3)')
    plt.xticks(range(9), labels=[str(i) for i in range(9)])
    plt.title("Frequency Comparison: Prime Residues vs π Digit Positions Mod 9")
    plt.xlabel("Residue Mod 9")
    plt.ylabel("Frequency")
    plt.legend()
    plt.show()

    # Step 5: Circular Mapping
    prime_angles = residues_to_angles(prime_digit_sums, 9)
    pi_angles = residues_to_angles(pi_positions_digit3, 9)

    fig, ax = plt.subplots(subplot_kw={'projection': 'polar'}, figsize=(8, 8))
    ax.scatter(prime_angles, [1] * len(prime_angles), color='blue', s=5, label='Prime Residues Mod 9')
    ax.scatter(pi_angles, [1.2] * len(pi_angles), color='red', s=5, label=f'π Positions of {target_digit} Mod 9')
    ax.set_title("Circular Mapping: Prime Residues vs π Residues Mod 9")
    ax.legend()
    plt.show()

if __name__ == "__main__":
    main()
