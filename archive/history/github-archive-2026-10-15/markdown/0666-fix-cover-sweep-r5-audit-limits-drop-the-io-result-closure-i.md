# #666 — fix(cover-sweep): R5 audit-limits — drop the io::Result closure in checkpoint save (#613 follow-up)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T16:41:30Z
- **Updated:** 2026-10-09T14:17:06Z
- **Closed:** 2026-08-13T17:01:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/666
- **Merged:** 2026-08-13T17:01:43Z
- **Merge commit:** cbd7207f8a1fbf153086192b0d22861538fee0f7
- **Base / Head:** main / UOR-Foundation:i613-r5-followup
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
#613 (PR #665) merged with `save_point_checkpoint` wrapping its I/O in a `(|| -> std::io::Result<()> { … })()` closure. The **R5** register-conformance gate (`xtask audit-limits`) forbids a shipped signature returning an unsanctioned error type — only `NotAProduct` / `ObservedBound` / `KappaError` / `SourceUnavailable` (plus serde/framework carve-outs) are allowed — so `io::Result<()>` trips it. That gate now **fails the register conformance job on `main`**, so this is a fast follow-up to unbreak it.

## What changed
Rewrote the persistence as inline best-effort steps: each fallible call (`to_vec_pretty`, `create_dir_all`, `write`, atomic `rename`) warns and returns on error, with **no `Result`-typed helper**. Behavior is unchanged — same tmp-file + atomic-rename, same warn-and-continue on failure.

## Why it slipped through
The register conformance job runs R1/R2/R3/**R5**/R6, but my local gate set had been running R1/R4/R2-R3/R6 and omitted **R5 `audit-limits`**. Added to the checklist going forward.

## Verification (local, matches the register conformance job)
- **R5** `cargo run -q -p xtask -- audit-limits`: *no shipped crate returns an unsanctioned error*.
- `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, and the `cover_sweep` unit tests (7 passed, incl. the checkpoint round-trip / mismatch / atomic-write guards) all green.

Refs #613

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

