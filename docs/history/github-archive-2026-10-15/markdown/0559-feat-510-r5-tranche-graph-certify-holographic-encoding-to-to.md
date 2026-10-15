# #559 — feat(#510): R5 tranche — graph-certify holographic encoding to total (44->40)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T22:58:04Z
- **Updated:** 2026-10-09T14:17:24Z
- **Closed:** 2026-08-09T23:17:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/559
- **Merged:** 2026-08-09T23:17:01Z
- **Merge commit:** aa3da047f16be28a84a314f4fa6f7c873111c121
- **Base / Head:** main / UOR-Foundation:r5/certify-holo
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

R5 rearchitecture (#510): third graph-certify tranche. Convert `holographic_encoding.rs` off `Result<_, DegeneracyError>`, dropping the `DegeneracyError` enum entirely. **The module is now fully sanctioned (0 audit sites).**

## Surfaces
- `validate_projection_family -> Option<String>` — a validator. `None` when the projection family is non-degenerate; `Some(reason)` naming the first degeneracy (empty set, single-node memorization, duplicate id/projection, empty/invalid recovery distribution, inconsistent distribution length). The held property is the absence of a degeneracy; the message preserves *which* one so callers keep their diagnostics.
- `recover_behavior_distribution -> Option<Vec<f64>>` — `None` when the request is not a product of these projections (unknown/duplicate active id, or the projections disagree on distribution length).
- `progressive_fidelity -> Option<Vec<ProgressiveFidelityPoint>>` and `ablation_curve -> Option<Vec<AblationPoint>>` — propagate the recovery `Option` via `?`, and `None` on an ablation id not in the active set.

## Ripple
All callers are the crate's `holographic_encoding_test`. The key flip: `validate_projection_family`'s old `Ok(())` = valid became `None` = valid, so the success test asserts `.is_none()` instead of `.expect()`; the five degeneracy tests assert `.is_some_and(reason contains keyword)`; the two propagation tests assert `.is_none()` instead of `expect_err`/`matches!`.

## Verification
audit-limits graph-certify **44 → 40** (holographic_encoding.rs now 0 sites). Full workspace build + wasm32 graph-compiler + clippy + fmt clean; graph-certify suite green (holographic test 6/6), BDD 100/100, audit-deferral clean.

Part of the #510 bottom-up sweep — graph-certify.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

