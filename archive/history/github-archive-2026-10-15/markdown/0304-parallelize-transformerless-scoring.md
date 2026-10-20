# #304 — Parallelize transformerless scoring

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T03:24:47Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-31T04:00:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/304
- **Merged:** 2026-07-31T04:00:23Z
- **Merge commit:** 22a9c763d27fc05cf1f90ecca1be81e2778b1533
- **Base / Head:** main / UOR-Foundation:agent/parallelize-transformerless-score
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

The transformerless score command was spending most of its time in serial CPU loops despite having independent per-position work.

This PR parallelizes:

- corpus observation extraction and store-code generation
- Gate C evaluation across held-out positions
- transition membership computation
- emission accumulation and quantization
- cover k-means assignments and coactivation counting

All reductions that affect artifact bytes or report floating-point totals remain ordered and deterministic.

## Performance

On the fixture command:

- Before: 145.7 s
- After: approximately 15–17 s with the release binary already built
- First run including rebuild: approximately 30 s

The generated report metrics and graph κ remain unchanged.

## Validation

- Targeted clippy with -D warnings
- uor-r4-graph-compiler tests
- uor-r4-graph-certify tests
- uor-r4-core tests
- cargo fmt --check
- Byte-for-byte comparison against the serial artifact/report

The existing R4G1 allocation census still reports 2 allocations / 594 bytes in its separate runtime test path; that issue is preserved as a known caveat and is not caused by the scoring changes.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

