---
id: owner-decision-float-path-scope
title: Decide whether a float (D37) path may count toward M1, M2 and the product
milestone: M1
area: architecture
status: done
open_to: lab
needs_gpu: false
depends_on: []
assignee: Casey-allard
---

## Why it matters

Stages 3–5 cannot start, or count toward a milestone, until this is settled.

## What is missing

Recorded on 24 Oct 2026: D73 in [DECISIONS](../../integration/DECISIONS.md) (the owner's ruling of 2026-10-24T03:41:41Z).

Before the ruling, D39 item 1 closed M1 only on one saved discrete rung-0 artifact, and the architect's rating of the F32 operators, 'RETIRE at tested scope for now', disagreed with D22's READY/PARKED. D73 records the owner's ruling on the first, and Claude's D68 reading of the second: an architect RETIRE reads as PARKED with its reopen condition. The ruling does not address D41 item 2(d), which bars a float student for distill-d1.

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

