# #74 — Richer teacher evidence: top-k > 3 observation records

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:31Z
- **Updated:** 2026-07-23T06:45:25Z
- **Closed:** 2026-07-23T06:45:25Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/74
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: nothing (independent of formula; changes record format — coordinate with any in-flight record work). Observation records currently store top-3 teacher tokens + weights. Residual/emission quality may improve with deeper evidence (top-8 or full top-k over a threshold). Measure the artifact-byte cost vs Gate C gain of top-8 records on the fixture corpus (v4 record format, backward-compatible loader like v3). Acceptance: byte-delta table and Gate C delta; adopt only if the agreement gain justifies the bytes (record the threshold).


---

## Definition of Done
- A v4 observation record format with top-8 teacher tokens + weights exists (backward-compatible loader, like v3), and its byte cost vs Gate C gain is measured.
- An adopt/reject decision is recorded with a documented threshold.

## Acceptance criteria
- [ ] Byte-delta per record and total corpus size delta reported.
- [ ] Gate C top-1 agreement and bits/token with top-8 evidence vs top-3, deterministic double-run.
- [ ] Adopt only if the agreement gain justifies the bytes per the recorded threshold; otherwise the table is attached as the rejection evidence.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

