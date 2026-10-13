import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import sympy

# 🛠️ Predefined Stabilization Anchors (Spherical Layers)
ANCHORS = [
    383180, 359400, 326012, 293122, 259590, 237656, 212982, 193160, 175980,
    151710, 135056, 120756, 98282, 80252, 72630, 58222, 51756, 39006, 32220,
    24806, 18906, 11990, 9506, 6320, 3782, 1892, 870, 182, 12
]

# 🧠 Residue Mapping Across Spherical Layers
def map_residues_on_sphere(n, anchors):
    """
    Map residues on spherical shells based on modular anchors.
    """
    residues = []
    current_n = n
    
    for i, anchor in enumerate(anchors):
        if anchor > current_n:
            continue
        
        residue = current_n % anchor
        theta = (residue / anchor) * 2 * np.pi  # Angular displacement
        phi = (i / len(anchors)) * np.pi  # Vertical layer distribution
        
        # Spherical to Cartesian conversion
        x = anchor * np.sin(phi) * np.cos(theta)
        y = anchor * np.sin(phi) * np.sin(theta)
        z = anchor * np.cos(phi)
        
        residues.append({
            'Residue': residue,
            'Anchor': anchor,
            'x': x,
            'y': y,
            'z': z
        })
        
        current_n = residue
    
    return residues

# 🌀 Plotting Prime Residues on Spherical Shells
def plot_spherical_prime_residues(n):
    """
    Plot prime residues on spherical shells.
    """
    residues = map_residues_on_sphere(n, ANCHORS)
    
    fig = plt.figure(figsize=(12, 10))
    ax = fig.add_subplot(111, projection='3d')
    
    # Plot each residue
    for point in residues:
        ax.scatter(point['x'], point['y'], point['z'], label=f"Residue: {point['Residue']}, Anchor: {point['Anchor']}")
    
    # Connect the residues with a path
    xs = [p['x'] for p in residues]
    ys = [p['y'] for p in residues]
    zs = [p['z'] for p in residues]
    ax.plot(xs, ys, zs, color='gray', linestyle='--', alpha=0.5)
    
    ax.set_title(f"3D Spherical Prime Residue Mapping for {n}")
    ax.set_xlabel("X Axis")
    ax.set_ylabel("Y Axis")
    ax.set_zlabel("Z Axis")
    plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.show()

# 🚀 Main Program
if __name__ == "__main__":
    try:
        number = int(input("Enter a number to visualize on the spherical prime map: "))
        plot_spherical_prime_residues(number)
    except ValueError:
        print("❌ Please enter a valid integer.")
    except KeyboardInterrupt:
        print("\n👋 Program interrupted. Goodbye!")
