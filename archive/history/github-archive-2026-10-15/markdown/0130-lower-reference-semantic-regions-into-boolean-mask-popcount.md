# #130 — Lower reference semantic regions into Boolean, mask, popcount, and fixed-point programs

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:55Z
- **Updated:** 2026-07-24T17:05:09Z
- **Closed:** 2026-07-24T17:05:09Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/130
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends the routing, ScoreQ, and operation-instrumentation work in #14, #15, and #19.
Source: PDF §§5, 7, and 17.

Implement the lowering boundary from the reference floating-point graph to the normative transformerless runtime representation.

## Scope

- Define lowering for region predicates and memberships into Boolean/signature, mask, Hamming/popcount, and integer operations.
- Lower transition/emission residuals into fixed-width ScoreQ/table programs with explicit saturation and tie-breaking semantics.
- Preserve overlap and typed edge semantics without duplicating evidence contributions.
- Keep scalar safe Rust as the normative implementation; optimized kernels remain optional and differential-tested.
- Emit a lowering witness that links each runtime contribution to a reference-IR contribution.

## Acceptance criteria

- A fixture lowers reference regions and transitions into a validated R4G1 artifact.
- Runtime outputs match the reference evaluator within the declared quantization tolerance on exhaustive small domains.
- Source/op-census checks show no float, multiply, divide, allocation, lock, or unbounded search in the deployed path.
- Saturation, overflow, candidate truncation, and fallback behavior are tested at boundary values.
- Lowering failure is explicit when a reference construct has no bounded runtime representation; it is never silently approximated.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

