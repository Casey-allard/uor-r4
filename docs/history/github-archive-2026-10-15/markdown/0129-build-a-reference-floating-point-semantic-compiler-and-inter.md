# #129 — Build a reference floating-point semantic compiler and intermediate representation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:52Z
- **Updated:** 2026-07-24T16:34:22Z
- **Closed:** 2026-07-24T16:34:22Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/129
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §§7, 11, and 17.

Make the compiler the primary research surface by introducing a reference floating-point graph compiler that can be evaluated before Boolean lowering or packed artifact emission.

## Scope

- Define an intermediate representation for observations, semantic states, regions, memberships, typed edges, transition scores, emission distributions, residuals, and provenance.
- Separate teacher probing, latent-state/region induction, transition discovery, predictive objective optimization, and artifact lowering into explicit stages.
- Allow slow floating-point and allocation-heavy reference code; do not couple it to the integer-only runtime contract.
- Preserve content-addressed inputs, ordered reductions, memory-budget behavior, and deterministic output semantics from the existing compiler plan.
- Emit a reference graph plus a report that can be consumed by the certifier and differential tests.

## Acceptance criteria

- A pinned mini-corpus compiles into a deterministic reference IR with stable CIDs.
- The reference graph can answer state transitions and emissions without loading a packed R4G1 artifact.
- Held-out evaluation reports teacher loss/agreement, entropy/objective components, overlap statistics, and artifact-cost estimates.
- A differential harness compares the reference graph with the existing induction pipeline so redesigns have an explicit baseline.
- The output boundary is documented: reference IR -> Boolean/integer lowering -> R4G1 -> validated runtime.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

