---
id: owner-decision-float-path-scope
title: Decide whether a float (D37) path may count toward M1, M2 and the product
milestone: M1
area: architecture
status: decision
open_to: lab
needs_gpu: false
depends_on: []
assignee: Casey-allard
---

## Why it matters

Stages 3–5 cannot start, or count toward a milestone, until this is settled.

## What is missing

D39 item 1 closes M1 only on one saved discrete rung-0 artifact, so a float trunk cannot close M1 without the owner redefining the milestone. The architect rates the F32 operators 'RETIRE at tested scope for now' and finds no site for E8 memory, while D22 keeps them READY/PARKED, and that disagreement is unresolved. D41 item 2(d) bars a float student for distill-d1 without a new owner decision.

## Done when

The owner records in DECISIONS.md whether float trunks and a float student count toward M1 and M2, and how the architect dispositions are reconciled with the READY/PARKED statuses.

## How to start

Open these first:

- `docs/integration/DECISIONS.md`
- `ROADMAP.md`
- `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md`

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Owner decision** (§5.1): only the owner decides this. The issue lays out the options and their consequences; labs do not act on their own reading of it.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:111`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:130`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:167`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:177`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)

