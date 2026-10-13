import os

sph_path = "/Users/adminamn/gemini-dev/extra_reading/sph_formal_specification.txt"
with open(sph_path, "r", encoding="utf-8") as f:
    lines = f.readlines()

print(f"Total lines: {len(lines)}")

# Look for section headers or interesting math symbols
keywords = ["Single Prime Hypothesis", "M_1", "omega", "psi", "gamma", "delta", "projection", "eigen", "quantum"]
found = {}
for idx, line in enumerate(lines):
    for kw in keywords:
        if kw.lower() in line.lower():
            if kw not in found:
                found[kw] = []
            if len(found[kw]) < 10:
                found[kw].append((idx + 1, line.strip()))

for kw, items in found.items():
    print(f"\nKeyword: {kw}")
    for idx, item in items:
        print(f"  Line {idx}: {item}")
