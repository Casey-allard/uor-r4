# #131 — Implement bounded future-state optimization and planning over graph transitions

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:58Z
- **Updated:** 2026-07-24T17:27:44Z
- **Closed:** 2026-07-24T17:27:44Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/131
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §§3, 10, 12, and 17.

Replace the overloaded "intent" framing with an explicit bounded future-state optimizer. A plan is a finite action trajectory that reaches a goal region while preserving constraints at every intermediate step.

## Scope

- Define goal, forbidden/constraint, action, cost, uncertainty, and bounded-plan types over the semantic state model.
- Implement deterministic reference planning with explicit horizon, frontier, candidate, and memory bounds.
- Define tie-breaking, unknown-transition handling, novelty-safe fallback, and abstention.
- Specify which planning behavior is compiler/reference-only and which fixed-capacity subset may be lowered into runtime state.
- Add evidence/provenance links for selected transitions and rejected alternatives.

## Acceptance criteria

- Planner terminates at the declared bound and never allocates on the runtime-facing path.
- For fixture graphs, plans satisfy goals and never enter forbidden states; invalid plans are rejected with focused errors.
- Replay produces identical plan, score, frontier trace, and witness bytes.
- Exhaustion, novel state, contradictory evidence, and saturated score cases return explicit statuses rather than fabricated plans.
- The plan cost includes bounded runtime/storage terms and is reported separately from teacher fidelity.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

