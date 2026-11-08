# ORDER: one-model-end-to-end, deepseek-28 (run the 2,048-step extension)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `f3a3b7869`.

## Goal

Execute the row-6 dose extension: retrain the reviewed reply-masked SFT recipe
from the same exact `(b64L)` seed-1 checkpoint, on the same mixed store, for
2,048 steps at a new root. Score every 256-step checkpoint, select the best
checkpoint by the registered deterministic rule, apply outcome rows 1-5, seal
and read back the evidence, and take the pod down.

This is a dose extension, not row 3's longer run from the selected step-512
checkpoint. No seed-2 run starts here. `m2-reply512` remains unavailable and no
substitute may be scored under that name.

## Cancel probe

- The reply-SFT data, held-out guard, base checkpoint and diagnostic inputs
  retain every full hash in the reviewed original sheet.
- The 1,024-step source seal remains unchanged.
- `uor-pod status` must retain room inside four running pods and $8/hour without
  touching Claude's busy pods.
- The new 2,048 output root must be absent.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-28/**`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2026-11-07.md`
- `docs/harness/prep/one-model-end-to-end-reply-sft-2048-2026-11-07.md`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The reviewed runner accepts only the registered 2,048-step dose and keeps
   1,024 as its legacy default.
2. The job scores checkpoints 256 through 2,048 and applies the same selector.
3. Outcome rows 1-5 are applied to the selected checkpoint; a second row-6
   outcome closes this dose rather than triggering another extension.
4. The evidence seals within 100 MiB, reads back, and the owned pod is deleted.
5. The REPORT and notebook record the result without a capability, mechanism or
   M2 claim.

## Stop and ask if

Ask means decide, record and continue. Never touch Claude's pods, branches,
leases or files. Never weaken a hash, gate, selector, stop, seal or geometry
constraint. Never cancel a healthy run for a self-set timer; renew and record it.
