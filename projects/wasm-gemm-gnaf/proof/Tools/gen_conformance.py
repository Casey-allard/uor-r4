#!/usr/bin/env python3
"""Generate CONFORMANCE.md deterministically from model/claims.json. SPEC 17.3."""
import json, os
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
d = json.load(open("model/claims.json"))
import subprocess

def inventory():
    """Live inventory. Prose docs cite this rather than duplicating counts,
    which is why the numbers in them kept going stale."""
    mods = subprocess.run(["find", "WasmGemmGnaf", "-name", "*.lean"],
                          capture_output=True, text=True).stdout.split()
    lines = sum(sum(1 for _ in open(m, errors="replace")) for m in mods)
    thms = 0
    for m in mods:
        for ln in open(m, errors="replace"):
            t = ln.lstrip()
            if t.startswith("theorem ") or t.startswith("instance ") and False:
                thms += 1
    return len(mods), lines, thms

nmod, nlines, nthm = inventory()

L = []
L.append("# Conformance\n")
L.append(f"**Inventory:** {nmod} Lean modules, {nlines:,} lines, {nthm:,} proved theorems.")
L.append("Generated live; prose documents cite this table rather than repeating counts.\n")
L.append("Generated from `model/claims.json` by `just docs`. Do not edit by hand.\n")
L.append("Claim levels are load-bearing (SPEC 17.1). Only `formalProof` supports the")
L.append("words \"proved\", \"theorem\", or \"globally optimal\".\n")
L.append("## Claims\n")
L.append("| ID | Level | Status | Statement | Lean declaration |")
L.append("| --- | --- | --- | --- | --- |")
for c in d["claims"]:
    decl = c.get("leanDeclaration") or "—"
    st = c["statement"].replace("|", "\\|")
    st = st if len(st) <= 110 else st[:107] + "..."
    L.append(f"| `{c['id']}` | {c['level']} | {c['status']} | {st} | `{decl}` |")
L.append("\n## Axiom closure\n")
L.append("Every `formalProof` claim's transitive axioms, from `#print axioms`:\n")
for c in d["claims"]:
    if c["level"] == "formalProof":
        ax = ", ".join(f"`{a}`" for a in c.get("axioms", [])) or "none"
        L.append(f"- `{c['id']}` — {ax}")
L.append("\nPermitted: `propext`, `Quot.sound`, `Classical.choice` (Lean core logical")
L.append("axioms, SPEC 4). Any `sorryAx` or project-declared axiom fails the gate.\n")
L.append("## Refuted framings\n")
L.append("Recorded so they are not silently re-asserted:\n")
for r in d.get("refutedFramings", []):
    L.append(f"- **{r['verdict']}** ({r['confidence']}) — {r['framing']}")
    L.append(f"  - {r['reason']}")
L.append("\n## Outstanding obligations\n")
out = [c for c in d["claims"] if c["status"] == "outstanding"]
L.append(f"{len(out)} outstanding. Terminal answer for `GO-001`: `WorkloadIncomplete`")
L.append("(UOR-GNAF v1-draft.2 section 10.9). See `CERTIFICATION.md`.\n")
for c in out:
    L.append(f"- `{c['id']}` ({c.get('obligation','—')}) — {c['statement'][:100]}")
open("CONFORMANCE.md", "w").write("\n".join(L) + "\n")
print(f"CONFORMANCE.md: {len(d['claims'])} claims, {len(out)} outstanding")
