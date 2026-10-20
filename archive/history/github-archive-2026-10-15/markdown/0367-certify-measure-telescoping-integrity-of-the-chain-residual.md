# #367 — certify: measure telescoping integrity of the chain residual sum (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T04:02:28Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-03T04:20:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/367
- **Merged:** 2026-08-03T04:20:57Z
- **Merge commit:** 6b22fa6bb146102fec3846cefc0ba7125defc4ff
- **Base / Head:** main / UOR-Foundation:graph-residual-calibration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measurement only. **Stacked on #366** — until that lands this PR also shows its
commit; the diff collapses to a single commit once #366 merges.

## Question

The residual is a telescoping sum: each level stores
`log P_level − log P_parent`, so a complete root-to-leaf chain sums to
`log P_leaf − log P_root`, and adding `root_score` recovers `log P_leaf`.
Truncation is per level, so a token kept at a descendant need not be kept at its
ancestors — and a missing level contributes 0 rather than its real correction.
The hypothesis was that broken sums explain why the residual still costs top-1
even after #366 fixed which tokens are kept.

## Result — largely refuted

| graph slice | ratio (shipped) | probability (#366) |
| --- | ---: | ---: |
| mean chain levels | 2.73 | 2.73 |
| mean levels emitting teacher | 0.01 | 1.60 |
| **telescoping COMPLETE** | 0.09% | **54.18%** |
| telescoping PARTIAL (broken sum) | 1.26% | 9.61% |

Partial sums are real but 9.61% — far too few to explain graph top-1 of 2.90%
against 8.76% with the residual disabled. Under corrected selection the sum is
intact for 54% of positions.

## The harder conclusion

Even where the chain is complete and the correction is mathematically right,
applying it makes both metrics worse (top-1 2.90% vs 8.76%; bits +1.32). So on
graph-status positions **`log P_region` is a worse next-token predictor than the
global unigram prior**.

That is not a defect in selection, telescoping, or scoring arithmetic — all now
measured and either fixed or exonerated. It says the region-conditional
distributions do not carry predictive value over the prior.

Most likely cause is **sparsity rather than a bug**: roughly 2,600 positions per
region against a 32,000-token vocabulary makes per-region next-token estimates
dominated by sampling noise, and add-one smoothing does not rescue them. The α
sweep independently agrees, peaking near **α ≈ 0.25** — the shape of a shrinkage
coefficient, not of a sign error (would peak at α < 0) or a plain scale error.

## Direction (not implemented here)

1. shrinkage toward the parent weighted by region evidence count (Witten-Bell /
   Jelinek-Mercer style) — the stored value is already a parent-relative delta,
   so this is a per-region weight, not a format change;
2. fewer/larger regions — trades resolution for estimate quality;
3. a better `Smoothing` variant — already a configurable enum from #67, so cheap
   to ablate.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
certify suite (11 files) green. No behavior change; adds four fields to the
per-status residual-influence block.

Related: #364, #366, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

