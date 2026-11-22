"""Pin each declaration's exact axiom set to what Lean observes.

LexLean's `axioms` field is an exact policy (LLV7005 otherwise) and verify
reports one violating declaration per run, so this loops: verify, read the
observed set from the diagnostic, record it in tools/axioms.json, regenerate
the model, repeat until verify passes. Run from the project root in WSL:
    python3 tools/pin_axioms.py ~/src/PrismPM/target/debug/lexlean
"""
import json, pathlib, re, subprocess, sys

root = pathlib.Path(__file__).resolve().parent.parent
lexlean = sys.argv[1] if len(sys.argv) > 1 else "lexlean"
pins = root / "tools" / "axioms.json"
observed = json.loads(pins.read_text(encoding="utf-8")) if pins.exists() else {}
pattern = re.compile(r"`[\w.]+\.(\w+)` violates its \w+ axiom policy: observed \[([^\]]*)\]")

for round_number in range(1, 80):
    subprocess.run([sys.executable if sys.executable else "python3", str(root / "tools" / "author.py")], check=True)
    subprocess.run([lexlean, "lock"], cwd=root, check=True, capture_output=True)
    run = subprocess.run([lexlean, "verify"], cwd=root, capture_output=True, text=True)
    output = run.stdout + run.stderr
    if run.returncode == 0:
        print(f"verify passed after {round_number} round(s); {len(observed)} declarations carry a nonempty axiom set")
        for line in output.splitlines()[-4:]:
            print("  ", line)
        sys.exit(0)
    found = pattern.search(output)
    if not found:
        print(output[-3000:])
        sys.exit(f"round {round_number}: verify failed for a reason other than axiom policy")
    name, axioms = found.group(1), sorted(a.strip() for a in found.group(2).split(",") if a.strip())
    if observed.get(name) == axioms:
        sys.exit(f"round {round_number}: {name} already pinned to {axioms}, verify still refuses")
    observed[name] = axioms
    pins.write_text(json.dumps(observed, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"round {round_number}: {name} observes {axioms}")
sys.exit("gave up after 80 rounds")
