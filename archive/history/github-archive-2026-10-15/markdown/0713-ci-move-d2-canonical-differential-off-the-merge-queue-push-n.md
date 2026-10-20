# #713 — ci: move D2 canonical-differential off the merge queue (push + nightly) + cache the compile

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T17:44:11Z
- **Updated:** 2026-08-14T18:04:53Z
- **Closed:** 2026-08-14T18:04:53Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/713
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem
The D2 canonical-differential acceptance jobs (`d2-differential` matrix + `d2-differential-compare`) download SmolLM2-135M and run `r4 compile --target 1000` on Linux + macOS, then compare every emitted byte for cross-OS determinism. They run on `merge_group` (and push), so **every merge-queue speculative build pays the full corpus compile** — and the queue builds up to `max_entries_to_build: 5` in parallel.

Since **#703 (B2, the uor-matmul exact-GEMM teacher migration)** the teacher runs correctly-rounded scalar GEMM instead of hardware BLAS, so that compile went from ~16 min (#699/#701) to **~50–58 min and climbing** (#703 49m, #706 55m, #708 58m). D2 is **not a required check** (required set: `fmt / clippy / tests / no_std / κ`, `cargo audit`, `fuzz smoke`, `wasm-pack`, `Gate C trend`), so it never gated a merge — it just burns ~an hour of runner time on every queued PR.

## Fix (approved)
- Run the D2 jobs on **push-to-main (post-merge) + a nightly cron (07:00 UTC) + workflow_dispatch**, NOT on `merge_group` or `pull_request`. Every merge is fast again; determinism is still verified on `main` within ~an hour of each merge (revertable) and nightly.
- Add a conservative `actions/cache` of the D2 manifest, keyed on `hashFiles('Cargo.lock', 'src/**/*.rs', 'crates/**/*.rs', 'scripts/d2_differential_compile.sh')`, so an unchanged compile path skips the corpus compile. The key is deliberately broad (any first-party Rust change recompiles) — a narrower compile-path-scoped key is unsafe because the teacher/compiler and unrelated code share crates and Rust builds are not byte-reproducible, so it could silently mask a real determinism regression. The nightly run bypasses the cache to re-verify against runner/toolchain drift.

## Notes
- Trade-off: determinism moves from pre-merge to post-merge detection on `main` (a rarely-regressing, easily-reverted property).
- Cross-OS byte equality is still asserted by the compare job on the restored manifests.
- actionlint + YAML validated. PR incoming.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

