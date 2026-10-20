# #775 — feat(#655-D1): additive release-bundle packaging helper

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T22:45:13Z
- **Updated:** 2026-08-17T23:04:43Z
- **Closed:** 2026-08-17T23:03:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/775
- **Merged:** 2026-08-17T23:03:23Z
- **Merge commit:** d37d71a238f711b54148ed9c1212ffc76f25da16
- **Base / Head:** main / UOR-Foundation:feat/655-d1-package-helper
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Implements #655-D1 per `docs/serving_release_packaging_655_d.md` (D0,
PR #774): a pure, additive helper that builds a `ReleaseBundleManifest`
by digesting an already-compiled R4G1 bundle's real files on disk. No
filesystem write yet, no compile/cover/score stage run or routed
through -- it only reads files the existing CLI pipeline already
produces.

## Why

C1c (PR #773) added a loader that verifies an optional
`release-bundle.json` sidecar, but nothing produces that sidecar yet.
D0 mapped the real bundle layout against `ReleaseBundleManifest`'s
fields; D1 is the first slice that turns that mapping into working code,
without yet wiring it to any CLI surface (that's D2).

## What it does

`package_release_bundle(physical_root, inputs) -> Result<ReleaseBundleManifest, PackageBundleError>`
blake3-hashes `graph/score.r4g1`, `tless_artifacts.bin`, `tokenizer.bin`
(optional), `graph/score_report.json`, and `graph-cover/cover_report.json`
(the `compile_report` mapping D0 decided), fills `abi` from
`AbiVersion::current()`, and takes `model_id`/`capability`/`uor_matmul`/
`tokenizer_adapter`/`provenance_note` as caller-supplied input via
`PackageInputs` -- confirmed via follow-up research this session that
none of those five fields has an on-disk producer in `physical_root`
(notably `tokenizer_adapter`: a bundle's `tokenizer.bin` is a different
format from the HF `tokenizer.json` a `TokenizerAdapter` is derived from,
and deriving one needs the original HF source snapshot, never present in
a compiled bundle directory). The result is validated before returning
-- `Ok` never carries a structurally invalid manifest.

## Why the `#![allow(dead_code)]`

Nothing calls `package_release_bundle` outside its own tests yet (D2 is
the CLI subcommand that will). Every item in the module is therefore
unreachable from any live root in a non-test build, which clippy
correctly flags -- confirmed empirically (12 `-D warnings` errors without
the allow, one per item, since dead-code reachability doesn't propagate
through an already-dead caller). A single module-level allow with a
comment was more honest than a dozen per-item ones. This mirrors how
`release_bundle.rs` (#655-C0) and `release_bundle_loader.rs` (#655-C1c)
each landed before their own consumer existed.

## Verification

Full ladder run locally in the worktree, all green: `cargo fmt --all --
--check`, `cargo clippy --workspace --all-targets --all-features --
-D warnings`, `cargo build --target wasm32-unknown-unknown -p
uor-r4-wasm-router --lib`, `cargo test --workspace` (zero failures).
8 unit tests (full bundle, absent optional tokenizer, each of the four
required files missing individually, invalid caller input, and
`InstructionChat` capability correctly rejected without a real tokenizer
adapter). One additional `#[ignore]`d, environment-gated test
(`UOR_R4_RELEASE_BUNDLE_PATH`, mirrors `crates/uor-r4-api/tests/api.rs`'s
own `UOR_R4_API_E2E_SOURCE` convention) run manually against the real
local bundle at `.uor-models/compiled/smollm2-135m-instruct/` -- passed,
confirming this isn't just synthetic-fixture-shaped code.

`just vv-register` was not runnable in this environment (`just` not
installed) -- same documented gap as PR #773; CI's own register
conformance job covers it.

## Next

Follow-up PR: D2 (a thin CLI subcommand that calls this helper and
writes `release-bundle.json` next to `physical_root`), then D3 (a golden
test confirming C1c's `verify_release_bundle_sidecar` returns `Some`
end-to-end against a real packaged bundle).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

