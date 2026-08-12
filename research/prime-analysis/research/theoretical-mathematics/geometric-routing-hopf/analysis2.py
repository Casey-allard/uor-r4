import numpy as np
import matplotlib.pyplot as plt

# Helper functions
def prime_digit_sums_mod9(n):
    """Calculate prime digit sums mod 9."""
    primes = []
    num = 2
    while len(primes) < n:
        if all(num % p != 0 for p in range(2, int(np.sqrt(num)) + 1)):
            primes.append(num)
        num += 1
    prime_sums = [sum(map(int, str(p))) % 9 for p in primes]
    return primes, prime_sums

def pi_digit_positions_mod9(target_digit, pi_digits):
    """Find positions of a target digit in Pi and reduce mod 9."""
    positions = [i + 1 for i, d in enumerate(pi_digits) if d == target_digit]
    positions_mod9 = [pos % 9 for pos in positions]
    return positions, positions_mod9

def gap_analysis(sequence):
    """Calculate gaps between consecutive positions in a sequence."""
    return np.diff(sequence)

# Generate Pi Digits
def generate_pi_digits(n):
    """Generate first n digits of Pi."""
    pi_str = "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
    pi_digits = [int(d) for d in pi_str[:n]]
    return pi_digits

# Main script
if __name__ == "__main__":
    num_primes = 200  # Number of primes to analyze
    pi_length = 100   # Number of Pi digits to analyze
    target_digit = 3  # Target digit for analysis

    # Step 1: Prime Analysis
    primes, prime_sums_mod9 = prime_digit_sums_mod9(num_primes)
    prime_gaps_mod9 = gap_analysis([i for i, _ in enumerate(prime_sums_mod9)])

    # Step 2: Pi Analysis
    pi_digits = generate_pi_digits(pi_length)
    pi_positions, pi_positions_mod9 = pi_digit_positions_mod9(target_digit, pi_digits)
    pi_gaps = gap_analysis(pi_positions)

    # Step 3: Plot Gaps
    plt.figure(figsize=(12, 6))
    plt.subplot(1, 2, 1)
    plt.plot(prime_gaps_mod9, marker='o', linestyle='-', label="Prime Gaps Mod 9")
    plt.title("Gaps Between Consecutive Prime Residues Mod 9")
    plt.xlabel("Index")
    plt.ylabel("Gap Size")
    plt.legend()

    plt.subplot(1, 2, 2)
    plt.plot(pi_gaps, marker='o', linestyle='-', color='red', label="Gaps Between 3s in π")
    plt.title("Gaps Between Positions of Digit 3 in π")
    plt.xlabel("Index")
    plt.ylabel("Gap Size")
    plt.legend()

    plt.tight_layout()
    plt.show()

    # Step 4: Overlay Histograms
    plt.figure(figsize=(10, 5))
    plt.hist(prime_gaps_mod9, bins=range(0, max(prime_gaps_mod9) + 1), alpha=0.7, label="Prime Gaps Mod 9")
    plt.hist(pi_gaps, bins=range(0, max(pi_gaps) + 1), alpha=0.5, color='red', label="Gaps Between 3s in π")
    plt.title("Overlay of Prime Gaps Mod 9 and Gaps Between Digit 3 in π")
    plt.xlabel("Gap Size")
    plt.ylabel("Frequency")
    plt.legend()
    plt.show()

    # Step 5: Polar Visualization for Alignments
    angles_prime = [2 * np.pi * gap / 9 for gap in prime_gaps_mod9]
    angles_pi = [2 * np.pi * gap / 9 for gap in pi_gaps]

    plt.figure(figsize=(10, 6))
    ax = plt.subplot(111, polar=True)
    ax.scatter(angles_prime, np.ones_like(angles_prime), label="Prime Gaps Mod 9", color='blue')
    ax.scatter(angles_pi, np.ones_like(angles_pi) * 1.2, label="Gaps Between 3s in π", color='red')
    plt.title("Polar Plot of Prime Gaps and Gaps Between Positions of 3 in π")
    plt.legend()
    plt.show()
