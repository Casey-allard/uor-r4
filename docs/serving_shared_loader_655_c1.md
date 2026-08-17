# Shared startup loader design (#655-C1)

This document proposes a concrete design for #655-C1 (the "real shared
startup loader" named in #655's own A-F decomposition), answering the
three "Next open questions for #655-C1" left open by
`docs/SERVING_MODEL_DISCOVERY.md` (#655-C1a, PR #739). It is a design and
decision record only — no server/CLI code path changes here, per this
project's own convention of scoping a multi-system change before touching
it (mirrors #655-C0/C1a's own additive-and-dormant shape, #515).

## Recap: what C1a already established

`docs/SERVING_MODEL_DISCOVERY.md` mapped three independent, disconnected
model-loading systems: `src/model.rs`'s `ModelStore` (CLI `ask`/`chat`),
`src/server.rs`'s ad hoc four-tier `#248` cascade (every HTTP path), and
`crates/uor-r4-api::release_bundle::ReleaseBundleManifest` (#655-C0, schema
only, read/written by nothing). Its own conclusion: `ReleaseBundleManifest`
fits the R4G1 tier well and should not be generalized to describe all four
tiers; `ModelStore` is a live system serving a real, different purpose and
should not be replaced wholesale.

## A fourth finding, not covered by C1a

`crates/uor-r4-api::compile` — the in-process compile pipeline that
produces `CompiledModel` (exactly the digest/ABI shape
`ReleaseBundleManifest::from_compiled_model` consumes) — is **not called
by any CLI or server code path today**. `grep` across `src/` and
`crates/` finds exactly one consumer: `crates/uor-r4-api/tests/api.rs`,
that crate's own test suite. The real production compile path is the
CLI's stage-based flow (`uor-r4-graph-cli::compile_hugging_face_with_progress`
→ `uor-r4-graph-compiler::compile` → `score_command`, orchestrated by
`scripts`/`Justfile` targets, not this API crate), which writes
`compiled.r4g1`/`score.r4g1`/`tless_artifacts.bin` etc. straight to disk
and computes no digests at any point.

This matters for sequencing: `ReleaseBundleManifest` is fully built and
tested, but nothing produces one from a real compile, because the thing
that would produce one (`uor-r4-api::compile`) isn't in the loop that real
compiles go through. A loader that requires a `ReleaseBundleManifest` to
load anything would find zero real bundles satisfy it on day one.

## Q1 — Does the loader read `ReleaseBundleManifest` for the R4G1 tier only?

**Yes**, confirming C1a's lean. Tiers 2-4 (`transformerless`,
`teacher-oracle`, `geometric`) keep their current ad hoc discovery
unchanged — `ReleaseBundleManifest`'s one-bundle-one-engine shape doesn't
fit them (C1a §4), and there is no product requirement forcing them onto a
shared schema. Scope C1 to the R4G1 tier's discovery, which is also the
tier #655-D's own text names as the packaging target.

## Q2 — Does `ModelStore` get pointed at the same manifest, or stay separate?

**Stay separate for this phase.** `ModelStore::is_compiled_bundle`
requires `tless_artifacts.bin` + `tless_store.bin` + `tokenizer.bin`
together — the pre-R4G1 bundle shape — and is a live system gating
`ask`/`chat` for real local installs today. Migrating it to
`ReleaseBundleManifest` is a breaking change to every existing local
`.uor-models` store's compatibility contract and deserves its own
explicit, separately-scoped decision (tracked as a follow-up once C1's
server-side piece is observed working, not decided implicitly here).

## Q3 — Sequencing against #655-D (packaging)

**The loader must tolerate a missing manifest as the common case, not the
exception**, for two independent reasons: no shipped bundle produces one
yet (see the fourth finding above), and #655-D hasn't landed. Building the
loader to *require* a manifest would make it untestable against any real
bundle until D ships, and D itself needs something to target. Breaking the
cycle:

1. The loader's manifest-parsing half (parse + `.validate()` a
   `ReleaseBundleManifest` from bytes) is already independently complete
   and tested in `release_bundle.rs` — nothing new needed there.
2. The loader's file-discovery half is added as **a strictly additive
   check layered on top of today's existing R4G1 discovery**
   (`resolve_loadable_compiled_bundle_with_authority`), not a
   replacement: if a `release-bundle.json` sidecar is present next to a
   resolved bundle's files, parse and validate it, verify its declared
   component digests against the actual file bytes already being loaded,
   and record a `verified: bool` alongside the existing resolved-bundle
   state. If absent, or a hash mismatch is found, behavior is **unchanged
   from today** — legacy discovery proceeds exactly as it does now. A
   verification failure is surfaced (logged, and worth a status field on
   `/v1/models`) but is not fatal, matching this project's D4 policy of
   typed decline over hard failure.
3. This makes the loader shippable and testable *before* #655-D lands,
   using a hand-built or test-fixture sidecar (exactly as
   `release_bundle.rs`'s own tests already do). #655-D then has a real
   target to write to — a `release-bundle.json` sidecar next to whatever
   directory layout D settles on — without the loader blocking on D or D
   blocking on the loader.
4. Computing real digests from a real compile still needs an answer
   eventually: either #655-D's packaging step calls
   `uor-r4-api::compile`'s pipeline directly (unifying it with the real
   compile path, a bigger change), or a smaller post-hoc "digest an
   already-compiled bundle's files" helper is added that doesn't require
   the CLI stage flow to change at all. Recommend the second for this
   phase — lower blast radius, and it's exactly what
   `ReleaseBundleManifest::from_compiled_model`'s sibling would need
   regardless of which compile path eventually produces a `CompiledModel`
   value to feed it.

## Proposed slice sequence

- **C1a** (done, PR #739): research, no code.
- **C1b** (this document): design/decision record, no code.
- **C1c**: additive helper — parse + validate a `release-bundle.json`
  sidecar if present next to an R4G1 bundle directory; verify its digests
  against the bundle's actual files; expose a `verified: bool` /
  `release_bundle: Option<ReleaseBundleManifest>` field on
  `ResolvedCompiledBundle`. No behavior change when the sidecar is
  absent (the case for every bundle that exists today) — unit-testable
  now with a synthetic sidecar fixture, without waiting on #655-D.
- **C1d** (after #655-D produces at least one real sidecar): surface
  `verified` on `/v1/models` and `/uor/v1/status`; confirm end-to-end
  against a real packaged bundle.
- **C1e** (separate, explicitly deferred issue, not part of this
  sequence): `ModelStore`/cascade reconciliation — Q2 above.

## Non-goals for C1

Does not change the default engine (#655-F). Does not touch
`transformerless`/`teacher-oracle`/`geometric` discovery. Does not require
or block on #655-D landing first (C1c is independently testable). Does not
migrate `ModelStore` (Q2/C1e). Does not make manifest presence mandatory
for a bundle to load — a missing or invalid sidecar is exactly as loadable
as it is today, never a new hard failure mode.
