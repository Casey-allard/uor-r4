# Anchor Chain Residue Evaluation Program
import math

# 🛠️ Hardcoded Stabilization Anchors
ANCHORS = [
    12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806, 32220,
    39006, 51756, 58222, 72630, 80252, 98282, 120756, 135056, 151710,
    175980, 193160, 212982, 237656, 259590, 293122, 326012, 359400, 383180
]

# 🧠 Validate Residue Path
def validate_residues(residues):
    """
    Evaluate residues for primality.
    """
    if 1 in residues and all(r != 0 for r in residues):
        return "Prime"
    elif 0 in residues:
        return "Composite"
    return "Indeterminate"

# 🌀 Residue Propagation
def propagate_residue(n, anchors):
    """
    Propagate the residue through the anchor chain.
    """
    residue_path = []
    current_n = n
    
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
    
    # Final Mod 6 Check
    final_residue = current_n % 6
    residue_path.append({
        "Layer": "Final Mod 6",
        "Residue": final_residue,
        "Anchor": 6,
        "Note": "Final Validation"
    })
    
    if final_residue == 1:
        return residue_path, "Prime"
    else:
        return residue_path, "Composite"

# 🧪 Main Evaluation Function
def anchor_chain_test(number):
    """
    Perform the full anchor chain evaluation.
    """
    print(f"🔍 Evaluating Number: {number}")
    residue_path, result = propagate_residue(number, ANCHORS)
    
    print("\n🔗 Residue Path:")
    for step in residue_path:
        print(step)
    
    print(f"\n🏁 Final Validation Result: {result}")
    return result

# 🚀 Main Program Loop
def main():
    """
    Main interactive loop.
    """
    print("🛠️ Modular Anchor Chain Residue Evaluation Program 🛠️")
    try:
        while True:
            number = int(input("\n🔢 Enter a number to test (or type 0 to exit): "))
            
            if number == 0:
                print("👋 Exiting program. Goodbye!")
                break
            
            anchor_chain_test(number)
    
    except ValueError:
        print("❌ Invalid input. Please enter integers only.")
    except KeyboardInterrupt:
        print("\n👋 Program interrupted. Goodbye!")


# Run the program
if __name__ == "__main__":
    main()
