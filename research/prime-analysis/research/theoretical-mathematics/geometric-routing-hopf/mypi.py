import matplotlib.pyplot as plt
import numpy as np

# First 100 digits of Pi (ignoring the decimal point)
pi_digits = [
    int(d) for d in "1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
]

# Map digits to mod 9
pi_mod9_residues = [digit % 9 for digit in pi_digits]

# Twin prime residues mod 9
twin_mod9_residues = [3, 5, 7, 8, 1, 2, 4]

def visualize_pi_and_twin_positions(mod9_residues, twin_residues):
    """
    Visualize Pi digits mod 9 and twin prime residues on a polar circular map.
    """
    # Convert modular values into angles
    pi_angles = [(2 * np.pi * i) / 9 for i in mod9_residues]
    pi_radii = [1] * len(pi_angles)  # All pi values on radius 1
    
    twin_angles = [(2 * np.pi * i) / 9 for i in twin_residues]
    twin_radii = [1.5] * len(twin_angles)  # Twin primes on radius 1.5

    # Create the plot
    fig, ax = plt.subplots(figsize=(12, 12), subplot_kw={'projection': 'polar'})
    ax.scatter(pi_angles, pi_radii, s=50, color='blue', alpha=0.7, label="π Digits Mod 9")
    ax.scatter(twin_angles, twin_radii, s=100, color='red', alpha=0.9, label="Twin Prime Residues Mod 9")

    # Configure aesthetics
    ax.set_title("Mapping π Digits Mod 9 and Twin Prime Residues")
    ax.set_theta_zero_location('N')
    ax.set_theta_direction(-1)
    ax.legend()
    plt.show()

# Run the visualization
visualize_pi_and_twin_positions(pi_mod9_residues, twin_mod9_residues)
