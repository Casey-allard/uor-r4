# #1769 — mqar-bench: dump_scores=1 — read-score instrumentation (T2 diagnostic)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T00:28:49Z
- **Updated:** 2026-10-09T13:22:39Z
- **Closed:** 2026-10-06T00:29:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1769
- **Merged:** 2026-10-06T00:29:55Z
- **Merge commit:** 88693553aecbc33f3d2a1514c2b4a872676d8b3b
- **Base / Head:** main / UOR-Foundation:deepseek/t2-argmax
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Opt-in instrumentation for the pre-registered T2 diagnostic: is the gold source the argmax of the read's score vector?

Adds `dump_scores=1` and `StackModel::read_weight_rows`, which appends an identity auxiliary value block to the read's value matrix — the mechanism the existing label-mask/span probes use — so output channel j of row t is exactly the read's own softmax weight on source j, sharing its scores, age and NoRead normalisation, removed before read.out. The optional dump runs AFTER the observed forward and is compared against it.

**Result (recorded, not changed here):** gold is the argmax in **1.14%** of last-layer rows, 2.92% pooled, 0.00% with NoRead competing. NoRead 0.60 and the largest entry in 99.3% of rows. The score is a smooth near-linear function of distance with **no content-matching spike at any head** — the read is a distance meter, not a content matcher.

**Safety:** default path byte-identical and runs no new code. Validated by run B's final log line being verbatim identical to sealed `ro-full-base`, logits bit-identical (max gap 0.0), dump-derived accuracy equal to the reported metric (0.03125), 14/14 example tests.

Roots: `reports/t2-argmax-20261005-1/-2` (newly claimed and sealed; no sealed root reused).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

