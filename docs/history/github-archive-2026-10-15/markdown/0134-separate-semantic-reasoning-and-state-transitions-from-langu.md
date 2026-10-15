# #134 — Separate semantic reasoning and state transitions from language emission

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:29:20Z
- **Updated:** 2026-07-24T18:41:00Z
- **Closed:** 2026-07-24T18:41:00Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/134
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Source: PDF §10.

Make the architectural boundary explicit: graph state transitions, belief updates, causal/goal evaluation, and bounded planning are the semantic operating system; language generation is one interface/emission adapter.

## Scope

- Define separate APIs/IR boundaries for semantic state propagation, transition traces, evidence, and optional token emission.
- Ensure the reference graph and planner can be evaluated without requiring a generated token stream.
- Treat emission as a typed edge/output surface with its own residuals, fallback status, and fidelity metrics.
- Update runtime/certifier terminology so plausible text is not used as a proxy for coherent internal transitions.
- Preserve the existing transformerless language path as a compatibility adapter during migration.

## Acceptance criteria

- A fixture exercises semantic transitions and planning with no language emission, then emits a token result through an explicit adapter.
- Certificates report state-transition coherence, evidence support, planning validity, and language fidelity as separate dimensions.
- The packed/runtime API makes the distinction visible in types or documented entry points.
- Novel, contradictory, and exhausted semantic states return explicit statuses before emission; no fabricated language output is treated as a successful transition.
- Existing `ask`/`chat` behavior remains compatible until a graph Gate-C-passing manifest is available.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

