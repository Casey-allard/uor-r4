# #548 — feat(#510): R5 tranche — graph-compiler rate-distortion analyzer to total (49→46)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T16:19:37Z
- **Updated:** 2026-10-09T14:17:25Z
- **Closed:** 2026-08-09T16:39:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/548
- **Merged:** 2026-08-09T16:39:26Z
- **Merge commit:** b0a4ee414517b74cc581e457b33ec803727fb2dc
- **Base / Head:** main / UOR-Foundation:r5/gc-rd
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `rate_distortion_compression.rs` off its `Result<_, CompressionAnalysisError>` surface to total forms. **The module is now fully total (0 sanctioned-violation sites).** The `CompressionAnalysisError` enum, its `Display`/`Error` impls, and `use std::fmt` are removed.

## Surfaces
- `analyze_rate_distortion -> Option<RateDistortionReport>` — `None` when `depth_tiers` is empty or contains a zero tier. No report is a product of those tiers, so it is the absence of a product, not a raised error.
- `validate_rate_budget` / `validate_distortion_threshold -> Option<String>` — `None` when every point holds, `Some(reason)` naming the first violating point. A validator is total; the held property is the absence of a violation. `Option<String>` rather than `ObservedBound` because the distortion threshold compares f32 values that `ObservedBound`'s i64 fields cannot carry, and the two validators share one uniform total surface.

## Ripple (same PR)
`tests/bdd.rs` — the `rd_error` World field is dropped for `rd_rejected: bool`; the execute step matches `Some`/`None`; the invalid-tier step asserts the analyzer returned `None`. The import drops `CompressionAnalysisError`.

## Verification
audit-limits graph-compiler **49 → 46** (rate_distortion_compression.rs 0 sites). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

