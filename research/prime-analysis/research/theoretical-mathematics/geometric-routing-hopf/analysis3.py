import numpy as np
import matplotlib.pyplot as plt

# Helper Functions
def prime_digit_sums_mod9(n):
    """Calculate prime digit sums mod 9."""
    primes = []
    num = 2
    while len(primes) < n:
        if all(num % p != 0 for p in range(2, int(np.sqrt(num)) + 1)):
            primes.append(num)
        num += 1
    prime_sums_mod9 = [sum(map(int, str(p))) % 9 for p in primes]
    return primes, prime_sums_mod9

def pi_digit_sums_mod9(pi_digits):
    """Calculate digit sums mod 9 for the Pi digits."""
    cumulative_sum = 0
    residues = []
    for i, d in enumerate(pi_digits):
        cumulative_sum += d
        residues.append(cumulative_sum % 9)
    return residues

def find_positions_of_zero(sequence):
    """Find positions where residues mod 9 are zero."""
    return [i for i, val in enumerate(sequence) if val == 0]

def generate_pi_digits(n):
    """Generate first n digits of Pi."""
    pi_str = "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
    return [int(d) for d in pi_str[:n]]

# Main Script
if __name__ == "__main__":
    num_primes = 200  # Number of primes to analyze
    pi_length = 100   # Number of Pi digits to analyze

    # Step 1: Prime Residues Mod 9
    primes, prime_sums_mod9 = prime_digit_sums_mod9(num_primes)
    prime_positions_mod9 = [i for i, val in enumerate(prime_sums_mod9) if val != 0]

    # Step 2: Pi Digit Sums Mod 9
    pi_digits = generate_pi_digits(pi_length)
    pi_residues = pi_digit_sums_mod9(pi_digits)
    zero_positions = find_positions_of_zero(pi_residues)

    # Step 3: Visualize
    plt.figure(figsize=(12, 6))

    # Subplot 1: Pi Digit Sums and Zero Positions
    plt.subplot(1, 2, 1)
    plt.plot(pi_residues, label="Pi Digit Sums Mod 9", marker='o')
    plt.scatter(zero_positions, [0]*len(zero_positions), color='red', zorder=5, label="Zero Positions")
    plt.title("Pi Digit Sums Mod 9")
    plt.xlabel("Index")
    plt.ylabel("Residue Mod 9")
    plt.legend()

    # Subplot 2: Prime Residues vs Pi Zero Positions
    plt.subplot(1, 2, 2)
    plt.scatter(prime_positions_mod9, [1]*len(prime_positions_mod9), label="Prime Residues Mod 9", color='blue')
    plt.scatter(zero_positions, [0]*len(zero_positions), label="Pi Zero Positions", color='red')
    plt.title("Prime Residues Mod 9 vs Pi Zero Positions")
    plt.xlabel("Index")
    plt.ylabel("Residue / Position")
    plt.legend()

    plt.tight_layout()
    plt.show()

    # Polar Visualization for Alignment
    angles_prime = [2 * np.pi * pos / num_primes for pos in prime_positions_mod9]
    angles_zero = [2 * np.pi * pos / pi_length for pos in zero_positions]

    plt.figure(figsize=(8, 6))
    ax = plt.subplot(111, polar=True)
    ax.scatter(angles_prime, np.ones_like(angles_prime), label="Prime Residues Mod 9", color='blue')
    ax.scatter(angles_zero, np.ones_like(angles_zero) * 1.2, label="Pi Zero Positions", color='red')
    plt.title("Polar Plot: Prime Residues vs Pi Zero Positions Mod 9")
    plt.legend()
    plt.show()
