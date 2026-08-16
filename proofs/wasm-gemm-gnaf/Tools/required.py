#!/usr/bin/env python3
"""Check SPEC section 15's required declarations against the COMPILED environment.

SPEC 15 lists the public declarations a conforming repository must have with
fully implemented bodies and proofs. The claims registry tracks obligations at a
coarser grain, so it understates the remaining proof surface. This derives the
inventory from SPEC.md itself and asks Lean whether each name exists, rather than
trusting a hand-maintained status field.
"""
import os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)

spec = open("SPEC.md", errors="replace").read()
m = re.search(r"## 15\. Required Lean declarations(.*?)\n## 16\.", spec, re.S)
if not m:
    print("SPEC section 15 not found"); sys.exit(1)
block = re.search(r"```lean\n(.*?)```", m.group(1), re.S)
names = [ln.strip() for ln in block.group(1).splitlines() if ln.strip()]

# Namespace each bare name under WasmGemmGnaf.
cands = {n: [f"WasmGemmGnaf.{n}"] for n in names}
# Theorems/ re-exports satisfy a requirement too.
for n in names:
    leaf = n.split(".")[-1]
    cands[n].append(f"WasmGemmGnaf.Theorems.{leaf}")

probe = ["import WasmGemmGnaf"]
for n in names:
    for c in cands[n]:
        probe.append(f"#print axioms {c}")
open(".required_probe.lean", "w").write("\n".join(probe) + "\n")
env = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
r = subprocess.run(["lean", ".required_probe.lean"], capture_output=True, text=True, env=env)
os.remove(".required_probe.lean")
out = r.stdout + r.stderr

present, missing = [], []
for n in names:
    if any(f"'{c}' " in out for c in cands[n]):
        present.append(n)
    else:
        missing.append(n)

print(f"SPEC 15 required declarations: {len(names)}")
print(f"  discharged : {len(present)}")
print(f"  outstanding: {len(missing)}")
if "--list" in sys.argv:
    for n in missing:
        print("    MISSING  " + n)
if "--check" in sys.argv and missing:
    sys.exit(1)
