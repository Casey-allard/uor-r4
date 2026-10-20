# #549 — feat(#510): R5 tranche — graph-compiler region-lowering to total (46→44)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T16:53:46Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-09T17:10:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/549
- **Merged:** 2026-08-09T17:10:08Z
- **Merge commit:** 9fbe9bad398a1811189fc1007cd25034ccb48c1e
- **Base / Head:** main / UOR-Foundation:r5/gc-lower
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): convert `lower_semantic_regions.rs` off its `Result<_, LoweringError>` surface to total forms. **The module is now fully total (0 sanctioned-violation sites).** The `LoweringError` enum, its `Display`/`Error` impls, and `use std::fmt` are removed.

## Surfaces
- `quantize_q88(f32) -> Option<Self>` — values beyond the Q8.8 range saturate (they are representable, flagged `saturated`), so the only input with no fixed-point image is a NaN, which yields `None`. The old `QuantizationOverflow` path never fired on finite overflow — it only rejected NaN.
- `BooleanLoweringCompiler::lower_region(...) -> Option<(LoweredBooleanRegion, LoweringWitnessEntry)>` — `None` when the region is unrepresentable in the integer runtime: a signature longer than the 64-bit mask (integer bound) or a radius outside the Hamming bound `[0, len]` (float range). The two conditions are of mixed integer/float kind, so they unify as `None` (the absence of a product) rather than a single sanctioned bound type.

## Ripple (same PR)
`tests/bdd.rs` — the `lower_error` World field is dropped for `lower_rejected: bool`; the attempt step sets it on `None`; the assertion checks it. The import drops `LoweringError`.

## Verification
audit-limits graph-compiler **46 → 44** (lower_semantic_regions.rs 0 sites). 86 lib tests, BDD 100/100, proof-model 24 tests, wasm32 + clippy + fmt clean, audit-deferral clean.

Part of the #510 bottom-up sweep (graph-compiler leaf).


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

