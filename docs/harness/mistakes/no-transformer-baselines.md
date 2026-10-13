---
id: no-transformer-baselines
title: Compute was spent on transformer controls the owner had withdrawn
date: 2026-10-03
severity: warn
class: compute
trigger: [transformer baseline, transformer control, dense comparator, compare to transformer, baseline, control arm]
paths: [docs/harness/prep/**, docs/mechanisms/**, docs/labs/**]
milestones: [m1, m2, m3, m4]
mechanism: ""
guard: prep.no_prohibited_teacher
evidence: [docs/labs/lab-pivot-2026-10-09/README.md, docs/labs/no-cpu-gpu-only-2026-10-12/README.md, docs/mechanisms/README.md]
status: active
---

## What happened

An audit at the time carried a kill rule of the form "geometric worse than transformer by more
than 0.05 → lose emitter role", so matched transformer controls were being run and paid for. On
2026-10-03 the owner withdrew it: *"it doesn't matter if the transformer beats our baseline right
now, as long as we can get the geometric mechanisms to WORK… constantly testing against
transformer baseline, we are throwing away a ton of work."*

The prohibition has since been reinforced twice: no new prohibited comparator runs (D23/D25),
and converted open-weight models are inactive historical evidence.

## Symptom

A pre-registration or PR whose control arm is a transformer, a dense model, or a converted
open-weight checkpoint. The geometric mechanism is present, but the comparison that consumes the
compute is against a baseline the owner has forbidden.

## Lesson

Do not spend compute on transformer baselines. Matched comparisons happen **within** the
geometric design — quaternion against U(1) lanes, Lorentz against a flat read, one geometric
variant against the previous best geometric model. Results already recorded against transformer
controls keep their scope as inactive evidence; they are not a template for the next run.

## How to apply

1. The control arm in a pre-registration must be another geometric configuration, or the frozen
   geometric parent.
2. The reference ladder is the project's own previous best artifact, not an outside model.
3. If someone asks for a transformer number, answer with the recorded historical values and their
   scope; do not re-run them.
