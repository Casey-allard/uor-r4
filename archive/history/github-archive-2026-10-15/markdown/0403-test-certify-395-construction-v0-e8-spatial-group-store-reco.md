# #403 — test(certify): #395 construction v0 — E8 spatial-group store recorded negative, mechanism diagnosed

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-04T01:17:03Z
- **Updated:** 2026-10-09T14:17:03Z
- **Closed:** 2026-08-04T01:33:01Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/403
- **Merged:** 2026-08-04T01:33:01Z
- **Merge commit:** 88a9b9dc7c3b8779dd20af3b6eda71efa9f5d718
- **Base / Head:** main / UOR-Foundation:i395-construction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Graded store keyed by E8 block-group codebooks at matched key budget: 23.0% top1 / WB 8.5579 / 47k keys vs shipped 34.8% / 8.4461 / 179k. Diagnosis: stage OOV rates 40-46% — the 9-block pattern space dwarfs the 255-entry cap, flooding half of all records into shared OOV buckets. Negative is scoped to THIS keying scheme; M1's representation-fidelity win (#397) stands, and residual-E8 (mirroring shipped residual-VQ shape) is explicitly not covered. Results comment on #395. Part of #393.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

