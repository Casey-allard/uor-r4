# #114 — revert(scorer): remove zero-information scoring variants (#102)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-23T16:23:01Z
- **Updated:** 2026-10-09T14:17:11Z
- **Closed:** 2026-07-23T16:33:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/114
- **Merged:** 2026-07-23T16:33:00Z
- **Merge commit:** 251d24bc406984c39fdbcc98331025f268003372
- **Base / Head:** main / UOR-Foundation:issue-102-remove-scoring-variants
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #102

## Decision: remove (the revert path)

The issue asked to either land the `662865f` revert adapted to current main, or justify keeping the variants with corrected wiring. Verification settled it:

- **Fake metric and CLI were already gone.** `610466b` (landed via #100) removed `rule12_interaction_residuals` and the `--scoring-variant` CLI. What remained was the `ScoringVariant` machinery (#80's `CloudSizeNormalized` / `MarginWeighted`).
- **The variants are zero-information.** Gate C on the checked-in fixture: `rule12_cloud_size_normalized` and `rule12_margin_weighted` are bit-identical to `rule12_precedence` (top-1 0.317086, 9.8612 bits/token). This is structural, not a wiring bug: 100% of held-out positions on every measured corpus are ExactContext status, and EXCT precedence dominates candidate scoring, so a residual-transform variant can never change the outcome. The columns cost two extra `GraphScorer` constructions + full scoring passes per evaluation and invite the "fake metric" criticism while measuring nothing.

## What changed

- `crates/uor-r4-core/src/transformerless/score_variant.rs` deleted; `pub mod score_variant` removed.
- `score_runtime.rs`: `ScoringVariant` re-export, struct field, setter/getter, `variant_residual`, and both call sites removed — chain emission residuals are added directly (exactly the `ChainTelescoped` behavior the deployed scorer already used).
- `score.rs`: variant scorers, accumulators, `GateCOutcome` rows removed; report **schema 5 → 6** with the history line recording why.
- `tests/score.rs`: variant JSON assertions removed; schema assertion bumped.

## Verification

- Fixture Gate C: schema 6, variant rows absent, `rule12_precedence` unchanged (0.317086 / 9.8612), `tla3_baseline` 11.8781
- `scripts/gate_c_trend.sh`: ok
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
- `cargo fmt --check`: clean
- `cargo test --workspace --offline`: green

`grep -r "ScoringVariant\|score_variant\|cloud_size_normalized\|margin_weighted" crates/ src/ tests/` → no code hits (only the schema-history comment). #80 is cross-referenced; its record already says "evaluated, trigger not fired" — this PR removes the now-empty measurement surface.

Refs #80, #99, #101


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

