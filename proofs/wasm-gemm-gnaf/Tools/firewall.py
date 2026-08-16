#!/usr/bin/env python3
"""Dependency firewall check. SPEC section 10.1.

  "Foundation, Wasm, Gemm, Cost, and the extensional definitions in
   Universal/Competitor.lean, Correct.lean and Feasible.lean SHALL NOT import
   GNAF, Atlas, Artifact, Universal/LowerBound, Universal/Argmin, or Theorems.
   A source-and-environment gate SHALL reject an artifact-, selector-, or
   conclusion-dependent scope predicate."

The point is that the competitor universe must be defined without reference to
the artifact that will be compared against it. If `ProfileValid` could see the
selected artifact, `GlobalOptimal` would be a statement about a universe built
around its own answer. This check makes that structurally impossible rather
than a convention someone remembers.
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)

PROTECTED = [
    "Foundation/", "Wasm/", "Gemm/", "Cost/",
    "Universal/Competitor.lean", "Universal/Correct.lean", "Universal/Feasible.lean",
]
FORBIDDEN = [
    "WasmGemmGnaf.GNAF", "WasmGemmGnaf.Atlas", "WasmGemmGnaf.Artifact",
    "WasmGemmGnaf.Universal.LowerBound", "WasmGemmGnaf.Universal.Argmin",
    "WasmGemmGnaf.Theorems",
]

violations = []
checked = 0
for r, _, fs in os.walk("WasmGemmGnaf"):
    for f in sorted(fs):
        if not f.endswith(".lean"):
            continue
        path = os.path.join(r, f)
        rel = path[len("WasmGemmGnaf/"):]
        if not any(rel == p or rel.startswith(p) for p in PROTECTED):
            continue
        checked += 1
        for i, line in enumerate(open(path), 1):
            m = re.match(r"\s*import\s+([A-Za-z0-9_.]+)", line)
            if not m:
                continue
            mod = m.group(1)
            for bad in FORBIDDEN:
                if mod == bad or mod.startswith(bad + "."):
                    violations.append(f"{path}:{i}: imports {mod}")

if violations:
    print("DEPENDENCY FIREWALL VIOLATED (SPEC 10.1):")
    for v in violations:
        print("  " + v)
    print("\nThe competitor universe must not depend on the artifact, the selector,")
    print("or any conclusion. Move the definition or invert the dependency.")
    sys.exit(1)

print(f"dependency firewall clean: {checked} protected modules, no forbidden import")
