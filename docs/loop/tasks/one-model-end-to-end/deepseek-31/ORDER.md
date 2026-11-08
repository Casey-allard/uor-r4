# ORDER: one-model-end-to-end, deepseek-31 (M-World ratio SFT)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `8688eb1a9`.

## Goal

Test one distinct answer-content intervention after the closed row-6 dose: fine-tune
the same `(b64L)` seed-1 model for 2,048 steps on OASST2 reply-masked data plus the
same excluded M-World store repeated three times. Keep the human continuation guard,
the per-turn panel and the exact selector. Scale only if outcome row 3 is reached.

## Cancel probe

- The base checkpoint, OASST2 and M-World sources are immutable and fully hashed.
- The new mixed store is uploaded and read back at
  `results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/inputs/`.
- The corpus-panel overlap audit is zero in all four categories.
- The 2,048 row-6 dose is closed; this is a different data-distribution
  intervention, not a second extension of that dose.
- `uor-pod status` must stay inside four pods and $8/hour without touching
  another lab's pods or leases.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-31/**`
- `docs/harness/prep/one-model-end-to-end-content-ratio-2026-11-08.md`
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The job and launcher have a closed `content-ratio` recipe pair with its own
   data paths, hashes, sheet and report; the legacy 1,024 recipe remains available.
2. The run uses the same geometry, optimization, checkpoint scoring, selector and
   CPU hardware-binding checks as the reviewed reply-SFT path.
3. The sealed run reaches outcome row 3 or closes this data intervention; it never
   authorizes a second ratio extension or seed 2 after a non-positive result.
4. The evidence reads back, the pod is deleted, and the result unit records the
   complete outcome without a mechanism, M2 or useful-chat claim.

Never touch another lab's pods, branches, leases or files. Never weaken a hash,
gate, selector, stop, seal, CPU-evidence or geometry constraint. Never cancel a
healthy run for a self-set timer.
