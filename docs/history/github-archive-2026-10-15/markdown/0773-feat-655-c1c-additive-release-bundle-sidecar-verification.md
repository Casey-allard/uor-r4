# #773 — feat(#655-C1c): additive release-bundle sidecar verification

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T21:04:45Z
- **Updated:** 2026-08-17T21:30:16Z
- **Closed:** 2026-08-17T21:25:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/773
- **Merged:** 2026-08-17T21:25:40Z
- **Merge commit:** 71d77101cdb4ed1c5be8cbf148c552bd5ccd78b1
- **Base / Head:** main / UOR-Foundation:feat/655-c1c-sidecar-verify
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Implements **C1c** from `docs/serving_shared_loader_655_c1.md` (#655-C1b,
PR #772): a new `src/release_bundle_loader.rs` module that parses and
verifies an optional `release-bundle.json` sidecar next to a resolved R4G1
bundle directory, checking the sidecar's declared `components.graph` and
`components.signature_artifact` blake3 digests against the bundle's actual
graph/teacher files on disk.

Wired into `resolve_loadable_compiled_bundle_with_authority` via a new
`ResolvedCompiledBundle.release_bundle: Option<ReleaseBundleManifest>`
field.

## Why this is safe to merge now

Strictly additive, per the design doc's Q3. A missing sidecar, a
parse/structural-validation failure, or a digest mismatch all resolve to
`None` — exactly today's behavior for **every bundle that exists today**,
since nothing produces a sidecar yet (#655-D packaging is still open).
Which bundle loads, and whether it loads, is unchanged either way. Unit
tests exercise all of these paths against synthetic fixtures.

## Scope

Verifies exactly the two components this resolution already has file
paths for (`graph`, `teacher`/`signature_artifact`). `score_report`,
`compile_report`, and `tokenizer` digests are structurally validated by
`ReleaseBundleManifest::validate()` but not re-hashed against disk — there
is no established path convention for locating those files independently
of a packaging step that doesn't exist yet. Documented as a gap for a
future #655-D/C1d slice, not silently skipped.

## Incidental change

Dropped `Eq` from `ResolvedCompiledBundle`'s derive (kept `PartialEq`),
since `ReleaseBundleManifest` doesn't implement `Eq`. One test-only enum
(`ConfiguredManagedBundle`) held a `Box<ResolvedCompiledBundle>` and also
derived `Eq`; adjusted the same way. Confirmed nothing in the codebase
depends on `Eq` for either type.

## Verification

- `cargo build --workspace` — clean
- `cargo fmt --check` — clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean
- `cargo test --workspace --offline` — all pass, 0 failures (7 new tests in
  `release_bundle_loader::tests`)
- `just vv-register` — not runnable in this environment (`just` not
  installed); no registry/model/deferral surface touched by this change,
  and CI's merge queue runs it regardless.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

