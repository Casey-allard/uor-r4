---
id: seed-lottery-variance-is-not-a-mechanism-result
title: A seed-to-seed lottery read as a mechanism result
date: 2026-10-12
severity: block
class: measurement
trigger: [seed lottery, only 1/3 seeds learned, seeds flipped, variance, single seed, seed 2 learned, escape step]
paths: [docs/labs/**, docs/harness/milestones.json, archive/integration/agent-execution-policy.md]
milestones: [m4]
mechanism: starling-read
guard: none: a person must judge whether a published number is inside its own seed spread; the panel's noise field is the witness
evidence: [archive/labs/starling-bootstrap-2026-10-12/README.md, archive/labs/starling-read-2026-10-11/README.md, archive/labs/read-binding-margin-2026-10-11/README.md]
status: active
---

## What happened

The starling read reached softmax-level recall at 1536 **when a seed learned** - 0.964 on
seed 2 with a 1/3 learn rate, and seeds flipped between schedules (WTR seed 2 learned
while A75 seed 2 failed). The bootstrap investigation then showed why: learning waits for
a heavy-tailed escape from the chance plateau, and within a fixed budget escape timing is
the seed-to-seed lottery. Recall tracks the budget left after escape (0.99 when escape
came at step 1900, about 0 when it never came). One earlier rung had reported both
starling arms "at chance" from a single configuration each.

## Symptom

A result PR quotes a headline-quality number from one seed, or an arm "fails" on a
configuration whose sibling seed passed. Words like "only 1/3 seeds learned", "seeds
flipped", or an unexplained spread across identical arms.

## Lesson

With a heavy-tailed escape, one seed measures the lottery, not the mechanism. Bars are
never narrower than the panel's noise, and decisive runs use >= 2 seeds or >= 200 fresh
rows, with the mechanism trained in rather than bolted on. Report the spread as
guard-sampling or initialization variance, explicitly: D22's constructor record says
seeds 1001/2001 vary guard sampling, so their spread is guard-sampling variance, not
independent initialization variance.

## How to apply

- Before publishing, check the number against the panel's own noise field
  (`uor-harness panel show -m m4`), and against every seed in the arm.
- Report per-seed values, not only the mean; state which variance the spread measures.
- A single-seed KEEP is not a KEEP: rerun with >= 2 seeds, or mark it NOT YET PROMOTED.
- Fix the lottery rather than averaging over it (the dense bootstrap and the KL during
  phase 1 exist because the escape was random).
