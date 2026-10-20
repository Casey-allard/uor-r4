# #404 — test(certify): #395 v1 residual-E8 — lattice snapping is free at 0.5rms (retraining-controlled)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T01:43:25Z
- **Updated:** 2026-10-09T14:17:04Z
- **Closed:** 2026-08-04T02:00:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/404
- **Merged:** 2026-08-04T02:00:13Z
- **Merge commit:** 558d8cdfd42831bb98b1885525344823ad89c59a
- **Base / Head:** main / UOR-Foundation:i395-rvq
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Five arms, identical RVQ skeleton/training/store/eval. rvq-e8@0.5rms matches the retrained f32 control to 0.1pp top1 and edges it on WB (7.9175 vs 7.9266): icosian-lattice codebooks cost nothing at the right coarseness, so store centroids can live on the lattice (integer/fixed-point, golden-coupled quaternion pairs, no-multiply-friendly decode) for free. Coarseness curve monotone above 0.5rms. Retrained-RVQ-beats-shipped (+2pp/−0.52 bits) recorded as an orthogonal compile-quality observation. Results comment on #395. Part of #393.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

