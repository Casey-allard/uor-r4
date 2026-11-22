"""Byte identity corpus for Publish: the generated core against the rule restated once in Python.

Writes model/publish.json: the model document bytes of a published application, the source manifest
bytes PrismPM embeds as Metadata, the publish decision on every row of evidence, and the served path.
core/tests/publish.rs feeds each case to the generated functions and compares byte for byte, then
composes a `.holo` through PrismPM's vendored archive code from the fixtures under
model/objects/publish/ and checks that the committed archive (the one the browser produced) is what
PrismPM's validator accepts, at the recorded address, and that one flipped byte is refused.

Run through tools/corpus.py, which writes every case file and runs the core's tests.
"""
import io, json, pathlib

root = pathlib.Path(__file__).resolve().parent.parent


def escape(v):
    # escapeJson: backslash first, then quote, newline, return, tab; nothing else is touched.
    return v.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")


def q(v):
    return '"' + escape(v) + '"'


def preimage(application, version, closure, attestation):
    # publishPreimage: sorted keys, no spaces, PrismPM's canonical JSON of the record.
    return '{"application":' + q(application) + ',"attestation":' + q(attestation) + ',"closure":' + q(closure) + ',"schema":"hologram-forge/published/1","version":' + q(version) + "}"


def source_manifest(lean_manifest, coverage, kernel, model_id, semantic_id, source_id):
    # sourceManifest: PrismPM's application_build.rs source_manifest shape, version 4.
    return ('{"files":[{"kind":"lexlean-build-manifest","sha256":' + q(lean_manifest) + '},{"kind":"lean4-prod-coverage","sha256":' + q(coverage)
            + '},{"kind":"lean4-prod-kernel-ir","sha256":' + q(kernel) + '},{"kind":"model-document","sha256":' + q(model_id)
            + '}],"semantic_id":' + q(semantic_id) + ',"source_id":' + q(source_id) + ',"version":4}')


def decision(closure, built, attestation):
    # publishDecision: no attestation, refuse; otherwise accept only when the closures are equal.
    if attestation is None:
        return "Refuse"
    return "Accept" if closure == built else "Refuse"


C1 = "79ae578ade8c74b59d33d820a969389b79c8d952f5660447a0fb6c98bbdf8d13"
C2 = "0" * 64
A1 = "7e9073ade09e6293a001bc106a2b8fb48f073b988b3755f1ce0d0713e652e441"
K1 = "blake3:" + "a1" * 32
H = "f52017c1dfe6c21e11b9eddfa99cacde3ca16a5892a69faab30cea40ea18d877"

cases = []


def case(name, application="scaffold", version=K1, closure=C1, built=C1, attestation=A1):
    cases.append({
        "name": name, "application": application, "version": version, "closure": closure, "built": built, "attestation": attestation,
        "preimage": preimage(application, version, closure, attestation or ""),
        "decision": decision(closure, built, attestation),
        "sourceManifest": source_manifest(H, H, H, C2, H, H),
        "sourceInputs": {"leanManifest": H, "coverage": H, "kernel": H, "modelId": C2, "semanticId": H, "sourceId": H},
        "holoPath": "/holo/" + K1 + "/",
    })


case("attested and matching")
case("closure drift", built=C2)
case("no attestation", attestation=None)
case("no attestation and drift", built=C2, attestation=None)
case("quotes in the name", application='say "hi"')
case("backslash in the name", application="a\\b")
case("newline in the name", application="line\nbreak")
case("tab in the name", application="tab\there")
case("unicode name", application="émoji 🚀")
case("empty name", application="")
case("empty version", version="")
case("other version", version="blake3:" + "b2" * 32)

out = root / "model" / "publish.json"
io.open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(cases, indent=1, ensure_ascii=False) + "\n")
print(f"wrote {out} ({len(cases)} cases)")
