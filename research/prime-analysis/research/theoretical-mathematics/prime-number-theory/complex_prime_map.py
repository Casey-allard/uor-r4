import math
import matplotlib.pyplot as plt
import cmath

# Precomputed Anchors
half_sphere_anchors = [
    12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806,
    32220, 39006, 51756, 58222, 72630, 80252, 98282, 120756,
    135056, 151710, 175980, 193160, 212982, 237656, 259590
]

# Harmonic Constants
harmonic_constants = {'sqrt2/2': math.sqrt(2) / 2, 'pi/4': math.pi / 4, 'phi³': (1 + math.sqrt(5)) ** 3 / 2}

# Function to Map Prime Residues on the Complex Plane
def map_residues_on_complex_plane(n, anchors):
    current_n = n
    residue_path = []
    
    valid_anchors = [a for a in anchors if a <= current_n]
    if not valid_anchors:
        return residue_path
    
    for anchor in reversed(valid_anchors):
        residue = current_n % anchor
        
        # Calculate polar representation
        r = residue
        θ = (residue % anchor) / anchor * 2 * math.pi  # Map to [0, 2π]
        z = cmath.rect(r, θ)  # Convert polar to complex plane
        
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
    z = cmath.rect(final_residue, final_residue / 6 * 2 * math.pi)
    residue_path.append({
        'Residue': final_residue,
        'Anchor': 6,
        'r': final_residue,
        'θ': final_residue / 6 * 2 * math.pi,
        'z': z,
        'Note': 'Final Mod 6'
    })
    
    return residue_path

# Visualization of Residues on the Complex Plane
def plot_complex_residues(n):
    residue_path = map_residues_on_complex_plane(n, half_sphere_anchors)
    
    fig, ax = plt.subplots(figsize=(10, 10))
    ax.axhline(0, color='grey', lw=0.5)
    ax.axvline(0, color='grey', lw=0.5)
    ax.grid(color='grey', linestyle='--', linewidth=0.5)
    
    for step in residue_path:
        z = step['z']
        ax.scatter(z.real, z.imag, label=f"Residue: {step['Residue']}, Anchor: {step['Anchor']}")
        ax.text(z.real, z.imag, f"{step['Residue']}", fontsize=8)
    
    ax.set_title(f"Prime Residue Path Mapping for {n} on the Complex Plane")
    ax.set_xlabel('Real Part')
    ax.set_ylabel('Imaginary Part')
    ax.legend()
    plt.show()

# User Interaction
def main():
    print("🛠️ Prime Residue Complex Mapping 🛠️")
    try:
        n = int(input("🔢 Enter a number to map on the complex plane: "))
        plot_complex_residues(n)
    except ValueError:
        print("❌ Invalid input. Please enter an integer number.")

# Entry Point
if __name__ == "__main__":
    main()
