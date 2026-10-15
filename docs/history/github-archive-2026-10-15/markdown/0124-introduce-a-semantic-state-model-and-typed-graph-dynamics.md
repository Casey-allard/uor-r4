# #124 — Introduce a semantic state model and typed graph dynamics

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:03:36Z
- **Updated:** 2026-07-24T15:33:02Z
- **Closed:** 2026-07-24T15:33:01Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/124
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §§2, 3, 12.

Introduce a reference semantic-state model that treats the graph as a state-transition system rather than only an associative lookup structure.

## Scope

- Define a compiler/reference representation for semantic states, observations, actions/operators, beliefs, goals, constraints, uncertainty, and bounded trajectories.
- Define typed transitions `T:S x A -> S` with explicit preconditions, postconditions, and failure/unknown outcomes.
- Represent semantic regions as subsets or predicates over `S`; keep the model independent of tokenizer-specific identities.
- Provide a deterministic transition evaluator suitable for differential testing against later packed execution.
- Keep this reference model out of the allocation-free deployed kernel until its representation and bounds are frozen.

## Acceptance criteria

- The model has explicit types and focused errors for invalid actions, missing transitions, forbidden states, and exceeded trajectory bounds.
- Unit/property tests cover deterministic replay, goal satisfaction, constraint preservation, and bounded termination.
- A small fixture demonstrates belief update -> transition -> goal/constraint evaluation without token generation.
- The design records which state fields are compile-time/reference-only and which fields can be lowered into fixed-capacity runtime state.
- Cross-reference the existing graph runtime and R4G1 contracts rather than introducing a second artifact identity model.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

