import numpy as np
import matplotlib.pyplot as plt
from collections import Counter

def reduce_to_single_digit(n):
    """Reduce a number to a single digit by summing its digits iteratively."""
    while n >= 10:
        n = sum(int(digit) for digit in str(n))
    return n

def calculate_prime_digit_sums(primes):
    """Calculate reduced digit sums mod 9 for a list of primes."""
    return [reduce_to_single_digit(p) % 9 for p in primes]

def calculate_gaps(sequence):
    """Calculate gaps between consecutive elements of a sequence."""
    return np.diff(sequence)

def plot_polar_alignment(prime_residues, pi_positions, title="Polar Plot: Prime Residues vs Pi Zero Positions Mod 9"):
    """Plot polar alignment of prime residues and pi zero positions."""
    angles_prime = [2 * np.pi * r / 9 for r in prime_residues]
    angles_pi = [2 * np.pi * p / 9 for p in pi_positions]

    plt.figure(figsize=(8, 8))
    ax = plt.subplot(111, polar=True)
    ax.scatter(angles_prime, np.ones_like(angles_prime), color='blue', label="Prime Residues Mod 9")
    ax.scatter(angles_pi, np.ones_like(angles_pi) + 0.2, color='red', label="Pi Zero Positions Mod 9")
    plt.title(title)
    plt.legend()
    plt.show()

# Sample Data: Replace these with your actual prime list and pi zero positions
primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71]
pi_zero_positions = [8, 14, 16, 23, 24, 26, 42, 45, 63, 85, 90]  # Example positions of 3 in π mod 9

# Step 1: Calculate prime digit sums and reduce them
prime_residues = calculate_prime_digit_sums(primes)

# Step 2: Polar alignment plot
plot_polar_alignment(prime_residues, pi_zero_positions, title="Polar Alignment: Reduced Prime Residues vs Pi Zero Positions Mod 9")

# Step 3: Gap Analysis
gaps_prime = calculate_gaps(prime_residues)
gaps_pi = calculate_gaps(pi_zero_positions)

# Plot Gap Distribution
plt.figure(figsize=(10, 6))
plt.hist(gaps_prime, bins=range(1, max(gaps_prime) + 2), alpha=0.6, label="Prime Gaps Mod 9", color='blue', edgecolor='black')
plt.hist(gaps_pi, bins=range(1, max(gaps_pi) + 2), alpha=0.6, label="Pi Zero Position Gaps Mod 9", color='orange', edgecolor='black')
plt.xlabel("Gap Size")
plt.ylabel("Frequency")
plt.title("Gap Distribution: Prime Digit Sums vs Pi Zero Positions Mod 9")
plt.legend()
plt.grid(axis="y", linestyle="--", alpha=0.7)
plt.show()
