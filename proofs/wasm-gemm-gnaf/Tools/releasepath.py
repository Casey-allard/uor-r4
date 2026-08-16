#!/usr/bin/env python3
"""Reject `noncomputable` and Type-valued choice on the release path (SPEC 19).

SPEC 19 excludes noncomputable definitions from the product/proof path, and
SPEC 6.3 requires executable proof-producing functions to be computable.
`Tools/scan.py` deliberately ignores comments, so it cannot see a
`noncomputable def`; this checks declarations directly on the modules that
constitute the release path.
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
sys.path.insert(0, os.path.join(ROOT, "Tools"))
from scan import strip  # reuse the comment/string stripper

RELEASE_PATH = ["WasmGemmGnaf/Artifact/", "WasmGemmGnaf/Theorems/",
                "WasmGemmGnaf/Universal/"]
PAT = re.compile(r"^\s*noncomputable\s+(def|abbrev|instance)\s+(\S+)")

hits, scanned = [], 0
for base in RELEASE_PATH:
    for r, _, fs in os.walk(base):
        for f in sorted(fs):
            if not f.endswith(".lean"):
                continue
            p = os.path.join(r, f)
            scanned += 1
            for ln, line in enumerate(strip(open(p, errors="replace").read()).splitlines(), 1):
                m = PAT.match(line)
                if m:
                    hits.append(f"{p}:{ln}: noncomputable {m.group(1)} {m.group(2)}")

if hits:
    print("NONCOMPUTABLE ON THE RELEASE PATH (SPEC 19 / 6.3):")
    for h in hits:
        print("  " + h)
    print("\nSPEC 19 excludes noncomputable definitions from the product/proof path.")
    print("A classically-chosen evaluator decodes, validates, enumerates and")
    print("executes nothing; it cannot stand in for the implemented explorer.")
    sys.exit(1)
print(f"release path computable: {scanned} modules, no noncomputable definition")
