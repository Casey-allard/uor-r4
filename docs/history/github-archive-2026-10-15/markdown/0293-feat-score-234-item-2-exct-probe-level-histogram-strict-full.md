# #293 — feat(score): #234 item 2 — EXCT probe-level histogram + strict full-code miss rate (schema 10)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T02:54:08Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-30T04:00:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/293
- **Merged:** 2026-07-30T04:00:07Z
- **Merge commit:** d3293ba5bd4cc6ce21ed18486d2cc1ac04370cd1
- **Base / Head:** main / UOR-Foundation:issue-234-exct-construction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Structural finding (details + measured table on #234): the Rule 2 EXCT probe stops at the deepest POPULATED graded prefix, and the root prefix is always populated with support — so the schema-9 status-based miss rate is structurally ~0 on every corpus and the declared 5% Gate C threshold was unreachable by any corpus construction. This PR lands the semantics-neutral instrumentation that makes the real quantity visible:

- `gate_c.rule12_exct_probe_levels`: per-position probe RESOLUTION level histogram (+ absent count, + strict full-depth-supported count)
- `distribution` (schema 10): `strict_exct_resolved_positions`, `strict_exct_miss_rate`, `exct_probe_level_histogram`; the Gate C validity verdict now judges the STRICT rate
- score-time stdout prints both rates + the histogram loudly

Measured on the pinned fixture harness: status miss 0.0% (30,036/30,036 ExactContext), strict full-code miss **50.3%**, levels L0:0 L1:2,099 L2:7,117 L3:5,901 L4:14,919. The existing D3 story split already exceeds the 5% threshold on the strict basis — **item 2 needs no new corpus**. The Rule 2 firing-depth question (serving semantics) is flagged ⚑ on the issue for Casey and does not block this.

Serving and scoring semantics are untouched; this is declaration/report-layer only.

Closes nothing; advances #234 (items 2+4).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

