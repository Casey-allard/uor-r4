# ORDER: one-model-end-to-end, deepseek-27 (record seed-1 result)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `8903c5551`.

## Goal

Record deepseek-26's completed seed-1 reply-masked SFT in its REPORT and the
float-training notebook, apply the registered deterministic selector and
first-match outcome map, and preserve the sealed artifact identities. This is a
laptop-only record unit: no pod, no model execution and no spend.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-27/**`
- `docs/loop/tasks/one-model-end-to-end/deepseek-26/REPORT.md`
- `docs/research/notebook/float-training.md`

## Done when

1. The selected checkpoint and outcome row are recorded.
2. Every reported hash/path is copied from the verified store readback.
3. Negative results and the distinction between configuration movement and
   mechanism/capability claims are explicit.
4. The next dependency named by the outcome map is recorded without starting it.

## Stop and ask if

Do not rent compute, alter a threshold, run seed 2, claim M2, or touch another
lab's work in this unit.
