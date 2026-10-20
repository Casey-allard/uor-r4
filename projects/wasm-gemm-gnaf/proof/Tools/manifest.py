#!/usr/bin/env python3
"""Generate MANIFEST.json. SPEC sections 4 and 5.

SPEC 4 fixes three ordered identity stages plus one external attestation, and
requires them ACYCLIC: no manifest contains its own identity, and no two stages
hash each other.

  1. SourceManifestCore        immutable authority + handwritten Lean + fixtures
                               + tool inputs. EXCLUDES every manifest and every
                               generated output.
  2. GeneratedProofInputBody   binds the source-core identity plus every GENERATED
                               Lean source on the final theorem path. Excludes its
                               own encoding and all later outputs.
     PreFinalEnvironmentBody   binds that identity plus toolchain/dependency
                               identities and the compiled environment digest.
  3. OutputManifestBody        binds the three identities above plus artifact, seal,
                               proof registry, generated docs and the frozen
                               reproducibility plan. EXCLUDES MANIFEST.json itself.

MANIFEST.json is the canonical encoding of stage 3; its own external digest is
reported by CI and never included in its own preimage.
"""
import hashlib, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(ROOT)

def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()

def ident(body):
    """Canonical identity of a first-order body: sha256 of its canonical JSON.
    Sorted keys and fixed separators make the encoding injective for these bodies."""
    return hashlib.sha256(
        json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

# Files that are GENERATED rather than handwritten. Excluded from the source core.
GENERATED = {"CONFORMANCE.md", "MANIFEST.json", "WasmGemmGnaf.lean"}
# Outputs, excluded from stages 1 and 2.
OUTPUT_DIRS = ("artifacts/",)
SKIP_DIRS = (".git/", ".lake/", "vendor/wasm-spec/README.md")

def collect(pred):
    out = []
    for r, ds, fs in os.walk("."):
        ds[:] = [d for d in ds if d not in (".git", ".lake")]
        for f in sorted(fs):
            p = os.path.normpath(os.path.join(r, f))
            if p.startswith("./"):
                p = p[2:]
            if any(p.startswith(s) for s in SKIP_DIRS):
                continue
            if pred(p):
                out.append(p)
    return sorted(out)

# ---- stage 1: source core -------------------------------------------------
def is_source(p):
    if os.path.basename(p) in GENERATED:
        return False
    if any(p.startswith(d) for d in OUTPUT_DIRS):
        return False
    return (p.startswith("WasmGemmGnaf/") or p.startswith("authority/")
            or p.startswith("model/") or p.startswith("Tools/")
            or p.startswith("fixtures/") or p.startswith("Tests/")
            or p.startswith(".github/")
            or p in ("SPEC.md", "README.md", "AGENTS.md", "VERIFICATION.md",
                     "CERTIFICATION.md", "Justfile", "lakefile.lean",
                     "lean-toolchain", "lake-manifest.json", ".gitignore",
                     "LICENSE-APACHE", "LICENSE-MIT"))

source_files = collect(is_source)
sourceCore = {
    "schemaVersion": 1,
    "stage": "SourceManifestCore",
    "excludes": ["every manifest", "every generated output", "artifacts/"],
    "files": [{"path": p, "sha256": sha(p), "bytes": os.path.getsize(p)}
              for p in source_files],
}
sourceCoreId = ident(sourceCore)

# ---- stage 2: generated proof inputs + pre-final environment ---------------
generated_present = [p for p in ("WasmGemmGnaf.lean",) if os.path.exists(p)]
generatedProofInput = {
    "schemaVersion": 1,
    "stage": "GeneratedProofInputBody",
    "sourceManifestCoreIdentity": sourceCoreId,
    "excludes": ["its own JSON encoding", "every later output"],
    "generatedLeanSources": [{"path": p, "sha256": sha(p),
                              "bytes": os.path.getsize(p)}
                             for p in generated_present],
    "note": ("Artifact/Bytes.lean is NOT present: no artifact has been emitted, "
             "because emission requires WS-001 and BI-002. SPEC 13 Phase F step 4 "
             "is therefore not reached."),
}
generatedProofInputId = ident(generatedProofInput)

tc = open("lean-toolchain").read().strip()
preFinalEnvironment = {
    "schemaVersion": 1,
    "stage": "PreFinalEnvironmentBody",
    "generatedProofInputIdentity": generatedProofInputId,
    "leanToolchain": tc,
    "leanCommit": "d024af099ca4bf2c86f649261ebf59565dc8c622",
    "dependencies": [],
    "compiledEnvironmentDigest": None,
    "note": ("compiledEnvironmentDigest is null: SPEC 13 Phase F step 5 records the "
             "checked final declaration-environment digest, which is only meaningful "
             "once the final theorem is on the path. GO-001 is outstanding."),
}
preFinalEnvironmentId = ident(preFinalEnvironment)

# ---- stage 3: output manifest ---------------------------------------------
artifacts = collect(lambda p: p.startswith("artifacts/") and not p.endswith("README.md"))
outputManifest = {
    "schemaVersion": 1,
    "stage": "OutputManifestBody",
    "sourceManifestCoreIdentity": sourceCoreId,
    "generatedProofInputIdentity": generatedProofInputId,
    "preFinalEnvironmentIdentity": preFinalEnvironmentId,
    "excludes": ["MANIFEST.json itself"],
    "artifact": [{"path": p, "sha256": sha(p)} for p in artifacts],
    "atlasSeal": None,
    "proofRegistry": {"path": "model/claims.json", "sha256": sha("model/claims.json")},
    "generatedDocumentation": [{"path": "CONFORMANCE.md",
                                "sha256": sha("CONFORMANCE.md")}]
                               if os.path.exists("CONFORMANCE.md") else [],
    "reproducibilityPlan": {"path": "model/reproducibility-plan.json",
                            "sha256": sha("model/reproducibility-plan.json")},
    "releaseStatus": {
        "GO-001": "outstanding",
        "answerClass": "WorkloadIncomplete",
        "authority": "UOR-GNAF v1-draft.2 section 10.9",
    },
    "extraFilesBeyondSpecTree": {
        "rule": "SPEC 5: additional files permitted only when owned by a layer and listed here.",
        "files": [
            {"path": "WasmGemmGnaf/Wasm/Fault.lean", "layer": "Wasm",
             "justification": "SPEC 7.1's SpecMachine exposes ONE Fault used by both decode and initial. Binary and Config own genuinely different failure sets under SPEC 7.1's ownership rule, so the unified type needs its own module. Both injections proved injective, images proved disjoint."},
            {"path": "WasmGemmGnaf/Foundation/SchemaRegistry.lean", "layer": "Foundation",
             "justification": "Required verbatim by SPEC 6.2 ('Foundation/SchemaRegistry.lean SHALL retain the finite registry'), which the SPEC 5 tree omits."},
            {"path": "WasmGemmGnaf/Atlas/CoverageScope.lean", "layer": "Atlas",
             "justification": "Hardening. Proves the seal's cover check is blind to the byte universe, so it cannot be cited as universal coverage (claim AT-001, falsifier M7)."},
            {"path": "WasmGemmGnaf/Universal/BilinearLowerBound.lean", "layer": "Universal",
             "justification": "Proved partial lower bound (claim LB-002); establishes that the open tensor-rank problem does not gate the release theorem."},
        ],
    },
}

manifest = {
    "schemaVersion": 1,
    "description": "Ordered acyclic identity stages. SPEC sections 4 and 5.",
    "acyclicity": ("Each stage binds only EARLIER stage identities. No stage contains "
                   "its own identity. MANIFEST.json is the canonical encoding of "
                   "OutputManifestBody and its own digest is never in its own preimage."),
    "sourceManifestCore": sourceCore,
    "sourceManifestCoreIdentity": sourceCoreId,
    "generatedProofInputBody": generatedProofInput,
    "generatedProofInputIdentity": generatedProofInputId,
    "preFinalEnvironmentBody": preFinalEnvironment,
    "preFinalEnvironmentIdentity": preFinalEnvironmentId,
    "outputManifestBody": outputManifest,
    "reproducibilityAttestation": None,
}

if "--check" in sys.argv:
    if not os.path.exists("MANIFEST.json"):
        print("MANIFEST.json missing — run `just manifest`"); sys.exit(1)
    cur = json.load(open("MANIFEST.json"))
    for k in ("sourceManifestCoreIdentity", "generatedProofInputIdentity",
              "preFinalEnvironmentIdentity"):
        if cur.get(k) != manifest[k]:
            print(f"MANIFEST.json STALE: {k} differs — run `just manifest`"); sys.exit(1)
    print(f"manifest current: {len(source_files)} source files, 3 identity stages")
else:
    open("MANIFEST.json", "w").write(json.dumps(manifest, indent=2) + "\n")
    print(f"MANIFEST.json: {len(source_files)} source files")
    print(f"  sourceManifestCore      {sourceCoreId[:16]}...")
    print(f"  generatedProofInput     {generatedProofInputId[:16]}...")
    print(f"  preFinalEnvironment     {preFinalEnvironmentId[:16]}...")
