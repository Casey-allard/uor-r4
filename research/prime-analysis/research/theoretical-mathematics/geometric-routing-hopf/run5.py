import math

# 🛠️ Precomputed Anchors (Stabilization Nodes)
ANCHORS = [
    383180, 359400, 326012, 293122, 259590, 237656, 212982, 193160, 175980,
    151710, 135056, 120756, 98282, 80252, 72630, 58222, 51756, 39006, 32220,
    24806, 18906, 11990, 9506, 6320, 3782, 1892, 870, 182, 12
]

# 🧠 Small Prime Safeguard for Final Validation
SMALL_PRIMES = [2, 3, 5, 7, 11, 13, 17, 19]


# 🌀 Residue Propagation
def propagate_residue(n, anchors):
    """
    Propagate residue reduction through the anchor chain.
    """
    residue_path = [{"Layer": "Initial", "Residue": n, "Anchor": None, "Note": "Start"}]
    current_n = n

    for layer, anchor in enumerate(anchors, start=1):
        if anchor > current_n:
            continue  # Skip anchors larger than the current number

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

        # Continue reduction with the residue
        current_n = residue

    # Final Mod 6 Validation
    final_residue = current_n % 6
    residue_path.append({
        "Layer": "Final Mod 6",
        "Residue": final_residue,
        "Anchor": 6,
        "Note": "Final Validation"
    })

    validation_result, validation_note = final_validation(final_residue, n)
    residue_path.append({
        "Layer": "Final Mod 6 Check",
        "Residue": final_residue,
        "Anchor": 6,
        "Note": validation_note
    })

    return residue_path, validation_result


# ✅ Final Validation Logic
def final_validation(residue, original_n):
    """
    Perform final validation with Mod 6 and small prime divisibility check.
    """
    if residue == 1:
        # Check divisibility by small primes
        for prime in SMALL_PRIMES:
            if original_n % prime == 0 and original_n != prime:
                return "Composite", f"Divisible by small prime {prime}"
        return "Prime", "Residue 1 and passed small prime check"
    elif residue == 0:
        return "Composite", "Final residue 0"
    else:
        return "Indeterminate", "Final residue does not align with prime pattern"


# 🧪 Prime Validation Function
def validate_prime(n):
    """
    Validate the primality of a number using modular anchor reduction.
    """
    print(f"🔍 Evaluating Number: {n}")
    residue_path, result = propagate_residue(n, ANCHORS)

    print("\n🔗 Residue Path:")
    for step in residue_path:
        print(step)

    print(f"\n🏁 Final Validation Result: {result}")
    return result


# 🚀 Main Program Loop
def main():
    """
    Main interactive loop for primality testing.
    """
    print("🛠️ Modular Anchor Chain Residue Evaluation Program 🛠️")
    try:
        while True:
            number = int(input("\n🔢 Enter a number to test (or type 0 to exit): "))
            if number == 0:
                print("👋 Exiting program. Goodbye!")
                break

            validate_prime(number)
    
    except ValueError:
        print("❌ Invalid input. Please enter integers only.")
    except KeyboardInterrupt:
        print("\n👋 Program interrupted. Goodbye!")


# Run the program
if __name__ == "__main__":
    main()
