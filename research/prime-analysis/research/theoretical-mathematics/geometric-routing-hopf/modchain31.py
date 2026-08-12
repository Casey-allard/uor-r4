import sympy
import math

# Define Constants
phi = (1 + math.sqrt(5)) / 2  # Golden ratio
harmonic_constants = {'sqrt2/2': math.sqrt(2) / 2, 'pi/4': math.pi / 4, 'phi³': (1 + math.sqrt(5)) ** 3 / 2}

# Precomputed Half-Sphere Anchors (from our discovery)
half_sphere_anchors = [
    12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806,
    32220, 39006, 51756, 58222, 72630, 80252, 98282, 120756,
    135056, 151710, 175980, 193160, 212982, 237656, 259590
]

# Prime Validation Function Using Modular Anchor Reduction
def validate_prime(n):
    current_n = n
    residue_path = [{'Layer': 'Initial', 'Residue': current_n, 'Anchor': None}]
    
    for anchor in reversed(half_sphere_anchors):
        if anchor > current_n:
            continue  # Skip anchors larger than the number
        
        residue = current_n % anchor
        
        if residue == current_n:
            residue_path.append({'Layer': f'Anchor {anchor}', 'Residue': residue, 'Anchor': anchor, 'Note': 'Skipped (Trivial Anchor)'})
            continue
        
        residue_path.append({'Layer': f'Anchor {anchor}', 'Residue': residue, 'Anchor': anchor, 'Note': 'Valid Reduction'})
        
        if residue == 1:
            residue_path.append({'Layer': f'Anchor {anchor}', 'Residue': residue, 'Anchor': anchor, 'Note': 'Prime Stop (Residue 1)'})
            return residue_path, "Prime"
        
        if residue == 0:
            residue_path.append({'Layer': f'Anchor {anchor}', 'Residue': residue, 'Anchor': anchor, 'Note': 'Composite (Residue 0)'})
            return residue_path, "Composite"
        
        current_n = residue
    
    # Final Mod 12 Validation
    final_residue = current_n % 12
    if final_residue == 1 and current_n > 12:
        residue_path.append({'Layer': 'Final Mod 12', 'Residue': final_residue, 'Anchor': 12, 'Note': 'Prime (Residue 1 from >12)'})
        return residue_path, "Prime"
    elif final_residue == 0:
        residue_path.append({'Layer': 'Final Mod 12', 'Residue': final_residue, 'Anchor': 12, 'Note': 'Composite (Residue 0)'})
        return residue_path, "Composite"
    else:
        residue_path.append({'Layer': 'Final Mod 12', 'Residue': final_residue, 'Anchor': 12, 'Note': 'Mapped to Mod 12 Terminal'})
        return residue_path, "Indeterminate"

# User Interaction
def main():
    print("🛠️ Modular Anchor Chain Reduction Program 🛠️")
    try:
        n = int(input("🔢 Enter a number to check for primality: "))
        residue_path, result = validate_prime(n)
        
        print("\n🔗 Residue Path:")
        for step in residue_path:
            print(f"Layer: {step['Layer']}, Residue: {step['Residue']}, Anchor: {step['Anchor']}, Note: {step['Note']}")
        
        print(f"\n✅ Final Validation Result: {result}")
    except ValueError:
        print("❌ Invalid input. Please enter an integer number.")

# Entry Point
if __name__ == "__main__":
    main()
