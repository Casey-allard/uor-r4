import numpy as np
import matplotlib.pyplot as plt
from sympy import isprime
from scipy.fft import fft

# Helper Functions
def generate_primes(n):
    """Generate first n primes."""
    primes = []
    num = 2
    while len(primes) < n:
        if isprime(num):
            primes.append(num)
        num += 1
    return primes

def prime_digit_sums_mod9(primes):
    """Calculate prime digit sums mod 9."""
    return [sum(map(int, str(p))) % 9 for p in primes]

def pi_digit_sums_mod9(pi_digits):
    """Calculate cumulative digit sums mod 9 for Pi digits."""
    cumulative_sum = 0
    residues = []
    for d in pi_digits:
        cumulative_sum += d
        residues.append(cumulative_sum % 9)
    return residues

def find_positions_of_zero(sequence):
    """Find positions where residues mod 9 are zero."""
    return [i for i, val in enumerate(sequence) if val == 0]

def fourier_transform(sequence):
    """Perform Fourier Transform and return amplitudes."""
    return np.abs(fft(sequence))

# Main Script
if __name__ == "__main__":
    num_primes = 500  # Number of primes to analyze
    pi_length = 1000   # Number of Pi digits to analyze

    # Generate Prime Data
    primes = generate_primes(num_primes)
    prime_sums_mod9 = prime_digit_sums_mod9(primes)

    # Pi Digit Sums Mod 9
    pi_digits = [int(d) for d in "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"[:pi_length]]
    pi_residues = pi_digit_sums_mod9(pi_digits)
    zero_positions = find_positions_of_zero(pi_residues)

    # Frequency of Prime Residues
    prime_residue_freq = {i: prime_sums_mod9.count(i) for i in range(9)}
    pi_zero_freq = {i: pi_residues.count(i) for i in range(9)}

    # Visualizations
    plt.figure(figsize=(14, 7))

    # Subplot 1: Histogram Comparison
    plt.subplot(1, 2, 1)
    plt.bar(prime_residue_freq.keys(), prime_residue_freq.values(), label="Prime Residues Mod 9", alpha=0.7)
    plt.bar(pi_zero_freq.keys(), pi_zero_freq.values(), label="Pi Residues Mod 9", alpha=0.7)
    plt.title("Histogram: Prime Residues vs Pi Residues Mod 9")
    plt.xlabel("Residue Mod 9")
    plt.ylabel("Frequency")
    plt.legend()

    # Subplot 2: Polar Plot Alignment
    angles_prime = [2 * np.pi * pos / num_primes for pos, val in enumerate(prime_sums_mod9) if val != 0]
    angles_pi = [2 * np.pi * pos / pi_length for pos in zero_positions]

    ax = plt.subplot(1, 2, 2, polar=True)
    ax.scatter(angles_prime, np.ones_like(angles_prime), label="Prime Residues", color='blue', s=10)
    ax.scatter(angles_pi, np.ones_like(angles_pi) * 1.1, label="Pi Zero Positions", color='red', s=15)
    plt.title("Polar Plot: Prime Residues vs Pi Zero Positions")
    plt.legend()

    plt.tight_layout()
    plt.show()

    # Fourier Analysis
    prime_fft = fourier_transform(prime_sums_mod9)
    pi_fft = fourier_transform(zero_positions)

    plt.figure(figsize=(10, 5))
    plt.plot(prime_fft[:50], label="Prime Residues FFT")
    plt.plot(pi_fft[:50], label="Pi Zero Positions FFT", alpha=0.7)
    plt.title("Fourier Analysis: Prime Residues vs Pi Zero Positions")
    plt.xlabel("Frequency")
    plt.ylabel("Amplitude")
    plt.legend()
    plt.show()

    # Print Frequency Counts
    print("Prime Residues Frequency Mod 9:", prime_residue_freq)
    print("Pi Zero Positions Frequency Mod 9:", pi_zero_freq)
