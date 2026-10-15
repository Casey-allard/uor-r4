# #80 — Evaluate overlap-node alternatives: cloud-size normalization and margin weighting

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:44Z
- **Updated:** 2026-07-23T14:24:38Z
- **Closed:** 2026-07-23T14:24:38Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/80
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64 must land first and show its numbers. Reserved candidate from the #64 redesign list that chain-telescoping did NOT implement: normalize emission sums by contributing-region count, or weight contributions by membership margin/support. If the chain-telescoped Gate C result still underperforms the baseline on uncovered contexts, implement margin-weighted chain selection (or normalization) as a measured variant and compare on the 4-way table. Acceptance: either a measured improvement adopted in the scorer, or a recorded negative result closing the candidate.


---

## Definition of Done
- The reserve candidates (cloud-size normalization, margin/support weighting) are evaluated as measured scorer variants against the chain-telescoped baseline — or closed untriggered because #64's numbers made them unnecessary.
- The outcome is a measured adoption or an attached negative result.

## Acceptance criteria
- [ ] Evaluation happens only after #64's 4-way table exists; the trigger condition (underperformance on uncovered contexts) is explicitly checked first.
- [ ] Each variant has its own Gate C row (top-1, bits/token) vs the chain-telescoped row.
- [ ] Adoption requires a non-noise improvement on the D3 held-out set; otherwise the negative result table is attached and the candidate closed.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

