import sympy
import math

# Precomputed Anchors
half_sphere_anchors = [
    12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806,
    32220, 39006, 51756, 58222, 72630, 80252, 98282, 120756,
    135056, 151710, 175980, 193160, 212982, 237656, 259590
]

# Harmonic Constants
harmonic_constants = {'sqrt2/2': math.sqrt(2) / 2, 'pi/4': math.pi / 4, 'phi³': (1 + math.sqrt(5)) ** 3 / 2}

# Recursive Mod 6 Validation
def recursive_mod6_validation(residue):
    final_residue = residue % 6
    if final_residue == 1:
        return "Prime (Mod 6 Validation)"
    elif final_residue == 0:
        return "Composite (Mod 6 Validation)"
    else:
        return f"Indeterminate (Residue {final_residue} in Mod 6)"

# Validate Number with Anchor Reduction
def validate_prime(n, anchors):
    current_n = n
    residue_path = [{"Layer": "Initial", "Residue": n, "Anchor": None}]
    
    valid_anchors = [a for a in anchors if a <= current_n]
    if not valid_anchors:
        return residue_path, "Indeterminate"
    
    for layer, anchor in enumerate(reversed(valid_anchors), start=1):
        residue = current_n % anchor
        
        residue_path.append({
            "Layer": f"Layer {layer}",
            "Residue": residue,
            "Anchor": anchor,
            "Note": "Valid Reduction"
        })
        
        if residue == 1:
            residue_path.append({
                "Layer": f"Layer {layer}",
                "Residue": residue,
                "Anchor": anchor,
                "Note": "Prime Stop (Residue 1)"
            })
            return residue_path, "Prime"
        
        if residue == 0:
            residue_path.append({
                "Layer": f"Layer {layer}",
                "Residue": residue,
                "Anchor": anchor,
                "Note": "Composite (Residue 0)"
            })
            return residue_path, "Composite"
        
        current_n = residue
    
    # Final Mod 12 Validation
    final_residue = current_n % 12
    if final_residue == 1 and current_n > 12:
        residue_path.append({
            "Layer": "Final Mod 12",
            "Residue": final_residue,
            "Anchor": 12,
            "Note": "Prime (Residue 1 from >12)"
        })
        return residue_path, "Prime"
    elif final_residue == 0:
        residue_path.append({
            "Layer": "Final Mod 12",
            "Residue": final_residue,
            "Anchor": 12,
            "Note": "Composite (Residue 0)"
        })
        return residue_path, "Composite"
    elif final_residue == 7:
        # Apply Recursive Mod 6 Validation
        mod6_result = recursive_mod6_validation(current_n)
        residue_path.append({
            "Layer": "Recursive Mod 6 Validation",
            "Residue": final_residue,
            "Anchor": 6,
            "Note": mod6_result
        })
        return residue_path, "Prime" if "Prime" in mod6_result else "Composite"
    else:
        residue_path.append({
            "Layer": "Final Mod 12",
            "Residue": final_residue,
            "Anchor": 12,
            "Note": "Mapped to Mod 12 Terminal"
        })
        return residue_path, "Indeterminate"

# User Interaction
def main():
    print("🛠️ Modular Anchor Chain Reduction Program 🛠️")
    try:
        n = int(input("🔢 Enter a number to check for primality: "))
        residue_path, result = validate_prime(n, half_sphere_anchors)
        
        print("\n🔗 Residue Path:")
        for step in residue_path:
            print(
                f"Layer: {step.get('Layer', 'Unknown')}, "
                f"Residue: {step.get('Residue', 'Unknown')}, "
                f"Anchor: {step.get('Anchor', 'Unknown')}, "
                f"Note: {step.get('Note', 'No Note Provided')}"
            )
        
        print(f"\n🏁 Final Validation Result: {result}")
    except ValueError:
        print("❌ Invalid input. Please enter an integer number.")

# Entry Point
if __name__ == "__main__":
    main()
