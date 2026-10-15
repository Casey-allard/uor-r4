# #109 — Façade, CLI subcommands, Wasm integration, and context scaling benchmark suite

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-23T15:48:50Z
- **Updated:** 2026-07-23T19:52:43Z
- **Closed:** 2026-07-23T19:52:43Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/109
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #103
Depends on: #108
Spec: `docs/r4_furey_quantum_geometric_plan.md` §Phase 6

Expose the Quantum Geometric Transformerless Engine through `uor-r4-wasm-router`, CLI subcommands (`r4 transformerless cd-compile` and `r4 transformerless quantum-eval`), and local HTTP `/api/tless/generate` server.
Add the end-to-end context scaling benchmark suite comparing bits/token, latency, and RAM footprint.

---

## Definition of Done
- CLI subcommands and Wasm bindings updated.
- End-to-end context scaling benchmark results ( = 10^3 \dots 10^6$ tokens) published.
- All CI quality gates (fmt, clippy, workspace tests, no_std, κ-reproduction) pass 100% green.

## Acceptance criteria
- [ ] `r4 transformerless cd-compile` and `quantum-eval` CLI flags functional.
- [ ] Context scaling benchmark table published in walkthrough.
- [ ] All workspace tests pass 100%.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

