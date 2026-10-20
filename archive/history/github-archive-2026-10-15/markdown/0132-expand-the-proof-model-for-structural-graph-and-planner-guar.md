# #132 — Expand the proof model for structural graph and planner guarantees

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:04:01Z
- **Updated:** 2026-07-24T18:10:28Z
- **Closed:** 2026-07-24T18:10:28Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/132
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends the executable proof model in #22 and the threat-model work in #32.
Source: PDF §13.

Expand formal work around properties that are actually structural and machine-checkable. Do not attempt to prove human-level reasoning or exact teacher equivalence.

## Scope

Add proof obligations and executable specifications for:

- graph determinism and canonical serialization;
- bounded memory, runtime, frontier, degree, overlap, and planner termination;
- route completeness under declared assumptions;
- provenance/evidence completeness and deletion traceability;
- replay determinism and artifact integrity;
- constraint preservation for certified transitions;
- novelty-safe fallback and explicit resolution statuses;
- no duplicate evidence contributions and safe fixed-point arithmetic.

Use property tests first and Kani where bounded model checking is appropriate. Evaluate heavier deductive tools only after the interfaces stabilize.

## Acceptance criteria

- `uor-r4-proof-model` has one named verifier/spec module per obligation family, with focused errors and no panic-based validation.
- Every entry in the proof-status matrix links to code, test, formal result, assumption, or explicit `Unproven`.
- CI rejects status/documentation drift and rejects unsubstantiated "machine-verified" wording.
- Differential tests compare executable specifications with the scalar runtime/reference planner.
- The proof suite covers malformed artifacts, boundary arithmetic, cycles/termination, empty frontiers, and fallback paths.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

