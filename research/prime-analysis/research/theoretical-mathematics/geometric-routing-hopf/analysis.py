import numpy as np
import matplotlib.pyplot as plt
from scipy.signal import correlate
from scipy.fft import fft, fftfreq

# Helper Functions
def prime_digit_sums_mod9(n):
    """Calculate prime numbers' digit sums mod 9."""
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
    """Generate first n digits of Pi (excluding the decimal)."""
    pi_str = "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
    pi_digits = [int(d) for d in pi_str[:n]]
    return pi_digits

# Analysis Functions
def cross_correlation(seq1, seq2):
    """Perform cross-correlation between two sequences."""
    return correlate(seq1, seq2, mode='full')

def fourier_analysis(gaps, title):
    """Perform Fourier Analysis on gap data and plot."""
    N = len(gaps)
    yf = fft(gaps)
    xf = fftfreq(N, 1)[:N // 2]

    plt.figure(figsize=(10, 5))
    plt.plot(xf, 2.0 / N * np.abs(yf[:N // 2]))
    plt.title(f"Fourier Analysis of {title}")
    plt.xlabel("Frequency")
    plt.ylabel("Amplitude")
    plt.grid()
    plt.show()

# Main Script
if __name__ == "__main__":
    num_primes = 500  # Number of primes to analyze
    pi_length = 100   # Number of Pi digits to analyze
    target_digit = 3  # Target digit in Pi for analysis

    # Step 1: Prime Analysis
    primes, prime_sums_mod9 = prime_digit_sums_mod9(num_primes)
    prime_gaps = gap_analysis([i for i, _ in enumerate(prime_sums_mod9)])

    # Step 2: Pi Analysis
    pi_digits = generate_pi_digits(pi_length)
    pi_positions, pi_positions_mod9 = pi_digit_positions_mod9(target_digit, pi_digits)
    pi_gaps = gap_analysis(pi_positions)

    # Step 3: Cross-Correlation
    correlation = cross_correlation(prime_sums_mod9, pi_positions_mod9)

    # Step 4: Visualizations
    plt.figure(figsize=(15, 5))
    plt.subplot(1, 2, 1)
    plt.hist(prime_sums_mod9, bins=range(10), color='blue', alpha=0.7, label="Prime Residues Mod 9")
    plt.hist(pi_positions_mod9, bins=range(10), color='red', alpha=0.5, label="Pi Positions Mod 9")
    plt.title("Histogram: Prime Residues vs Pi Digit Positions Mod 9")
    plt.xlabel("Residue Mod 9")
    plt.ylabel("Frequency")
    plt.legend()

    plt.subplot(1, 2, 2)
    plt.plot(correlation, color='purple')
    plt.title("Cross-Correlation: Prime Residues vs Pi Positions")
    plt.xlabel("Lag")
    plt.ylabel("Correlation")

    plt.tight_layout()
    plt.show()

    # Step 5: Fourier Analysis
    fourier_analysis(prime_gaps, "Prime Gaps Mod 9")
    fourier_analysis(pi_gaps, "Gaps Between Positions of Digit 3 in Pi Mod 9")

    # Step 6: Heatmap Visualization
    heatmap_data = np.zeros((9, 9))
    for i in prime_sums_mod9:
        for j in pi_positions_mod9:
            heatmap_data[i][j] += 1

    plt.figure(figsize=(8, 6))
    plt.imshow(heatmap_data, cmap='hot', interpolation='nearest')
    plt.title("Heatmap: Prime Residues vs Pi Positions Mod 9")
    plt.xlabel("Pi Digit Positions Mod 9")
    plt.ylabel("Prime Residues Mod 9")
    plt.colorbar(label="Frequency")
    plt.show()
