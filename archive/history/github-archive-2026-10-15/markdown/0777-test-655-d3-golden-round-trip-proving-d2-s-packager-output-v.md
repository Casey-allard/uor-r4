# #777 — test(#655-D3): golden round-trip proving D2's packager output verifies

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T00:46:41Z
- **Updated:** 2026-08-18T01:04:22Z
- **Closed:** 2026-08-18T01:03:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/777
- **Merged:** 2026-08-18T01:03:29Z
- **Merge commit:** 9468efb03c78ee8807324289249268a6b2695e15
- **Base / Head:** main / UOR-Foundation:feat/655-d3-golden-e2e-test
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Closes the loop that #655-D0/D1/D2 and #655-C1c left open: `package_release_bundle`
(D1) and `verify_release_bundle_sidecar` (C1c) were each independently
well-tested against synthetic fixtures, but nothing proved the two halves
actually compose against the exact bytes and sidecar path #655-D2's CLI
command produces. This PR adds exactly that -- a golden round-trip test,
no production code changes.

## Tests added (`src/release_bundle_packager.rs`)

- `packaged_bundle_is_accepted_by_the_loaders_sidecar_verifier` --
  packages a `Continuation` bundle, writes `release-bundle.json` the same
  way `package_release_bundle_command` (`src/main.rs`) does
  (`serde_json::to_vec_pretty` to
  `release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME`), and asserts
  `verify_release_bundle_sidecar` returns `Some` of that exact manifest.
- `instruction_chat_bundle_with_real_tokenizer_adapter_round_trips_through_the_loader`
  -- same round trip for an `InstructionChat` bundle carrying a real
  (non-default) `tokenizer_adapter`, mirroring D2's `--source` +
  `--tokenizer-family`/`--tokenizer-version` path rather than only the
  simpler default-adapter case.

## Why these tests live here, not in `tests/`

`verify_release_bundle_sidecar` is `pub(crate)` (D2 only elevated the
sidecar file-name constant and the packaging path to `pub` -- main.rs
writes a sidecar, it never reads one back). An integration test under
`tests/` compiles as a separate crate and can only see `pub` items, so
it cannot call the verifier. These tests live inside
`release_bundle_packager.rs`'s own `#[cfg(test)]` module instead, which
also gives them access to that module's private relative-path constants
(`GRAPH_RELATIVE_PATH` etc.) as the single source of truth for the
on-disk layout, rather than re-hardcoding paths that could drift.

## Verification ladder (all green, run locally on this branch)

- `cargo fmt --check`
- `cargo clippy -p uor-r4-wasm-router --all-targets --all-features -- -D warnings`
- `cargo test -p uor-r4-wasm-router --lib release_bundle_packager::` -- 10/10 passed, 1 ignored (env-gated)
- `UOR_R4_RELEASE_BUNDLE_PATH=.../smollm2-135m-instruct cargo test ... -- --ignored packages_the_real_local_bundle` -- passed
- `cargo build -p uor-r4-wasm-router --lib --target wasm32-unknown-unknown` -- clean
- `cargo test --workspace` -- 154 suites, 0 failures
- Register conformance gate (R1-R6): `check-model`, `repo-conformance`, `audit-deferral`, `audit-limits`, `cargo deny check` -- all passed

## Next

With D0-D3 done, #655-D (packaging) is complete. The next natural slice
is **#655-C1d**: surface `verified` on `/v1/models` and
`/uor/v1/status`, now that there's a proven, tested path from a real
compiled bundle to a sidecar the server-side loader actually accepts.
No default-engine change here or in C1d -- #655-F still requires
Casey's explicit sign-off per the issue's standing decisions.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

