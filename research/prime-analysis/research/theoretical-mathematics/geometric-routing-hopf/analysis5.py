import sympy
import numpy as np
import matplotlib.pyplot as plt
from collections import Counter

# Function to calculate digit sum mod 9
def digit_sum_mod_9(n):
    return sum(map(int, str(n))) % 9

# Extract digits of π and reduce mod 9
def pi_digits_mod_9(target_digit, num_digits=1000):
    from mpmath import mp
    mp.dps = num_digits  # Set precision
    pi_str = str(mp.pi)[2:num_digits+2]  # Get π digits after the decimal point

    pi_digit_sums = [sum(map(int, str(int(d)))) % 9 for d in pi_str]
    zero_positions = [i for i, d in enumerate(pi_digit_sums) if d == target_digit]

    return pi_digit_sums, zero_positions

# Step 1: Calculate Prime Digit Sums Mod 9
def calculate_prime_digit_sums_mod_9(limit=1000):
    primes = list(sympy.primerange(2, limit))
    prime_digit_sums = [digit_sum_mod_9(p) for p in primes]
    return primes, prime_digit_sums

# Step 2: Analyze Prime Frequencies and Zero Positions
def analyze_frequencies(prime_digit_sums, zero_positions):
    prime_freq = Counter(prime_digit_sums)
    zero_freq = Counter([pos % 9 for pos in zero_positions])
    return prime_freq, zero_freq

# Step 3: Visualize Results
def plot_results(prime_freq, zero_freq, prime_digit_sums, zero_positions):
    fig, axes = plt.subplots(1, 2, figsize=(14, 6))

    # Histogram: Prime Residues vs π Zero Positions Mod 9
    labels = list(range(9))
    prime_counts = [prime_freq.get(i, 0) for i in labels]
    zero_counts = [zero_freq.get(i, 0) for i in labels]

    axes[0].bar(labels, prime_counts, color='tab:blue', label='Prime Residues Mod 9')
    axes[0].bar(labels, zero_counts, color='tab:orange', label='Pi Zero Positions Mod 9')
    axes[0].set_title("Histogram: Prime Residues vs Pi Zero Positions Mod 9")
    axes[0].set_xlabel("Residue Mod 9")
    axes[0].set_ylabel("Frequency")
    axes[0].legend()

    # Polar Plot: Prime Digit Sums vs Pi Zero Positions
    polar_labels = np.array(prime_digit_sums) / max(prime_digit_sums)
    polar_positions = np.array(zero_positions) / max(zero_positions)

    ax = plt.subplot(1, 2, 2, projection='polar')
    ax.scatter(polar_labels * 2 * np.pi, np.ones_like(polar_labels), color='blue', label='Prime Residues')
    ax.scatter(polar_positions * 2 * np.pi, np.ones_like(polar_positions), color='red', label='Pi Zero Positions')
    ax.set_title("Polar Plot: Prime Residues vs Pi Zero Positions Mod 9")
    ax.legend()

    plt.tight_layout()
    plt.show()

    # Fourier Analysis
    plt.figure(figsize=(10, 5))
    fft_primes = np.fft.fft(prime_digit_sums)
    fft_zeros = np.fft.fft(zero_positions)

    plt.plot(np.abs(fft_primes[:50]), label="Prime Residues FFT")
    plt.plot(np.abs(fft_zeros[:50]), label="Pi Zero Positions FFT")
    plt.title("Fourier Analysis: Prime Residues vs Pi Zero Positions")
    plt.xlabel("Frequency")
    plt.ylabel("Amplitude")
    plt.legend()
    plt.show()

# Main Execution
if __name__ == "__main__":
    # Parameters
    PRIME_LIMIT = 1000   # Upper limit for prime calculation
    PI_DIGITS = 1000     # Number of π digits to analyze
    TARGET_DIGIT = 0     # Digit to find zero positions in π (default 0 mod 9)

    print("Calculating prime digit sums mod 9...")
    primes, prime_digit_sums = calculate_prime_digit_sums_mod_9(PRIME_LIMIT)

    print("Calculating π digit sums mod 9 and zero positions...")
    pi_digit_sums, zero_positions = pi_digits_mod_9(TARGET_DIGIT, PI_DIGITS)

    print("Analyzing frequencies...")
    prime_freq, zero_freq = analyze_frequencies(prime_digit_sums, zero_positions)

    print("Prime Residues Frequency Mod 9:", prime_freq)
    print("Pi Zero Positions Frequency Mod 9:", zero_freq)

    print("Visualizing results...")
    plot_results(prime_freq, zero_freq, prime_digit_sums, zero_positions)
