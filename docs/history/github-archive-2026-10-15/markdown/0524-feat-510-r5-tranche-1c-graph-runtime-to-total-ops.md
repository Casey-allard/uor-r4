# #524 — feat(#510): R5 tranche 1c — graph-runtime to total ops

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T02:32:41Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-09T02:57:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/524
- **Merged:** 2026-08-09T02:57:44Z
- **Merge commit:** 56c284c60fbb7bb14df1581aeb14ca74a57807d0
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t1c-graph-runtime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Third R5 tranche (issue #510): converts `uor-r4-graph-runtime` to the sanctioned error surface. Continues the bottom-up march after graph-format (#518, #520, #521, #522, #523).

## What changes
The crate's private `RuntimeError` / `AccumulatorError` enums are removed. Every function either becomes **total** or narrows to a **sanctioned** type (`NotAProduct` / `ObservedBound` / `KappaError`):

- `engine::R4G1Runtime::parse` → `Result<Self, NotAProduct>` (was `FormatError`; now propagates the sanctioned parse verdict directly).
- `engine::R4G1Runtime::try_push_patch` → `Option<&'static str>` (None = pushed, Some = rejection reason; a non-parsing patch folds to a rejection reason).
- `engine::R4G1Runtime::step` → `(u32, ResolutionStatus)` — the `Result` was never `Err`; an empty graph resolves to `(0, BackedOff)`.
- `patch_chain::PatchChain::try_push_patch` → `Option<&'static str>`.
- `routing::evaluate_route` → `Option<&'a [u8]>` (None = malformed ROUT program: absent head, out-of-bounds field, or unknown opcode).
- `scoring::accumulate_reference` → `Option<ScoreQ>` (None = duplicate evidence id); `AccumulatorError` removed.
- Dormant packed kernels (`evaluate_routing_program`, `evaluate_typed_transition`, `evaluate_no_alloc_predict_step`) → total `u32`.

## Dormant mechanisms preserved
Per the preserve-and-gate policy (#515), no mechanism is deleted. The `packed-routing-dormant` markers and their `open` ledger claims are kept intact for later activation; only the arbitrary `max_steps == 0` guard (an unsanctioned limit) is dropped.

## Verification
- `cargo build`/`test`/`clippy`/`--no-default-features` (no_std) all green for the crate and workspace.
- `xtask audit-deferral` clean; graph-runtime no longer appears in the `audit-limits` inventory.
- No external code depended on `RuntimeError` (only a re-export and a certify test comment).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

