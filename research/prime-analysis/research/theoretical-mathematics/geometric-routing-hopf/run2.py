import sympy

# Updated Anchor List
ANCHORS = [
    12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806, 32220, 
    39006, 51756, 58222, 72630, 80252, 98282, 120756, 135056, 151710, 
    175980, 193160, 212982, 237656, 259590, 293122, 326012, 359400, 383180
]

# Validate Prime Using Anchors
def validate_prime(n, anchors):
    current_n = n
    residue_path = [{"Layer": "Initial", "Residue": n, "Anchor": None}]
    
    for layer, anchor in enumerate(anchors, start=1):
        if anchor > current_n:
            continue
        
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
    
    # Final Mod 6 Validation
    final_residue = current_n % 6
    if final_residue == 1 and current_n > 6:
        residue_path.append({
            "Layer": "Final Mod 6",
            "Residue": final_residue,
            "Anchor": 6,
            "Note": "Prime (Residue 1 from >6)"
        })
        return residue_path, "Prime"
    elif final_residue == 0:
        residue_path.append({
            "Layer": "Final Mod 6",
            "Residue": final_residue,
            "Anchor": 6,
            "Note": "Composite (Residue 0)"
        })
        return residue_path, "Composite"
    else:
        residue_path.append({
            "Layer": "Final Mod 6",
            "Residue": final_residue,
            "Anchor": 6,
            "Note": "Mapped to Mod 6 Terminal"
        })
        return residue_path, "Indeterminate"

# Main Program
def main():
    n = int(input("Enter a number to test for primality: "))
    
    residue_path, result = validate_prime(n, ANCHORS)
    
    print("\n🔗 Residue Path:")
    for step in residue_path:
        print(step)
    
    print(f"\n🏁 Final Validation Result: {result}")

if __name__ == "__main__":
    main()
