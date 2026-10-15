# #391 — feat(certify): #390 — null-adjusted generalization rows (schema 17)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T20:07:10Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-08-03T20:21:19Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/391
- **Merged:** 2026-08-03T20:21:19Z
- **Merge commit:** 49b13072aae56f6efd11ee7c4ac7580f5ca968c3
- **Base / Head:** main / UOR-Foundation:i390-null-rows
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements #390. Report-only (no scoring/κ change): `gate_c.rule12_generalization` (the EXCT-free graph+novel headline) and `gate_c.nulls` (train-unigram analytic baselines for the full and generalization slices, with train/held-out counts). Residuals now read as **gaps against a null**, and cross-cover comparisons carry their granularity confound explicitly.

Validated on the fixture (worktree-isolated build):

| row | pos | top-1 | bits |
|---|---:|---:|---:|
| rule12 blended | 100,306 | 36.47% | 8.3534 |
| unigram null (all) | 100,306 | 6.28% | 8.6998 |
| **rule12_generalization** | 14,877 | **1.98%** | **17.0907** |
| **unigram null (gen)** | 14,877 | **7.49%** | **9.1527** |

The null-adjusted reading is the sharpest in-band statement yet of the #364/#234 blended-metric concern: the +30pp blended margin is carried entirely by the memorizable population, while the generalization slice sits **below the constant-predictor null on top-1 and ~8 bits above a unigram model**. Every future improvement claim on generalization now has a floor it must clear in the report itself.

Schema 16 → 17 (#293 convention). Gates from cleaned targets incl. lib-only clippy `-D warnings`.

Closes #390. Refs #364, #234, #374.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

