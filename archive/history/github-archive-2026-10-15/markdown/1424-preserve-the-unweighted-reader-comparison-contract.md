# #1424 — Preserve the unweighted reader comparison contract

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T07:25:23Z
- **Updated:** 2026-10-09T13:17:50Z
- **Closed:** 2026-09-27T07:26:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1424
- **Merged:** 2026-09-27T07:26:24Z
- **Merge commit:** e81abd07336ea2062b28bf879b0ca2acc3793407
- **Base / Head:** main / UOR-Foundation:codex/reader-objective-guard-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

After `Campaign.end_weight` was added, the fixed reader comparator could accept a weighted campaign while reporting an unweighted Dot/Lorentz/Affine study. It now rejects every explicit objective override, while retaining compatibility with omitted and null fields.

The existing configuration regression now covers all three geometries through actual Campaign deserialization, accepting omitted/null and rejecting weights1.0/2.5. Explicit1.0 is outside this frozen protocol; no mathematical inequivalence is claimed. The running study uses its unchanged older executable/campaigns and needs no replay.

Corrects related source/result descriptions: training `batch_mean_nll` is weighted, shard-local normalization is not a global weighted mean, and rebased witnesses match non-timing trajectory content rather than report bytes. Updates measured cost components and documents the preserved-receipt correction of duplicate shared elapsed time. Training arithmetic, old reports and the INERT result are preserved.

Validation on source `1df9c903`: three focused optimized Rust comparator checks passed (129 unrelated checks filtered), build/check46.57s, reported maxchildRSS634142720B. Touched-package formatting, claim wording and diff checks passed; independent source review complete. Subsequent source change is a documentation comment only. No new model fit, reload, generation or seal replay.

References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

