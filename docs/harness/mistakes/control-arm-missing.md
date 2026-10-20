---
id: control-arm-missing
title: A memory gain measured with no no-memory control
date: 2026-10-10
severity: block
class: measurement
trigger: [no-memory control, control arm, matched control, dose response, 22/40, memory arm, same recipe]
paths: [docs/labs/**, docs/mechanisms/exact-addressed-memory.md, docs/integration/current-state.md]
milestones: [m1]
mechanism: exact-addressed-memory
guard: none: a control's absence is a design decision made before compute; the pre-registration sheet is where it is caught
evidence: [archive/labs/addressed-memory-2026-10-10/README.md, archive/labs/mixture-dose-2026-10-10/README.md, archive/labs/ptr-addr-2026-10-10/README.md]
status: active
---

## What happened

An addressed-memory arm read **22/40**, which the **no-memory control at the shorter fit
also read**, so the reading was not evidence of the memory mechanism. The same cycle then
paid for the control properly: the four-arm dose pattern (10 % recall share) showed the
memory half doubling while the reply half was "statistically indistinguishable from the
base", and abstention was bought separately at 25 % (0/24 at 10 %, 15/24 at 25 %).

## Symptom

A mechanism arm improves a number and the record compares it against the frozen parent, an
older artifact or an aggregate score - but not against a matched arm of the same recipe
with the mechanism off at the same seeds, steps and data split.

## Lesson

An arm is only a mechanism result when a matched control at the same recipe could have
produced the same number. Without it, the gain may be the recipe, the dose, the fit
length or the seed. The related trap is comparing to the frozen parent instead of row by
row against each prior artifact; an aggregate can hide the rows the mechanism did not fix.

## How to apply

- Include the no-memory/matched control in the pre-registration before compute
  (`uor-harness prep new -m m1`); it is part of "what a fair test needs"
  in [the mechanism brief](../../mechanisms/exact-addressed-memory.md).
- Report the control arm beside the treatment arm, same panel, same rows.
- Compare row by row against each prior artifact, not only against the parent or a mean.
- If the control cannot be run, say what it would have separated; an uncontrolled gain is
  NOT YET PROMOTED, not a KEEP.
