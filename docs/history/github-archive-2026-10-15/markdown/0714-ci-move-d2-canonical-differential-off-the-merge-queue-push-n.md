# #714 — ci: move D2 canonical-differential off the merge queue (push + nightly) + cache

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T17:45:18Z
- **Updated:** 2026-10-09T13:08:41Z
- **Closed:** 2026-08-14T18:04:52Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/714
- **Merged:** 2026-08-14T18:04:52Z
- **Merge commit:** 97abea369e30cdebbe19a7b68df4fa1b6e7af2d5
- **Base / Head:** main / UOR-Foundation:ci-d2-off-merge-queue
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #713.

The D2 acceptance jobs (`d2-differential` matrix + `d2-differential-compare`) download SmolLM2-135M and run `r4 compile --target 1000` on Linux + macOS, then byte-compare for cross-OS determinism. They ran on `merge_group`, so **every merge-queue speculative build paid the full corpus compile** (queue builds up to 5 in parallel). Since #703 (the uor-matmul exact-GEMM teacher) that compile is ~50–58 min and climbing. D2 is **not a required check** — it never gated a merge, it just burned ~an hour of runner time per queued PR.

## Change
- **Move D2 off the merge queue**: `d2-differential` + `d2-differential-compare` now run on **push-to-main + a nightly cron (07:00 UTC) + workflow_dispatch**, not on `merge_group`/`pull_request`. Every merge is fast again; determinism is still verified on `main` within ~1h of each merge (revertable) and nightly.
- **Cache the D2 manifest**, keyed conservatively on `hashFiles('Cargo.lock', 'src/**/*.rs', 'crates/**/*.rs', 'scripts/d2_differential_compile.sh')`, so an unchanged compile path skips the corpus compile. Broad key on purpose — a narrower, compile-path-scoped key is unsafe (the teacher/compiler and unrelated code share crates, and Rust builds aren't byte-reproducible, so it could silently mask a determinism regression). The nightly run bypasses the cache to re-verify against runner/toolchain drift.

## Verification / notes
- YAML parse + `actionlint` clean (0 errors).
- D2 is not a required status check (required set: `fmt / clippy / tests / no_std / κ`, `cargo audit`, `fuzz smoke`, `wasm-pack`, `Gate C trend`), so removing it from the queue cannot wedge merges.
- This PR does not exercise D2 on its own PR/queue run (D2 no longer runs there) — expected; its required checks run and gate as usual.
- Trade-off: determinism detection moves pre-merge → post-merge on `main`, for a rarely-regressing, easily-reverted property.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

