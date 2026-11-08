# REPORT: one-model-end-to-end, deepseek-26

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only so far; no pod, model step
or spend.

## What did not work, first

- **The controller repair had no execution authority.** The active deepseek-25
  ORDER limited that unit to pre-run repair and explicitly prohibited a pod.
  D67 returned NO-GO until a separate execution ORDER existed.
- **The pre-registration and selector drifted during review.** The retired early
  continuation stop survived in one sheet sentence, free-generation rendering
  was implicit, and guard-safe/first-result selection lacked mutation coverage.
  The reconciled contract is recorded in deepseek-25's DECISION and report.

## What changed

- This unit is the active execution authorization for one seed: one
  DeepSeek-owned RTX 5090, the already reviewed reply-masked SFT chain, and no
  seed 2.
- The run cannot start until the merged pin reproduces the runtime recorded by
  fresh FIT and D67 GO in the reply-SFT sheet.

## Measurements

No model or GPU step has run. The reviewed resource projection remains one
RTX 5090 for 36-39 minutes normally and 78 minutes with one retry, within the
four-pod and $8/hour owner caps.

## Checks run at the head

- The active ORDER is registered by the repository policy hook.
- The execution gate, float-training tests and input bindings are recorded in
  the deepseek-25 report; this unit rechecks the merged pin and reviewed runtime
  before rental.

Not run yet: pod creation, remote CUDA build, model training, export, panel
scoring, store write and seal readback.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
geometry rules were used directly.

## Still open

- Fresh FIT and D67 GO for this ORDER's final runtime.
- Execute seed 1, read back the seal, take the pod down, and apply the
  deterministic selected-checkpoint outcome map.

## Plain summary

This unit authorizes one reviewed reply-SFT seed and records that no model work
has started yet.
