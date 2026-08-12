import sympy
import math

# Precompute Anchors
def precompute_anchors(max_prime=1000):
    primes = list(sympy.primerange(2, max_prime))
    anchors = {p: p * (p + 1) for p in primes}
    return sorted(anchors.items(), key=lambda x: x[1], reverse=True)

# Validate Number with Anchor Reduction
def validate_prime(n, anchors):
    current_n = n
    residue_path = [{"Layer": "Initial", "Residue": n, "Anchor": None}]
    
    for layer in range(20):
        valid_anchors = [a for p, a in anchors if a <= current_n]
        if not valid_anchors:
            break
        
        current_anchor = valid_anchors[0]
        residue = current_n % current_anchor
        
        if residue == current_n:
            residue_path.append({"Layer": f"Layer {layer+1}", "Residue": residue, "Anchor": current_anchor, "Note": "Skipped (Trivial Anchor)"})
            continue
        
        residue_path.append({"Layer": f"Layer {layer+1}", "Residue": residue, "Anchor": current_anchor, "Note": "Valid Reduction"})
        
        if residue == 1:
            residue_path.append({"Layer": f"Layer {layer+1}", "Residue": residue, "Anchor": current_anchor, "Note": "Prime Stop (Residue 1)"})
            return residue_path, "Prime"
        
        if residue == 0:
            residue_path.append({"Layer": f"Layer {layer+1}", "Residue": residue, "Anchor": current_anchor, "Note": "Composite (Residue 0)"})
            return residue_path, "Composite"
        
        current_n = residue
    
    # Final Mod 12 Validation
    final_residue = current_n % 12
    if final_residue == 1 and current_n > 12:
        residue_path.append({"Layer": "Final Mod 12", "Residue": final_residue, "Anchor": 12, "Note": "Prime (Residue 1 from >12)"})
        return residue_path, "Prime"
    elif final_residue == 0:
        residue_path.append({"Layer": "Final Mod 12", "Residue": final_residue, "Anchor": 12, "Note": "Composite (Residue 0)"})
        return residue_path, "Composite"
    else:
        residue_path.append({"Layer": "Final Mod 12", "Residue": final_residue, "Anchor": 12, "Note": "Mapped to Mod 12 Terminal"})
        return residue_path, "Indeterminate"

# User Prompt
def main():
    max_prime = 1000
    anchors = precompute_anchors(max_prime)
    n = int(input("Enter a number to test for primality: "))
    
    residue_path, result = validate_prime(n, anchors)
    print("\nResidue Path:")
    for step in residue_path:
        print(step)
    print(f"\nFinal Validation Result: {result}")

if __name__ == "__main__":
    main()
