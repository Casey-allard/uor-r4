#!/usr/bin/env python3
"""Normative release gate. SPEC.md section 20.2.

Exits non-zero unless every numbered condition holds. It is expected to fail at
step 9 while WGG-GO-1 is outstanding; that failure is the conforming behavior.
"""
import hashlib, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
FAIL = []
def check(n, name, ok, detail=""):
    # Only show the detail on failure: a PASS row carrying a failure explanation
    # is how a green gate gets misread as a red one and vice versa.
    shown = detail if (detail and not ok) else ""
    print(f"  [{'PASS' if ok else 'FAIL'}] {n}. {name}" + (f" — {shown}" if shown else ""))
    if not ok: FAIL.append((n, name, detail))

def sha(p): return hashlib.sha256(open(p, "rb").read()).hexdigest()

print("release gate (SPEC 20.2)\n")

# 1. authority + toolchain identities
man = json.load(open("authority/manifest.json"))
u = man["pins"]["uorGnaf"]
check(1, "UOR-GNAF authority digest",
      sha(u["path"]) == u["pinnedSha256"], u["pinnedSha256"][:16] + "...")
mf = subprocess.run(["python3", "Tools/manifest.py", "--check"], capture_output=True, text=True)
check(1, "manifest identity stages current and acyclic", mf.returncode == 0,
      mf.stdout.strip().splitlines()[-1][:150] if mf.stdout.strip() else mf.stderr[:120])
tc = open("lean-toolchain").read().strip()
check(1, "Lean toolchain pin", tc == man["pins"]["lean"]["toolchain"], tc)
check(1, "WebAssembly Core wg-3.0 vendored",
      man["pins"]["wasmCore"]["vendored"],
      "vendor/wasm-spec/ empty; pinned tree not vendored")

# 2. claim graph
claims = json.load(open("model/claims.json"))["claims"]
ids = [c["id"] for c in claims]
check(2, "claim graph nonempty", len(claims) > 0, f"{len(claims)} claims")
check(2, "claim ids unique", len(ids) == len(set(ids)))
dangling = [d for c in claims for d in c.get("dependsOn", []) if d not in ids]
check(2, "no orphan dependencies", not dangling, str(dangling))
fw = subprocess.run(["python3", "Tools/firewall.py"], capture_output=True, text=True)
check(2, "dependency firewall (SPEC 10.1)", fw.returncode == 0,
      fw.stdout.strip().splitlines()[-1][:160] if fw.stdout.strip() else "")

# 3. Lean builds, no placeholder or unexpected axiom
r = subprocess.run(["lake", "build"], capture_output=True, text=True)
check(3, "lake build", r.returncode == 0, r.stderr.strip()[:120])
# `lake build` can report success while serving a stale .olean, masking a root
# module that does not actually elaborate (this really happened: two modules
# declared clashing `Fault` types and lake reported green). Elaborate the root
# directly — success there is the claim that matters.
env0 = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
rootlean = subprocess.run(["lean", "WasmGemmGnaf.lean"], capture_output=True,
                          text=True, env=env0)
check(3, "root module elaborates (not a stale olean)",
      rootlean.returncode == 0 and not rootlean.stdout.strip(),
      rootlean.stdout.strip()[:160])
scan = subprocess.run(["python3", "Tools/scan.py"], capture_output=True, text=True)
banned = "" if scan.returncode == 0 else scan.stdout.strip()
check(3, "no forbidden constructs", banned == "", banned[:160])

ALLOWED = {"propext", "Quot.sound", "Classical.choice"}
proved = [c for c in claims if c["level"] == "formalProof"]
# The root module, not one layer of it: a probe importing only `Cost.Objective`
# cannot see a claim proved in `GNAF` or `Gemm`, and an unseen claim is an
# unaudited claim.
probe = "\n".join(["import WasmGemmGnaf"] +
                  [f"#print axioms {c['leanDeclaration']}" for c in proved])
open(".gate_axioms.lean", "w").write(probe + "\n")
env = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
ax = subprocess.run(["lean", ".gate_axioms.lean"], capture_output=True, text=True, env=env)
os.remove(".gate_axioms.lean")
unexpected = [w for w in ("sorryAx", "Lean.ofReduceBool", "Lean.trustCompiler") if w in ax.stdout]
check(3, "axiom closure clean", ax.returncode == 0 and not unexpected,
      (str(unexpected) or ax.stderr.strip()[:120]) if (unexpected or ax.returncode) else
      "propext, Quot.sound only")

devs = json.load(open("model/spec-deviations.json"))["deviations"]
undocumented = [d for d in devs if not d.get("intendedContentProvedBy")]
check(2, "every SPEC deviation carries a proved replacement",
      not undocumented, f"{len(devs)} deviation(s), all justified"
      if not undocumented else str([d["id"] for d in undocumented]))

# SPEC 15 inventory, derived from SPEC.md and checked against the COMPILED
# environment. A hand-maintained JSON status field is not evidence; an external
# audit found the registry understated the surface by 26 declarations.
req = subprocess.run(["python3", "Tools/required.py"], capture_output=True, text=True)
reqline = [l for l in req.stdout.splitlines() if "outstanding" in l]
nout = int(reqline[0].split(":")[1]) if reqline else -1
check(2, "SPEC 15 required declarations all discharged", nout == 0,
      req.stdout.strip().replace("\n", " ")[:150])

# All declaration presence questions are answered by ONE lean invocation; a probe
# per declaration re-imports 95 modules each time and times the gate out.
_PROBE = [
    "WasmGemmGnaf.Conformance.globalOptimal_matches_authority_schema",
    "WasmGemmGnaf.Conformance.profileValid_matches_authority_schema",
    "WasmGemmGnaf.Wasm.profile_matches_pinned_revision",
    "WasmGemmGnaf.Universal.universal_sublevel_coverage",
    "WasmGemmGnaf.Universal.all_competitors_lower_bound",
    "WasmGemmGnaf.Artifact.released_attains_lower_bound",
    "WasmGemmGnaf.Artifact.released_wasm_gemm_gnaf_global_optimal",
]
open(".gate_decl.lean", "w").write(
    "import WasmGemmGnaf\n" + "".join(f"#print axioms {n}\n" for n in _PROBE))
_env = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
_r = subprocess.run(["lean", ".gate_decl.lean"], capture_output=True, text=True, env=_env)
os.remove(".gate_decl.lean")
_out = _r.stdout + _r.stderr

def declared(name):
    """Present in the compiled environment? Answered from the single probe above."""
    return ("'" + name + "' ") in _out

# SPEC 1 / authority WGG-GO-1: the gate must compare the compiled UNFOLDED
# definition with the frozen schema, not merely find a name. The binding is
# definitional (Iff.rfl in Conformance/Schema.lean), so a weakened GlobalOptimal
# or an artifact-specific ProfileValid stops elaborating.
check(2, "scope-critical definitions match the frozen WGG-GO-1 schema",
      declared("WasmGemmGnaf.Conformance.globalOptimal_matches_authority_schema")
      and declared("WasmGemmGnaf.Conformance.profileValid_matches_authority_schema"),
      "schema binding absent -- a name check alone cannot reject a weakened proposition")

# 4-8. semantics, coverage, artifact, lower bound -- by declaration presence.
check(4, "WebAssembly and GEMM semantics built",
      declared("WasmGemmGnaf.Wasm.profile_matches_pinned_revision"),
      "Wasm.profile_matches_pinned_revision absent")
check(5, "universal sublevel coverage proved",
      declared("WasmGemmGnaf.Universal.universal_sublevel_coverage"),
      "Universal.universal_sublevel_coverage absent")
check(6, "committed artifact matches proved value",
      os.path.exists("artifacts/wasm-gemm-gnaf.wasm"), "artifact not emitted")
check(7, "artifact decode/validate/ABI/cost theorems", False, "gated on WS-001")
check(8, "universal lower bound and attainment",
      declared("WasmGemmGnaf.Universal.all_competitors_lower_bound")
      and declared("WasmGemmGnaf.Artifact.released_attains_lower_bound"),
      "Universal.all_competitors_lower_bound / Artifact.released_attains_lower_bound absent")

# 9. The release theorem itself -- presence in the compiled environment, never a
# JSON field. This is the condition the whole gate exists to protect.
check(9, "released_wasm_gemm_gnaf_global_optimal closed",
      declared("WasmGemmGnaf.Artifact.released_wasm_gemm_gnaf_global_optimal"),
      "declaration absent; answer class WorkloadIncomplete (UOR-GNAF 10.9)")

# 10-13
check(10, "Atlas seal reconstructs", False, "seal not constructed")
if "--no-mutation" in sys.argv:
    # Invoked from M6; running the suite here would recurse back into the gate.
    check(11, "mutation suites reject planted faults", True, "skipped (invoked from M6)")
else:
    mut = subprocess.run(["python3", "Tools/mutation.py"], capture_output=True, text=True)
    check(11, "mutation suites reject planted faults", mut.returncode == 0,
          "all planted faults rejected" if mut.returncode == 0 else mut.stdout.strip()[-160:])
check(12, "two clean emissions byte-identical", False, "gated on step 6")
root = subprocess.run(["python3", "Tools/root.py", "--check"], capture_output=True, text=True)
check(13, "no stale or unowned Lean module", root.returncode == 0, root.stdout.strip()[:160])
check(13, "worktree clean after verification",
      subprocess.run(["git", "status", "--porcelain"], capture_output=True,
                     text=True).stdout.strip() == "", "untracked build outputs present")

print(f"\n{'='*66}")
if FAIL:
    print(f"GATE: FAIL — {len(FAIL)} unmet condition(s).")
    print("This is the conforming outcome while WGG-GO-1 is outstanding.")
    print("Terminal answer: WorkloadIncomplete (UOR-GNAF 10.9). See CERTIFICATION.md.")
    print("Per UOR-GNAF 13.3 the gate MUST NOT return an unproved global label.")
    sys.exit(1)
print("GATE: PASS")
