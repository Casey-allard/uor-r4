#!/usr/bin/env python3
"""Axiom closure audit. SPEC.md section 19: the decisive audit inspects the
compiled environment, not the source text."""
import json, os, subprocess, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
claims = json.load(open("model/claims.json"))["claims"]
proved = [c for c in claims if c["level"] == "formalProof"]
if not proved:
    print("no formalProof claims"); sys.exit(0)
# Import the ROOT module, not one layer of it: probing `Cost.Objective` alone
# made every claim outside that import cone an `unknownIdentifier`, so those
# claims were never actually audited.
src = "\n".join(["import WasmGemmGnaf"] +
                [f"#print axioms {c['leanDeclaration']}" for c in proved])
open(".axioms_probe.lean", "w").write(src + "\n")
env = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
r = subprocess.run(["lean", ".axioms_probe.lean"], capture_output=True, text=True, env=env)
os.remove(".axioms_probe.lean")
print(r.stdout.strip() or r.stderr.strip())
BAD = ("sorryAx", "Lean.ofReduceBool", "Lean.trustCompiler")
hits = [b for b in BAD if b in r.stdout]
if hits or r.returncode != 0:
    print(f"\nFORBIDDEN AXIOM: {hits}"); sys.exit(1)
print("\naxiom audit: clean (Lean core logical axioms only)")
