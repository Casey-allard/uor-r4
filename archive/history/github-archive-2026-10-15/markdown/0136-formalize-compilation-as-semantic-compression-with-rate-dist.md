# #136 — Formalize compilation as semantic compression with rate-distortion tradeoffs

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:29:27Z
- **Updated:** 2026-07-24T19:27:37Z
- **Closed:** 2026-07-24T19:27:37Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/136
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends #24 and complements #127 and #129.
Source: PDF §7.

Formalize the compiler as a map `C: Theta -> G`, from a trained teacher's behavioral parameterization to a compiled holographic graph artifact. Treat the result as lossy semantic compression, not as an unqualified equivalent model.

## Scope

- Define the teacher behavior surface being sampled and the graph artifact produced by compilation.
- Specify distortion measures for predictive behavior, future-state prediction, semantic reuse, and intervention response.
- Measure rate terms including artifact bytes, bytes read, operation count, frontier/degree budgets, and retained state.
- Compare graph depth, overlap, residual, and fallback choices on rate-distortion curves.
- Record what is intentionally discarded as surface detail and what future-relevant information must be retained.

## Acceptance criteria

- A pinned mini-corpus produces a deterministic compression report with teacher loss/distortion, rate terms, and confidence intervals.
- Reports include depth-wise and ablation-wise curves, not only one composite score.
- A baseline is defined for the current TLA/TLS path and the reference graph compiler.
- The report distinguishes empirical distortion from structural guarantees and records unsupported extrapolations.
- Compiler objective versions and report schema changes are content-addressed and reproducible.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

