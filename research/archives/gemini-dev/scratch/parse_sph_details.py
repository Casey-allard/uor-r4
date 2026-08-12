import re

sph_path = "/Users/adminamn/gemini-dev/extra_reading/sph_formal_specification.txt"
with open(sph_path, "r", encoding="utf-8") as f:
    text = f.read()

# Let's search for classes or terms related to observables
terms = ["stratum", "catastrophe", "winding", "monodromy", "commutator", "curvature", "popcount", "cascade"]
for term in terms:
    print(f"\n=== Matches for: {term} ===")
    matches = [line for line in text.split("\n") if re.search(r'\b' + term + r'\b', line, re.IGNORECASE)]
    for m in matches[:15]:
        print(" ", m.strip())
