# #365 — certify: measure emission selection against likelihood (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T02:58:59Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-03T03:15:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/365
- **Merged:** 2026-08-03T03:15:01Z
- **Merge commit:** 4da1214ed668320c96e6f9ea32f0d4e2ff549e39
- **Base / Head:** main / UOR-Foundation:graph-emission-selection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measures the root cause behind #364. Instrumentation only — no change to
emission selection, scoring, the runtime, or the artifact.

## What it measures

Region emissions are built as a log-ratio against the parent, then truncated:

```rust
let ln = lp_n - lp_p;            // log P_region(token) − log P_parent(token)
residuals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
residuals.truncate(config.emission_entries);   // DEFAULT_EMISSION_ENTRIES = 64
```

Ranking by ratio keeps the tokens most **over-represented** against the parent,
not the most **likely** next tokens. This compares that selection against a
top-E-by-probability one over the same region evidence:

```
regions=38  mean_overlap_with_top_count=0.0078  mean_probability_mass_kept=0.0157  (E=64)
```

- **Overlap 0.78%** — the two selections are nearly disjoint, ~0.5 tokens in
  common out of 64.
- **Mass kept 1.57%** — the emitted list covers 1.6% of what actually happens
  next in that region.

So 98.4% of each region's real next-token mass carries residual 0 and is scored
at the bare global prior, while 64 rare-but-distinctive tokens receive large
positive boosts.

## Why it matters

`root_score + log-ratio = log P_region` is the correct decomposition. The bug is
the **truncation**: dropped tokens silently take ratio 0 rather than their true
correction, and ranking by ratio drops precisely the tokens carrying the mass.

This accounts for every measurement in the #358 / #360 / #363 thread — teacher
present in region emissions 3.8%, 89% of winners on-chain, median teacher rank
63, α = 0 improving both top-1 and bits, graph slice at 17.10 bits/token against
exact_context's 6.83, and 82% of graph-slice retrieval coming from the bare
prior. Components previously suspected and cleared: ranking/calibration, residual
sign and scale, chain selection.

## Scope

Adds `EmissionSelectionStats` and one stderr line at compile time. Selection
behavior is unchanged — fixing it is #364, and should be measured on the graph
slice with bits/token using the per-status instrumentation now on `main`.

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify --all-targets -D warnings`,
and the certify suite (11 files) are green on this base.

Related: #364, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

