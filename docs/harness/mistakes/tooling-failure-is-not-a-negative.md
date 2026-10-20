---
id: tooling-failure-is-not-a-negative
title: A tooling, numerical or infrastructure failure is a blocker, not a negative
date: 2026-10-10
severity: block
class: process
trigger: [numerical failure, solver tolerance, crash, replay failed, run did not qualify, tooling interruption, UNFINISHED_NOT_NEGATIVE]
paths: [docs/integration/DECISIONS.md, docs/labs/**, docs/integration/current-state.md, scripts/harness/*.py]
milestones: [m2, m3]
mechanism: protected-legal-construction
guard: prlint.prereg_before_compute
evidence: [docs/integration/DECISIONS.md, archive/labs/d22-constructor-2026-10-10/README.md, archive/labs/whole-path-implementation-2026-10-12/README.md]
status: active
---

## What happened

The M2 constructor line produced a run of failures that were recorded as results: a
`Singular matrix` from the direct legal construction, a numerical replay that did not
qualify, a producer interrupted for a session handover. D22 §1 was written because those
were being counted toward the three negatives that force a pivot, and because the run
that was stopped mid-flight was labelled `UNFINISHED_NOT_NEGATIVE` rather than filed as a
closed mechanism.

## Symptom

A PR or record says the mechanism failed and quotes a solver error, a `CUDA` compile
error, a missing field, an interrupted job or a "did not qualify" replay. The count on
the line goes up. The next step becomes another constructor instead of fixing the tool.

## Lesson

A tooling, numerical or infrastructure failure closes nothing: it is a blocker to fix
and the count does not move. The repository already wrote this down in
[DECISIONS.md D22](../../integration/DECISIONS.md) and it was still violated repeatedly,
because the count was maintained by hand in PR prose and nothing checked it.

## How to apply

- State the failure as a blocker with the exact error, and name who can remove it.
- Do not increment the line count; do not close the mechanism (stopping a novel
  mechanism needs the owner).
- The PR body must still cite its pre-registration: `uor-harness lint
  check --body-file <body>` fails `prlint.prereg_before_compute` when a body reports
  numbers without one.
