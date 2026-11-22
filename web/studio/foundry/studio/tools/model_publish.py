"""Declarations of the Publish module, as data. Imported by tools/author.py beside the workspace and
inference declarations; not run on its own.

Publish turns a project version into a Hologram application: a `.holo` v4 archive composed by
PrismPM's own archive code (vendored verbatim into core/src/holo/, checked against the pinned
PrismPM commit by the lane). The model owns what PrismPM does not: the bytes of the model document
that names the published application (publishPreimage), the bytes of the source manifest PrismPM
embeds as the Metadata section (sourceManifest, PrismPM's own shape, version 4), the decision that
gates publishing on the evidence (publishDecision: the shell closure in the request must equal the
closure the lane recorded beside the core, and the lane's attestation must be present), and the path
a published application is served under on the page origin (holoPath).

String equality is not kernel reducible, so the decision's Accept row and the mismatch row are pinned
by the corpus (model/publish.json); the missing attestation row is a theorem because it matches on
the option, not on a string.
"""
import importlib.util, pathlib

HERE = pathlib.Path(__file__).resolve().parent


def _load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ws = _load("model_workspace")
STRING, opt, named = ws.STRING, ws.opt, ws.named
var, s, call, ctor, if_, match, branch, equal, eq, join, strings, q = ws.var, ws.s, ws.call, ws.ctor, ws.if_, ws.match, ws.branch, ws.equal, ws.eq, ws.join, ws.strings, ws.q
definition, theorem = ws.definition, ws.theorem


def preimage_body(application, version, closure, attestation):
    # The model document of a published application: keys sorted, no spaces, no trailing newline,
    # as PrismPM's canonical JSON spells an object. PrismPM derives model_id (SHA-256) and the
    # model content kappa (BLAKE3) from exactly these bytes.
    return join(strings(s('{"application":'), q(application), s(',"attestation":'), q(attestation), s(',"closure":'), q(closure),
                        s(',"schema":"hologram-forge/published/1","version":'), q(version), s("}")))


def manifest_body(lean_manifest, coverage, kernel, model_id, semantic_id, source_id):
    # PrismPM's source manifest (crates/prismpm/src/application_build.rs, fn source_manifest):
    # the four evidence files by kind with their SHA-256, the LexLean semantic and source
    # identities, version 4. Embedded as the archive's Metadata section.
    return join(strings(s('{"files":[{"kind":"lexlean-build-manifest","sha256":'), q(lean_manifest),
                        s('},{"kind":"lean4-prod-coverage","sha256":'), q(coverage),
                        s('},{"kind":"lean4-prod-kernel-ir","sha256":'), q(kernel),
                        s('},{"kind":"model-document","sha256":'), q(model_id),
                        s('}],"semantic_id":'), q(semantic_id), s(',"source_id":'), q(source_id), s(',"version":4}')))


decls = [
    definition("publishPreimage", [("application", STRING), ("version", STRING), ("closure", STRING), ("attestation", STRING)], STRING,
        preimage_body(var("application"), var("version"), var("closure"), var("attestation"))),
    definition("sourceManifest", [("leanManifest", STRING), ("coverage", STRING), ("kernel", STRING), ("modelId", STRING), ("semanticId", STRING), ("sourceId", STRING)], STRING,
        manifest_body(var("leanManifest"), var("coverage"), var("kernel"), var("modelId"), var("semanticId"), var("sourceId"))),
    # Accept only when the lane attested the model (an attestation id is present) and the shell the
    # request came from is the shell the lane projected beside this core (closure equality).
    definition("publishDecision", [("closure", STRING), ("built", STRING), ("attestation", opt(STRING))], named("Decision"),
        match(var("attestation"),
            branch("Option.none", [], ctor("Decision.Refuse")),
            branch("Option.some", ["id"], if_(equal(var("closure"), var("built")), ctor("Decision.Accept"), ctor("Decision.Refuse"))))),
    # A published application is served under its own archive address on the page origin.
    definition("holoPath", [("kappa", STRING)], STRING, join(strings(s("/holo/"), var("kappa"), s("/")))),

    theorem("publish_refused_without_attestation",
        eq(call("publishDecision", s("c"), s("c"), ctor("Option.none", targs=[STRING])), ctor("Decision.Refuse"))),
    theorem("holo_path_shape", eq(call("holoPath", s("k")), join(strings(s("/holo/"), s("k"), s("/"))))),
    theorem("publish_preimage_shape",
        eq(call("publishPreimage", s("a"), s("v"), s("c"), s("t")), preimage_body(s("a"), s("v"), s("c"), s("t")))),
]
