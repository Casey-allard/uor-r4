# #499 — test(#457): marginal-inconsistency cheap-check — the overlapping blocks DO disagree

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T12:31:50Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-08T12:51:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/499
- **Merged:** 2026-08-08T12:51:13Z
- **Merge commit:** d0de35f9b8d12031cc8701d7b5396bc83cf09e05
- **Base / Head:** main / UOR-Foundation:test/457-marginal-inconsistency
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #457 (**kept open** for the definitive Arm B). The measurement-discipline cheap check before building the IPF operator: is the current scoring marginal-inconsistent *enough* for an IPF consistency operator to matter?

## What
`crates/uor-r4-graph-cli/tests/marginal_inconsistency_457.rs` (`#[ignore]`d). For each held-out position it attributes the position to its active cover regions (`binary_memberships` across depths) and measures whether those co-activated blocks agree on the next-token distribution — pairwise Jensen–Shannon divergence + argmax agreement over the root-mass + residual support — against a shuffled-region null.

## Measured (default cover, 5000 held-out of the 500k fixture) — hypothesis REVERSED
Positions activate a mean of **7.52 regions** (heavy overlap, not nested-only):

| | mean pairwise JS | argmax agreement |
|---|---:|---:|
| real co-activated regions | 0.5511 bits | 0.3468 |
| null random regions | 0.6019 bits | 0.2611 |

The falsifier passes (real regions agree more than random → the co-activation carries real structure), but the **absolute disagreement is large**: a position's ~7.5 blocks concur on the next token only ~35% of the time. So the inconsistency is **not** too small to matter — the hypothesis that overlapping regions concur (making IPF moot) is **refuted**, and an IPF **Arm B is warranted**, not ruled out.

## Why this still isn't a positive (and #457 stays open)
#456 measured the reconstruction that *combines* these regions to be **sub-unigram**, and #459 that k≥3 marginals are **counting noise** at D3 scale. So the disagreement is very likely disagreement among near-noise residuals, which IPF would reconcile into a consistent-but-still-noisy joint rather than into signal. The definitive test is **Arm B** (IPF-consistent reconstruction vs backoff+sum on held-out bits/top-1) — likely negative given #456/#459, but not settled by this cheap check. This PR records the cheap-check data (the exit criteria's marginal-inconsistency deliverable); #457 stays open for Arm B.

fmt / clippy `--all-targets --all-features -D warnings` clean; `#[ignore]`d so no effect on the default test run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

