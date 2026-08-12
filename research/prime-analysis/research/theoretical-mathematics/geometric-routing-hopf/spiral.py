import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np

# Predefined anchors (as derived in our earlier analysis)
half_sphere_anchors = [
    259590, 237656, 212982, 193160, 175980, 151710, 135056,
    120756, 98282, 80252, 72630, 58222, 51756, 39006,
    32220, 24806, 18906, 11990, 9506, 6320, 3782, 1892,
    870, 182, 12
]

# Define 3D Spiral Prime Residue Mapping
def map_residues_on_3d_spiral(n, anchors):
    current_n = n
    residue_path = []

    valid_anchors = [a for a in anchors if a <= current_n]
    if not valid_anchors:
        return residue_path

    for anchor in reversed(valid_anchors):
        residue = current_n % anchor

        # Calculate polar and vertical positioning for 3D
        r = residue
        θ = (residue % anchor) / anchor * 2 * np.pi  # Map to [0, 2π]
        z = anchor  # Use the anchor as the z-axis coordinate

        # Convert polar to Cartesian coordinates
        x = r * np.cos(θ)
        y = r * np.sin(θ)
        
        residue_path.append({
            'Residue': residue,
            'Anchor': anchor,
            'r': r,
            'θ': θ,
            'z': z,
            'x': x,
            'y': y
        })
        
        if residue == 1 or residue == 0:
            break

        current_n = residue

    # Final Mod 6 Validation
    final_residue = current_n % 6
    θ = final_residue / 6 * 2 * np.pi
    x = final_residue * np.cos(θ)
    y = final_residue * np.sin(θ)
    z = 6  # Final validation layer
    
    residue_path.append({
        'Residue': final_residue,
        'Anchor': 6,
        'r': final_residue,
        'θ': θ,
        'z': z,
        'x': x,
        'y': y,
        'Note': 'Final Mod 6'
    })

    return residue_path

# Plotting the 3D Spiral
def plot_3d_spiral(n):
    residue_path = map_residues_on_3d_spiral(n, half_sphere_anchors)
    
    fig = plt.figure(figsize=(12, 10))
    ax = fig.add_subplot(111, projection='3d')
    
    # Plot each residue point
    xs, ys, zs = [], [], []
    for step in residue_path:
        xs.append(step['x'])
        ys.append(step['y'])
        zs.append(step['z'])
        ax.scatter(step['x'], step['y'], step['z'], label=f"Residue: {step['Residue']}, Anchor: {step['Anchor']}")

    # Connect points with a line
    ax.plot(xs, ys, zs, color='grey', linestyle='--')

    ax.set_title(f"3D Prime Residue Spiral Mapping for {n}")
    ax.set_xlabel('X (Real Part)')
    ax.set_ylabel('Y (Imaginary Part)')
    ax.set_zlabel('Z (Anchor Layers)')
    ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.show()

# Test with a sample prime number
if __name__ == "__main__":
    sample_prime_3d = int(input("Enter a number to map on the 3D spiral: "))
    plot_3d_spiral(sample_prime_3d)
