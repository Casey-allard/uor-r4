# #79 — Repetition control in the graph scoring path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:41Z
- **Updated:** 2026-07-23T07:19:34Z
- **Closed:** 2026-07-23T07:19:34Z
- **Labels:** graph-compiler
- **Milestone:** Phase 5: Allocation-free graph runtime
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/79
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64. The legacy runtime penalizes recently-emitted tokens via the recency Vec (now TokenState); the graph scorer has no equivalent, so greedy graph generation can loop. Add a bounded, integer-only repetition mechanism to the graph scorer (recency window penalty in ScoreQ, or coverage-aware suppression), measured by repetition rate on a fixed probe set. Acceptance: repetition rate reported in score_report.json; no quality regression on Gate C; allocation-free and integer-only.


---

## Definition of Done
- A bounded, integer-only repetition mechanism exists in the graph scorer (ScoreQ recency penalty or coverage-aware suppression).
- Repetition rate is measured and reported without Gate C quality regression.

## Acceptance criteria
- [ ] Repetition rate on a fixed probe set is reported in score_report.json (before vs after).
- [ ] Gate C top-1 agreement and bits/token unchanged within noise.
- [ ] The mechanism is allocation-free and integer-only (census + source scan).

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

