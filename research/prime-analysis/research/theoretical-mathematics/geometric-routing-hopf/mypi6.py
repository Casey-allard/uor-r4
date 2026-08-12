import matplotlib.pyplot as plt
import numpy as np
from sympy import isprime, primerange
from mpmath import mp

# 1. Set up precision for π and compute first n digits
def get_pi_digits(n):
    mp.dps = n + 2  # Decimal places (extra 2 for rounding issues)
    pi_str = str(mp.pi)[2:]  # Remove "3."
    return [int(digit) for digit in pi_str[:n]]

# 2. Compute prime digit sums mod 9
def prime_digit_sums_mod9(limit):
    primes = list(primerange(2, limit))
    digit_sums_mod9 = [(sum(int(d) for d in str(p)) % 9) for p in primes]
    return primes, digit_sums_mod9

# 3. Find positions of a target digit in π mod 9
def positions_of_digit_mod9(digits, target_digit):
    positions = [i for i, d in enumerate(digits) if d == target_digit]
    reduced_positions = [pos % 9 for pos in positions]
    return positions, reduced_positions

# 4. Calculate gaps between positions of a target digit
def calculate_gaps(positions):
    return np.diff(positions)

# 5. Main Analysis Function
def main():
    # User-defined parameters
    pi_digit_count = 1000  # Number of π digits
    prime_limit = 8000     # Upper bound for prime numbers
    target_digit = 3       # Digit in π to analyze (e.g., '3')

    # Step 1: Get π digits
    print("Extracting π digits...")
    pi_digits = get_pi_digits(pi_digit_count)

    # Step 2: Calculate prime digit sums mod 9
    print("Calculating prime digit sums mod 9...")
    primes, prime_sums_mod9 = prime_digit_sums_mod9(prime_limit)

    # Step 3: Find target digit positions in π mod 9
    print("Finding target digit positions in π...")
    positions, reduced_positions = positions_of_digit_mod9(pi_digits, target_digit)
    gaps = calculate_gaps(positions)

    # Step 4: Frequency Analysis
    prime_mod9_freq = {i: prime_sums_mod9.count(i) for i in range(9)}
    pi_mod9_freq = {i: reduced_positions.count(i) for i in range(9)}

    # Step 5: Visualization
    print("Visualizing results...")
    fig, ax = plt.subplots(1, 2, figsize=(14, 6))

    # Bar chart: Frequency comparison
    ax[0].bar(prime_mod9_freq.keys(), prime_mod9_freq.values(), label="Prime Residues Mod 9", color="blue", alpha=0.7)
    ax[0].bar(pi_mod9_freq.keys(), pi_mod9_freq.values(), label=f"π Positions of {target_digit} Mod 9", color="red", alpha=0.5)
    ax[0].set_title("Frequency Comparison: Prime Residues vs π Digit Positions Mod 9")
    ax[0].set_xlabel("Residue Mod 9")
    ax[0].set_ylabel("Frequency")
    ax[0].legend()

    # Circular mapping: Polar plot
    theta_primes = [2 * np.pi * (i / 9) for i in prime_sums_mod9]
    theta_pi = [2 * np.pi * (i / 9) for i in reduced_positions]
    r_primes = np.ones(len(theta_primes))
    r_pi = np.ones(len(theta_pi)) * 1.2

    ax_polar = plt.subplot(122, polar=True)
    ax_polar.scatter(theta_primes, r_primes, c="blue", s=10, label="Prime Residues Mod 9")
    ax_polar.scatter(theta_pi, r_pi, c="red", s=20, label=f"π Positions of {target_digit} Mod 9")
    ax_polar.set_title("Circular Mapping: Prime Residues vs π Residues Mod 9")
    ax_polar.legend()

    plt.tight_layout()
    plt.show()

    # Step 6: Print results
    print("\nFrequency of Prime Residues Mod 9:", prime_mod9_freq)
    print(f"Frequency of π Residues Mod 9 (digit {target_digit}):", pi_mod9_freq)
    print("Gaps between positions of target digit in π:", gaps)

if __name__ == "__main__":
    main()
