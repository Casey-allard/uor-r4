# #369 — certify: refute sparsity explanation; control for cover predictiveness (#364)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T04:51:21Z
- **Updated:** 2026-10-09T13:09:09Z
- **Closed:** 2026-08-03T06:04:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/369
- **Merged:** 2026-08-03T06:04:38Z
- **Merge commit:** 9928abbb12dfb0bb83cc212f01a54dde6296cc0e
- **Base / Head:** main / UOR-Foundation:graph-cover-predictive
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two measurements closing out #364's diagnosis. **Stacked on #366/#367** — the
diff collapses once those land.

## 1. Shrinkage refutes the sparsity explanation (`8ca83a2`)

Added opt-in per-region shrinkage (`--emission-shrinkage witten-bell`, scaling
each region's residual by `n/(n+T)` at compile time) to test whether the
residual's remaining harm is sparse-estimate noise.

| variant | graph top-1 | graph bits |
| --- | ---: | ---: |
| probability + none | 2.90% | 13.32 |
| probability + witten-bell | 2.90% | 13.31 |
| **root-only baseline** | **8.76%** | **12.00** |

It changed nothing, because λ = 0.9992 — and the reason refutes the hypothesis:

```
regions=38  mean_n=3,105,750  mean_types=2,198
```

Regions average **3.1 million** next-token observations over ~2,200 types.
Densely estimated, not noisy. The "~2,600 positions per region" figure I put in
#364 and #367 was wrong — taken from the Gate C held-out count rather than the
training corpus. Withdrawn.

## 2. The cover IS predictive — 1.41× (`387a20c`)

Control: does a context's **own** region emit its actual continuation more often
than an **unrelated** one? The unrelated node is chosen by a fixed integer hash of
the position index — deterministic, no RNG, independent of the context's
geometry.

| selection | own region | unrelated region | **lift** |
| --- | ---: | ---: | ---: |
| ratio (shipped) | 0.74% | 0.48% | 1.55× |
| **probability (#366)** | **60.36%** | **42.69%** | **1.41×** |

**The geometric premise holds.** Routing carries real next-token information.

But the 42.69% baseline is the key figure: an unrelated region's top-64 contains
the teacher nearly half the time. Emission lists are largely the same
globally-common tokens across regions, so specialization is thin.

## What this closes

The regions are **not mis-estimated** (3.1M observations each), **not
meaningless** (1.41× lift), but **mostly generic with a thin specialized
margin** — and the scorer applies that margin as though it were a full
correction.

That accounts for every result in the thread: why no positive α helps, why
`log P_region` loses to the unigram prior despite abundant data, why shrinkage by
evidence count was beside the point (estimates are fine; *contrast* is weak), and
why fixing emission selection improved the candidate set without making the
residual helpful.

## Direction

The lever is **contrast**, not estimation — weight conditioning by how far a
region's distribution actually departs from the global prior, rather than
uniformly. That is a different quantity from the evidence-count shrinkage tested
here and is the natural next experiment.

## Verification

`cargo fmt`, `cargo clippy -p uor-r4-graph-certify -p uor-r4-graph-cli
--all-targets -D warnings`, certify suite (11 files) green. Both additions are
measurement-only; `--emission-shrinkage` defaults to `none`, preserving shipped
behavior byte-exactly.

Related: #364, #366, #367, #362.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

