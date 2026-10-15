# #778 — feat(#655-C1d): surface verified on /v1/models and /uor/v1/status

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T01:38:47Z
- **Updated:** 2026-08-18T02:00:44Z
- **Closed:** 2026-08-18T02:00:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/778
- **Merged:** 2026-08-18T02:00:05Z
- **Merge commit:** 888e8a39aba02d7b69acbb9c8566e38087ea26f6
- **Base / Head:** main / UOR-Foundation:feat/655-c1d-verified-status
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Surfaces a `verified` signal on `GET /v1/models` and `GET /uor/v1/status`,
per the C1 design doc's own stated sequence
(`docs/serving_shared_loader_655_c1.md`, "C1d ... surface `verified` on
`/v1/models` and `/uor/v1/status`; confirm end-to-end"). This was blocked
until #655-D3 (merged, PR #777) proved a real D2-produced sidecar is
actually accepted by C1c's `verify_release_bundle_sidecar` -- it is now
unblocked.

## Scoping research

Before writing code, a subagent mapped the current state of both
handlers (route registration, response builders, whether
`ResolvedCompiledBundle.release_bundle` -- populated since #655-C1c --
was already reachable). Key findings that shaped this PR:

- **Verification is already eager/cached, not lazy.**
  `verify_release_bundle_sidecar` does real file I/O and blake3-hashes
  the entire graph + teacher/signature-artifact files. That work already
  happens exactly once per bundle load/reload
  (`resolve_loadable_compiled_bundle_with_authority`), not per HTTP
  request. This PR only reads the already-cached
  `ResolvedCompiledBundle.release_bundle` field -- it never calls
  `verify_release_bundle_sidecar` from a request handler.
- `/uor/v1/status` already had direct access to `active_bundle` inline;
  `/v1/models` did not -- `active_models` discarded the resolved bundle
  entirely, returning a bare `(String, u64)`. Plumbing was required
  through `active_models`, `openai_model_object`, and `models_list_body`.
- A sibling status surface, `/api/r4g1/status`
  (`R4g1CompileStatus::json`), mirrors some of the same `active_bundle`
  fields and has a parity test against `/uor/v1/status`
  (`both_status_surfaces_report_the_exact_optional_source_execution_pair`).
  Confirmed that test asserts specific shared fields, not exhaustive JSON
  equality -- adding `verified` only to `/uor/v1/status` and `/v1/models`
  (the two surfaces C1d's own scope names) does not break it, and does
  not touch `/api/r4g1/status`.
- The field name `verified` was effectively pre-decided by the C1b design
  doc itself, so this wasn't a fresh naming decision.

## Changes (`src/server.rs`)

- New shared helper `active_bundle_verified(Option<&ResolvedCompiledBundle>) -> bool`,
  used by both endpoints so they cannot silently drift from each other.
  `false` covers every negative case uniformly (no active bundle, no
  sidecar, digest mismatch) -- matches the existing readiness-boolean
  convention (`r4g1_ready`, `teacher_ready`, ...) rather than the
  `Option<&str>` "nothing to report" convention used for error fields.
- `uor_status_json` gains one new `"verified"` key.
- `active_models`/`loadable_models_in` now return `(String, u64, bool)`;
  `openai_model_object` takes a `verified: bool` and adds it to the JSON
  object as an additive field (not part of the pinned OpenAI schema's
  required fields, so conforming clients ignore it); `models_list_body`
  threads it through.
- Deliberately does **not** touch `/api/r4g1/status` -- out of C1d's
  stated scope.

## Tests

- `active_bundle_verified_reflects_the_cached_release_bundle_state` --
  direct unit coverage of the shared helper (no active bundle / bundle
  without a release_bundle / bundle with one).
- `uor_status_json_reports_verified_from_the_active_bundles_release_bundle`
  -- same three cases through the real JSON builder.
- `openai_model_object_carries_every_required_field` (updated) +
  `openai_model_object_verified_field_reflects_the_caller_supplied_flag`
  (new).
- **Golden path**, mirroring #655-D3's own style:
  `loadable_models_reports_verified_true_for_a_bundle_with_a_matching_sidecar`
  writes a real `release-bundle.json` sidecar with digests computed from
  real on-disk bytes (the same way #655-D2's CLI command does) and
  confirms `verified: true` flows all the way through
  `resolve_loadable_compiled_bundle_with_authority` ->
  `verify_release_bundle_sidecar` -> `active_bundle_verified` ->
  `loadable_models_in` -> `models_list_body`'s JSON -- not just true by
  construction.
- Existing tests updated for the new 3-tuple shape; the sorted-inventory
  test gained assertions that synthetic bundles without a sidecar report
  `verified: false` at both the tuple and JSON layers.

## Verification ladder (all green, run locally on this branch)

- `cargo fmt --check`
- `cargo clippy -p uor-r4-wasm-router --all-targets --all-features -- -D warnings`
- Targeted `server::` tests (verified/openai_model/loadable_models): 10/10 passed
- `cargo build -p uor-r4-wasm-router --lib --target wasm32-unknown-unknown` -- clean
- `cargo test --workspace` -- 154 suites, 0 failures
- Register conformance gate (R1-R6): `check-model`, `repo-conformance`, `audit-deferral`, `audit-limits`, `cargo deny check` -- all passed

## Next

With C1a/C1b/C1c/C1d and D0-D3 all done, #655's C track is complete
except **C1e** (`ModelStore`/cascade reconciliation, explicitly deferred
to its own future issue per the C1b design doc). The remaining open
sub-tracks are **E** (Production/Experimental engine profiles) and **F**
(default-engine flip, which still requires Casey's explicit sign-off --
not touched here or anywhere in this PR).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

