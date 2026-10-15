# #776 — feat(#655-D2): r4 package-release-bundle CLI command

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T23:34:53Z
- **Updated:** 2026-08-17T23:57:40Z
- **Closed:** 2026-08-17T23:53:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/776
- **Merged:** 2026-08-17T23:53:13Z
- **Merge commit:** b65d027b2bee6e84136f2256c7488ec91d45fecf
- **Base / Head:** main / UOR-Foundation:feat/655-d2-package-cli
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Implements #655-D2 per `docs/serving_release_packaging_655_d.md`: a new
`r4 package-release-bundle` CLI command that calls D1's
`package_release_bundle` (PR #775) and writes its result to
`<compiled>/release-bundle.json` -- the first thing that actually
produces the sidecar C1c's `verify_release_bundle_sidecar` (PR #773)
already knows how to verify.

## Why

D1 built the pure digesting helper but nothing called it. D2 makes it
reachable from a real CLI surface, closing the loop that C1c opened.

## What it does

```
r4 package-release-bundle --compiled <dir> --capability continuation
r4 package-release-bundle --compiled <dir> --capability instruction-chat \
    --source <hf-snapshot-dir> --tokenizer-family hf-byte-bpe --tokenizer-version 1
```

`--capability continuation` packages with an empty `tokenizer_adapter`
(valid -- `ReleaseBundleManifest::validate` only requires a real adapter
family for instruction-chat bundles). `--capability instruction-chat`
requires `--source` (the original HF snapshot dir) plus
`--tokenizer-family`/`--tokenizer-version`, and resolves a *real*
`TokenizerAdapter` via `uor_r4_core::transformerless::hf_bpe`'s
`resolve_source_tokenizer` -- confirmed this session that
`physical_root` alone cannot supply one (a bundle's `tokenizer.bin` is a
different format from the `tokenizer.json`/`spiece.model` a
`TokenizerAdapter` is derived from). Tokenizer identity is required, not
inferred, matching #718's existing policy (the same one `r4 compile`'s
own `--tokenizer-family`/`--tokenizer-version` pair enforces).

`uor-matmul` rev/operation-profile/license are CLI flags defaulting to
the project's standing #655 pin and the `exact-gemm-float`/`MIT`
placeholder values already used throughout this codebase's test
fixtures -- kept overridable rather than hardcoded, since neither has a
real producer yet (D0's finding) and upstream `uor-matmul`'s own
`Cargo.toml` (`Apache-2.0`) and `LICENSE` file (MIT license text)
currently disagree with each other.

## Visibility changes

`release_bundle_loader`/`release_bundle_packager` go from `pub(crate)`
to `pub` in `lib.rs`, since `main.rs` (a separate crate from the library)
now needs to call into `release_bundle_packager::package_release_bundle`
and reference `release_bundle_loader::RELEASE_BUNDLE_SIDECAR_FILE_NAME`
(elevated too, so the writer here and C1c's reader share one filename
constant instead of two copies that could drift). Removed D1's
`#![allow(dead_code)]` from `release_bundle_packager.rs` -- confirmed via
clippy that a `pub` item's dead-code lint is satisfied by potential
external use regardless of whether `main.rs`'s own crate target calls it
(unlike the `pub(crate)` case, where clippy flagged all 12 items
individually with no caller).

## Verification

Full ladder green: `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets --all-features -- -D warnings`, `cargo build
--target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, `cargo test
--workspace` (all passed, zero failures; D1's own 8 unit tests
unaffected by the visibility change).

Manually smoke-tested the built binary against the real local
`smollm2-135m-instruct` bundle: both `--capability continuation` and
`--capability instruction-chat` (`--source` pointed at the matching
`.uor-models/sources/smollm2-135m-instruct` snapshot) produced
structurally valid manifests -- the instruction-chat run's
`tokenizer_adapter` carried a real, non-empty `family`/`tokenizer_cid`,
not a placeholder. Also confirmed both guard-rail error paths (missing
`--source` for instruction-chat; `--source` given without the tokenizer
flags) fail with clear messages and a non-zero exit.

`just vv-register` was not runnable in this environment (`just` not
installed) -- same documented gap as PR #773/#775; CI's own register
conformance job covers it.

## Next

D3: a golden test confirming C1c's `verify_release_bundle_sidecar`
returns `Some(..)` against a bundle this command actually wrote (not
just a hand-built fixture), closing out the loop back to the blocked
C1d.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

