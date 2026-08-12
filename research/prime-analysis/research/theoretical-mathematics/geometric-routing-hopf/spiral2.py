import matplotlib.pyplot as plt
import numpy as np

# Predefined anchors (as derived earlier)
half_sphere_anchors = [
    259590, 237656, 212982, 193160, 175980, 151710, 135056,
    120756, 98282, 80252, 72630, 58222, 51756, 39006,
    32220, 24806, 18906, 11990, 9506, 6320, 3782, 1892,
    870, 182, 12
]

# Map Residues on 3D Polar Spiral
def map_residues_on_3d_polar(n, anchors):
    current_n = n
    residue_path = []

    valid_anchors = [a for a in anchors if a <= current_n]
    if not valid_anchors:
        return residue_path

    for anchor in reversed(valid_anchors):
        residue = current_n % anchor

        # Cylindrical coordinates for polar plotting
        r = residue
        θ = (residue % anchor) / anchor * 2 * np.pi  # Angle in radians
        z = anchor  # Anchor as the vertical layer

        residue_path.append({
            'Residue': residue,
            'Anchor': anchor,
            'r': r,
            'θ': θ,
            'z': z
        })

        if residue == 1 or residue == 0:
            break

        current_n = residue

    # Final Mod 6 Validation
    final_residue = current_n % 6
    θ = final_residue / 6 * 2 * np.pi
    r = final_residue
    z = 6  # Final validation layer

    residue_path.append({
        'Residue': final_residue,
        'Anchor': 6,
        'r': r,
        'θ': θ,
        'z': z,
        'Note': 'Final Mod 6'
    })

    return residue_path

# Plotting the 3D Polar Spiral
def plot_3d_polar_spiral(n):
    residue_path = map_residues_on_3d_polar(n, half_sphere_anchors)
    
    fig = plt.figure(figsize=(12, 10))
    ax = fig.add_subplot(111, projection='3d')

    # Extract coordinates
    xs = [step['r'] * np.cos(step['θ']) for step in residue_path]
    ys = [step['r'] * np.sin(step['θ']) for step in residue_path]
    zs = [step['z'] for step in residue_path]

    # Plot points
    for step, x, y, z in zip(residue_path, xs, ys, zs):
        ax.scatter(x, y, z, label=f"Residue: {step['Residue']}, Anchor: {step['Anchor']}")
    
    # Connect the points
    ax.plot(xs, ys, zs, color='grey', linestyle='--')

    ax.set_title(f"3D Polar Prime Residue Spiral Mapping for {n}")
    ax.set_xlabel('X (Polar X)')
    ax.set_ylabel('Y (Polar Y)')
    ax.set_zlabel('Z (Anchor Layers)')
    ax.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.show()

# Test with a sample number
if __name__ == "__main__":
    sample_prime_polar = int(input("Enter a number to map on the 3D polar spiral: "))
    plot_3d_polar_spiral(sample_prime_polar)
