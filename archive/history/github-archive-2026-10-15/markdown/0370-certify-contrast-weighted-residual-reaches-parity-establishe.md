# #370 — certify: contrast-weighted residual — reaches parity, establishes the ceiling (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T05:54:20Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-03T06:24:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/370
- **Merged:** 2026-08-03T06:24:26Z
- **Merge commit:** 9c4b24b4f4493e34b56751ed7fd3f7d8526b6a7e
- **Base / Head:** main / UOR-Foundation:graph-contrast-weighting
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Direction 2 from #364, plus the measurement that closes the scoring-side
investigation. **Stacked on #366/#367/#369.**

## Contrast weighting works

Weights each region's residual by `1 − overlap(region top-E, global prior top-E)`
— a region whose most-likely tokens *are* the prior's conditions on nothing.
Distinct from the evidence-count shrinkage in #369, which measured λ = 0.9992 and
was never the binding constraint.

| variant | graph top-1 | graph bits |
| --- | ---: | ---: |
| probability + none | 2.90% | 13.32 |
| probability + witten-bell | 2.90% | 13.31 |
| **probability + contrast** | **5.42%** | **12.01** |
| **root-only baseline** | **8.76%** | **12.00** |

Nearly doubles top-1 and **closes the bits gap entirely** (12.01 vs 12.00). It
also beats the equivalent global constant (α = 0.5 → 5.26%), so per-region
contrast carries real variance.

## …and establishes the ceiling

It does not clear the bar. Conditioning is now **break-even on coherence and
still negative on top-1**. At this cover's specialization, the best geometric
emission conditioning achieves is **parity with not conditioning at all**.

## Why: the cover is uniformly semi-generic

```
contrast  mean=0.5012  min=0.3906  max=0.7500   (38 regions)
```

Every region shares 25–61% of its top-E with the global prior. Even the most
distinctive shares a quarter; none approaches high specialization.

**The architectural statement:** the cover is induced by context-bundle geometry,
but predictive value requires grouping by *continuation* — and the two are only
weakly aligned (1.41× own-region lift, #369). Tuning granularity does not address
that; it needs an induction objective optimizing continuation discriminativeness
rather than signature-space compactness.

## What this closes

Every scoring-side component is now measured:

| component | verdict |
| --- | --- |
| emission selection | defect, fixed (#366) |
| ranking / calibration | exonerated |
| residual sign, scale | exonerated |
| chain selection | exonerated |
| telescoping | intact |
| estimate sparsity | refuted (#369) |
| conditioning weight | **optimized — parity, not gain** |

The remaining lever is region construction, upstream of the scorer entirely.

## Bearing on other work

- **#362**: compare trigram rows against a geometric path whose realistic
  contribution is parity with a unigram prior — not against zero, not against the
  shipped 1.96%.
- **#290 / #359 / #361**: those optimize emission-residual machinery now measured
  at break-even on the graph slice. The speedups are real; worth knowing what
  they accelerate.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify -p uor-r4-graph-cli
--all-targets -D warnings`, certify suite (11 files) green. `--emission-shrinkage`
defaults to `none`, preserving shipped behavior byte-exactly.

Related: #364, #366, #367, #369, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5169569617 — 2026-08-03T17:22:20Z — auser

**Correction to this PR's body.** It states the cover is "induced by context-bundle geometry" and that fixing the weak alignment "would need an induction objective optimizing continuation discriminativeness." That is wrong — `induction.rs` already gates splits on next-token entropy reduction (`entropy_gain_bits`, default 0.25) and scores an information-bottleneck term including `I(Z;X) − β·I(Z;Y_future)`.

The corrected statement: candidate partitions come from geometry, and continuation structure only *vetoes* them — entropy gain is an acceptance filter, not the maximand. So the cover captures continuation structure only where it aligns with signature-space geometry. All measurements in this PR stand; the explanation of the cause does not. Full correction on #364.


---
_Comment/review ids in this file: 5169569617_

