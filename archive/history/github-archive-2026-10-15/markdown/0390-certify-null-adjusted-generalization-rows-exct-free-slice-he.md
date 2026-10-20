# #390 — certify: null-adjusted generalization rows — EXCT-free slice headline + analytic unigram null + granularity counts (schema 17)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T19:10:13Z
- **Updated:** 2026-08-03T20:21:21Z
- **Closed:** 2026-08-03T20:21:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/390
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Gate C reports lack a null-adjusted generalization number — every headline is blended and every cross-cover comparison is granularity-confounded

Three standing observations converge on the same missing instrument:

1. **The blended-metric problem** (#364 real-bundle analysis, #234): rule12 headlines blend the exact-context-resolved population with the generalization population; n-gram/EXCT improvements move the blend without touching generalization. The per-status split exists (schema 10+), but there is no headline "generalization slice" row and no baseline that slice must beat.
2. **`docs/transformerless/BASELINE.md` §4.1's own caveat**: "Until an EXCT-free evaluation exists, the stop-or-redesign signal … should be read as 'the current judge measures exact-context memory.'"
3. **Cross-cover incomparability**: bits/token compared across covers of different region counts (e.g. the #374 sweep tables, 38 vs 102 regions) is confounded — a finer partition mechanically shifts what an address-free predictor achieves, so raw values across granularities are not the same quantity. Comparisons need a matched null.

### Scope (report-only; no scoring change, no artifact/κ change)

- **Generalization-slice headline**: an aggregated `rule12_generalization` row = the graph+novel population under rule12 (positions, top-1, bits) — the EXCT-free number, promoted from per-status footnote to first-class row.
- **Analytic unigram null**: from the TRAIN distribution, (a) null top-1 = frequency of the train-majority token on the held-out slice, (b) null bits = train-unigram cross-entropy on held-out (smoothed floor for unseen). Computed for the full population and for the generalization slice. Every residual then reports as a **gap vs the null**, not a raw value.
- **Granularity declaration**: region count and EXCT context count recorded beside the nulls, so any cross-cover comparison carries its granularity and readers can demand matched nulls.

Schema 16 → 17 (additive, #293 convention). Defaults byte-exact; measurement semantics unchanged — only reporting.

Refs #364, #234, #374.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

