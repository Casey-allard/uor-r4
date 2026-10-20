# #125 — Unify semantic, causal, temporal, constraint, and provenance edges over one node space

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:40Z
- **Updated:** 2026-07-24T17:00:03Z
- **Closed:** 2026-07-24T17:00:03Z
- **Labels:** graph-compiler
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/125
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Depends on the R4G1 and transition work tracked by #11 and #18.
Source: PDF §4 and §9.

Extend the graph model so one canonical node identity can participate in multiple edge algebras. Do not create separate semantic, belief, intent, causal, and provenance graphs with duplicated node identity.

## Scope

- Specify edge kinds and their payload/algebra: similarity, causal, temporal, constraint, goal-progress, evidence/provenance, refinement/abstraction, predictive-forward, and evidential-predecessor.
- Define which edge kinds are directed, whether they may form cycles, and which require reverse indexes.
- Map the reference edge model to R4G1 sections/feature bits without destabilizing existing packed widths.
- Define canonical ordering and contribution IDs so multiple edge kinds cannot double-count the same evidence.
- Preserve bounded degree, overlap, and frontier declarations in HEAD.

## Acceptance criteria

- The edge algebra is documented with a stable discriminant and validation rules.
- A shared-node fixture exercises at least three edge kinds and round-trips through the reference and packed representations.
- Forward/reverse consistency and no-dangling-reference checks cover every declared index.
- Loader validation rejects unknown mandatory edge kinds, invalid payloads, and undeclared cycles.
- Existing R4G1/graph-runtime tests remain green and legacy artifacts remain readable.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

