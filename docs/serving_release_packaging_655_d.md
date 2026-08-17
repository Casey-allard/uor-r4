# Release bundle packaging design (#655-D)

This document scopes #655-D ("produce + package one minimal evaluated
serving bundle") before any packaging code lands, mirroring the
research-then-decide convention already used for C1
(`docs/SERVING_MODEL_DISCOVERY.md` → `docs/serving_shared_loader_655_c1.md`
→ PR #773's additive verifier). It is a design and decision record only —
no server/CLI code path changes here.

## Recap: what C1 already established

`ReleaseBundleManifest` (`crates/uor-r4-api::release_bundle`) is a
schema-only type: parse, `.validate()`, and (since PR #773)
`src/release_bundle_loader.rs` verifies an optional `release-bundle.json`
sidecar's declared component digests against a resolved R4G1 bundle's
actual files. Nothing produces that sidecar yet — every real bundle takes
the `None` path today, which is by design (C1b, Q3). D's job is to make
the `Some` path real for at least one bundle.

## How a real bundle is produced today

The CLI compile pipeline writes into `.uor-models/compiled/<slug>/` (the
`physical_root` `resolve_loadable_compiled_bundle_with_authority` reads
from, `src/server.rs:2220-2290`):

- `compile` (`compile_hugging_face_with_progress_and_session`,
  `crates/uor-r4-graph-cli/src/lib.rs:6662-6925`) writes `tokenizer.bin`,
  `hamming_calibration.json`, `hierarchical_codes.json`,
  `tless_artifacts.bin`, `tless_store.bin`, `space_manifest.json`, plus a
  raw corpus pair.
- `transformerless cover` (`cover_command_with_authority`, graph-cli
  `lib.rs:4439-4700`) writes `graph-cover/cover.r4g1` +
  `graph-cover/cover_report.json`.
- `transformerless score` (`score_command_with_authority`, graph-cli
  `lib.rs:4981-5312`) writes `graph/score.r4g1` + `graph/score_report.json`.

`resolve_loadable_compiled_bundle_with_authority` reads `graph` as
`physical_root.join("graph/score.r4g1")` (falling back to a legacy
top-level `compiled.r4g1`, `server.rs:2249-2250`) and `teacher` as
`physical_root.join("tless_artifacts.bin")` (`server.rs:2252`) — so a
`release-bundle.json` sidecar belongs at `physical_root`'s root, exactly
where C1c's `verify_release_bundle_sidecar` already looks
(`RELEASE_BUNDLE_SIDECAR_FILE_NAME`, `release_bundle_loader.rs`).

This is a real, already-running pipeline, not a hypothetical one: a
complete bundle produced by exactly this flow already exists locally
(gitignored, `.gitignore:29`) at
`.uor-models/compiled/smollm2-135m-instruct/`, with the file layout above
plus (from an older run) a legacy top-level `compiled.r4g1` +
`compile_report.json` pair. `.uor-models/manifests` — written by the
separate `import`/`ModelStore` content-addressing system — is empty; that
system has never been pointed at this bundle and is out of scope here
(C1b Q2/C1e).

## `ReleaseBundleManifest` field-to-file mapping

The manifest's `components` block declares four `blake3:<hex>` digests:
`graph`, `signature_artifact`, `tokenizer` (optional), `score_report`, and
`compile_report`. Mapped against the real files above:

`components.graph` ↔ `graph/score.r4g1`, `components.signature_artifact`
↔ `tless_artifacts.bin`, `components.tokenizer` ↔ `tokenizer.bin`,
`components.score_report` ↔ `graph/score_report.json` — all four have an
obvious, already-real, literal producer.

`components.compile_report` is the one field without a literal-filename
match. `uor_r4_graph_compiler::compile()`
(`crates/uor-r4-graph-compiler/src/lib.rs:393`) writes a file named
`compile_report.json`, and the name match is tempting, but that function
is not what the real CLI pipeline runs for the cover stage — the real
pipeline's cover stage is `cover_command_with_authority`, which writes
`graph-cover/cover_report.json` instead. These are two separate cover
implementations (see next section), not a rename of one file. **Decision:
map `components.compile_report` to `graph-cover/cover_report.json`,
documented as a deliberate semantic mapping** (the cover stage is this
bundle's compile-report-equivalent artifact), not a literal filename
echo. If a future bundle shape adds a real `compile_report.json`
alongside `cover_report.json`, this mapping is revisited then.

`abi`, `capability`, `model_id`, and `uor_matmul` have no on-disk producer
at all — they are caller-supplied policy, exactly as C1b's own Q3
anticipated ("`uor_matmul` [pinned rev] has no producer anywhere; every
test/doc hand-supplies a fixed rev/profile/license"). A packaging helper
supplies these as fixed values from this project's own standing pins
(`serving_655.md`'s `uor-matmul` pin) and caller-provided identity
(`model_id`, `capability`), not derived from any file.

## Is `uor-r4-api::compile` a shortcut here?

No. Confirmed still uncalled outside its own crate's tests (`grep` across
`src/` and `crates/*/src` for `uor_r4_api::compile::`/`CompiledModel`
matches only `crates/uor-r4-api/src/{compile.rs,release_bundle.rs}` and
`crates/uor-r4-api/tests/api.rs`). It does call real stage-1
(`compile_hugging_face_with_progress`) and stage-3 (`score_command`)
functions, but for stage 2 it calls `uor_r4_graph_compiler::compile()`
directly — the *other* cover implementation, not the CLI's own
`cover_command_with_authority` — and its `work_dir` layout
(`work/graph/`, `work/scored/`) doesn't match the real convention
(`graph-cover/`, `graph/`) established above. Routing D through
`uor-r4-api::compile` would mean unifying two divergent cover
implementations and two divergent directory layouts as a prerequisite —
a bigger, riskier change than packaging needs, and not blocking:
**packaging reads already-produced files off disk; it does not need to
own or change how they get produced.**

## Proposed slice sequence

- **D0** (this document): research + decision record, no code.
- **D1**: a small, additive, pure `package_release_bundle` helper —
  given a `physical_root` directory plus caller-supplied `model_id`,
  `capability`, and the standing `uor_matmul` pin, blake3-hash the four
  real files above (skip the tokenizer digest if the file is absent,
  matching `components.tokenizer`'s `Option`ality) and construct +
  `.validate()` a `ReleaseBundleManifest`. No filesystem *write* yet —
  return the manifest value, unit-tested against a synthetic fixture
  (mirroring `release_bundle_loader.rs`'s own test helpers) plus an
  `#[ignore]`-by-default integration test against the real local bundle
  at `.uor-models/compiled/smollm2-135m-instruct/` (mirrors
  `crates/uor-r4-api/tests/api.rs`'s own `#[ignore]`d end-to-end
  convention — keeps CI independent of a multi-GB local model store).
- **D2**: a thin CLI subcommand (e.g. `r4 package-release-bundle
  <physical_root>`) that calls D1's helper and writes
  `release-bundle.json` next to the bundle it describes — the first time
  anything produces the sidecar C1c already knows how to verify.
- **D3**: run D2 against the real local bundle, confirm C1c's
  `verify_release_bundle_sidecar` returns `Some(..)` end-to-end, and
  capture that as a golden/fixture-backed regression test. This is the
  handoff point to **C1d** (surface `verified` on `/v1/models` and
  `/uor/v1/status`), which was blocked on exactly this.

D1-D3 do not touch the default engine, the #248 cascade, or
`ModelStore`/`import` — they are purely additive, matching this project's
established pattern of landing dormant/additive infrastructure before any
default-affecting change (#515). The broader D scope in #655's own issue
text (quality/score-report *evaluation* gating, resource requirements,
license/provenance text, distributing the bundle as part of the release
artifact itself) is real remaining D work beyond D1-D3, deliberately not
decided here — D1-D3 is the minimal slice that makes the sidecar real
without deciding those larger policy questions prematurely.

## Open question deliberately deferred, not decided here

Whether packaging should *gate* on `score_report.json`'s quality signal
(e.g. refuse to package below some threshold) or merely *record* it
verbatim in the manifest. D1-D3 record it verbatim (no new gate,
consistent with typed-decline-over-hard-failure — a low-quality bundle is
still an honestly-described bundle); introducing a packaging-time quality
gate is real policy work for a later D slice, not implied by anything
here.

## Non-goals for D0-D3

Does not change the default engine (#655-F, Casey sign-off required).
Does not migrate or touch `ModelStore`/`import` (C1e). Does not change
what `resolve_loadable_compiled_bundle_with_authority` reads or how C1c
verifies — it produces the file C1c already knows how to consume. Does
not unify `uor-r4-api::compile`'s divergent cover-stage/work_dir layout
with the real CLI pipeline. Does not distribute the packaged bundle as
part of any release/install artifact yet (later D work, once at least one
bundle can be packaged at all).
