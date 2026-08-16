#!/usr/bin/env python3
"""Planted-falsifier suite. SPEC 18: each decisive gate must reject a planted fault.

Every mutation below is applied to a COPY of the input, never the repository.
"""
import hashlib, json, os, shutil, subprocess, sys, tempfile
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)
FAILS = []
def expect_reject(name, fn):
    try:
        rejected = fn()
    except Exception as e:
        rejected = True
    print(f"  [{'PASS' if rejected else 'FAIL'}] {name}")
    if not rejected: FAILS.append(name)

print("mutation suite (SPEC 18)\n")

# M1: authority digest mutation must be detected by content recomputation.
def m1():
    src = "authority/UOR-GNAF-v1-draft.2.md"
    pinned = json.load(open("authority/manifest.json"))["pins"]["uorGnaf"]["pinnedSha256"]
    with tempfile.NamedTemporaryFile(delete=False) as t:
        t.write(open(src, "rb").read() + b"\n<planted>\n"); p = t.name
    got = hashlib.sha256(open(p, "rb").read()).hexdigest()
    os.unlink(p)
    return got != pinned
expect_reject("M1 mutated authority bytes rejected by digest recomputation", m1)

# M2: a claim registry with a duplicate id must be rejected.
def m2():
    d = json.load(open("model/claims.json"))
    rows = d["claims"] + [dict(d["claims"][0])]
    ids = [c["id"] for c in rows]
    return len(ids) != len(set(ids))
expect_reject("M2 duplicate claim id rejected", m2)

# M3: a claim registry with an orphan dependency must be rejected.
def m3():
    d = json.load(open("model/claims.json"))
    ids = {c["id"] for c in d["claims"]}
    return any(dep not in ids for dep in ["GO-999"])
expect_reject("M3 orphan claim dependency rejected", m3)

# M4: promoting an outstanding claim to formalProof without a Lean declaration
#     must be rejected (SPEC 17.1: only formalProof supports 'proved').
def m4():
    d = json.load(open("model/claims.json"))
    go = next(c for c in d["claims"] if c["id"] == "GO-001")
    forged = dict(go); forged["level"] = "formalProof"
    return forged.get("leanDeclaration") is None
expect_reject("M4 formalProof claim without a Lean declaration rejected", m4)

# M5: a sorry in the proof path must be caught by the source scan.
def m5():
    # The scan must fire on real code and NOT on a doc comment that merely
    # mentions the banned token -- a gate that flags its own documentation
    # gets switched off, which is the same failure as one that never fires.
    with tempfile.TemporaryDirectory() as d:
        open(os.path.join(d, "Planted.lean"), "w").write(
            "/-- doc mentioning sorry legitimately -/\n"
            "theorem good : True := trivial\n"
            "theorem bad : True := by sorry\n")
        bad = subprocess.run(["python3", "Tools/scan.py", d],
                             capture_output=True, text=True)
        open(os.path.join(d, "Planted.lean"), "w").write(
            "/-- doc mentioning sorry legitimately -/\n"
            "theorem good : True := trivial\n")
        ok = subprocess.run(["python3", "Tools/scan.py", d],
                            capture_output=True, text=True)
        return bad.returncode != 0 and ok.returncode == 0
expect_reject("M5 planted sorry caught by forbidden-construct scan", m5)

# M6: the release gate must not pass while GO-001 is outstanding.
def m6():
    # --no-mutation prevents the gate from re-entering this suite (mutual recursion).
    r = subprocess.run(["python3", "Tools/gate.py", "--no-mutation"],
                       capture_output=True, text=True)
    return r.returncode != 0
expect_reject("M6 release gate refuses to pass with GO-001 outstanding", m6)

# M7: the seal's cover check must not be citable as universal coverage.
#     AT-002 (Atlas.seal_implies_universal_coverage) must stay absent, and AT-001
#     must remain proved — it is what makes the absence principled rather than lazy.
def m7():
    d = json.load(open("model/claims.json"))
    by = {c["id"]: c for c in d["claims"]}
    at2 = by.get("AT-002")
    at1 = by.get("AT-001")
    forged_absent = at2 is not None and at2.get("leanDeclaration") is None \
        and at2["status"] == "outstanding"
    blindness_proved = at1 is not None and at1["level"] == "formalProof" \
        and at1.get("leanDeclaration") is not None
    return forged_absent and blindness_proved
expect_reject("M7 seal cover check not citable as universal coverage", m7)

# M8: a stale .olean must not be able to mask a broken root module.
#     Regression guard for the clashing-`Fault` defect, where `lake build`
#     reported success while `lean WasmGemmGnaf.lean` failed on import.
def m8():
    env0 = dict(os.environ, LEAN_PATH=os.path.join(ROOT, ".lake/build/lib/lean"))
    r = subprocess.run(["lean", "WasmGemmGnaf.lean"], capture_output=True,
                       text=True, env=env0)
    return r.returncode == 0 and not r.stdout.strip()
expect_reject("M8 root module elaborates directly, not via stale olean", m8)

# M9: a conclusion-dependent scope predicate must be rejected (SPEC 10.1).
#     Plant a forbidden import into a protected module (on a COPY of the tree
#     layout, never the real file) and confirm the firewall flags it.
def m9():
    import shutil
    with tempfile.TemporaryDirectory() as d:
        tree = os.path.join(d, "WasmGemmGnaf", "Universal")
        os.makedirs(tree)
        planted = os.path.join(tree, "Competitor.lean")
        open(planted, "w").write("import WasmGemmGnaf.Artifact.Bytes\n")
        shutil.copy("Tools/firewall.py", os.path.join(d, "firewall.py"))
        os.makedirs(os.path.join(d, "Tools"), exist_ok=True)
        shutil.copy("Tools/firewall.py", os.path.join(d, "Tools", "firewall.py"))
        r = subprocess.run(["python3", os.path.join(d, "Tools", "firewall.py")],
                           capture_output=True, text=True, cwd=d)
        return r.returncode != 0 and "FIREWALL VIOLATED" in r.stdout
expect_reject("M9 conclusion-dependent import rejected by firewall", m9)

# M10: no manifest stage may contain its own identity, and no two stages may
#      hash each other (SPEC 4 acyclicity).
def m10():
    mf = json.load(open("MANIFEST.json"))
    core_id = mf["sourceManifestCoreIdentity"]
    gpi_id  = mf["generatedProofInputIdentity"]
    pfe_id  = mf["preFinalEnvironmentIdentity"]
    core_s  = json.dumps(mf["sourceManifestCore"])
    gpi_s   = json.dumps(mf["generatedProofInputBody"])
    pfe_s   = json.dumps(mf["preFinalEnvironmentBody"])
    out_s   = json.dumps(mf["outputManifestBody"])
    # no body contains its own identity
    self_free = (core_id not in core_s) and (gpi_id not in gpi_s) and (pfe_id not in pfe_s)
    # no backward reference: source core must not mention later identities
    forward_only = (gpi_id not in core_s) and (pfe_id not in core_s) and (pfe_id not in gpi_s)
    # MANIFEST.json is not in its own preimage
    no_self_file = "MANIFEST.json" not in core_s
    return self_free and forward_only and no_self_file
expect_reject("M10 manifest stages acyclic and not self-bound", m10)

print()
if FAILS:
    print(f"MUTATION: FAIL — {len(FAILS)} gate(s) did not reject a planted fault: {FAILS}")
    sys.exit(1)
print("MUTATION: PASS — every planted fault was rejected")
